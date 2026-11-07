//! The checkbox of the design system, from the Brilliant `Checkbox` component: a box checked,
//! unchecked or mixed, each drawn in its default, hover, focus and disabled state, in the colors
//! of the current theme, followed by an optional label.
//!
//! ```ignore
//! ui.add(Checkbox::new(&mut snap.grid, language.label(Label::SnapGrid)));
//! ```
//!
//! Disable it the egui way, with `ui.add_enabled(false, Checkbox::new(..))`, or with
//! `ui.add_enabled_ui(false, ..)` around it.

mod style;

use bevy_egui::egui;
use style::{CheckboxState, CheckboxValue, paint};

use crate::ui_components::palette::Palette;

const BOX_SIZE: f32 = 15.0;
const CORNER_RADIUS: u8 = 3;
/// Between the box and the label.
const LABEL_GAP: f32 = 7.0;
/// The width of the focus ring around the box, and how strongly it shows.
const FOCUS_SPREAD: f32 = 2.0;
const FOCUS_OPACITY: f32 = 0.45;
const MARK_WIDTH: f32 = 1.4;

/// A checkbox bound to a `bool`, added with `ui.add(...)`; a click toggles the value and marks
/// the response as changed.
#[must_use = "add the checkbox with `ui.add(checkbox)`"]
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    text: egui::WidgetText,
    mixed: bool,
}

impl<'a> Checkbox<'a> {
    pub fn new(checked: &'a mut bool, text: impl Into<egui::WidgetText>) -> Self {
        Self {
            checked,
            text: text.into(),
            mixed: false,
        }
    }

    /// Shows a dash instead of the value, for a setting that differs across the selection.
    /// A click then checks the box.
    // Unused until a panel edits several pieces at once.
    #[allow(dead_code)]
    pub fn mixed(mut self, mixed: bool) -> Self {
        self.mixed = mixed;
        self
    }
}

impl egui::Widget for Checkbox<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self {
            checked,
            text,
            mixed,
        } = self;
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
        let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click());

        let mut mixed = mixed;
        if response.clicked() {
            *checked = mixed || !*checked;
            mixed = false;
            response.mark_changed();
        }
        response.widget_info(|| {
            if mixed {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Checkbox,
                    ui.is_enabled(),
                    galley.text(),
                )
            } else {
                egui::WidgetInfo::selected(
                    egui::WidgetType::Checkbox,
                    ui.is_enabled(),
                    *checked,
                    galley.text(),
                )
            }
        });

        if ui.is_rect_visible(rect) {
            let value = CheckboxValue::new(*checked, mixed);
            let hovered = response.hovered() || response.is_pointer_button_down_on();
            let state = CheckboxState::new(ui.is_enabled(), hovered);
            let painter = ui.painter();
            let colors = &Palette::of(painter).checkbox;
            let style = paint(colors, value, state);
            let focused = response.has_focus() && state != CheckboxState::Disabled;

            let box_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left(), rect.center().y - BOX_SIZE / 2.0),
                egui::Vec2::splat(BOX_SIZE),
            );
            if focused {
                painter.rect_filled(
                    box_rect.expand(FOCUS_SPREAD),
                    CORNER_RADIUS + FOCUS_SPREAD as u8,
                    colors.focus.linear_multiply(FOCUS_OPACITY),
                );
            }
            // A focused box always has the focus color on its edge, even when unchecked.
            let stroke = if focused { colors.focus } else { style.stroke };
            painter.rect(
                box_rect,
                CORNER_RADIUS,
                style.fill,
                egui::Stroke::new(1.0, stroke),
                egui::StrokeKind::Inside,
            );
            if let Some(mark) = style.mark {
                paint_mark(
                    painter,
                    box_rect,
                    value,
                    egui::Stroke::new(MARK_WIDTH, mark),
                );
            }

            if !galley.is_empty() {
                let text_pos = egui::pos2(
                    box_rect.right() + LABEL_GAP,
                    rect.center().y - galley.size().y / 2.0,
                );
                painter.galley(text_pos, galley, style.text);
            }
        }
        response
    }
}

/// The check of a checked box or the dash of a mixed one, in the 10 px area the design leaves
/// inside the 15 px box.
fn paint_mark(
    painter: &egui::Painter,
    box_rect: egui::Rect,
    value: CheckboxValue,
    stroke: egui::Stroke,
) {
    let at = |x: f32, y: f32| box_rect.min + egui::vec2(x, y);
    match value {
        CheckboxValue::On => {
            painter.add(egui::Shape::line(
                vec![at(3.0, 7.6), at(5.7, 10.1), at(12.0, 4.4)],
                stroke,
            ));
        }
        CheckboxValue::Mixed => {
            let y = box_rect.center().y;
            painter.line_segment(
                [
                    egui::pos2(box_rect.left() + 3.0, y),
                    egui::pos2(box_rect.right() - 3.0, y),
                ],
                stroke,
            );
        }
        CheckboxValue::Off => {}
    }
}
