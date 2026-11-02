use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

const ICON_FONT: &str = "leathercraft-icons";

/// Glyphs of the icon font, in its private use area.
pub mod glyph {
    pub const TRASH: char = '\u{E110}';
    pub const LOCK: char = '\u{E115}';
    pub const UNLOCK: char = '\u{E116}';
}

fn icon_font() -> egui::FontFamily {
    egui::FontFamily::Name(ICON_FONT.into())
}

/// Registers the icon font as its own family; run once, it takes effect from the next pass.
pub fn install_icon_font(mut contexts: EguiContexts) -> Result {
    let data = egui::FontData::from_static(include_bytes!("../res/leathercrafticons.ttf"));
    contexts
        .ctx_mut()?
        .add_font(egui::epaint::text::FontInsert::new(
            ICON_FONT,
            data,
            vec![egui::epaint::text::InsertFontFamily {
                family: icon_font(),
                priority: egui::epaint::text::FontPriority::Highest,
            }],
        ));
    Ok(())
}

/// Paints a glyph of the icon font centered in `rect`, in the stroke's color.
pub fn paint_glyph(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke, glyph: char) {
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::new(rect.height() * 0.75, icon_font()),
        stroke.color,
    );
}

/// A square button with a painted icon; the closure gets the button's rect and stroke.
pub fn icon_button(
    ui: &mut egui::Ui,
    size: f32,
    selected: bool,
    name: &str,
    paint: impl FnOnce(&egui::Painter, egui::Rect, egui::Stroke),
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, name)
    });
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, selected);
        ui.painter().rect(
            rect,
            visuals.corner_radius,
            visuals.weak_bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        paint(ui.painter(), rect, visuals.fg_stroke);
    }
    response.on_hover_text(name)
}
