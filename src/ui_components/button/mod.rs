//! The text button of the design system: three kinds, each drawn in its default, hover and
//! disabled state, in the colors of the current theme.
//!
//! Disable it the egui way, with `ui.add_enabled(false, Button::primary(..))`.

mod style;

pub use style::ButtonKind;

use bevy_egui::egui;
use style::{ButtonState, paint};

use crate::ui_components::palette::Palette;

const PADDING: egui::Vec2 = egui::vec2(16.0, 6.0);
const CORNER_RADIUS: u8 = 5;
const MIN_WIDTH: f32 = 64.0;

/// A button with a text label, added with `ui.add(...)`.
#[must_use = "add the button with `ui.add(button)`"]
pub struct Button {
    text: egui::WidgetText,
    kind: ButtonKind,
}

impl Button {
    pub fn new(kind: ButtonKind, text: impl Into<egui::WidgetText>) -> Self {
        Self {
            text: text.into(),
            kind,
        }
    }

    /// The main action of a dialog, such as OK.
    pub fn primary(text: impl Into<egui::WidgetText>) -> Self {
        Self::new(ButtonKind::Primary, text)
    }

    /// The action beside the main one, such as Cancel.
    pub fn secondary(text: impl Into<egui::WidgetText>) -> Self {
        Self::new(ButtonKind::Secondary, text)
    }

    /// A helper action, such as Reset to defaults.
    pub fn outline(text: impl Into<egui::WidgetText>) -> Self {
        Self::new(ButtonKind::Outline, text)
    }
}

impl egui::Widget for Button {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let galley = self.text.into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Button,
        );
        let size = egui::vec2(
            (galley.size().x + 2.0 * PADDING.x).max(MIN_WIDTH),
            galley.size().y + 2.0 * PADDING.y,
        );
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), galley.text())
        });

        if ui.is_rect_visible(rect) {
            let state = if !ui.is_enabled() {
                ButtonState::Disabled
            } else if response.hovered() || response.is_pointer_button_down_on() {
                ButtonState::Hover
            } else {
                ButtonState::Default
            };
            let painter = ui.painter();
            let colors = &Palette::of(painter).button;
            let style = paint(colors, self.kind, state);
            painter.rect(
                rect,
                CORNER_RADIUS,
                style.fill.unwrap_or(egui::Color32::TRANSPARENT),
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
            let text_pos = rect.center() - galley.size() / 2.0;
            painter.galley(text_pos, galley, style.text);
        }
        response
    }
}
