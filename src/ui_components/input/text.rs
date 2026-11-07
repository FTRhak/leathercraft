//! The plain text field.

use bevy_egui::egui;

use super::{Field, Metrics, escaped, show_edit};

/// The same frame as the search field, without its icon.
const METRICS: Metrics = Metrics {
    height: 30.0,
    padding: 10.0,
    corner_radius: 6,
    gap: 8.0,
};

/// A single-line text field, added with `ui.add(...)`.
#[must_use = "add the field with `ui.add(input)`"]
pub struct TextInput<'a> {
    text: &'a mut String,
    hint: Option<egui::WidgetText>,
    width: Option<f32>,
    id_salt: Option<egui::Id>,
}

impl<'a> TextInput<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint: None,
            width: None,
            id_salt: None,
        }
    }

    /// The text shown while the field is empty.
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
    pub fn id_salt(mut self, id_salt: impl std::hash::Hash) -> Self {
        self.id_salt = Some(egui::Id::new(id_salt));
        self
    }
}

impl egui::Widget for TextInput<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let field = Field::allocate(ui, self.id_salt, self.width, &METRICS);
        let state = field.state(ui, false);
        let style = field.paint_frame(ui, &METRICS, state);
        field.forward_focus(ui);

        let mut edit = egui::TextEdit::singleline(self.text)
            .id(field.edit_id())
            .text_color(style.text);
        if let Some(hint) = self.hint {
            edit = edit.hint_text(hint.color(style.placeholder));
        }
        let content = field.rect.shrink2(egui::vec2(METRICS.padding, 0.0));
        let output = show_edit(ui, content, edit);
        escaped(ui, &output.response);
        output.response | field.response
    }
}
