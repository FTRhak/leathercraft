//! The app logo of the title bar: the same picture as the icon of the executable
//! (`assets/leathercraft.ico`), bundled as a 256 px PNG and scaled down once to the exact pixel
//! size it is shown at, so it stays sharp at any display scale.

use bevy_egui::egui;

use crate::app_icon::LOGO_PNG;

/// The logo as a texture `side` pixels square, decoded on first use and kept in the context.
pub(super) fn texture(ctx: &egui::Context, side: u32) -> egui::TextureHandle {
    let id = egui::Id::new(("title_bar_logo", side));
    if let Some(texture) = ctx.data(|data| data.get_temp::<egui::TextureHandle>(id)) {
        return texture;
    }
    let mut picture = image::load_from_memory_with_format(LOGO_PNG, image::ImageFormat::Png)
        .expect("the bundled logo is a valid PNG")
        .into_rgba8();
    // Premultiplied before scaling, so the transparent corners don't darken the edge.
    for pixel in picture.pixels_mut() {
        let alpha = u16::from(pixel[3]);
        for channel in &mut pixel.0[..3] {
            *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
        }
    }
    let scaled =
        image::imageops::resize(&picture, side, side, image::imageops::FilterType::Lanczos3);
    let size = [side as usize, side as usize];
    let image = egui::ColorImage::from_rgba_premultiplied(size, scaled.as_raw());
    let texture = ctx.load_texture("title_bar_logo", image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    texture
}
