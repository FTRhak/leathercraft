//! Scale, camera limits and defaults of the 3D preview.

/// Scene units per millimeter; the scene is scaled up, so this is not a meters conversion.
pub(super) const MM_TO_SCENE: f64 = 0.01;
pub(super) const MIN_DISTANCE: f32 = 1.0;
pub(super) const MAX_DISTANCE: f32 = 60.0;
pub(super) const DEFAULT_DISTANCE: f32 = 3.2;
pub(super) const DEFAULT_AZIMUTH: f32 = 0.72;
pub(super) const DEFAULT_ELEVATION: f32 = 0.78;
pub(super) const MIN_ELEVATION: f32 = 0.12;
pub(super) const MAX_ELEVATION: f32 = 1.48;
/// The ground plane's opacity.
pub(super) const GROUND_OPACITY: f32 = 0.05;
