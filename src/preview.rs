use crate::document::ActiveProject;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;
use bevy_egui::egui;
use earcutr::earcut;
use leathercraft::model::PatternPiece;

// Scene units per millimeter; the scene is scaled up, so this is not a meters conversion.
const MM_TO_SCENE: f64 = 0.01;
const PIECE_THICKNESS: f32 = 0.035;
const MIN_DISTANCE: f32 = 1.0;
const MAX_DISTANCE: f32 = 60.0;

#[derive(Component)]
pub struct PreviewCamera;

#[derive(Component)]
pub struct PreviewPiece;

#[derive(Resource)]
pub struct PreviewSettings {
    pub azimuth: f32,
    pub elevation: f32,
    pub distance: f32,
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self {
            azimuth: 0.72,
            elevation: 0.78,
            distance: 3.2,
        }
    }
}

pub fn setup_scene(
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
        MeshMaterial3d(materials.add(Color::srgb(0.13, 0.15, 0.18))),
        Transform::from_xyz(0.0, -0.02, 0.0),
    ));
}

pub fn rebuild_preview(
    mut commands: Commands,
    project: Res<ActiveProject>,
    existing: Query<Entity, With<PreviewPiece>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut settings: ResMut<PreviewSettings>,
) {
    if !project.is_changed() {
        return;
    }

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Some(bounds) = project.bounds() else {
        settings.distance = 3.2;
        return;
    };
    // Meshes are built around the origin, which is where the camera orbits.
    let center_x = ((bounds.min_x + bounds.max_x) * 0.5 * MM_TO_SCENE) as f32;
    let center_z = ((bounds.min_y + bounds.max_y) * 0.5 * MM_TO_SCENE) as f32;
    settings.distance = ((bounds.width().max(bounds.height()) * MM_TO_SCENE) as f32 * 2.7)
        .clamp(MIN_DISTANCE, MAX_DISTANCE);

    let leather = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.23, 0.105),
        perceptual_roughness: 0.88,
        ..default()
    });

    for piece in &project.pieces {
        match build_piece_mesh(piece, center_x, center_z) {
            Ok(mesh) => {
                commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(leather.clone()),
                    PreviewPiece,
                ));
            }
            Err(error) => {
                bevy::log::error!("Could not render piece '{}': {error}", piece.name);
            }
        }
    }
}

pub fn update_camera(
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

pub fn show_controls(ui: &mut egui::Ui, settings: &mut PreviewSettings) {
    ui.label("Camera");
    ui.add(egui::Slider::new(&mut settings.azimuth, 0.0..=std::f32::consts::TAU).text("Orbit"));
    ui.add(egui::Slider::new(&mut settings.elevation, 0.12..=1.48).text("Tilt"));
    ui.add(egui::Slider::new(&mut settings.distance, MIN_DISTANCE..=MAX_DISTANCE).text("Zoom"));
}

fn build_piece_mesh(piece: &PatternPiece, center_x: f32, center_z: f32) -> Result<Mesh, String> {
    let points = &piece.points;
    let mut flat_points = Vec::with_capacity(points.len() * 2);
    for point in points {
        flat_points.push(point.x * MM_TO_SCENE - f64::from(center_x));
        flat_points.push(point.y * MM_TO_SCENE - f64::from(center_z));
    }
    let triangles =
        earcut(&flat_points, &[], 2).map_err(|error| format!("triangulation failed: {error}"))?;

    let mut positions = Vec::<[f32; 3]>::with_capacity(points.len() * 2 + points.len() * 4);
    let mut normals = Vec::<[f32; 3]>::with_capacity(points.len() * 2 + points.len() * 4);
    let mut indices = Vec::<u32>::new();
    for point in points {
        let x = (point.x * MM_TO_SCENE - f64::from(center_x)) as f32;
        let z = (point.y * MM_TO_SCENE - f64::from(center_z)) as f32;
        positions.push([x, 0.0, z]);
        normals.push([0.0, -1.0, 0.0]);
    }
    for point in points {
        let x = (point.x * MM_TO_SCENE - f64::from(center_x)) as f32;
        let z = (point.y * MM_TO_SCENE - f64::from(center_z)) as f32;
        positions.push([x, PIECE_THICKNESS, z]);
        normals.push([0.0, 1.0, 0.0]);
    }

    for triangle in triangles.chunks_exact(3) {
        let a = triangle[0] as u32;
        let b = triangle[1] as u32;
        let c = triangle[2] as u32;
        let pa = flat_points[triangle[0] * 2..triangle[0] * 2 + 2].to_vec();
        let pb = flat_points[triangle[1] * 2..triangle[1] * 2 + 2].to_vec();
        let pc = flat_points[triangle[2] * 2..triangle[2] * 2 + 2].to_vec();
        let normal_y = (pb[1] - pa[1]) * (pc[0] - pa[0]) - (pb[0] - pa[0]) * (pc[1] - pa[1]);
        if normal_y >= 0.0 {
            indices.extend_from_slice(&[a, c, b]);
            indices.extend_from_slice(&[
                a + points.len() as u32,
                b + points.len() as u32,
                c + points.len() as u32,
            ]);
        } else {
            indices.extend_from_slice(&[a, b, c]);
            indices.extend_from_slice(&[
                a + points.len() as u32,
                c + points.len() as u32,
                b + points.len() as u32,
            ]);
        }
    }

    // Pieces can be wound either way; walls must still face away from the interior.
    let outward: f32 = if piece.signed_area() >= 0.0 {
        1.0
    } else {
        -1.0
    };
    for index in 0..points.len() {
        let next = (index + 1) % points.len();
        let a = points[index];
        let b = points[next];
        let dx = ((b.x - a.x) * MM_TO_SCENE) as f32;
        let dz = ((b.y - a.y) * MM_TO_SCENE) as f32;
        let length = dx.hypot(dz).max(f32::EPSILON);
        let normal = [outward * dz / length, 0.0, -outward * dx / length];
        let base = positions.len() as u32;
        let ax = (a.x * MM_TO_SCENE - f64::from(center_x)) as f32;
        let az = (a.y * MM_TO_SCENE - f64::from(center_z)) as f32;
        let bx = (b.x * MM_TO_SCENE - f64::from(center_x)) as f32;
        let bz = (b.y * MM_TO_SCENE - f64::from(center_z)) as f32;
        positions.extend_from_slice(&[
            [ax, 0.0, az],
            [ax, PIECE_THICKNESS, az],
            [bx, PIECE_THICKNESS, bz],
            [bx, 0.0, bz],
        ]);
        normals.extend_from_slice(&[normal; 4]);
        if outward > 0.0 {
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        } else {
            indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    Ok(mesh)
}
