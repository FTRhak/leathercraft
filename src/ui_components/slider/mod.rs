//! The slider of the design system, from the Opacity row of the "Background Images Panel"
//! boards of the Brilliant design: a field as tall as a number field, filled up to its value,
//! with the value in the middle. A click or drag anywhere sets the value from the pointer; with
//! Ctrl the drag moves it ten times slower, for fine steps.
//!
//! ```ignore
//! let response = ui.add(Slider::new(&mut image.opacity, 0.0..=1.0).percent());
//! if response.changed() { … }
//! ```
//!
//! Disable it the egui way, with `ui.add_enabled(false, Slider::new(..))`.

use std::ops::RangeInclusive;

use bevy_egui::egui;

use crate::ui_components::palette::Palette;

const HEIGHT: f32 = 26.0;
const CORNER_RADIUS: u8 = 5;
const STROKE: f32 = 1.0;
/// With Ctrl held, a drag moves the value this many times slower.
const FINE: f32 = 0.1;

/// A slider bound to an `f32`, added with `ui.add(...)`; the response is marked changed when
/// the value changes.
#[must_use = "add the slider with `ui.add(slider)`"]
pub struct Slider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    percent: bool,
    decimals: usize,
    width: Option<f32>,
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            percent: false,
            decimals: 2,
            width: None,
        }
    }

    /// Shows the value as a whole percentage of the range, as for an opacity.
    pub fn percent(mut self) -> Self {
        self.percent = true;
        self
    }

    /// The width of the slider; the available width by default.
    #[allow(dead_code)]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    fn fraction(&self, value: f32) -> f32 {
        let (min, max) = (*self.range.start(), *self.range.end());
        if max > min {
            ((value - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn text(&self, value: f32) -> String {
        if self.percent {
            format!("{:.0} %", self.fraction(value) * 100.0)
        } else {
            format!("{value:.*}", self.decimals)
        }
    }
}

impl egui::Widget for Slider<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let (rect, mut response) =
            ui.allocate_exact_size(egui::vec2(width, HEIGHT), egui::Sense::click_and_drag());
        let (min, max) = (*self.range.start(), *self.range.end());

        if ui.is_enabled() {
            let fine = ui.input(|input| input.modifiers.ctrl || input.modifiers.command);
            let mut value = *self.value;
            if fine && response.dragged() {
                let delta = response.drag_delta().x / rect.width() * (max - min) * FINE;
                value += delta;
            } else if (response.clicked() || response.dragged())
                && let Some(pointer) = response.interact_pointer_pos()
            {
                let fraction = ((pointer.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                value = min + fraction * (max - min);
            }
            let value = value.clamp(min, max);
            if value != *self.value {
                *self.value = value;
                response.mark_changed();
            }
        }

        if ui.is_rect_visible(rect) {
            let palette = Palette::of(ui.painter());
            let field = if !ui.is_enabled() {
                &palette.input.disabled
            } else if response.hovered() || response.dragged() {
                &palette.input.hover
            } else {
                &palette.input.default
            };
            let painter = ui.painter();
            painter.rect_filled(rect, CORNER_RADIUS, field.fill);
            let fill_width = rect.width() * self.fraction(*self.value);
            if fill_width > 0.0 {
                let filled = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, HEIGHT));
                let colors = &palette.background;
                let (fill, edge) = if ui.is_enabled() {
                    (colors.slider_fill, colors.slider_edge)
                } else {
                    (field.fill, field.stroke)
                };
                let radius = egui::CornerRadius {
                    nw: CORNER_RADIUS,
                    sw: CORNER_RADIUS,
                    ne: 0,
                    se: 0,
                };
                painter.rect_filled(filled, radius, fill);
                if fill_width < rect.width() - 1.0 {
                    painter.line_segment(
                        [filled.right_top(), filled.right_bottom()],
                        egui::Stroke::new(STROKE, edge),
                    );
                }
            }
            painter.rect_stroke(
                rect,
                CORNER_RADIUS,
                egui::Stroke::new(STROKE, field.stroke),
                egui::StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.text(*self.value),
                egui::TextStyle::Monospace.resolve(ui.style()),
                field.text,
            );
        }
        let text = self.text(*self.value);
        response.widget_info(|| {
            egui::WidgetInfo::slider(ui.is_enabled(), f64::from(*self.value), text.clone())
        });
        response
    }
}
