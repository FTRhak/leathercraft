//! The input fields of the design system, from the Brilliant `Number Input` and `Search Field`
//! components: a plain [`TextInput`], a [`NumberInput`] with a unit and a stepper, and a
//! [`SearchInput`] with a search icon and a clear button. Each is drawn in its default, hover,
//! focus and disabled state; a number can also be drawn in the accent state, for a value that
//! matches a canvas layer.
//!
//! ```ignore
//! ui.add(TextInput::new(&mut name).hint(language.label(Label::PieceName)));
//! ui.add(NumberInput::new(&mut thickness_mm).range(0.4..=8.0).step(0.1).decimals(1).unit("mm"));
//! ui.add(SearchInput::new(&mut filter).hint(language.label(Label::SearchHotkeys)));
//! ```
//!
//! Each is an `egui::Widget`; check `changed()` on the response. Disable one the egui way, with
//! `ui.add_enabled(false, ..)`. Esc leaves a focused field without closing the dialog around it;
//! a number being typed is then dropped, and is otherwise committed on Enter or when the field
//! loses focus.

mod number;
mod search;
mod style;
mod text;

pub use number::NumberInput;
pub use search::SearchInput;
pub use text::TextInput;

use bevy_egui::egui;
use style::{InputState, paint};

use crate::ui_components::palette::{InputPaint, Palette};

const STROKE: f32 = 1.0;
/// The stroke of the painted icons: search, clear and stepper.
const ICON_STROKE: f32 = 1.2;

/// The size and spacing of one kind of field.
struct Metrics {
    height: f32,
    padding: f32,
    corner_radius: u8,
    /// Between the icon, the text and the trailing control.
    gap: f32,
}

/// The frame of a field: allocated, with its click response, before its text is laid out.
struct Field {
    id: egui::Id,
    rect: egui::Rect,
    response: egui::Response,
}

impl Field {
    /// Allocates a field of `width`, or egui's `text_edit_width`, at the height of `metrics`.
    fn allocate(
        ui: &mut egui::Ui,
        id_salt: Option<egui::Id>,
        width: Option<f32>,
        metrics: &Metrics,
    ) -> Self {
        let width = width.unwrap_or_else(|| ui.spacing().text_edit_width);
        let (auto_id, rect) = ui.allocate_space(egui::vec2(width, metrics.height));
        let id = id_salt.map_or(auto_id, |salt| ui.make_persistent_id(salt));
        let response = ui
            .interact(rect, id, egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::Text);
        Self { id, rect, response }
    }

    /// The id of the text edit inside the field.
    fn edit_id(&self) -> egui::Id {
        self.id.with("edit")
    }

    /// The state to draw in, from the focus of the text edit, before it is shown.
    fn state(&self, ui: &egui::Ui, accent: bool) -> InputState {
        let focused = ui.memory(|memory| memory.has_focus(self.edit_id()));
        let hovered = ui.rect_contains_pointer(self.rect);
        InputState::new(ui.is_enabled(), focused, accent, hovered)
    }

    /// Paints the background and border; returns the paint for the content.
    fn paint_frame(
        &self,
        ui: &egui::Ui,
        metrics: &Metrics,
        state: InputState,
    ) -> &'static InputPaint {
        let style = paint(&Palette::of(ui.painter()).input, state);
        if ui.is_rect_visible(self.rect) {
            ui.painter().rect(
                self.rect,
                metrics.corner_radius,
                style.fill,
                egui::Stroke::new(STROKE, style.stroke),
                egui::StrokeKind::Inside,
            );
        }
        style
    }

    /// A click on the padding, the unit or the icon puts the cursor in the text.
    fn forward_focus(&self, ui: &egui::Ui) {
        if self.response.clicked() {
            ui.memory_mut(|memory| memory.request_focus(self.edit_id()));
        }
    }
}

/// Shows `edit` without a frame, vertically centered in `rect`.
fn show_edit(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    edit: egui::TextEdit,
) -> egui::text_edit::TextEditOutput {
    let edit = edit
        .frame(false)
        .margin(egui::Margin::ZERO)
        .desired_width(rect.width());
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| edit.show(ui),
    )
    .inner
}

/// Whether the field was left with Esc; the key is then consumed, so that a dialog around the
/// field stays open.
fn escaped(ui: &egui::Ui, response: &egui::Response) -> bool {
    response.lost_focus()
        && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
}

/// A magnifying glass, as wide and tall as `rect`.
fn paint_search_icon(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let radius = rect.width() * 0.36;
    let center = rect.min + egui::vec2(radius, radius) + egui::Vec2::splat(stroke.width / 2.0);
    painter.circle_stroke(center, radius, stroke);
    let handle = std::f32::consts::FRAC_1_SQRT_2 * radius;
    painter.line_segment(
        [
            center + egui::Vec2::splat(handle),
            rect.max - egui::Vec2::splat(stroke.width / 2.0),
        ],
        stroke,
    );
}

/// A cross in a circle, as wide and tall as `rect`.
fn paint_clear_icon(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let center = rect.center();
    let radius = rect.width() / 2.0 - stroke.width / 2.0;
    painter.circle_stroke(center, radius, stroke);
    let arm = radius * 0.4;
    painter.line_segment(
        [center - egui::vec2(arm, arm), center + egui::vec2(arm, arm)],
        stroke,
    );
    painter.line_segment(
        [
            center + egui::vec2(-arm, arm),
            center + egui::vec2(arm, -arm),
        ],
        stroke,
    );
}

/// A caret pointing up above one pointing down, as wide and tall as `rect`.
fn paint_stepper_icon(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let rect = rect.shrink(stroke.width / 2.0);
    let caret = rect.height() * 0.3;
    let up = [
        egui::pos2(rect.left(), rect.top() + caret),
        rect.center_top(),
        egui::pos2(rect.right(), rect.top() + caret),
    ];
    let down = [
        egui::pos2(rect.left(), rect.bottom() - caret),
        rect.center_bottom(),
        egui::pos2(rect.right(), rect.bottom() - caret),
    ];
    painter.add(egui::Shape::line(up.to_vec(), stroke));
    painter.add(egui::Shape::line(down.to_vec(), stroke));
}
