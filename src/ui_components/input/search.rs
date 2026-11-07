//! The search field: a text field with a search icon, and a clear button while it has text.

use bevy_egui::egui;

use super::{Field, ICON_STROKE, Metrics, escaped, paint_clear_icon, paint_search_icon, show_edit};
use crate::ui_components::palette::InputPaint;

const METRICS: Metrics = Metrics {
    height: 30.0,
    padding: 10.0,
    corner_radius: 6,
    gap: 8.0,
};
const ICON_SIZE: f32 = 13.0;

/// A single-line search field, added with `ui.add(...)`. Its response is `changed()` when the
/// text is typed or cleared.
#[must_use = "add the field with `ui.add(input)`"]
pub struct SearchInput<'a> {
    text: &'a mut String,
    hint: Option<egui::WidgetText>,
    width: Option<f32>,
    id_salt: Option<egui::Id>,
}

impl<'a> SearchInput<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint: None,
            width: None,
            id_salt: None,
        }
    }

    /// The text shown while the field is empty, such as "Search…".
    pub fn hint(mut self, hint: impl Into<egui::WidgetText>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// The width of the field; egui's `text_edit_width` by default.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Keeps the cursor and the focus when fields before this one come and go.
    // No search field sits in a changing list yet.
    #[allow(dead_code)]
    pub fn id_salt(mut self, id_salt: impl std::hash::Hash) -> Self {
        self.id_salt = Some(egui::Id::new(id_salt));
        self
    }
}

impl egui::Widget for SearchInput<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self {
            text,
            hint,
            width,
            id_salt,
        } = self;
        let field = Field::allocate(ui, id_salt, width, &METRICS);
        let state = field.state(ui, false);
        let style = field.paint_frame(ui, &METRICS, state);
        field.forward_focus(ui);

        let content = field.rect.shrink2(egui::vec2(METRICS.padding, 0.0));
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(content.left() + ICON_SIZE / 2.0, content.center().y),
            egui::Vec2::splat(ICON_SIZE),
        );
        let clear_rect = egui::Rect::from_center_size(
            egui::pos2(content.right() - ICON_SIZE / 2.0, content.center().y),
            egui::Vec2::splat(ICON_SIZE),
        );

        // The clear button sits on top of the field, so it is added after it but before the text.
        let mut cleared = false;
        let has_text = !text.is_empty();
        if has_text {
            let clear = ui
                .interact(clear_rect, field.id.with("clear"), egui::Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            let color = if clear.hovered() && ui.is_enabled() {
                style.text
            } else {
                style.placeholder
            };
            paint_clear_icon(
                ui.painter(),
                clear_rect,
                egui::Stroke::new(ICON_STROKE, color),
            );
            if clear.clicked() {
                text.clear();
                cleared = true;
                ui.memory_mut(|memory| memory.request_focus(field.edit_id()));
            }
        }

        let focused = ui.memory(|memory| memory.has_focus(field.edit_id()));
        paint_search_icon(
            ui.painter(),
            icon_rect,
            egui::Stroke::new(ICON_STROKE, icon_color(style, text.is_empty() && !focused)),
        );

        let right = if has_text {
            clear_rect.left() - METRICS.gap
        } else {
            content.right()
        };
        let text_rect =
            egui::Rect::from_x_y_ranges(icon_rect.right() + METRICS.gap..=right, content.y_range());
        let mut edit = egui::TextEdit::singleline(text)
            .id(field.edit_id())
            .text_color(style.text);
        if let Some(hint) = hint {
            edit = edit.hint_text(hint.color(style.placeholder));
        }
        let output = show_edit(ui, text_rect, edit);
        escaped(ui, &output.response);
        let mut response = output.response | field.response;
        if cleared {
            response.mark_changed();
        }
        response
    }
}

/// The icon of an empty field that isn't being typed in is as dim as its hint, as in the
/// design's empty state.
fn icon_color(style: &InputPaint, idle: bool) -> egui::Color32 {
    if idle { style.placeholder } else { style.icon }
}
