//! State for the 3D preview camera and view.

use super::constants::{DEFAULT_AZIMUTH, DEFAULT_DISTANCE, DEFAULT_ELEVATION};
use super::mode::PreviewMode;
use bevy::prelude::*;

#[derive(Resource)]
pub struct PreviewSettings {
    pub(super) azimuth: f32,
    pub(super) elevation: f32,
    pub(super) distance: f32,
    /// The point the camera orbits and looks at, moved by panning.
    pub(super) target: Vec3,
    /// The distance that frames the whole scene, which a reset of the view returns to.
    pub(super) fit_distance: f32,
    pub(super) mode: PreviewMode,
    /// Set by Sew again: start the simulation over.
    pub(super) resew: bool,
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self {
            azimuth: DEFAULT_AZIMUTH,
            elevation: DEFAULT_ELEVATION,
            distance: DEFAULT_DISTANCE,
            target: Vec3::ZERO,
            fit_distance: DEFAULT_DISTANCE,
            mode: PreviewMode::default(),
            resew: false,
        }
    }
}
