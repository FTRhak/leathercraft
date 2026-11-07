mod document;
mod editor;
mod panels;
mod preview;
mod settings;
mod ui_components;

use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use document::{ActiveProject, HistoryMessage, ProjectMessage};
use editor::{EditorState, ViewMode};
use leathercraft::model::Project;
use preview::PreviewPlugin;
use ui_components::dock::PanelDock;

fn main() {
    let language = settings::load_language();
    let preferences = settings::load_preferences();
    let project = Project {
        default_material: preferences.default_material,
        ..default()
    };
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
        .insert_resource(ActiveProject::new(project))
        .insert_resource(preferences)
        .insert_resource(EditorState::new(language, settings::load_snap()))
        .insert_resource(PanelDock::load(language))
        .init_resource::<ViewMode>()
        .add_systems(
            EguiPrimaryContextPass,
            (
                ui_components::icons::install_icon_font.run_if(run_once),
                settings::apply_theme,
                // The icon font is bound only from the next pass, so the UI starts one pass later.
                (editor::editor_ui, document::apply_project_events)
                    .chain()
                    .run_if(not(run_once)),
            ),
        )
        .run();
}
