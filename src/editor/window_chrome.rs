//! The window is created without the system title bar and frame, so the app's own title bar
//! moves, resizes, minimizes, maximizes and closes it.

use super::confirm_discard::confirm_discard;
use crate::ui_components::title_bar::WindowCommand;
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
use leathercraft::i18n::Language;
use leathercraft::model::Project;

/// Whether the main window is maximized, which Bevy's `Window` doesn't report.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowMaximized(pub bool);

/// Reads the maximized state from winit, which also changes when the system maximizes the
/// window (Win+Up, snapping to the top of the screen). Runs on the main thread, where winit
/// keeps its windows.
pub fn track_maximized(
    _main_thread: NonSendMarker,
    window: Single<Entity, With<PrimaryWindow>>,
    mut maximized: ResMut<WindowMaximized>,
) {
    let now = WINIT_WINDOWS.with_borrow(|windows| {
        windows
            .get_window(*window)
            .is_some_and(|window| window.is_maximized())
    });
    maximized.set_if_neq(WindowMaximized(now));
}

/// Applies what the title bar asked of the window. Closing asks first if the project has
/// unsaved changes.
pub(super) fn apply(
    command: WindowCommand,
    window: &mut Window,
    maximized: bool,
    project: &Project,
    language: Language,
    exit: &mut MessageWriter<AppExit>,
) {
    match command {
        WindowCommand::Minimize => window.set_minimized(true),
        WindowCommand::ToggleMaximize => window.set_maximized(!maximized),
        WindowCommand::Close => {
            if confirm_discard(project, language) {
                exit.write(AppExit::Success);
            }
        }
        WindowCommand::StartMove => window.start_drag_move(),
        WindowCommand::StartResize(direction) => window.start_drag_resize(direction),
    }
}
