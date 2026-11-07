//! The icon button of the Align panel, from the Brilliant `AlignBtn` component: a 32 × 28 px
//! button with a glyph of the icon font, drawn default, hover or disabled in the colors of the
//! current theme. Its name is shown as a tooltip.
//!
//! ```ignore
//! if ui.add(AlignButton::new(glyph::ALIGN_LEFT, language.label(Label::AlignLeft))).clicked() {
//!     // ...
//! }
//! ```
//!
//! Disable it the egui way, with `ui.add_enabled(false, AlignButton::new(..))`.

mod style;

use bevy_egui::egui;
use style::{AlignButtonState, paint};

use crate::ui_components::icons::paint_glyph;
use crate::ui_components::palette::Palette;

pub const SIZE: egui::Vec2 = egui::vec2(32.0, 28.0);
/// The box the glyph is painted in; the icon font fills three quarters of it, 15 px.
const ICON_BOX: f32 = 20.0;
const CORNER_RADIUS: u8 = 5;

/// An align button, added with `ui.add(...)`.
#[must_use = "add the align button with `ui.add(button)`"]
pub struct AlignButton {
    glyph: char,
    name: String,
}

impl AlignButton {
    /// A button showing `glyph`; `name` is its tooltip.
    pub fn new(glyph: char, name: impl Into<String>) -> Self {
        Self {
            glyph,
            name: name.into(),
        }
    }
}

impl egui::Widget for AlignButton {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self { glyph, name } = self;
        let (rect, response) = ui.allocate_exact_size(SIZE, egui::Sense::click());
        let enabled = ui.is_enabled();
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, &name));

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() || response.is_pointer_button_down_on();
            let painter = ui.painter();
            let colors = &Palette::of(painter).align_button;
            let style = paint(colors, AlignButtonState::new(enabled, hovered));
            painter.rect(
                rect,
                CORNER_RADIUS,
                style.fill,
                egui::Stroke::new(1.0, style.stroke),
                egui::StrokeKind::Inside,
            );
            if response.has_focus() {
                painter.rect_stroke(
                    rect.expand(2.0),
                    CORNER_RADIUS + 2,
                    egui::Stroke::new(1.0, colors.focus),
                    egui::StrokeKind::Inside,
                );
            }
            let icon = egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(ICON_BOX));
            paint_glyph(painter, icon, egui::Stroke::new(1.5, style.icon), glyph);
        }
        response.on_hover_text(name)
    }
}
