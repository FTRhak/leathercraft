//! The flat preview: each piece extruded from its region, with its stitch holes cut through.

use super::MM_TO_SCENE;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use earcutr::earcut;
use leathercraft::boolean::Region;
use leathercraft::model::{Point, signed_area};

pub(super) fn build_piece_mesh(
    region: &Region,
    thickness_mm: f64,
    center_x: f32,
    center_z: f32,
) -> Result<Mesh, String> {
    let points: Vec<Point> = region.vertices().copied().collect();
    let thickness = (thickness_mm * MM_TO_SCENE) as f32;
    let mut flat_points = Vec::with_capacity(points.len() * 2);
    for point in &points {
        flat_points.push(point.x * MM_TO_SCENE - f64::from(center_x));
        flat_points.push(point.y * MM_TO_SCENE - f64::from(center_z));
    }
    let triangles = earcut(&flat_points, &region.hole_indices(), 2)
        .map_err(|error| format!("triangulation failed: {error}"))?;

    let mut positions = Vec::<[f32; 3]>::with_capacity(points.len() * 2 + points.len() * 4);
    let mut normals = Vec::<[f32; 3]>::with_capacity(points.len() * 2 + points.len() * 4);
    let mut indices = Vec::<u32>::new();
    for point in &points {
        let x = (point.x * MM_TO_SCENE - f64::from(center_x)) as f32;
        let z = (point.y * MM_TO_SCENE - f64::from(center_z)) as f32;
        positions.push([x, 0.0, z]);
        normals.push([0.0, -1.0, 0.0]);
    }
    for point in &points {
        let x = (point.x * MM_TO_SCENE - f64::from(center_x)) as f32;
        let z = (point.y * MM_TO_SCENE - f64::from(center_z)) as f32;
        positions.push([x, thickness, z]);
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

    // Rings can be wound either way; walls must still face away from the leather,
    // which for a hole means toward its own center.
    for (ring_index, ring) in region.rings().enumerate() {
        let is_hole = ring_index > 0;
        let outward: f32 = if (signed_area(ring) >= 0.0) != is_hole {
            1.0
        } else {
            -1.0
        };
        for index in 0..ring.len() {
            let next = (index + 1) % ring.len();
            let a = ring[index];
            let b = ring[next];
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
                [ax, thickness, az],
                [bx, thickness, bz],
                [bx, 0.0, bz],
            ]);
            normals.extend_from_slice(&[normal; 4]);
            if outward > 0.0 {
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            } else {
                indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
            }
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
