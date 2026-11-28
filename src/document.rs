use crate::editor::EditorState;
use bevy::prelude::*;
use leathercraft::events::ProjectEvent;
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
    /// Starts with `project`, as the first entry of a new history.
    pub fn new(project: Project) -> Self {
        let history = History::new(&project, leathercraft::i18n::Message::ActionNewProject);
        Self { project, history }
    }

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
        // Rejected events and saves leave the content as it was, and background images aren't
        // part of what the preview builds, so neither must rebuild it.
        let ActiveProject { project, history } = active.bypass_change_detection();
        match project.apply(event.clone()) {
            Ok(change) => {
                history.record(event, change, project);
                if change.affects_pattern() {
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
