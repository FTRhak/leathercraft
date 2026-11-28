//! The decoded pictures of the project's background images, as egui textures.
//!
//! Interface state: the project only stores each image's path and placement
//! (`leathercraft::background`). A picture is decoded the first time it is drawn, and again
//! after Reload; a file that can't be read is remembered as failed, so it isn't retried every
//! frame. Loading is synchronous: a large photo stalls one frame when it is added or opened,
//! which could move to a thread if it gets in the way.

use bevy_egui::egui;
use image::{DynamicImage, ImageDecoder, ImageReader};
use leathercraft::i18n::Message;
use leathercraft::model::Project;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Larger pictures are scaled down once on load, to bound the texture memory.
const MAX_SIDE_PX: u32 = 4096;

/// The extensions the Add image dialog offers.
pub const EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "tif", "tiff"];

pub enum Picture {
    Loaded {
        texture: egui::TextureHandle,
        /// The size of the file's picture, before any scaling down.
        size_px: [u32; 2],
        kilobytes: u64,
    },
    Failed(Message),
}

#[derive(Default)]
pub struct BackgroundTextures {
    pictures: HashMap<PathBuf, Picture>,
}

impl BackgroundTextures {
    /// The picture at `path`, decoded on first use.
    pub fn get(&mut self, ctx: &egui::Context, path: &Path) -> &Picture {
        self.pictures
            .entry(path.to_path_buf())
            .or_insert_with(|| load(ctx, path))
    }

    /// Forgets the picture, so that the next `get` reads the file again.
    pub fn reload(&mut self, path: &Path) {
        self.pictures.remove(path);
    }

    /// Drops the pictures no image of the project uses any more.
    pub fn retain_used(&mut self, project: &Project) {
        if self.pictures.len() <= project.background_images.len() {
            return;
        }
        self.pictures.retain(|path, _| {
            project
                .background_images
                .iter()
                .any(|image| image.path == *path)
        });
    }
}

fn load(ctx: &egui::Context, path: &Path) -> Picture {
    let failed = |detail: String| {
        bevy::log::warn!(
            "Could not load background image {}: {detail}",
            path.display()
        );
        Picture::Failed(Message::ImageLoadFailed {
            path: path.display().to_string(),
            detail,
        })
    };
    let kilobytes = std::fs::metadata(path).map_or(0, |metadata| metadata.len() / 1024);
    let picture = match decode(path) {
        Ok(picture) => picture,
        Err(detail) => return failed(detail),
    };
    let size_px = [picture.width(), picture.height()];
    // egui panics on a texture past the device's limit, which can be as low as 2048 px.
    let device_max = ctx.input(|input| input.max_texture_side);
    let max_side = u32::try_from(device_max)
        .unwrap_or(u32::MAX)
        .clamp(1, MAX_SIDE_PX);
    let picture = if size_px[0].max(size_px[1]) > max_side {
        bevy::log::info!(
            "Scaling background image {} from {}x{} to fit {max_side} px",
            path.display(),
            size_px[0],
            size_px[1]
        );
        picture.resize(max_side, max_side, image::imageops::FilterType::Triangle)
    } else {
        picture
    };
    let rgba = picture.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    // Should the scaling still leave it too large, a warning beats a panic.
    if size[0].max(size[1]) > device_max || size[0] == 0 || size[1] == 0 {
        return failed(format!(
            "{}×{} px is larger than this graphics device allows ({device_max} px)",
            size[0], size[1]
        ));
    }
    let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    let name = format!("background:{}", path.display());
    let texture = ctx.load_texture(name, color, egui::TextureOptions::LINEAR);
    Picture::Loaded {
        texture,
        size_px,
        kilobytes,
    }
}

/// Decodes the file, turned upright by its EXIF orientation, as phone photos need.
fn decode(path: &Path) -> Result<DynamicImage, String> {
    let reader = ImageReader::open(path)
        .map_err(|error| error.to_string())?
        .with_guessed_format()
        .map_err(|error| error.to_string())?;
    let mut decoder = reader.into_decoder().map_err(|error| error.to_string())?;
    let orientation = decoder.orientation().ok();
    let mut picture = DynamicImage::from_decoder(decoder).map_err(|error| error.to_string())?;
    if let Some(orientation) = orientation {
        picture.apply_orientation(orientation);
    }
    Ok(picture)
}
