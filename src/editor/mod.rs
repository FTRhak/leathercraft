mod canvas;
mod confirm_discard;
mod drag;
mod drag_state_struct;
mod editor_state_struct;
mod grid;
mod panel_viewer;
mod shape_draft_struct;
mod ui;

pub use editor_state_struct::EditorState;
pub use ui::editor_ui;

use crate::document::ProjectMessage;
use bevy::prelude::*;
use canvas::canvas;
use drag_state_struct::{DragState, DragTarget};
use leathercraft::events::ProjectEvent;
use leathercraft::exports::ExportError;
use leathercraft::i18n::{Language, Message};
use leathercraft::model::{Project, Units};
use leathercraft::persistence;
use shape_draft_struct::ShapeDraft;

const PROJECT_EXTENSION: &str = "leathercraft";

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Editor2d,
    Preview3d,
}

/// A part of a shape picked with the Edge tool: an edge of a polygon, or a vertex of any shape
/// with corners, by index into `Shape::corners`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapePart {
    Edge(usize),
    Vertex(usize),
}

fn send(events: &mut MessageWriter<ProjectMessage>, event: ProjectEvent) {
    events.write(ProjectMessage(event));
}

fn export(
    project: &Project,
    state: &mut EditorState,
    label: &str,
    extension: &str,
    encode: fn(&Project) -> std::result::Result<String, ExportError>,
) {
    let contents = match encode(project) {
        Ok(contents) => contents,
        Err(error) => {
            state.report_error(error.into());
            return;
        }
    };
    let Some(mut path) = rfd::FileDialog::new()
        .add_filter(label, &[extension])
        .save_file()
    else {
        return;
    };
    if path.extension().is_none() {
        path.set_extension(extension);
    }
    match persistence::write_export(&path, &contents) {
        Ok(()) => state.report(Message::Exported {
            path: path.display().to_string(),
        }),
        Err(error) => state.report_error(Message::CouldNotWrite {
            path: path.display().to_string(),
            detail: error.to_string(),
        }),
    }
}

pub(crate) fn format_length(language: Language, units: Units, millimeters: f64) -> String {
    format!(
        "{:.2} {}",
        units.from_millimeters(millimeters),
        language.unit_symbol(units)
    )
}
