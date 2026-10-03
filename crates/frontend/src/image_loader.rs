use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use gpui::{Image, ImageFormat, RenderImage};
use image::Frame;
use std::sync::Arc;

struct LoadedImage {
    format: ImageFormat,
    bytes: Vec<u8>,
}

/// One small runtime and one client for every picture the window loads. Each
/// image used to get its own OS thread, its own tokio runtime and a fresh
/// `reqwest::get` — no connection reuse, no HTTP/2, no timeouts, and no limit
/// on how many ran at once when a catalogue page opened.
static RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("noro-images")
        .enable_all()
        .build()
        .expect("image runtime")
});

static CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(concat!("noro-launcher/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(std::time::Duration::from_secs(10))
        .read_timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default()
});

/// Downloads in flight at once.
static SLOTS: std::sync::LazyLock<tokio::sync::Semaphore> =
    std::sync::LazyLock::new(|| tokio::sync::Semaphore::new(8));

/// Runs `work` on the image runtime and waits for it from any executor —
/// GPUI's included.
async fn on_runtime<T: Send + 'static>(
    work: impl std::future::Future<Output = Result<T, String>> + Send + 'static,
) -> Result<T, String> {
    RUNTIME
        .spawn(work)
        .await
        .map_err(|_| "image loader stopped".to_string())?
}

/// CPU work (decoding, resizing) off both the UI thread and the runtime's
/// workers.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    on_runtime(async move {
        tokio::task::spawn_blocking(work)
            .await
            .map_err(|_| "image decoder stopped".to_string())?
    })
    .await
}

pub async fn load_image_from_url(url: String) -> Result<Arc<Image>, String> {
    if url.starts_with("data:") {
        return decode_data_url(&url);
    }
    let image = fetch_image(url).await?;
    Ok(Arc::new(Image::from_bytes(image.format, image.bytes)))
}

/// Like `load_image_from_url`, but shrinks anything larger than `max_side`
/// first.
///
/// Art comes off the master at whatever size it was uploaded — a build icon at
/// 1024×1024 that gets drawn the size of a fingernail, a background at full
/// width. GPUI decodes those to RGBA and keeps them both in the heap and as a
/// GPU texture, so every pixel we never show is paid for twice.
///
/// Not for skins: they are 64×64 pixel art and go through the 3D renderer.
pub async fn load_image_capped(url: String, max_side: u32) -> Result<Arc<Image>, String> {
    if url.starts_with("data:") {
        return decode_data_url(&url);
    }
    let image = fetch_image(url).await?;
    blocking(move || {
        let (format, bytes) = match downscale(&image.bytes, max_side) {
            Some(smaller) => smaller,
            None => (image.format, image.bytes),
        };
        Ok(Arc::new(Image::from_bytes(format, bytes)))
    })
    .await
}

/// `None` when the image is already small enough or can't be decoded.
///
/// Keeps an alpha channel as PNG and sends everything else to JPEG — re-encoding
/// a photographic background as PNG would undo the saving in transit.
fn downscale(bytes: &[u8], max_side: u32) -> Option<(ImageFormat, Vec<u8>)> {
    let img = image::load_from_memory(bytes).ok()?;
    if img.width() <= max_side && img.height() <= max_side {
        return None;
    }
    let scaled = img.resize(max_side, max_side, image::imageops::FilterType::Lanczos3);
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    if img.color().has_alpha() {
        scaled.write_to(&mut cursor, image::ImageFormat::Png).ok()?;
        Some((ImageFormat::Png, out))
    } else {
        scaled
            .into_rgb8()
            .write_to(&mut cursor, image::ImageFormat::Jpeg)
            .ok()?;
        Some((ImageFormat::Jpeg, out))
    }
}

pub async fn load_image_and_bytes(url: String) -> Result<(Arc<Image>, Vec<u8>), String> {
    if url.starts_with("data:") {
        let img = decode_data_url(&url)?;
        let b64 = url.split(',').nth(1).ok_or("invalid data URL")?;
        let bytes = B64.decode(b64).map_err(|e| e.to_string())?;
        return Ok((img, bytes));
    }
    let image = fetch_image(url).await?;
    let img = Arc::new(Image::from_bytes(image.format, image.bytes.clone()));
    Ok((img, image.bytes))
}

fn normalize_image_bytes(bytes: Vec<u8>, fallback_format: ImageFormat) -> (ImageFormat, Vec<u8>) {
    if bytes.starts_with(b"\x89PNG") {
        return (ImageFormat::Png, bytes);
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return (ImageFormat::Jpeg, bytes);
    }
    if let Ok(dyn_img) = image::load_from_memory(&bytes) {
        let mut png_bytes = Vec::new();
        if dyn_img
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .is_ok()
        {
            return (ImageFormat::Png, png_bytes);
        }
    }
    (fallback_format, bytes)
}

fn decode_data_url(url: &str) -> Result<Arc<Image>, String> {
    let b64 = url.split(',').nth(1).ok_or("invalid data URL")?;
    let bytes = B64.decode(b64).map_err(|e| e.to_string())?;
    let format = image_format_from_bytes(&bytes)
        .ok_or_else(|| "unknown image format in data URL".to_string())?;
    let (final_format, final_bytes) = normalize_image_bytes(bytes, format);
    Ok(Arc::new(Image::from_bytes(final_format, final_bytes)))
}

/// Pictures kept on disk by URL. Every start used to download every icon and
/// background again, and offline the window had none at all.
mod disk {
    use std::path::PathBuf;
    use std::time::Duration;

    /// Younger than this is used without asking the network.
    pub const FRESH: Duration = Duration::from_secs(24 * 3600);
    /// Not refreshed for this long: nothing shows it any more.
    const KEEP: Duration = Duration::from_secs(30 * 24 * 3600);

    fn dir() -> Option<PathBuf> {
        Some(
            dirs::data_dir()?
                .join(schema::launcher_dir_name())
                .join("cache")
                .join("images"),
        )
    }

    pub fn path(url: &str) -> Option<PathBuf> {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut h);
        Some(dir()?.join(format!("{:016x}", h.finish())))
    }

    fn type_path(path: &std::path::Path) -> PathBuf {
        path.with_extension("type")
    }

    /// The bytes, their content type if one was recorded, and their age.
    pub fn read(path: &std::path::Path) -> Option<(Vec<u8>, Option<String>, Duration)> {
        let age = std::fs::metadata(path)
            .ok()?
            .modified()
            .ok()?
            .elapsed()
            .unwrap_or_default();
        let bytes = std::fs::read(path).ok()?;
        let content_type = std::fs::read_to_string(type_path(path)).ok();
        Some((bytes, content_type, age))
    }

    pub fn write(path: &std::path::Path, bytes: &[u8], content_type: Option<&str>) {
        let Some(parent) = path.parent() else {
            return;
        };
        let _ = std::fs::create_dir_all(parent);
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, bytes).is_ok() && std::fs::rename(&tmp, path).is_ok() {
            match content_type {
                Some(t) => {
                    let _ = std::fs::write(type_path(path), t);
                }
                None => {
                    let _ = std::fs::remove_file(type_path(path));
                }
            }
        }
    }

    /// Once per run, in the background.
    pub fn prune() {
        let Some(dir) = dir() else {
            return;
        };
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let stale = entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|age| age > KEEP);
            if stale {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

/// The body as it came, with the declared content type: from the disk while
/// it is fresh, from the network otherwise, and from the disk again, however
/// old, when the network fails.
async fn fetch_raw(url: String) -> Result<(Vec<u8>, Option<String>), String> {
    static PRUNE: std::sync::Once = std::sync::Once::new();
    PRUNE.call_once(|| {
        RUNTIME.spawn_blocking(disk::prune);
    });
    on_runtime(async move {
        let path = disk::path(&url);
        let cached = match path.clone() {
            Some(p) => tokio::task::spawn_blocking(move || disk::read(&p))
                .await
                .ok()
                .flatten(),
            None => None,
        };
        if let Some((bytes, content_type, age)) = &cached {
            if *age < disk::FRESH {
                return Ok((bytes.clone(), content_type.clone()));
            }
        }
        match fetch_network(&url).await {
            Ok((bytes, content_type)) => {
                if let Some(p) = path {
                    let (b, t) = (bytes.clone(), content_type.clone());
                    let _ = tokio::task::spawn_blocking(move || disk::write(&p, &b, t.as_deref()))
                        .await;
                }
                Ok((bytes, content_type))
            }
            Err(e) => match cached {
                Some((bytes, content_type, _)) => Ok((bytes, content_type)),
                None => Err(e),
            },
        }
    })
    .await
}

async fn fetch_network(url: &str) -> Result<(Vec<u8>, Option<String>), String> {
    let _slot = SLOTS.acquire().await.map_err(|e| e.to_string())?;
    let response = CLIENT.get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .map(str::trim)
        .map(str::to_string);
    let bytes = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
    Ok((bytes, content_type))
}

async fn fetch_image(url: String) -> Result<LoadedImage, String> {
    let (bytes, content_type) = fetch_raw(url.clone()).await?;
    let format = image_format_from_bytes(&bytes)
        .or_else(|| {
            content_type
                .as_deref()
                .and_then(ImageFormat::from_mime_type)
        })
        .or_else(|| image_format_from_url(&url))
        .ok_or_else(|| "unknown image format".to_string())?;
    blocking(move || {
        let (format, bytes) = normalize_image_bytes(bytes, format);
        Ok(LoadedImage { format, bytes })
    })
    .await
}

fn image_format_from_bytes(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(ImageFormat::Png);
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some(ImageFormat::Jpeg);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(ImageFormat::Gif);
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some(ImageFormat::Webp);
    }
    if bytes.starts_with(b"BM") {
        return Some(ImageFormat::Bmp);
    }
    if bytes.starts_with(b"\0\0\x01\0") {
        return Some(ImageFormat::Ico);
    }
    if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        return Some(ImageFormat::Tiff);
    }
    if bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
        return Some(ImageFormat::Svg);
    }
    None
}

fn image_format_from_url(url: &str) -> Option<ImageFormat> {
    let clean = url
        .split('?')
        .next()
        .unwrap_or(url)
        .split('#')
        .next()
        .unwrap_or(url);
    let ext = clean.rsplit('.').next()?.to_ascii_lowercase();
    match ext.as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "webp" => Some(ImageFormat::Webp),
        "gif" => Some(ImageFormat::Gif),
        "svg" => Some(ImageFormat::Svg),
        "bmp" => Some(ImageFormat::Bmp),
        "tif" | "tiff" => Some(ImageFormat::Tiff),
        "ico" => Some(ImageFormat::Ico),
        "pnm" | "pbm" | "ppm" | "pgm" => Some(ImageFormat::Pnm),
        _ => None,
    }
}

/// Like `load_image_capped`, but hands back pixels instead of a compressed
/// image.
///
/// `Image` holds the PNG or WebP bytes and goes through GPUI's asset cache,
/// which decodes it on a background task the first time it is drawn. That is
/// fine for one picture and ruinous for a list: the catalogue draws twenty-one
/// icons, and at sixty frames a second the measured cost was 311 ms a frame —
/// six fps — against 30 ms on the same screen without them.
///
/// `RenderImage` is already decoded, so drawing it is an atlas lookup. The same
/// reasoning is written down in `skin::preview`, which draws a frame every
/// 16 ms and could never have used `Image`.
///
/// The caller owns the texture: GPUI keeps every `RenderImage` in the sprite
/// atlas by id and never evicts one on its own, so a picture that is replaced
/// has to be handed back with `cx.drop_image`.
pub async fn load_render_image_capped(
    url: String,
    max_side: u32,
) -> Result<Arc<RenderImage>, String> {
    let bytes = if url.starts_with("data:") {
        let b64 = url.split(',').nth(1).ok_or("invalid data URL")?;
        B64.decode(b64).map_err(|e| e.to_string())?
    } else {
        // Decoded to pixels right away, so the PNG round trip that `Image`
        // needs for WebP and friends would be wasted here.
        fetch_raw(url).await?.0
    };
    blocking(move || decode_to_frame(&bytes, max_side)).await
}

/// Decode, shrink and swap to BGRA — the order GPUI uploads textures in.
fn decode_to_frame(bytes: &[u8], max_side: u32) -> Result<Arc<RenderImage>, String> {
    let decoded = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let scaled = if decoded.width() > max_side || decoded.height() > max_side {
        decoded.resize(max_side, max_side, image::imageops::FilterType::Triangle)
    } else {
        decoded
    };
    let mut rgba = scaled.into_rgba8();
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Ok(Arc::new(RenderImage::new(vec![Frame::new(rgba)])))
}
