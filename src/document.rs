use crate::editor::EditorState;
use bevy::prelude::*;
use leathercraft::events::{ProjectChange, ProjectEvent};
use leathercraft::model::Project;

/// The library stays free of Bevy, so the project becomes a resource through this wrapper.
/// The private field and read-only `Deref` leave `apply_project_events` as the only writer.
#[derive(Resource, Default, Deref)]
pub struct ActiveProject(Project);

#[derive(Message, Clone, Debug)]
pub struct ProjectMessage(pub ProjectEvent);

pub fn apply_project_events(
    mut messages: MessageReader<ProjectMessage>,
    mut project: ResMut<ActiveProject>,
    mut state: ResMut<EditorState>,
) {
    for ProjectMessage(event) in messages.read() {
        // Rejected events and saves leave the content as it was, so the preview must not rebuild.
        match project.bypass_change_detection().0.apply(event.clone()) {
            Ok(change) => {
                if change != ProjectChange::Saved {
                    project.set_changed();
                }
                state.on_project_change(change, &project);
            }
            Err(error) => state.report_error(error),
        }
    }
    state.clear_released_drag();
}
