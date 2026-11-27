//! The sewn preview: the pieces of `leathercraft::drape::Drape`, rebuilt from the simulation
//! each frame until it settles, with the thread drawn along the seams.

use super::MM_TO_SCENE;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;
use leathercraft::drape::{Drape, DrapedPiece, Vec3 as DrapeVec};

/// The waxed thread: light, so that it shows on dark and light leather.
const THREAD_COLOR: Color = Color::srgb(0.93, 0.89, 0.78);

/// The simulation behind the sewn preview, and the meshes it updates.
#[derive(Resource, Default)]
pub(super) struct SewnModel {
    drape: Option<Drape>,
    /// The model point shown at the scene's origin, where the camera orbits.
    center: DrapeVec,
    pieces: Vec<Handle<Mesh>>,
    thread: Option<Handle<Mesh>>,
}

fn to_scene(point: DrapeVec, center: DrapeVec) -> [f32; 3] {
    [
        ((point.x - center.x) * MM_TO_SCENE) as f32,
        ((point.y - center.y) * MM_TO_SCENE) as f32,
        ((point.z - center.z) * MM_TO_SCENE) as f32,
    ]
}

/// The piece as a shell of its thickness around the simulated surface: the grain and flesh
/// faces, and a wall along its outline and holes.
fn piece_mesh(drape: &Drape, piece: &DrapedPiece, center: DrapeVec) -> Mesh {
    let positions = drape.positions();
    let normals = drape.vertex_normals(piece);
    let half = piece.material.thickness_mm * 0.5;
    let start = piece.vertices.start;
    let count = piece.vertices.len();

    let mut out_positions = Vec::with_capacity(count * 2 + piece.boundary.len() * 4);
    let mut out_normals = Vec::with_capacity(out_positions.capacity());
    for sign in [1.0, -1.0] {
        for (offset, &normal) in normals.iter().enumerate() {
            let point = positions[start + offset] + normal * (half * sign);
            out_positions.push(to_scene(point, center));
            let n = normal * sign;
            out_normals.push([n.x as f32, n.y as f32, n.z as f32]);
        }
    }
    let mut indices = Vec::with_capacity(piece.triangles.len() * 6 + piece.boundary.len() * 6);
    for triangle in &piece.triangles {
        let [a, b, c] = triangle.map(|v| (v - start) as u32);
        indices.extend_from_slice(&[a, b, c]);
        let flesh = count as u32;
        indices.extend_from_slice(&[a + flesh, c + flesh, b + flesh]);
    }
    for &[a, b] in &piece.boundary {
        let (ia, ib) = (a - start, b - start);
        let edge = positions[b] - positions[a];
        let outward = edge.cross(normals[ia] + normals[ib]).normalized();
        let base = out_positions.len() as u32;
        for (vertex, sign) in [(ia, 1.0), (ib, 1.0), (ib, -1.0), (ia, -1.0)] {
            let point = positions[start + vertex] + normals[vertex] * (half * sign);
            out_positions.push(to_scene(point, center));
            out_normals.push([outward.x as f32, outward.y as f32, outward.z as f32]);
        }
        indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, out_positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, out_normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn thread_mesh(drape: &Drape, center: DrapeVec) -> Mesh {
    let segments = drape.thread_segments();
    let positions: Vec<[f32; 3]> = segments
        .iter()
        .flat_map(|segment| segment.map(|point| to_scene(point, center)))
        .collect();
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh
}

impl SewnModel {
    /// Lays the project's pieces out to be sewn and spawns their meshes, each tagged with
    /// `tag`. Returns the size of the model, in mm, for the camera distance.
    pub(super) fn rebuild(
        &mut self,
        project: &leathercraft::model::Project,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        tag: impl Component + Clone,
    ) -> Option<f64> {
        let drape = Drape::new(project);
        self.pieces.clear();
        self.thread = None;
        let (min, max) = drape.bounds()?;
        // Centered over the origin and standing on the ground: the lowest middle surface is
        // lifted by half the thickest leather.
        let thickest = drape
            .pieces()
            .iter()
            .map(|piece| piece.material.thickness_mm)
            .fold(0.0, f64::max);
        self.center = DrapeVec::new(
            (min.x + max.x) * 0.5,
            min.y - thickest * 0.5,
            (min.z + max.z) * 0.5,
        );

        for piece in drape.pieces() {
            let handle = meshes.add(piece_mesh(&drape, piece, self.center));
            let [red, green, blue] = piece.material.color.rgb();
            let leather = materials.add(StandardMaterial {
                base_color: Color::srgb_u8(red, green, blue),
                perceptual_roughness: 0.88,
                double_sided: true,
                cull_mode: None,
                ..default()
            });
            commands.spawn((
                Mesh3d(handle.clone()),
                MeshMaterial3d(leather),
                // The mesh moves while it is sewn, past the bounds it was spawned with.
                bevy::camera::visibility::NoFrustumCulling,
                tag.clone(),
            ));
            self.pieces.push(handle);
        }
        let thread = meshes.add(thread_mesh(&drape, self.center));
        commands.spawn((
            Mesh3d(thread.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: THREAD_COLOR,
                unlit: true,
                ..default()
            })),
            bevy::camera::visibility::NoFrustumCulling,
            tag,
        ));
        self.thread = Some(thread);

        let size = (max.x - min.x).max(max.y - min.y).max(max.z - min.z);
        self.drape = Some(drape);
        Some(size)
    }

    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }

    /// Runs one frame of the simulation and updates the meshes, until it settles.
    pub(super) fn step(&mut self, meshes: &mut Assets<Mesh>) {
        let Some(drape) = self.drape.as_mut() else {
            return;
        };
        if drape.is_settled() {
            return;
        }
        drape.step();
        for (piece, handle) in drape.pieces().iter().zip(&self.pieces) {
            if let Some(mesh) = meshes.get_mut(handle) {
                *mesh = piece_mesh(drape, piece, self.center);
            }
        }
        if let Some(mesh) = self.thread.as_ref().and_then(|h| meshes.get_mut(h)) {
            *mesh = thread_mesh(drape, self.center);
        }
    }
}
