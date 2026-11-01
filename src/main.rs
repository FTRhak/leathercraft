mod document;
mod editor;
mod panels;
mod preview;
mod settings;

use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use document::{ActiveProject, ProjectMessage};
use editor::{EditorState, ViewMode};
use preview::PreviewPlugin;

fn main() {
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
        .init_resource::<ActiveProject>()
        .insert_resource(EditorState::new(settings::load_language()))
        .init_resource::<ViewMode>()
        .add_systems(
            EguiPrimaryContextPass,
            (editor::editor_ui, document::apply_project_events).chain(),
        )
        .run();
}
