mod document;
mod editor;
mod preview;

use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use document::{ActiveProject, ProjectMessage};
use editor::{EditorState, ViewMode};
use preview::PreviewSettings;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Leathercraft Pattern Studio".to_owned(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_message::<ProjectMessage>()
        .init_resource::<ActiveProject>()
        .init_resource::<EditorState>()
        .init_resource::<ViewMode>()
        .init_resource::<PreviewSettings>()
        .add_systems(Startup, preview::setup_scene)
        .add_systems(
            EguiPrimaryContextPass,
            (editor::editor_ui, document::apply_project_events).chain(),
        )
        .add_systems(Update, (preview::rebuild_preview, preview::update_camera))
        .run();
}
