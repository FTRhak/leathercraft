//! The number field: a value in a monospaced font, an optional unit after it, and a stepper.

use std::ops::RangeInclusive;

use bevy_egui::egui::{self, Key, Modifiers, emath::Numeric};

use super::{Field, ICON_STROKE, Metrics, escaped, paint_stepper_icon, show_edit};

const METRICS: Metrics = Metrics {
    height: 26.0,
    padding: 8.0,
    corner_radius: 5,
    gap: 4.0,
};
const WIDTH: f32 = 96.0;
const STEPPER_SIZE: egui::Vec2 = egui::vec2(7.0, 12.0);
/// Steps taken by one press of an arrow key with Shift.
const SHIFT_STEPS: f64 = 10.0;

type GetSet<'a> = Box<dyn FnMut(Option<f64>) -> f64 + 'a>;

/// A number field, added with `ui.add(...)`. The value can be typed, committed with Enter or
/// when the field loses focus, or stepped with the stepper or the arrow keys (Shift for ten
/// steps). It is always kept within its range.
#[must_use = "add the field with `ui.add(input)`"]
pub struct NumberInput<'a> {
    get_set: GetSet<'a>,
    integral: bool,
    range: RangeInclusive<f64>,
    step: f64,
    decimals: usize,
    unit: Option<egui::WidgetText>,
    width: f32,
    accent: bool,
    id_salt: Option<egui::Id>,
}

impl<'a> NumberInput<'a> {
    pub fn new<N: Numeric>(value: &'a mut N) -> Self {
        Self {
            get_set: Box::new(move |set: Option<f64>| {
                if let Some(set) = set {
                    *value = N::from_f64(set);
                }
                value.to_f64()
            }),
            integral: N::INTEGRAL,
            range: N::MIN.to_f64()..=N::MAX.to_f64(),
            step: 1.0,
            decimals: if N::INTEGRAL { 0 } else { 2 },
            unit: None,
            width: WIDTH,
            accent: false,
            id_salt: None,
        }
    }

    /// The values the field accepts; typed or stepped values are clamped into it.
    pub fn range<N: Numeric>(mut self, range: RangeInclusive<N>) -> Self {
        self.range = range.start().to_f64()..=range.end().to_f64();
        self
    }

    /// How much the stepper and the arrow keys change the value; 1 by default.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// How many decimals the value is shown and kept with; 2 by default, 0 for integers.
    pub fn decimals(mut self, decimals: usize) -> Self {
        if !self.integral {
            self.decimals = decimals;
        }
        self
    }

    /// The unit shown after the value, such as "mm".
    pub fn unit(mut self, unit: impl Into<egui::WidgetText>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// The width of the field; 96 points by default.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Draws the value in the accent color, for a value that matches a canvas layer.
    pub fn accent(mut self, accent: bool) -> Self {
        self.accent = accent;
        self
    }

    /// Keeps the typed text and the focus when fields before this one come and go.
    pub fn id_salt(mut self, id_salt: impl std::hash::Hash) -> Self {
        self.id_salt = Some(egui::Id::new(id_salt));
        self
    }

    fn value(&mut self) -> f64 {
        (self.get_set)(None)
    }

    /// The value as it is shown while the field isn't being typed in.
    fn current_text(&mut self) -> String {
        let value = self.value();
        self.format(value)
    }

    fn format(&self, value: f64) -> String {
        format!("{value:.*}", self.decimals)
    }

    /// `value` clamped into the range and rounded to the shown decimals.
    fn clamp(&self, value: f64) -> f64 {
        let factor = 10f64.powi(self.decimals as i32);
        let rounded = (value * factor).round() / factor;
        rounded.clamp(*self.range.start(), *self.range.end())
    }

    /// Sets the value, if it differs; returns whether it did.
    fn set(&mut self, value: f64) -> bool {
        let value = self.clamp(value);
        let changed = value != self.value();
        if changed {
            (self.get_set)(Some(value));
        }
        changed
    }
}

/// A typed number; a comma is read as the decimal point.
fn parse(text: &str) -> Option<f64> {
    text.trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

/// The steps asked for with the arrow keys this frame, consumed so they don't move the cursor.
fn arrow_steps(ui: &egui::Ui) -> f64 {
    ui.input_mut(|input| {
        let mut steps = 0.0;
        for (key, sign) in [(Key::ArrowUp, 1.0), (Key::ArrowDown, -1.0)] {
            // Shift first: the plain shortcut also matches presses with Shift.
            steps += SHIFT_STEPS * sign * input.count_and_consume_key(Modifiers::SHIFT, key) as f64;
            steps += sign * input.count_and_consume_key(Modifiers::NONE, key) as f64;
        }
        steps
    })
}

impl egui::Widget for NumberInput<'_> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let field = Field::allocate(ui, self.id_salt, Some(self.width), &METRICS);
        let edit_id = field.edit_id();
        let state = field.state(ui, self.accent);
        let style = field.paint_frame(ui, &METRICS, state);
        field.forward_focus(ui);

        let mut changed = false;
        let was_focused = ui.memory(|memory| memory.has_focus(edit_id));
        let mut text = if was_focused {
            ui.data(|data| data.get_temp::<String>(edit_id))
        } else {
            None
        }
        .unwrap_or_else(|| self.current_text());

        if was_focused && ui.is_enabled() {
            let steps = arrow_steps(ui);
            if steps != 0.0 {
                let current = parse(&text).unwrap_or_else(|| self.value());
                changed |= self.set(current + steps * self.step);
                text = self.current_text();
            }
        }

        let content = field.rect.shrink2(egui::vec2(METRICS.padding, 0.0));
        let stepper_rect = egui::Rect::from_center_size(
            egui::pos2(content.right() - STEPPER_SIZE.x / 2.0, content.center().y),
            STEPPER_SIZE,
        );
        // The whole height beside the caret, so that either half is easy to hit.
        let stepper_hit = egui::Rect::from_x_y_ranges(
            stepper_rect.left() - METRICS.gap..=field.rect.right(),
            field.rect.y_range(),
        );
        let stepper = ui
            .interact(stepper_hit, field.id.with("stepper"), egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::Default);
        if stepper.clicked() {
            let up = stepper
                .interact_pointer_pos()
                .is_some_and(|pos| pos.y < stepper_hit.center().y);
            let current = self.value();
            let sign = if up { 1.0 } else { -1.0 };
            changed |= self.set(current + sign * self.step);
            text = self.current_text();
        }
        let stepper_color = if stepper.hovered() && ui.is_enabled() {
            style.text
        } else {
            style.icon
        };
        paint_stepper_icon(
            ui.painter(),
            stepper_rect,
            egui::Stroke::new(ICON_STROKE, stepper_color),
        );

        let unit = self.unit.take().map(|unit| {
            unit.into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                egui::TextStyle::Monospace,
            )
        });
        let unit_width = unit
            .as_ref()
            .map_or(0.0, |unit| unit.size().x + METRICS.gap);
        let text_rect = egui::Rect::from_x_y_ranges(
            content.left()..=(stepper_rect.left() - METRICS.gap - unit_width).max(content.left()),
            content.y_range(),
        );
        let edit = egui::TextEdit::singleline(&mut text)
            .id(edit_id)
            .font(egui::TextStyle::Monospace)
            .text_color(style.text);
        let output = show_edit(ui, text_rect, edit);
        let response = &output.response;

        if let Some(unit) = unit {
            let pos = egui::pos2(
                output.galley_pos.x + output.galley.size().x + METRICS.gap,
                content.center().y - unit.size().y / 2.0,
            );
            ui.painter().galley(pos, unit, style.unit);
        }

        if response.gained_focus() {
            // Typing replaces the value rather than adding to it.
            let mut edit_state = output.state.clone();
            edit_state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::select_all(&output.galley)));
            edit_state.store(ui.ctx(), edit_id);
        }
        if response.has_focus() {
            ui.data_mut(|data| data.insert_temp(edit_id, text.clone()));
        } else if response.lost_focus() {
            ui.data_mut(|data| data.remove::<String>(edit_id));
            if !escaped(ui, response)
                && let Some(value) = parse(&text)
            {
                changed |= self.set(value);
            }
        }

        let mut response = output.response | field.response | stepper;
        if changed {
            response.mark_changed();
        }
        response
    }
}
