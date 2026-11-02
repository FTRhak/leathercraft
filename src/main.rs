mod document;
mod editor;
mod panels;
mod preview;
mod settings;

use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use document::{ActiveProject, HistoryMessage, ProjectMessage};
use editor::{EditorState, ViewMode};
use panels::dock::PanelDock;
use preview::PreviewPlugin;

fn main() {
    let language = settings::load_language();
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Leathercraft Pattern Studio".to_owned(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((EguiPlugin::default(), PreviewPlugin))
        .add_message::<ProjectMessage>()
        .add_message::<HistoryMessage>()
        .init_resource::<ActiveProject>()
        .insert_resource(EditorState::new(language))
        .insert_resource(PanelDock::load(language))
        .init_resource::<ViewMode>()
        .add_systems(
            EguiPrimaryContextPass,
            (
                panels::install_icon_font.run_if(run_once),
                (editor::editor_ui, document::apply_project_events).chain(),
            ),
        )
        .run();
}
