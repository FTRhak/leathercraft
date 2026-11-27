//! Camera navigation of the 3D preview: orbiting, panning and zooming with the mouse.

use super::constants::{
    DEFAULT_AZIMUTH, DEFAULT_ELEVATION, MAX_DISTANCE, MAX_ELEVATION, MIN_DISTANCE, MIN_ELEVATION,
};
use super::{PreviewCamera, PreviewSettings};
use bevy::prelude::*;
use bevy_egui::egui;

/// Radians the camera turns per point the mouse moves.
const ORBIT_SPEED: f32 = 0.008;
/// Matches the zoom speed of the 2D canvas.
const ZOOM_SPEED: f32 = 0.002;
/// Bevy's default vertical field of view, to pan the scene along with the cursor.
const FIELD_OF_VIEW: f32 = std::f32::consts::FRAC_PI_4;

impl PreviewSettings {
    /// Turns the camera back to where it starts, framing the whole scene.
    fn reset_view(&mut self) {
        self.azimuth = DEFAULT_AZIMUTH;
        self.elevation = DEFAULT_ELEVATION;
        self.distance = self.fit_distance;
        self.target = Vec3::ZERO;
    }

    /// The camera's position relative to its target.
    fn camera_offset(&self) -> Vec3 {
        let horizontal = self.distance * self.elevation.cos();
        Vec3::new(
            horizontal * self.azimuth.cos(),
            self.distance * self.elevation.sin(),
            horizontal * self.azimuth.sin(),
        )
    }
}

pub(super) fn update_camera(
    settings: Res<PreviewSettings>,
    mut cameras: Query<&mut Transform, With<PreviewCamera>>,
) {
    if !settings.is_changed() {
        return;
    }

    let position = settings.target + settings.camera_offset();
    for mut transform in &mut cameras {
        *transform = Transform::from_translation(position).looking_at(settings.target, Vec3::Y);
    }
}

/// Moves the camera with the mouse over the preview: the primary button orbits, the secondary
/// or middle one pans (as on the 2D canvas), the wheel zooms and a double click resets the view.
/// `settings` is only written on input, so the camera doesn't update every frame.
pub fn navigate(ui: &mut egui::Ui, settings: &mut ResMut<PreviewSettings>) {
    let rect = ui.max_rect();
    let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

    if response.double_clicked() {
        settings.reset_view();
        return;
    }
    let delta = response.drag_delta();
    if delta != egui::Vec2::ZERO {
        if response.dragged_by(egui::PointerButton::Primary) {
            let settings = &mut **settings;
            settings.azimuth =
                (settings.azimuth + delta.x * ORBIT_SPEED).rem_euclid(std::f32::consts::TAU);
            settings.elevation =
                (settings.elevation + delta.y * ORBIT_SPEED).clamp(MIN_ELEVATION, MAX_ELEVATION);
        } else if response.dragged_by(egui::PointerButton::Secondary)
            || response.dragged_by(egui::PointerButton::Middle)
        {
            // The camera renders to the whole window, so its scale follows the window's height.
            let window_height = ui.ctx().content_rect().height().max(1.0);
            let scene_per_point =
                2.0 * settings.distance * (FIELD_OF_VIEW * 0.5).tan() / window_height;
            let forward = -settings.camera_offset().normalize();
            let right = forward.cross(Vec3::Y).normalize();
            let up = right.cross(forward);
            let shift = (up * delta.y - right * delta.x) * scene_per_point;
            settings.target += shift;
        }
    }
    if response.hovered() {
        let scroll = ui.input(|input| input.smooth_scroll_delta.y);
        if scroll != 0.0 {
            settings.distance = (settings.distance * (-scroll * ZOOM_SPEED).exp())
                .clamp(MIN_DISTANCE, MAX_DISTANCE);
        }
    }
}
