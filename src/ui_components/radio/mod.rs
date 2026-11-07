//! The radio button of the design system, from the Brilliant `Radio` component: a round box,
//! on (with a dot) or off, drawn in its default, hover, focus and disabled state, followed by an
//! optional label. It has the sizes of the checkbox and shares its `Palette::checkbox` colors.
//!
//! ```ignore
//! ui.add(Radio::new(theme == Theme::Dark, language.label(Label::ThemeDark)));
//! radio_value(ui, &mut theme, Theme::Dark, language.label(Label::ThemeDark));
//! ```
//!
//! Disable it the egui way, with `ui.add_enabled(false, Radio::new(..))`.

mod style;

use bevy_egui::egui;
use style::{RadioState, paint};

use crate::ui_components::palette::Palette;

/// The diameter of the circle.
const BOX_SIZE: f32 = 15.0;
/// The diameter of the dot of a selected radio button.
const DOT_SIZE: f32 = 6.0;
/// Between the circle and the label.
const LABEL_GAP: f32 = 7.0;
/// The width of the focus ring around the circle, and how strongly it shows.
const FOCUS_SPREAD: f32 = 2.0;
const FOCUS_OPACITY: f32 = 0.45;

/// A radio button showing whether its option is the chosen one, added with `ui.add(...)`. It
/// doesn't change anything itself: check `clicked()` on the response, or use `radio_value`.
#[must_use = "add the radio button with `ui.add(radio)`"]
pub struct Radio {
    selected: bool,
    text: egui::WidgetText,
}

impl Radio {
    pub fn new(selected: bool, text: impl Into<egui::WidgetText>) -> Self {
        Self {
            selected,
            text: text.into(),
        }
    }
}

impl egui::Widget for Radio {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self { selected, text } = self;
        let galley = text.into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Body,
        );
        let label_width = if galley.is_empty() {
            0.0
        } else {
            LABEL_GAP + galley.size().x
        };
        let size = egui::vec2(BOX_SIZE + label_width, BOX_SIZE.max(galley.size().y));
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::RadioButton,
                ui.is_enabled(),
                selected,
                galley.text(),
            )
        });

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() || response.is_pointer_button_down_on();
            let state = RadioState::new(ui.is_enabled(), hovered);
            let painter = ui.painter();
            let colors = &Palette::of(painter).checkbox;
            let style = paint(colors, selected, state);
            let focused = response.has_focus() && state != RadioState::Disabled;

            let center = egui::pos2(rect.left() + BOX_SIZE / 2.0, rect.center().y);
            let radius = BOX_SIZE / 2.0;
            if focused {
                painter.circle_filled(
                    center,
                    radius + FOCUS_SPREAD,
                    colors.focus.linear_multiply(FOCUS_OPACITY),
                );
            }
            // A focused button always has the focus color on its edge, even when off.
            let stroke = if focused { colors.focus } else { style.stroke };
            // The stroke sits inside the circle, as in the checkbox.
            painter.circle(
                center,
                radius - 0.5,
                style.fill,
                egui::Stroke::new(1.0, stroke),
            );
            if let Some(dot) = style.mark {
                painter.circle_filled(center, DOT_SIZE / 2.0, dot);
            }

            if !galley.is_empty() {
                let text_pos = egui::pos2(
                    rect.left() + BOX_SIZE + LABEL_GAP,
                    rect.center().y - galley.size().y / 2.0,
                );
                painter.galley(text_pos, galley, style.text);
            }
        }
        response
    }
}

/// The radio counterpart of `ui.radio_value`: a radio button for `alternative`, selected while
/// `current` equals it, that sets `current` to it on click and marks the response as changed.
pub fn radio_value<T: PartialEq>(
    ui: &mut egui::Ui,
    current: &mut T,
    alternative: T,
    text: impl Into<egui::WidgetText>,
) -> egui::Response {
    let mut response = ui.add(Radio::new(*current == alternative, text));
    if response.clicked() && *current != alternative {
        *current = alternative;
        response.mark_changed();
    }
    response
}
