//! The sewn preview (`docs/sewn-preview.md`): the project's pieces sewn together along their
//! seam links, as a simulation the 3D preview steps frame by frame. Each piece is a triangle
//! mesh that keeps its size, bends as easily as its material allows, and folds most easily
//! along its centerlines; each pair of holes is drawn together as the link's assembly lays
//! the pieces, with the slack of its stitch. There is no gravity and no collision.

mod delaunay;
mod placement;
mod sheet;
mod solver;
mod vec3;

pub use solver::Anchor;
pub use vec3::Vec3;

use crate::boolean::Region;
use crate::material::Material;
use crate::model::{Point, Project, signed_area};
use crate::seams::stitch_line;
use crate::sewing::{Assembly, OnTop, Sewing, StitchMethod, Thread};
use crate::shapes::Shape;
use placement::{Edge, Pose};
use sheet::{Sheet, segment_distance};
use solver::{Bend, Distance, Seam, SeamKind, Solver};
use std::collections::HashMap;
use std::ops::Range;

/// About how many vertices all the meshes share; the spacing follows from the total area.
const TARGET_VERTICES: f64 = 2_500.0;
/// The finest spacing, in mm.
const MIN_SPACING_MM: f64 = 1.0;
/// How far above the leather the thread's surface is drawn, in mm, so that it isn't hidden in it.
const THREAD_LIFT_MM: f64 = 0.03;
/// Each vertex starts this many spacings off the plane at most, so that a piece that has to
/// curve can leave it.
const NOISE: f64 = 0.02;

/// How two linked pieces lie against each other, from the link's assembly.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Join {
    /// B lies on A: on A's grain side when `side` is 1, its flesh side when -1, `gap` mm apart
    /// between the middles of the two leathers. `turned_over` when B faces the other way.
    Stacked {
        side: f64,
        turned_over: bool,
        gap: f64,
    },
    /// Edge to edge, the paired holes `gap` mm apart.
    Beside { gap: f64 },
}

/// The slack the stitch leaves between two stacked holes, in mm.
fn slack(stitch: StitchMethod) -> f64 {
    match stitch {
        StitchMethod::Saddle => 0.0,
        StitchMethod::Running => 0.3,
        StitchMethod::Baseball => 0.15,
        StitchMethod::Whip => 0.4,
        StitchMethod::Cross => 0.5,
    }
}

/// One piece of the sewn model.
pub struct DrapedPiece {
    pub piece: u64,
    pub material: Material,
    /// Its vertices in `Drape::positions`.
    pub vertices: Range<usize>,
    /// Indices into `Drape::positions`; `(b - a) × (c - a)` points out of the grain side.
    pub triangles: Vec<[usize; 3]>,
    /// The edges of a single triangle, in that triangle's order: the outline and the holes.
    pub boundary: Vec<[usize; 2]>,
}

/// A seam as the thread is drawn along it.
pub struct SeamThread {
    pub stitch: StitchMethod,
    pub thread: Thread,
    /// How many holes are sewn back over at each end.
    pub lock_holes: u32,
    /// The holes of each pair, in the order they are sewn.
    pub a: Vec<Anchor>,
    pub b: Vec<Anchor>,
    pub thickness_a: f64,
    pub thickness_b: f64,
}

/// A stretch of thread on a face of the leather, its middle line lying half the thread's
/// height off the face.
#[derive(Clone, Copy, Debug)]
pub struct ThreadSegment {
    /// Its seam in `Drape::threads`.
    pub seam: usize,
    pub ends: [Vec3; 2],
    /// Out of the face it lies on, for the side a flat thread lies flat against.
    pub normal: Vec3,
}

/// The sewn model and its simulation.
pub struct Drape {
    pieces: Vec<DrapedPiece>,
    threads: Vec<SeamThread>,
    solver: Solver,
}

struct Built {
    index: usize,
    sheet: Sheet,
    region: Region,
    lines: Vec<(Point, Point)>,
    material: Material,
    offset: usize,
}

/// Where a pattern point lies on the sheet, as a triangle and weights; outside every
/// triangle, the nearest one.
fn attach(sheet: &Sheet, offset: usize, point: Point) -> Anchor {
    let mut best = (f64::INFINITY, [0, 0, 0], [1.0, 0.0, 0.0]);
    for triangle in &sheet.triangles {
        let [a, b, c] = triangle.map(|v| sheet.points[v]);
        let det = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
        if det.abs() < 1.0e-12 {
            continue;
        }
        let wa = ((b.y - c.y) * (point.x - c.x) + (c.x - b.x) * (point.y - c.y)) / det;
        let wb = ((c.y - a.y) * (point.x - c.x) + (a.x - c.x) * (point.y - c.y)) / det;
        let wc = 1.0 - wa - wb;
        let outside = (-wa).max(-wb).max(-wc).max(0.0);
        if outside < best.0 {
            best = (outside, *triangle, [wa, wb, wc]);
            if outside == 0.0 {
                break;
            }
        }
    }
    let (_, vertices, weights) = best;
    // Outside, the weights are clamped onto the triangle and renormalized.
    let clamped = weights.map(|w: f64| w.max(0.0));
    let sum: f64 = clamped.iter().sum::<f64>().max(1.0e-12);
    Anchor {
        vertices: vertices.map(|v| v + offset),
        weights: clamped.map(|w| w / sum),
    }
}

fn region_area(region: &Region) -> f64 {
    signed_area(&region.outline).abs()
        - region
            .holes
            .iter()
            .map(|hole| signed_area(hole).abs())
            .sum::<f64>()
}

/// The mean distance from the points to the piece's outline or holes.
fn edge_distance(region: &Region, points: &[Point]) -> f64 {
    let distance = |point: Point| {
        region
            .rings()
            .flat_map(|ring| (0..ring.len()).map(move |i| (ring[i], ring[(i + 1) % ring.len()])))
            .map(|(a, b)| segment_distance(point, a, b))
            .fold(f64::INFINITY, f64::min)
    };
    points.iter().map(|&p| distance(p)).sum::<f64>() / points.len().max(1) as f64
}

fn join_of(sewing: &Sewing, thickness: f64, butt_gap: impl FnOnce() -> f64) -> Join {
    match sewing.assembly {
        Assembly::FleshToFlesh | Assembly::Folded => Join::Stacked {
            side: -1.0,
            turned_over: true,
            gap: thickness,
        },
        Assembly::Lapped => Join::Stacked {
            side: if sewing.on_top == OnTop::B { 1.0 } else { -1.0 },
            turned_over: false,
            gap: thickness,
        },
        Assembly::Butted => Join::Beside { gap: butt_gap() },
        // Sewn grain to grain and turned out: the pieces open away from the seam.
        Assembly::Turned => Join::Beside { gap: thickness },
    }
}

impl Drape {
    /// The project's pieces laid onto each other and ready to be sewn. Pieces whose shapes
    /// don't combine into one outline are left out, as in the flat preview, and so are the
    /// links that use them.
    pub fn new(project: &Project) -> Self {
        let candidates: Vec<(usize, Region)> = project
            .pieces
            .iter()
            .enumerate()
            .filter_map(|(index, piece)| Some((index, piece.region().ok()?)))
            .collect();
        let total_area: f64 = candidates.iter().map(|(_, r)| region_area(r)).sum();
        let spacing = (total_area / (0.866 * TARGET_VERTICES))
            .sqrt()
            .max(MIN_SPACING_MM);

        let mut built: Vec<Built> = Vec::new();
        let mut offset = 0;
        for (index, region) in candidates {
            let piece = &project.pieces[index];
            let lines: Vec<(Point, Point)> = piece
                .children
                .iter()
                .filter_map(|child| match &child.shape {
                    Shape::Line(line) => Some((line.start, line.end)),
                    _ => None,
                })
                .collect();
            let Some(sheet) = sheet::build(&region, &lines, spacing) else {
                continue;
            };
            let count = sheet.points.len();
            built.push(Built {
                index,
                sheet,
                region,
                lines,
                material: piece.material,
                offset,
            });
            offset += count;
        }
        let by_piece: HashMap<u64, usize> = built
            .iter()
            .enumerate()
            .map(|(slot, b)| (project.pieces[b.index].id, slot))
            .collect();

        // The pieces' own frames: (x, lift, y), lifted only where a piece folds onto itself.
        let mut local: Vec<Vec<Vec3>> = built
            .iter()
            .map(|b| {
                b.sheet
                    .points
                    .iter()
                    .map(|p| Vec3::new(p.x, 0.0, p.y))
                    .collect()
            })
            .collect();

        struct Link {
            a: usize,
            b: usize,
            holes: Vec<(Point, Point)>,
            sewing: Sewing,
        }
        let mut links = Vec::new();
        for link in &project.seam_links {
            let (Some(&a), Some(&b)) = (by_piece.get(&link.a.piece), by_piece.get(&link.b.piece))
            else {
                continue;
            };
            let (Some(line_a), Some(line_b)) = (
                stitch_line(project, link.a.piece, link.a.child),
                stitch_line(project, link.b.piece, link.b.child),
            ) else {
                continue;
            };
            let Ok(pairs) = link.pairs(project) else {
                continue;
            };
            let (holes_a, holes_b) = (line_a.holes(), line_b.holes());
            let holes: Vec<(Point, Point)> = pairs
                .pairs
                .iter()
                .filter_map(|pair| Some((holes_a.get(pair.a)?.center, holes_b.get(pair.b)?.center)))
                .collect();
            if !holes.is_empty() {
                links.push(Link {
                    a,
                    b,
                    holes,
                    sewing: link.sewing,
                });
            }
        }

        // Every hole as a point of its sheet.
        let anchors: Vec<Vec<(Anchor, Anchor)>> = links
            .iter()
            .map(|link| {
                let (a, b) = (&built[link.a], &built[link.b]);
                link.holes
                    .iter()
                    .map(|&(pa, pb)| {
                        (
                            attach(&a.sheet, a.offset, pa),
                            attach(&b.sheet, b.offset, pb),
                        )
                    })
                    .collect()
            })
            .collect();
        let joins: Vec<Join> = links
            .iter()
            .map(|link| {
                let thickness = (built[link.a].material.thickness_mm
                    + built[link.b].material.thickness_mm)
                    * 0.5;
                // Butted edges touch, so the holes are as far apart as each is from its edge.
                join_of(&link.sewing, thickness, || {
                    let side_a: Vec<Point> = link.holes.iter().map(|h| h.0).collect();
                    let side_b: Vec<Point> = link.holes.iter().map(|h| h.1).collect();
                    edge_distance(&built[link.a].region, &side_a)
                        + edge_distance(&built[link.b].region, &side_b)
                })
            })
            .collect();

        // A piece sewn to itself is folded or rolled first, by its first such link; the
        // simulation does the rest.
        let mut shaped = vec![false; built.len()];
        for (link, join) in links.iter().zip(&joins) {
            if link.a != link.b || std::mem::replace(&mut shaped[link.a], true) {
                continue;
            }
            let b = &built[link.a];
            let points = &mut local[link.a];
            match *join {
                Join::Stacked {
                    turned_over: true,
                    gap,
                    ..
                } => placement::prefold(points, &link.holes, &b.lines, gap),
                Join::Stacked { .. } => placement::preroll(points, &link.holes, 0.0),
                Join::Beside { gap } => placement::preroll(points, &link.holes, gap),
            }
        }
        let flat: Vec<Vec3> = local.iter().flatten().copied().collect();
        let in_plane = |anchor: &Anchor| {
            let p = anchor.position(&flat);
            Point::new(p.x, p.z)
        };

        let edges: Vec<Edge> = links
            .iter()
            .zip(&anchors)
            .zip(&joins)
            .filter(|((link, _), _)| link.a != link.b)
            .map(|((link, pairs), &join)| Edge {
                a: link.a,
                b: link.b,
                pairs: pairs
                    .iter()
                    .map(|(a, b)| (in_plane(a), in_plane(b)))
                    .collect(),
                join,
                b_on_top: link.sewing.on_top == OnTop::B,
            })
            .collect();
        let areas: Vec<f64> = built.iter().map(|b| region_area(&b.region)).collect();
        let centroids: Vec<Point> = local
            .iter()
            .map(|points| {
                let n = points.len().max(1) as f64;
                Point::new(
                    points.iter().map(|p| p.x).sum::<f64>() / n,
                    points.iter().map(|p| p.z).sum::<f64>() / n,
                )
            })
            .collect();
        let poses: Vec<Pose> = placement::place(built.len(), &edges, &areas, &centroids);

        let mut positions = Vec::with_capacity(offset);
        for (slot, points) in local.iter().enumerate() {
            for (i, &point) in points.iter().enumerate() {
                let mut world = poses[slot].place(point);
                let seed = (built[slot].offset + i) as f64;
                world.y += ((seed * 91.345).sin() * 47_453.547).fract() * NOISE * spacing;
                positions.push(world);
            }
        }

        let mut inverse_mass = vec![0.0; offset];
        let mut distances = Vec::new();
        let mut bends = Vec::new();
        let mut pieces = Vec::with_capacity(built.len());
        for b in &built {
            let sheet = &b.sheet;
            let mut area = vec![0.0; sheet.points.len()];
            // Each edge with the vertices across it, in each of its triangles.
            let mut across: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
            for &[i, j, k] in &sheet.triangles {
                let (p, q, r) = (sheet.points[i], sheet.points[j], sheet.points[k]);
                let third = ((q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x)).abs() / 6.0;
                for (v, (e0, e1)) in [(k, (i, j)), (i, (j, k)), (j, (k, i))] {
                    area[v] += third;
                    across.entry((e0.min(e1), e0.max(e1))).or_default().push(v);
                }
            }
            let mass_scale = b.material.thickness_mm / 1.2 / (0.866 * spacing * spacing);
            for (v, &a) in area.iter().enumerate() {
                inverse_mass[b.offset + v] = 1.0 / (a * mass_scale).max(1.0e-3);
            }
            let rest = |i: usize, j: usize| {
                let (p, q) = (sheet.points[i], sheet.points[j]);
                (p.x - q.x).hypot(p.y - q.y)
            };
            let stretch = solver::stretch_compliance(&b.material);
            let bending = solver::bending_compliance(&b.material);
            let crease = solver::crease_compliance(&b.material);
            let mut boundary = Vec::new();
            for (&(i, j), opposite) in &across {
                distances.push(Distance {
                    a: b.offset + i,
                    b: b.offset + j,
                    rest: rest(i, j),
                    compliance: stretch,
                });
                if let [c, d] = opposite[..] {
                    let compliance = if sheet.creases.contains(&(i, j)) {
                        crease
                    } else {
                        bending
                    };
                    let at = |v: usize| [sheet.points[v].x, sheet.points[v].y];
                    bends.push(Bend::new(
                        [i, j, c, d].map(|v| b.offset + v),
                        [at(i), at(j), at(c), at(d)],
                        compliance,
                    ));
                }
            }
            for &[i, j, k] in &sheet.triangles {
                for (e0, e1) in [(i, j), (j, k), (k, i)] {
                    if across[&(e0.min(e1), e0.max(e1))].len() == 1 {
                        boundary.push([b.offset + e0, b.offset + e1]);
                    }
                }
            }
            pieces.push(DrapedPiece {
                piece: project.pieces[b.index].id,
                material: b.material,
                vertices: b.offset..b.offset + sheet.points.len(),
                triangles: sheet
                    .triangles
                    .iter()
                    .map(|t| t.map(|v| v + b.offset))
                    .collect(),
                boundary,
            });
        }

        let mut seams = Vec::new();
        let mut threads = Vec::new();
        for ((link, pairs), join) in links.iter().zip(&anchors).zip(&joins) {
            let kind = match *join {
                Join::Stacked { side, gap, .. } => SeamKind::Stacked {
                    offset: side * gap,
                    slack: slack(link.sewing.stitch),
                },
                Join::Beside { gap } => SeamKind::Beside { gap },
            };
            for &(a, b) in pairs {
                seams.push(Seam {
                    a,
                    b,
                    kind,
                    start: 0.0,
                });
            }
            threads.push(SeamThread {
                stitch: link.sewing.stitch,
                thread: link.sewing.thread,
                lock_holes: link.sewing.lock_holes,
                a: pairs.iter().map(|p| p.0).collect(),
                b: pairs.iter().map(|p| p.1).collect(),
                thickness_a: built[link.a].material.thickness_mm,
                thickness_b: built[link.b].material.thickness_mm,
            });
        }

        Self {
            pieces,
            threads,
            solver: Solver::new(positions, inverse_mass, distances, bends, seams),
        }
    }

    pub fn pieces(&self) -> &[DrapedPiece] {
        &self.pieces
    }

    /// The seams' threads, in the order of `ThreadSegment::seam`.
    pub fn threads(&self) -> &[SeamThread] {
        &self.threads
    }

    pub fn positions(&self) -> &[Vec3] {
        &self.solver.positions
    }

    /// Whether the pieces have stopped moving, or the simulation ran out of time.
    pub fn is_settled(&self) -> bool {
        self.solver.settled
    }

    /// One frame of simulation; returns whether anything moved.
    pub fn step(&mut self) -> bool {
        self.solver.step()
    }

    /// The box around every vertex, as (min, max).
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let positions = self.positions();
        let first = *positions.first()?;
        Some(positions.iter().fold((first, first), |(min, max), p| {
            (
                Vec3::new(min.x.min(p.x), min.y.min(p.y), min.z.min(p.z)),
                Vec3::new(max.x.max(p.x), max.y.max(p.y), max.z.max(p.z)),
            )
        }))
    }

    /// Each vertex's normal out of the grain side, averaged over its triangles by area.
    pub fn vertex_normals(&self, piece: &DrapedPiece) -> Vec<Vec3> {
        let positions = self.positions();
        let start = piece.vertices.start;
        let mut normals = vec![Vec3::ZERO; piece.vertices.len()];
        let mut fallback = vec![Vec3::ZERO; piece.vertices.len()];
        for &[a, b, c] in &piece.triangles {
            let normal = (positions[b] - positions[a]).cross(positions[c] - positions[a]);
            for v in [a, b, c] {
                normals[v - start] += normal;
                fallback[v - start] = normal;
            }
        }
        normals
            .iter()
            .zip(&fallback)
            .map(|(&sum, &one)| {
                // Across a fold the normals cancel out; then any one triangle's will do.
                if sum.length() > 0.2 * one.length() {
                    sum.normalized()
                } else {
                    one.normalized()
                }
            })
            .collect()
    }

    /// The thread on the leather's faces, as segments: along the seam on both faces for a
    /// saddle stitch, every other stitch on each face for a running stitch (both faces where it
    /// is sewn back over its lock holes), crossing from one side's hole to the other's next
    /// one for the baseball, whip and cross stitches. Each lies on the face by half its
    /// thread's height.
    pub fn thread_segments(&self) -> Vec<ThreadSegment> {
        let positions = self.positions();
        let mut segments = Vec::new();
        for (seam, thread) in self.threads.iter().enumerate() {
            let (_, height) = thread.thread.profile_mm();
            // The face's normal at the hole, and the thread's middle line over it.
            let face = |anchor: &Anchor, thickness: f64, grain: bool| {
                let sign = if grain { 1.0 } else { -1.0 };
                let normal = anchor.normal(positions) * sign;
                let lift = thickness * 0.5 + THREAD_LIFT_MM + height * 0.5;
                (anchor.position(positions) + normal * lift, normal)
            };
            let mut push = |(p0, n0): (Vec3, Vec3), (p1, n1): (Vec3, Vec3)| {
                segments.push(ThreadSegment {
                    seam,
                    ends: [p0, p1],
                    normal: (n0 + n1).normalized(),
                })
            };
            let (ta, tb) = (thread.thickness_a, thread.thickness_b);
            let stitches = thread.a.len().saturating_sub(1);
            let lock = thread.lock_holes as usize;
            for k in 0..stitches {
                let (a0, a1, b0, b1) = (
                    &thread.a[k],
                    &thread.a[k + 1],
                    &thread.b[k],
                    &thread.b[k + 1],
                );
                match thread.stitch {
                    StitchMethod::Saddle | StitchMethod::Running => {
                        let locked = k < lock || k + lock >= stitches;
                        let faces: &[bool] = match thread.stitch {
                            StitchMethod::Saddle => &[true, false],
                            _ if locked => &[true, false],
                            _ if k % 2 == 0 => &[true],
                            _ => &[false],
                        };
                        for &grain in faces {
                            push(face(a0, ta, grain), face(a1, ta, grain));
                            push(face(b0, tb, grain), face(b1, tb, grain));
                        }
                    }
                    StitchMethod::Baseball | StitchMethod::Whip | StitchMethod::Cross => {
                        push(face(a0, ta, true), face(b1, tb, true));
                        if thread.stitch != StitchMethod::Whip {
                            push(face(b0, tb, true), face(a1, ta, true));
                        }
                    }
                }
            }
        }
        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Flexibility;
    use crate::model::{BooleanOp, ChildShape, PatternPiece};
    use crate::seams::{SeamLink, SeamSide};
    use crate::shapes::{HoleKind, Line, LineStyle, Rectangle, StitchLine};

    fn rectangle(x: f64, y: f64, width: f64, height: f64) -> Shape {
        Shape::Rectangle(Rectangle {
            center: Point::new(x + width * 0.5, y + height * 0.5),
            width,
            height,
            rotation: 0.0,
            radii: Vec::new(),
        })
    }

    fn stitch(from: Point, to: Point) -> Shape {
        Shape::StitchLine(StitchLine::new(vec![from, to], HoleKind::Round, 4.0, 1.0).unwrap())
    }

    fn child(id: u64, shape: Shape) -> ChildShape {
        ChildShape {
            id,
            op: BooleanOp::Subtract,
            shape,
        }
    }

    fn link(id: u64, a: (u64, u64, usize), b: (u64, u64, usize), assembly: Assembly) -> SeamLink {
        let side = |(piece, child, holes): (u64, u64, usize)| SeamSide {
            piece,
            child,
            from: 0,
            to: holes - 1,
            reversed: false,
        };
        SeamLink {
            id,
            a: side(a),
            b: side(b),
            offset: 0,
            manual: Vec::new(),
            sewing: Sewing {
                assembly,
                ..Sewing::default()
            },
        }
    }

    fn hole_count(project: &Project, piece: u64, child: u64) -> usize {
        stitch_line(project, piece, child).unwrap().hole_count()
    }

    /// The largest distance between paired holes after settling, beyond their wanted gap.
    fn settle(project: &Project) -> (Drape, f64) {
        let mut drape = Drape::new(project);
        while drape.step() {}
        let positions = drape.positions();
        let worst = drape
            .solver
            .seams
            .iter()
            .map(|seam| {
                let (a, b) = (seam.a.position(positions), seam.b.position(positions));
                match seam.kind {
                    SeamKind::Stacked { offset, slack } => {
                        let target = a + seam.a.normal(positions) * offset;
                        ((b - target).length() - slack).max(0.0)
                    }
                    SeamKind::Beside { gap } => ((b - a).length() - gap).abs(),
                }
            })
            .fold(0.0, f64::max);
        (drape, worst)
    }

    /// A card holder: one strip folded on its centerline, its two long edges sewn together.
    fn folded_strip(flexibility: Flexibility) -> Project {
        let mut piece = PatternPiece::new(1, "Strip", rectangle(0.0, 0.0, 160.0, 60.0));
        piece.material.flexibility = flexibility;
        piece.children = vec![
            child(1, stitch(Point::new(4.0, 4.0), Point::new(76.0, 4.0))),
            child(2, stitch(Point::new(84.0, 4.0), Point::new(156.0, 4.0))),
            child(3, stitch(Point::new(4.0, 56.0), Point::new(76.0, 56.0))),
            child(4, stitch(Point::new(84.0, 56.0), Point::new(156.0, 56.0))),
            child(
                5,
                Shape::Line(Line {
                    start: Point::new(80.0, -5.0),
                    end: Point::new(80.0, 65.0),
                    style: LineStyle::Solid,
                }),
            ),
        ];
        let mut project = Project::default();
        project.pieces.push(piece);
        let n = hole_count(&project, 1, 1);
        let mut bottom = link(1, (1, 1, n), (1, 2, n), Assembly::Folded);
        bottom.b.reversed = true;
        let mut top = link(2, (1, 3, n), (1, 4, n), Assembly::Folded);
        top.b.reversed = true;
        project.seam_links = vec![bottom, top];
        project
    }

    #[test]
    fn a_strip_folds_on_its_centerline_and_its_seams_close() {
        let project = folded_strip(Flexibility::Stiff);
        let (drape, worst) = settle(&project);
        assert!(worst < 0.5, "seams open by {worst} mm");
        let (min, max) = drape.bounds().unwrap();
        // Folded in half: about 80 mm long, and only a few thicknesses high.
        assert!((max.x - min.x) < 90.0, "length {}", max.x - min.x);
        assert!((max.y - min.y) < 4.0, "height {}", max.y - min.y);
    }

    #[test]
    fn two_panels_are_sewn_flesh_to_flesh() {
        let mut project = Project::default();
        for (id, x) in [(1, 0.0), (2, 120.0)] {
            let mut piece = PatternPiece::new(id, "Panel", rectangle(x, 0.0, 100.0, 60.0));
            piece.children = vec![child(
                1,
                stitch(Point::new(x + 4.0, 4.0), Point::new(x + 96.0, 4.0)),
            )];
            project.pieces.push(piece);
        }
        let n = hole_count(&project, 1, 1);
        // Turned over onto A, B's holes run the other way.
        let mut seam = link(1, (1, 1, n), (2, 1, n), Assembly::FleshToFlesh);
        seam.b.reversed = true;
        project.seam_links = vec![seam];
        let (drape, worst) = settle(&project);
        assert!(worst < 0.3, "seam open by {worst} mm");
        let (min, max) = drape.bounds().unwrap();
        // The second panel lies on the first: one panel's footprint.
        assert!((max.x - min.x) < 110.0 && (max.z - min.z) < 70.0);
    }

    #[test]
    fn butted_pieces_lie_edge_to_edge() {
        let mut project = Project::default();
        for (id, x, seam_x) in [(1, 0.0, 96.0), (2, 200.0, 204.0)] {
            let mut piece = PatternPiece::new(id, "Panel", rectangle(x, 0.0, 100.0, 60.0));
            piece.children = vec![child(
                1,
                stitch(Point::new(seam_x, 4.0), Point::new(seam_x, 56.0)),
            )];
            project.pieces.push(piece);
        }
        let n = hole_count(&project, 1, 1);
        project.seam_links = vec![link(1, (1, 1, n), (2, 1, n), Assembly::Butted)];
        let (drape, worst) = settle(&project);
        assert!(worst < 0.3, "seam off by {worst} mm");
        let (min, max) = drape.bounds().unwrap();
        assert!(
            ((max.x - min.x) - 200.0).abs() < 3.0,
            "width {}",
            max.x - min.x
        );
    }

    #[test]
    fn a_strip_butted_to_itself_rolls_into_a_cuff() {
        let mut piece = PatternPiece::new(1, "Cuff", rectangle(0.0, 0.0, 150.0, 30.0));
        piece.children = vec![
            child(1, stitch(Point::new(4.0, 3.0), Point::new(4.0, 27.0))),
            child(2, stitch(Point::new(146.0, 3.0), Point::new(146.0, 27.0))),
        ];
        let mut project = Project::default();
        project.pieces.push(piece);
        let n = hole_count(&project, 1, 1);
        project.seam_links = vec![link(1, (1, 1, n), (1, 2, n), Assembly::Butted)];
        let (drape, worst) = settle(&project);
        assert!(worst < 0.3, "seam off by {worst} mm");
        let (min, max) = drape.bounds().unwrap();
        // 150 mm around: about 48 mm across.
        let diameter = 150.0 / std::f64::consts::PI;
        assert!(
            ((max.x - min.x) - diameter).abs() < 4.0,
            "across {}",
            max.x - min.x
        );
        assert!(
            ((max.y - min.y) - diameter).abs() < 4.0,
            "high {}",
            max.y - min.y
        );
    }

    #[test]
    fn unlinked_pieces_stay_flat_and_still() {
        let mut project = Project::default();
        project.pieces.push(PatternPiece::new(
            1,
            "Panel",
            rectangle(0.0, 0.0, 50.0, 30.0),
        ));
        let mut drape = Drape::new(&project);
        assert!(drape.is_settled() && !drape.step());
        let (min, max) = drape.bounds().unwrap();
        assert!(max.y - min.y < 0.1);
        assert!(drape.thread_segments().is_empty());
    }

    #[test]
    fn a_folded_strip_shows_its_thread() {
        let mut project = folded_strip(Flexibility::Medium);
        let navy = crate::sewing::ThreadShade::Navy.color();
        for link in &mut project.seam_links {
            link.sewing.thread.color = navy;
        }
        let drape = Drape::new(&project);
        let segments = drape.thread_segments();
        assert!(!segments.is_empty());
        assert_eq!(drape.threads().len(), 2);
        assert!(drape.threads().iter().all(|t| t.thread.color == navy));
        assert!(segments.iter().all(|segment| segment.seam < 2));
        assert_eq!(drape.pieces().len(), 1);
        assert!(!drape.pieces()[0].boundary.is_empty());
    }

    #[test]
    fn a_running_stitch_covers_both_faces_over_its_lock_holes() {
        let count = |lock_holes: u32| {
            let mut project = folded_strip(Flexibility::Medium);
            for link in &mut project.seam_links {
                link.sewing.stitch = StitchMethod::Running;
                link.sewing.lock_holes = lock_holes;
            }
            Drape::new(&project).thread_segments().len()
        };
        // Two locked stitches at each end of two links: each gains its other face, on both
        // sides of the link.
        assert_eq!(count(2), count(0) + 2 * 4 * 2);
    }

    #[test]
    fn a_thicker_thread_lies_further_off_the_leather() {
        let lift = |diameter_mm: f64| {
            let mut project = folded_strip(Flexibility::Medium);
            for link in &mut project.seam_links {
                link.sewing.thread.diameter_mm = diameter_mm;
            }
            let drape = Drape::new(&project);
            let segment = drape.thread_segments()[0];
            let anchor = &drape.threads()[segment.seam].a[0];
            (segment.ends[0] - anchor.position(drape.positions())).length()
        };
        assert!((lift(1.2) - lift(0.4) - 0.4).abs() < 1.0e-9);
    }
}
