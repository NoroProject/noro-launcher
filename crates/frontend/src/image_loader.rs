use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use gpui::{Image, ImageFormat};
use std::sync::Arc;

struct LoadedImage {
    format: ImageFormat,
    bytes: Vec<u8>,
}

pub async fn load_image_from_url(url: String) -> Result<Arc<Image>, String> {
    if url.starts_with("data:") {
        return decode_data_url(&url);
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let result =
            fetch_image(url).map(|image| Arc::new(Image::from_bytes(image.format, image.bytes)));
        let _ = tx.send(result);
    });
    rx.await.map_err(|_| "image loader stopped".to_string())?
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
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let result = fetch_image(url).map(|image| {
            let (format, bytes) = match downscale(&image.bytes, max_side) {
                Some(smaller) => smaller,
                None => (image.format, image.bytes),
            };
            Arc::new(Image::from_bytes(format, bytes))
        });
        let _ = tx.send(result);
    });
    rx.await.map_err(|_| "image loader stopped".to_string())?
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
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let result = fetch_image_and_bytes(url);
        let _ = tx.send(result);
    });
    rx.await.map_err(|_| "image loader stopped".to_string())?
}

fn fetch_image_and_bytes(url: String) -> Result<(Arc<Image>, Vec<u8>), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    runtime.block_on(async move {
        let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
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
        let format = image_format_from_bytes(&bytes)
            .or_else(|| {
                content_type
                    .as_deref()
                    .and_then(ImageFormat::from_mime_type)
            })
            .or_else(|| image_format_from_url(&url))
            .ok_or_else(|| "unknown image format".to_string())?;

        let (final_format, final_bytes) = normalize_image_bytes(bytes, format);
        let img = Arc::new(Image::from_bytes(final_format, final_bytes.clone()));
        Ok((img, final_bytes))
    })
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

fn fetch_image(url: String) -> Result<LoadedImage, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    runtime.block_on(async move {
        let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
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
        let format = image_format_from_bytes(&bytes)
            .or_else(|| {
                content_type
                    .as_deref()
                    .and_then(ImageFormat::from_mime_type)
            })
            .or_else(|| image_format_from_url(&url))
            .ok_or_else(|| "unknown image format".to_string())?;

        let (final_format, final_bytes) = normalize_image_bytes(bytes, format);
        Ok(LoadedImage {
            format: final_format,
            bytes: final_bytes,
        })
    })
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
