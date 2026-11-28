//! The 3D preview: the pieces either flat, each extruded on its own (`flat`), or sewn
//! together along their seam links (`sewn`, `docs/sewn-preview.md`).

mod constants;
mod controls;
mod flat;
mod mode;
mod navigation;
mod settings;
mod sewn;

pub use controls::show_controls;
pub use mode::PreviewMode;
pub use navigation::navigate;
use navigation::update_camera;
pub use settings::PreviewSettings;

use crate::document::ActiveProject;
use crate::editor::ViewMode;
use crate::settings::{Preferences, Theme};
use bevy::prelude::*;
use constants::{DEFAULT_DISTANCE, GROUND_OPACITY, MAX_DISTANCE, MIN_DISTANCE, MM_TO_SCENE};
use sewn::SewnModel;

pub struct PreviewPlugin;

impl Plugin for PreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PreviewSettings>()
            .init_resource::<SewnModel>()
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                (
                    (rebuild_preview, step_sewn).chain(),
                    update_camera,
                    apply_theme,
                ),
            );
    }
}

#[derive(Component, Clone)]
struct PreviewPiece;

#[derive(Component)]
struct PreviewCamera;

#[derive(Component)]
struct Ground;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.2, 2.8, 3.4).looking_at(Vec3::ZERO, Vec3::Y),
        PreviewCamera,
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(2.5, 5.0, 3.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(24.0, 24.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: ground_color(Theme::default()),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.02, 0.0),
        Ground,
    ));
}

/// The background behind the ground plane.
fn clear_color(theme: Theme) -> Color {
    match theme {
        Theme::Dark => Color::srgb_u8(43, 44, 47),
        Theme::Light => Color::srgb_u8(214, 217, 222),
    }
}

/// Nearly see-through, so the ground hints at the floor without hiding the model's underside.
fn ground_color(theme: Theme) -> Color {
    let color = match theme {
        Theme::Dark => Color::srgb(0.13, 0.15, 0.18),
        Theme::Light => Color::srgb(0.72, 0.74, 0.77),
    };
    color.with_alpha(GROUND_OPACITY)
}

/// Follows the interface theme, so the preview doesn't stay dark between light panels.
fn apply_theme(
    preferences: Res<Preferences>,
    mut clear: ResMut<ClearColor>,
    ground: Query<&MeshMaterial3d<StandardMaterial>, With<Ground>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut applied: Local<Option<Theme>>,
) {
    let theme = preferences.theme;
    if *applied == Some(theme) {
        return;
    }
    clear.0 = clear_color(theme);
    for handle in &ground {
        if let Some(material) = materials.get_mut(&handle.0) {
            material.base_color = ground_color(theme);
        }
    }
    *applied = Some(theme);
}

/// What the scene was last built from.
#[derive(Default)]
struct Built {
    /// The project changed since.
    stale: bool,
    mode: Option<PreviewMode>,
}

/// Rebuilds the scene when the project or the mode changed, once the preview is shown: the
/// sewn model is laid out anew each time, which is too slow to do on every edit in 2D.
#[allow(clippy::too_many_arguments)]
fn rebuild_preview(
    mut commands: Commands,
    project: Res<ActiveProject>,
    view_mode: Res<ViewMode>,
    existing: Query<Entity, With<PreviewPiece>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut settings: ResMut<PreviewSettings>,
    mut sewn: ResMut<SewnModel>,
    mut built: Local<Built>,
) {
    if project.is_changed() {
        built.stale = true;
    }
    let mode = settings.mode;
    let current = !built.stale && built.mode == Some(mode) && !settings.resew;
    if current || *view_mode != ViewMode::Preview3d {
        return;
    }
    settings.bypass_change_detection().resew = false;
    *built = Built {
        stale: false,
        mode: Some(mode),
    };

    for entity in &existing {
        commands.entity(entity).despawn();
    }
    sewn.clear();
    let size_mm = match mode {
        PreviewMode::Flat => spawn_flat(&project, &mut commands, &mut meshes, &mut materials),
        PreviewMode::Sewn => sewn.rebuild(
            &project,
            &mut commands,
            &mut meshes,
            &mut materials,
            PreviewPiece,
        ),
    };
    settings.fit_distance = size_mm.map_or(DEFAULT_DISTANCE, |size| {
        ((size * MM_TO_SCENE) as f32 * 2.7).clamp(MIN_DISTANCE, MAX_DISTANCE)
    });
    // The scene is built around the origin, so a pan away from it would lose the model.
    settings.distance = settings.fit_distance;
    settings.target = Vec3::ZERO;
}

/// Spawns each piece flat, centered on the origin; returns the project's size in mm.
fn spawn_flat(
    project: &ActiveProject,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<f64> {
    let bounds = project.bounds()?;
    // Meshes are built around the origin, which is where the camera orbits.
    let center_x = ((bounds.min_x + bounds.max_x) * 0.5 * MM_TO_SCENE) as f32;
    let center_z = ((bounds.min_y + bounds.max_y) * 0.5 * MM_TO_SCENE) as f32;
    for piece in &project.pieces {
        // A piece whose shapes don't combine into one outline is flagged in the editor instead.
        let Ok(region) = piece.punched_region() else {
            continue;
        };
        match flat::build_piece_mesh(&region, piece.material.thickness_mm, center_x, center_z) {
            Ok(mesh) => {
                let [red, green, blue] = piece.material.color.rgb();
                let leather = materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(red, green, blue),
                    perceptual_roughness: 0.88,
                    ..default()
                });
                commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(leather),
                    PreviewPiece,
                ));
            }
            Err(error) => {
                bevy::log::error!("Could not render piece '{}': {error}", piece.name);
            }
        }
    }
    Some(bounds.width().max(bounds.height()))
}

/// Sews the model a frame further while the preview shows it.
fn step_sewn(
    view_mode: Res<ViewMode>,
    settings: Res<PreviewSettings>,
    mut sewn: ResMut<SewnModel>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if *view_mode == ViewMode::Preview3d && settings.mode == PreviewMode::Sewn {
        sewn.step(&mut meshes);
    }
}
