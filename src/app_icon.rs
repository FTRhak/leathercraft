//! The icon of the running window, shown on the taskbar and in Alt+Tab. Bevy has no setting for
//! it, so it is set on the winit window once that exists.
//!
//! On Windows it comes from the `APP_ICON` resource that `build.rs` embeds in the executable, so
//! the system picks the best of its sizes; elsewhere from the bundled logo PNG.

use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
use winit::window::Icon;

/// The logo at 256 px, edge to edge: the same picture as `assets/leathercraft.ico`.
pub const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

/// Sets the icon of the primary window, once, as soon as its winit window exists.
/// `NonSendMarker` keeps it on the main thread, which holds `WINIT_WINDOWS`.
pub fn set_window_icon(
    mut done: Local<bool>,
    primary: Query<Entity, With<PrimaryWindow>>,
    _main_thread: NonSendMarker,
) {
    if *done {
        return;
    }
    let Ok(entity) = primary.single() else {
        return;
    };
    WINIT_WINDOWS.with_borrow(|windows| {
        let Some(window) = windows.get_window(entity) else {
            return;
        };
        *done = true;
        apply(window);
    });
}

#[cfg(windows)]
fn apply(window: &winit::window::Window) {
    use winit::platform::windows::{IconExtWindows, WindowExtWindows};
    // `None` lets Windows load the size each place needs, from the sizes in the `.ico`.
    let load = || Icon::from_resource_name("APP_ICON", None);
    match (load(), load()) {
        (Ok(small), Ok(large)) => {
            window.set_window_icon(Some(small));
            window.set_taskbar_icon(Some(large));
        }
        (Err(error), _) | (_, Err(error)) => warn!("could not load the app icon: {error}"),
    }
}

#[cfg(not(windows))]
fn apply(window: &winit::window::Window) {
    let picture = match image::load_from_memory_with_format(LOGO_PNG, image::ImageFormat::Png) {
        Ok(picture) => picture.into_rgba8(),
        Err(error) => return warn!("could not decode the app icon: {error}"),
    };
    let (width, height) = picture.dimensions();
    match Icon::from_rgba(picture.into_raw(), width, height) {
        Ok(icon) => window.set_window_icon(Some(icon)),
        Err(error) => warn!("could not create the app icon: {error}"),
    }
}
