use crate::editor::EditorState;
use bevy::prelude::*;
use leathercraft::events::{ProjectChange, ProjectEvent};
use leathercraft::history::{History, HistoryCommand};
use leathercraft::model::Project;
use std::ops::Deref;

/// The library stays free of Bevy, so the project becomes a resource through this wrapper.
/// The private fields and read-only `Deref` leave `apply_project_events` as the only writer.
#[derive(Resource, Default)]
pub struct ActiveProject {
    project: Project,
    history: History,
}

impl ActiveProject {
    pub fn history(&self) -> &History {
        &self.history
    }
}

impl Deref for ActiveProject {
    type Target = Project;

    fn deref(&self) -> &Project {
        &self.project
    }
}

#[derive(Message, Clone, Debug)]
pub struct ProjectMessage(pub ProjectEvent);

#[derive(Message, Clone, Copy, Debug)]
pub struct HistoryMessage(pub HistoryCommand);

pub fn apply_project_events(
    mut messages: MessageReader<ProjectMessage>,
    mut history_messages: MessageReader<HistoryMessage>,
    mut active: ResMut<ActiveProject>,
    mut state: ResMut<EditorState>,
) {
    for ProjectMessage(event) in messages.read() {
        // Rejected events and saves leave the content as it was, so the preview must not rebuild.
        let ActiveProject { project, history } = active.bypass_change_detection();
        match project.apply(event.clone()) {
            Ok(change) => {
                history.record(event, change, project);
                if change != ProjectChange::Saved {
                    active.set_changed();
                }
                state.on_project_change(change, &active);
            }
            Err(error) => state.report_error(error),
        }
    }
    for HistoryMessage(command) in history_messages.read() {
        let ActiveProject { project, history } = active.bypass_change_detection();
        if let Some(status) = history.apply(*command, project) {
            active.set_changed();
            state.on_history_restore(status, &active);
        }
    }
    state.clear_released_drag();
}
