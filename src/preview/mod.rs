//! The 3D preview: the pieces either flat, each extruded on its own (`flat`), or sewn
//! together along their seam links (`sewn`, `docs/sewn-preview.md`).

mod flat;
mod sewn;

use crate::document::ActiveProject;
use crate::editor::ViewMode;
use crate::settings::{Preferences, Theme};
use crate::ui_components::button::Button;
use crate::ui_components::group_button::GroupButton;
use bevy::prelude::*;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use sewn::SewnModel;

// Scene units per millimeter; the scene is scaled up, so this is not a meters conversion.
const MM_TO_SCENE: f64 = 0.01;
const MIN_DISTANCE: f32 = 1.0;
const MAX_DISTANCE: f32 = 60.0;
const DEFAULT_DISTANCE: f32 = 3.2;

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

/// What the preview shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PreviewMode {
    /// Each piece flat on the ground, with its holes cut through.
    Flat,
    /// The pieces sewn together along their seam links.
    #[default]
    Sewn,
}

#[derive(Resource)]
pub struct PreviewSettings {
    azimuth: f32,
    elevation: f32,
    distance: f32,
    mode: PreviewMode,
    /// Set by Sew again: start the simulation over.
    resew: bool,
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self {
            azimuth: 0.72,
            elevation: 0.78,
            distance: DEFAULT_DISTANCE,
            mode: PreviewMode::default(),
            resew: false,
        }
    }
}

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
        MeshMaterial3d(materials.add(ground_color(Theme::default()))),
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

fn ground_color(theme: Theme) -> Color {
    match theme {
        Theme::Dark => Color::srgb(0.13, 0.15, 0.18),
        Theme::Light => Color::srgb(0.72, 0.74, 0.77),
    }
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
    settings.distance = size_mm.map_or(DEFAULT_DISTANCE, |size| {
        ((size * MM_TO_SCENE) as f32 * 2.7).clamp(MIN_DISTANCE, MAX_DISTANCE)
    });
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

fn update_camera(
    settings: Res<PreviewSettings>,
    mut cameras: Query<&mut Transform, With<PreviewCamera>>,
) {
    if !settings.is_changed() {
        return;
    }

    let horizontal = settings.distance * settings.elevation.cos();
    let offset = Vec3::new(
        horizontal * settings.azimuth.cos(),
        settings.distance * settings.elevation.sin(),
        horizontal * settings.azimuth.sin(),
    );
    for mut transform in &mut cameras {
        *transform = Transform::from_translation(offset).looking_at(Vec3::ZERO, Vec3::Y);
    }
}

pub fn show_controls(ui: &mut egui::Ui, settings: &mut PreviewSettings, language: Language) {
    ui.label(language.label(Label::PreviewShows));
    ui.horizontal(|ui| {
        ui.add(
            GroupButton::new(&mut settings.mode)
                .option(PreviewMode::Sewn, language.label(Label::PreviewSewn))
                .option(PreviewMode::Flat, language.label(Label::PreviewFlat)),
        );
        if settings.mode == PreviewMode::Sewn
            && ui
                .add(Button::outline(language.label(Label::SewAgain)))
                .on_hover_text(language.label(Label::SewAgainHint))
                .clicked()
        {
            settings.resew = true;
        }
    });
    ui.label(language.label(Label::Camera));
    ui.add(
        egui::Slider::new(&mut settings.azimuth, 0.0..=std::f32::consts::TAU)
            .text(language.label(Label::Orbit)),
    );
    ui.add(
        egui::Slider::new(&mut settings.elevation, 0.12..=1.48).text(language.label(Label::Tilt)),
    );
    ui.add(
        egui::Slider::new(&mut settings.distance, MIN_DISTANCE..=MAX_DISTANCE)
            .text(language.label(Label::Zoom)),
    );
}
