use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

const ICON_FONT: &str = "leathercraft-icons";

/// Glyphs of the icon font, in its private use area.
pub mod glyph {
    pub const TRASH: char = '\u{E110}';
    pub const LOCK: char = '\u{E115}';
    pub const UNLOCK: char = '\u{E116}';
    pub const HISTORY: char = '\u{E121}';
    pub const SELECT: char = '\u{E130}';
    pub const DRAW: char = '\u{E132}';
    pub const RECTANGLE: char = '\u{E133}';
    pub const ELLIPSE: char = '\u{E134}';
    pub const EDGE: char = '\u{E135}';
    pub const STITCH_HOLES: char = '\u{E136}';
    pub const CENTERLINE: char = '\u{E137}';
    pub const REGULAR_POLYGON: char = '\u{E138}';
    pub const NEW_PIECE: char = '\u{E13C}';
    pub const SUBTRACT: char = '\u{E13D}';
    pub const INTERSECT: char = '\u{E13F}';
    pub const UNION: char = '\u{E13E}';
    pub const ALIGN_LEFT: char = '\u{E140}';
    pub const ALIGN_CENTER_X: char = '\u{E141}';
    pub const ALIGN_RIGHT: char = '\u{E142}';
    pub const ALIGN_TOP: char = '\u{E143}';
    pub const ALIGN_CENTER_Y: char = '\u{E144}';
    pub const ALIGN_BOTTOM: char = '\u{E145}';
    pub const DISTRIBUTE_TOP: char = '\u{E146}';
    pub const DISTRIBUTE_CENTER_Y: char = '\u{E147}';
    pub const DISTRIBUTE_BOTTOM: char = '\u{E148}';
    pub const DISTRIBUTE_LEFT: char = '\u{E149}';
    pub const DISTRIBUTE_CENTER_X: char = '\u{E14A}';
    pub const DISTRIBUTE_RIGHT: char = '\u{E14B}';
}

fn icon_font() -> egui::FontFamily {
    egui::FontFamily::Name(ICON_FONT.into())
}

/// Registers the icon font as its own family; run once, it takes effect from the next pass,
/// so nothing may use the family in the pass that installs it.
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

/// A glyph of the icon font followed by text, both in `text_style` and the widget's text color.
pub fn icon_text(
    style: &egui::Style,
    text_style: egui::TextStyle,
    glyph: char,
    text: &str,
) -> egui::WidgetText {
    let size = text_style.resolve(style).size;
    let mut job = egui::text::LayoutJob::default();
    let align = egui::Align::Center;
    egui::RichText::new(format!("{glyph} "))
        .font(egui::FontId::new(size, icon_font()))
        .append_to(&mut job, style, text_style.clone().into(), align);
    egui::RichText::new(text).append_to(&mut job, style, text_style.into(), align);
    job.into()
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
