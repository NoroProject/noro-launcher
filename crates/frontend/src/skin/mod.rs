//! 3D Minecraft skin renderer: software rasterizer with z-buffer, painter sort,
//! two-light shading, overlay layers, slim/legacy models and capes.

mod geometry;
mod math;
mod model;
mod part;
mod preview;
mod project;
mod raster;

use image::{DynamicImage, GenericImageView, RgbaImage};
use math::Mat3;
use part::BodyPartType;
use project::{project_part, ProjectedQuad};
use raster::Texture;

pub use preview::{render_view, PREVIEW_H, PREVIEW_W};

/// Widest and tallest the model gets at any angle, in model units (brute-forced
/// upstream). Used to fit the figure into the output regardless of rotation.
const MAX_WIDTH: f64 = 20.407_198_535_851_574;
const MAX_HEIGHT: f64 = 34.651_839_777_997_37;

/// Camera and animation state for a single frame.
pub struct View {
    pub yaw: f64,
    pub pitch: f64,
    /// Idle loop progress in `[0, 1)`.
    pub sway: f64,
    pub zoom: f64,
    /// Shift of the figure in model units, positive moves it down.
    pub y_offset: f64,
}

impl Default for View {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            sway: 0.0,
            zoom: 1.0,
            y_offset: 0.0,
        }
    }
}

struct Skin {
    image: DynamicImage,
    /// 64×32 texture of pre-1.8 skins: no second layer, mirrored left limbs.
    legacy: bool,
    uv_scale: f64,
}

fn load_skin(bytes: &[u8]) -> Option<Skin> {
    let image = image::load_from_memory(bytes).ok()?;
    let (w, h) = (image.width(), image.height());
    if w < 64 || w % 64 != 0 {
        return None;
    }
    let legacy = h * 2 == w;
    if !legacy && h != w {
        return None;
    }
    Some(Skin {
        image,
        legacy,
        uv_scale: w as f64 / 64.0,
    })
}

/// Slim skins leave the extra arm column of the texture transparent.
fn is_slim(skin: &Skin) -> bool {
    if skin.legacy {
        return false;
    }
    let s = skin.uv_scale as u32;
    skin.image.get_pixel(54 * s, 20 * s)[3] < 20
}

/// Render one frame onto a transparent RGBA canvas. `None` if the skin is not a
/// valid 64×64 / 64×32 layout.
pub fn render_rgba(
    skin_png: &[u8],
    cape_png: Option<&[u8]>,
    width: u32,
    height: u32,
    view: &View,
) -> Option<RgbaImage> {
    let skin = load_skin(skin_png)?;
    let cape = cape_png.and_then(|b| image::load_from_memory(b).ok());
    let slim = is_slim(&skin);

    let rot = Mat3::rotation_yx(view.yaw.to_radians(), view.pitch.to_radians());
    let mut quads: Vec<ProjectedQuad> = Vec::new();
    for def in model::player_model(skin.legacy, slim) {
        project_part(&def, &rot, view.sway, &mut quads);
    }
    if cape.is_some() {
        project_part(&model::cape_part(), &rot, view.sway, &mut quads);
    }
    // Painter's algorithm: farthest quads first, so blended overlays land on top.
    quads.sort_by(|a, b| a.avg_z.total_cmp(&b.avg_z));

    let scale = (width as f64 / MAX_WIDTH).min(height as f64 / MAX_HEIGHT) * view.zoom;
    let offset_x = width as f64 / 2.0;
    let offset_y = height as f64 / 2.0 + view.y_offset * scale;

    let skin_tex = Texture {
        image: &skin.image,
        uv_scale: skin.uv_scale,
    };
    let cape_tex = cape.as_ref().map(|c| Texture {
        image: c,
        uv_scale: c.width() as f64 / 64.0,
    });

    let mut out = RgbaImage::new(width, height);
    let mut zbuf = vec![f64::MIN; (width * height) as usize];
    for mut quad in quads {
        for v in &mut quad.verts {
            v.x = v.x * scale + offset_x;
            v.y = v.y * scale + offset_y;
        }
        let tex = if quad.kind == BodyPartType::Cape {
            cape_tex.as_ref()
        } else {
            Some(&skin_tex)
        };
        if let Some(tex) = tex {
            raster::rasterize_quad(&quad, tex, &mut out, &mut zbuf);
        }
    }
    Some(out)
}
