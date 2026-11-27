//! The physics of the sewn preview: an XPBD solver over every piece's mesh, with no gravity.
//! Mesh edges keep the leather from stretching, a distance across each pair of triangles
//! keeps it from bending, as stiff as its material and much softer along centerlines, and the
//! paired holes of each seam are pulled together the way the assembly lays them.

use super::vec3::Vec3;
use crate::material::{Flexibility, Material};

/// Solver steps per frame; more make every constraint stiffer.
pub(super) const SUBSTEPS: usize = 8;
const DT: f64 = 1.0 / 60.0 / SUBSTEPS as f64;
/// The velocity kept after each substep; there is no gravity, so motion only has to die out.
const DAMPING: f64 = 0.9;
/// The frames over which the seams close, so that they draw the pieces in instead of
/// snapping them.
pub(super) const RAMP_FRAMES: u32 = 90;
/// The longest a model is simulated, in frames.
pub(super) const MAX_FRAMES: u32 = 60 * 20;
/// A model whose points move less than this per substep, in mm, has settled.
const SETTLED_MM: f64 = 2.0e-3;

/// How hard the leather resists bending, as XPBD compliance over the step squared: higher is
/// softer. The leather is about 1.2 mm thick; thicker leather is stiffer.
pub(super) fn bending_compliance(material: &Material) -> f64 {
    let base = match material.flexibility {
        Flexibility::VerySoft => 20.0,
        Flexibility::Soft => 8.0,
        Flexibility::SemiSoft => 3.0,
        Flexibility::Medium => 1.0,
        Flexibility::Stiff => 0.3,
        Flexibility::VeryStiff => 0.08,
        Flexibility::Rigid => 0.02,
    };
    let thickness = (material.thickness_mm / 1.2).powi(2).clamp(0.25, 6.0);
    base / thickness
}

/// How much the leather gives along its surface: soft leather stretches a little.
pub(super) fn stretch_compliance(material: &Material) -> f64 {
    match material.flexibility {
        Flexibility::VerySoft => 2.0e-3,
        Flexibility::Soft => 1.0e-3,
        Flexibility::SemiSoft => 5.0e-4,
        Flexibility::Medium => 2.0e-4,
        Flexibility::Stiff => 1.0e-4,
        Flexibility::VeryStiff => 5.0e-5,
        Flexibility::Rigid => 2.0e-5,
    }
}

/// A centerline marks a fold: it bends this much more easily than the leather around it, and
/// never less easily than `CREASE_COMPLIANCE`.
const CREASE_SOFTENING: f64 = 400.0;
const CREASE_COMPLIANCE: f64 = 200.0;

pub(super) fn crease_compliance(material: &Material) -> f64 {
    (bending_compliance(material) * CREASE_SOFTENING).max(CREASE_COMPLIANCE)
}

/// Two points kept at their distance in the pattern.
pub(super) struct Distance {
    pub a: usize,
    pub b: usize,
    pub rest: f64,
    pub compliance: f64,
}

/// The bend across one inner edge, in the quadratic model of Bergou et al. (2006): the
/// weighted sum of the edge's two ends and the two vertices across it is zero while the
/// leather there is flat, and grows with the bend.
pub(super) struct Bend {
    pub vertices: [usize; 4],
    pub weights: [f64; 4],
    pub compliance: f64,
}

fn cotangent(at: [f64; 2], to: [f64; 2], other: [f64; 2]) -> f64 {
    let u = [to[0] - at[0], to[1] - at[1]];
    let v = [other[0] - at[0], other[1] - at[1]];
    let cross = (u[0] * v[1] - u[1] * v[0]).abs().max(1.0e-12);
    (u[0] * v[0] + u[1] * v[1]) / cross
}

impl Bend {
    /// The edge `ends`, with `across` the vertices of its two triangles that aren't on it, all
    /// as they lie in the pattern. `compliance` is per unit of area.
    pub fn new(vertices: [usize; 4], rest: [[f64; 2]; 4], compliance: f64) -> Self {
        let [x0, x1, x2, x3] = rest;
        let (c01, c02) = (cotangent(x0, x1, x2), cotangent(x0, x1, x3));
        let (c03, c04) = (cotangent(x1, x0, x2), cotangent(x1, x0, x3));
        let area = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
            ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() * 0.5
        };
        let areas = area(x0, x1, x2) + area(x0, x1, x3);
        Self {
            vertices,
            weights: [c03 + c04, c01 + c02, -c01 - c03, -c02 - c04],
            compliance: compliance * areas / 3.0,
        }
    }
}

/// A point inside a triangle, by its barycentric weights.
#[derive(Clone, Copy, Debug)]
pub struct Anchor {
    pub vertices: [usize; 3],
    pub weights: [f64; 3],
}

impl Anchor {
    pub(super) fn position(&self, positions: &[Vec3]) -> Vec3 {
        (0..3).fold(Vec3::ZERO, |sum, k| {
            sum + positions[self.vertices[k]] * self.weights[k]
        })
    }

    /// The normal of its triangle, out of the grain side.
    pub(super) fn normal(&self, positions: &[Vec3]) -> Vec3 {
        let [a, b, c] = self.vertices.map(|v| positions[v]);
        (b - a).cross(c - a).normalized()
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum SeamKind {
    /// B's hole lies `offset` along A's grain normal from A's hole, within `slack`.
    Stacked { offset: f64, slack: f64 },
    /// The holes are `gap` apart.
    Beside { gap: f64 },
}

pub(super) struct Seam {
    pub a: Anchor,
    pub b: Anchor,
    pub kind: SeamKind,
    /// The distance the seam starts from, closed over `RAMP_FRAMES`.
    pub start: f64,
}

impl Seam {
    /// The vector the seam measures, from where B's hole should be to where it is.
    fn offset(&self, positions: &[Vec3]) -> Vec3 {
        let (a, b) = (self.a.position(positions), self.b.position(positions));
        match self.kind {
            SeamKind::Stacked { offset, .. } => b - (a + self.a.normal(positions) * offset),
            SeamKind::Beside { .. } => b - a,
        }
    }

    pub(super) fn measure_start(&mut self, positions: &[Vec3]) {
        self.start = self.offset(positions).length();
    }
}

pub(super) struct Solver {
    pub positions: Vec<Vec3>,
    previous: Vec<Vec3>,
    velocities: Vec<Vec3>,
    pub inverse_mass: Vec<f64>,
    pub distances: Vec<Distance>,
    pub bends: Vec<Bend>,
    pub seams: Vec<Seam>,
    pub frame: u32,
    pub settled: bool,
}

impl Solver {
    pub fn new(
        positions: Vec<Vec3>,
        inverse_mass: Vec<f64>,
        distances: Vec<Distance>,
        bends: Vec<Bend>,
        mut seams: Vec<Seam>,
    ) -> Self {
        for seam in &mut seams {
            seam.measure_start(&positions);
        }
        let count = positions.len();
        Self {
            previous: positions.clone(),
            positions,
            velocities: vec![Vec3::ZERO; count],
            inverse_mass,
            distances,
            bends,
            settled: seams.is_empty(),
            seams,
            frame: 0,
        }
    }

    /// One frame of simulation. Returns whether the model is still moving.
    pub fn step(&mut self) -> bool {
        if self.settled {
            return false;
        }
        let ramp = (f64::from(self.frame + 1) / f64::from(RAMP_FRAMES)).min(1.0);
        let mut moved = 0.0_f64;
        for _ in 0..SUBSTEPS {
            for ((position, previous), velocity) in self
                .positions
                .iter_mut()
                .zip(&mut self.previous)
                .zip(&self.velocities)
            {
                *previous = *position;
                *position += *velocity * DT;
            }
            self.solve_distances();
            self.solve_bends();
            self.solve_seams(ramp);
            moved = 0.0;
            for ((position, previous), velocity) in self
                .positions
                .iter()
                .zip(&self.previous)
                .zip(&mut self.velocities)
            {
                let step = *position - *previous;
                moved = moved.max(step.length());
                *velocity = step * (DAMPING / DT);
            }
        }
        self.frame += 1;
        let closed = self.frame >= RAMP_FRAMES + 30;
        if (closed && moved < SETTLED_MM) || self.frame >= MAX_FRAMES {
            self.settled = true;
        }
        !self.settled
    }

    fn solve_distances(&mut self) {
        let positions = &mut self.positions;
        let inverse_mass = &self.inverse_mass;
        for constraint in &self.distances {
            let (wa, wb) = (inverse_mass[constraint.a], inverse_mass[constraint.b]);
            let delta = positions[constraint.b] - positions[constraint.a];
            let length = delta.length();
            if length < 1.0e-12 {
                continue;
            }
            let lambda = -(length - constraint.rest) / (wa + wb + constraint.compliance);
            let correction = delta * (lambda / length);
            positions[constraint.a] -= correction * wa;
            positions[constraint.b] += correction * wb;
        }
    }

    fn solve_bends(&mut self) {
        let positions = &mut self.positions;
        let inverse_mass = &self.inverse_mass;
        for bend in &self.bends {
            let mut sum = Vec3::ZERO;
            let mut denominator = bend.compliance;
            for (&v, &k) in bend.vertices.iter().zip(&bend.weights) {
                sum += positions[v] * k;
                denominator += inverse_mass[v] * k * k;
            }
            if denominator < 1.0e-12 {
                continue;
            }
            for (&v, &k) in bend.vertices.iter().zip(&bend.weights) {
                positions[v] -= sum * (inverse_mass[v] * k / denominator);
            }
        }
    }

    fn solve_seams(&mut self, ramp: f64) {
        for index in 0..self.seams.len() {
            let seam = &self.seams[index];
            let offset = seam.offset(&self.positions);
            let length = offset.length();
            let error = match seam.kind {
                SeamKind::Stacked { slack, .. } => {
                    let allowed = (seam.start + (slack - seam.start) * ramp).max(slack);
                    (length - allowed).max(0.0)
                }
                SeamKind::Beside { gap } => length - (seam.start + (gap - seam.start) * ramp),
            };
            if error.abs() < 1.0e-9 || length < 1.0e-12 {
                continue;
            }
            let weight = |anchor: &Anchor| {
                (0..3)
                    .map(|k| self.inverse_mass[anchor.vertices[k]] * anchor.weights[k].powi(2))
                    .sum::<f64>()
            };
            let total = weight(&seam.a) + weight(&seam.b);
            if total < 1.0e-12 {
                continue;
            }
            let direction = offset * (1.0 / length);
            let lambda = -error / total;
            let (a, b) = (seam.a, seam.b);
            for k in 0..3 {
                let v = a.vertices[k];
                self.positions[v] -= direction * (lambda * a.weights[k] * self.inverse_mass[v]);
            }
            for k in 0..3 {
                let v = b.vertices[k];
                self.positions[v] += direction * (lambda * b.weights[k] * self.inverse_mass[v]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stiffer_and_thicker_leather_bends_less() {
        let mut material = Material::default();
        let medium = bending_compliance(&material);
        material.flexibility = Flexibility::Rigid;
        let rigid = bending_compliance(&material);
        material.thickness_mm = 3.0;
        assert!(rigid < medium && bending_compliance(&material) < rigid);
        assert!(crease_compliance(&material) > 100.0 * bending_compliance(&material));
    }

    #[test]
    fn a_flat_hinge_has_no_bend_and_a_folded_one_does() {
        let rest = [[0.0, 0.0], [1.0, 0.0], [0.2, 1.0], [0.5, -1.0]];
        let bend = Bend::new([0, 1, 2, 3], rest, 1.0);
        let sum = |points: &[Vec3]| (0..4).fold(Vec3::ZERO, |s, i| s + points[i] * bend.weights[i]);
        let flat: Vec<Vec3> = rest.iter().map(|p| Vec3::new(p[0], 0.0, p[1])).collect();
        assert!(sum(&flat).length() < 1.0e-12);
        let mut folded = flat.clone();
        folded[3] = Vec3::new(0.5, 1.0, 0.0);
        assert!(sum(&folded).length() > 0.5);
    }

    #[test]
    fn a_seam_closes_and_stays_closed() {
        // Two loose points, one seam between them.
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(11.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 1.0),
        ];
        let anchor = |first: usize| Anchor {
            vertices: [first, first + 1, first + 2],
            weights: [1.0, 0.0, 0.0],
        };
        let seam = Seam {
            a: anchor(0),
            b: anchor(3),
            kind: SeamKind::Beside { gap: 2.0 },
            start: 0.0,
        };
        let mut solver = Solver::new(positions, vec![1.0; 6], Vec::new(), Vec::new(), vec![seam]);
        while solver.step() {}
        let gap = (solver.positions[3] - solver.positions[0]).length();
        assert!((gap - 2.0).abs() < 0.01, "gap {gap}");
    }
}
