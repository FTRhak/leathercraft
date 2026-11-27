//! Where each piece starts before it is sewn: a piece folded onto itself is folded first, then
//! every piece is laid onto the one it is sewn to, with its holes over its partner's, the right
//! way up for the assembly. The simulation then closes what a rigid placement can't.

use super::Join;
use super::vec3::Vec3;
use crate::model::Point;

/// A rigid move in the pattern's plane, turned over first when `mirror`: a piece turned over
/// has its grain side down.
#[derive(Clone, Copy, Debug)]
pub(super) struct Iso {
    cos: f64,
    sin: f64,
    pub mirror: bool,
    tx: f64,
    ty: f64,
}

impl Iso {
    pub const IDENTITY: Self = Self {
        cos: 1.0,
        sin: 0.0,
        mirror: false,
        tx: 0.0,
        ty: 0.0,
    };

    pub fn apply(&self, point: Point) -> Point {
        let y = if self.mirror { -point.y } else { point.y };
        Point::new(
            self.cos * point.x - self.sin * y + self.tx,
            self.sin * point.x + self.cos * y + self.ty,
        )
    }

    /// The world Y of the grain side's normal.
    pub fn grain_up(&self) -> f64 {
        if self.mirror { -1.0 } else { 1.0 }
    }
}

/// Where a piece lies: its move in the plane and its height above the root piece.
#[derive(Clone, Copy, Debug)]
pub(super) struct Pose {
    pub iso: Iso,
    pub height: f64,
}

impl Pose {
    /// A point of the piece's own frame, (x, lift, y), in the model.
    pub fn place(&self, local: Vec3) -> Vec3 {
        let planar = self.iso.apply(Point::new(local.x, local.z));
        Vec3::new(
            planar.x,
            self.height + self.iso.grain_up() * local.y,
            planar.y,
        )
    }
}

fn mean(points: &[Point]) -> Point {
    let n = points.len().max(1) as f64;
    Point::new(
        points.iter().map(|p| p.x).sum::<f64>() / n,
        points.iter().map(|p| p.y).sum::<f64>() / n,
    )
}

/// The main direction of the points, or `None` when they all but coincide.
fn principal_direction(points: &[Point]) -> Option<Point> {
    let center = mean(points);
    let (mut xx, mut xy, mut yy) = (0.0, 0.0, 0.0);
    for point in points {
        let (dx, dy) = (point.x - center.x, point.y - center.y);
        xx += dx * dx;
        xy += dx * dy;
        yy += dy * dy;
    }
    if xx + yy < 1.0e-6 {
        return None;
    }
    let angle = 0.5 * (2.0 * xy).atan2(xx - yy);
    Some(Point::new(angle.cos(), angle.sin()))
}

/// The rigid move, turned over or not as asked, that best lays `from` onto `to`.
pub(super) fn fit(from: &[Point], to: &[Point], mirror: bool) -> Iso {
    let from: Vec<Point> = from
        .iter()
        .map(|p| Point::new(p.x, if mirror { -p.y } else { p.y }))
        .collect();
    let (cf, ct) = (mean(&from), mean(to));
    let (mut a, mut b) = (0.0, 0.0);
    for (u, v) in from.iter().zip(to) {
        let (ux, uy) = (u.x - cf.x, u.y - cf.y);
        let (vx, vy) = (v.x - ct.x, v.y - ct.y);
        a += ux * vx + uy * vy;
        b += ux * vy - uy * vx;
    }
    let angle = if a.abs() + b.abs() > 1.0e-12 {
        b.atan2(a)
    } else {
        0.0
    };
    let (sin, cos) = angle.sin_cos();
    Iso {
        cos,
        sin,
        mirror,
        tx: ct.x - (cos * cf.x - sin * cf.y),
        ty: ct.y - (sin * cf.x + cos * cf.y),
    }
}

/// Folds the side of a piece that holds the B holes of a link to itself over onto the flesh
/// side of the other, `gap` mm under it, along a centerline between the two sides if there is
/// one, else along the line the pairs are symmetric about.
pub(super) fn prefold(
    local: &mut [Vec3],
    pairs: &[(Point, Point)],
    centerlines: &[(Point, Point)],
    gap: f64,
) {
    let Some(&(first_a, first_b)) = pairs.first() else {
        return;
    };
    let side_a: Vec<Point> = pairs.iter().map(|pair| pair.0).collect();
    let side_b: Vec<Point> = pairs.iter().map(|pair| pair.1).collect();
    let (mean_a, mean_b) = (mean(&side_a), mean(&side_b));
    let midpoints: Vec<Point> = pairs
        .iter()
        .map(|(a, b)| Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5))
        .collect();
    let across = |o: Point, d: Point, p: Point| d.x * (p.y - o.y) - d.y * (p.x - o.x);

    let mut origin = mean(&midpoints);
    let mut direction = match principal_direction(&midpoints) {
        Some(d) => d,
        None => {
            let (dx, dy) = (first_b.x - first_a.x, first_b.y - first_a.y);
            let length = dx.hypot(dy);
            if length < 1.0e-6 {
                return;
            }
            Point::new(-dy / length, dx / length)
        }
    };
    // A centerline with the two sides on either side of it is where the piece folds.
    let mut best_alignment = 0.94;
    for &(start, end) in centerlines {
        let (dx, dy) = (end.x - start.x, end.y - start.y);
        let length = dx.hypot(dy);
        if length < 1.0e-6 {
            continue;
        }
        let d = Point::new(dx / length, dy / length);
        let separates = across(start, d, mean_a) * across(start, d, mean_b) < 0.0;
        let alignment = (d.x * direction.x + d.y * direction.y).abs();
        if separates && alignment > best_alignment {
            best_alignment = alignment;
            origin = start;
            direction = d;
        }
    }

    // `normal` points toward side B, which is the side folded over.
    let mut normal = Point::new(-direction.y, direction.x);
    let toward_b = (mean_b.x - origin.x) * normal.x + (mean_b.y - origin.y) * normal.y;
    if toward_b < 0.0 {
        normal = Point::new(-normal.x, -normal.y);
    }
    // Bent around a half circle as wide as the gap, so that the folded side keeps its size
    // and lies flat under the other, `gap` below it.
    let radius = (gap * 0.5).max(1.0e-3);
    let arc = std::f64::consts::PI * radius;
    for point in local.iter_mut() {
        let (px, py) = (point.x - origin.x, point.z - origin.y);
        let s = px * normal.x + py * normal.y;
        if s <= 0.0 {
            continue;
        }
        let along = px * direction.x + py * direction.y;
        let (out, lift) = if s < arc {
            let angle = s / radius;
            (radius * angle.sin(), -radius * (1.0 - angle.cos()))
        } else {
            (arc - s, -gap)
        };
        point.x = origin.x + direction.x * along + normal.x * out;
        point.z = origin.y + direction.y * along + normal.y * out;
        point.y = lift;
    }
}

/// Rolls a piece whose two edges are sewn together edge to edge, or lapped, into a tube
/// around the flesh side: the distance from the A holes to the B holes, plus `gap`, goes once
/// around.
pub(super) fn preroll(local: &mut [Vec3], pairs: &[(Point, Point)], gap: f64) {
    let side_a: Vec<Point> = pairs.iter().map(|pair| pair.0).collect();
    let side_b: Vec<Point> = pairs.iter().map(|pair| pair.1).collect();
    let (mean_a, mean_b) = (mean(&side_a), mean(&side_b));
    let (dx, dy) = (mean_b.x - mean_a.x, mean_b.y - mean_a.y);
    let span = dx.hypot(dy);
    if span < 1.0e-6 {
        return;
    }
    let u = Point::new(dx / span, dy / span);
    let radius = (span + gap) / std::f64::consts::TAU;
    for point in local.iter_mut() {
        let (px, py) = (point.x - mean_a.x, point.z - mean_a.y);
        let s = px * u.x + py * u.y;
        let angle = s / radius;
        let shift = radius * angle.sin() - s;
        point.x += u.x * shift;
        point.z += u.y * shift;
        point.y = -radius * (1.0 - angle.cos());
    }
}

/// One link between two different pieces, as placement sees it: the pairs' positions in each
/// piece's own frame.
pub(super) struct Edge {
    pub a: usize,
    pub b: usize,
    pub pairs: Vec<(Point, Point)>,
    pub join: Join,
    /// Whether the assembly wants B, not A, the grain side up.
    pub b_on_top: bool,
}

/// The pose of every piece: each group of pieces sewn together is laid onto its piece with the
/// most links, which stays where it is drawn; pieces sewn to nothing stay as drawn.
pub(super) fn place(count: usize, edges: &[Edge], areas: &[f64], centroids: &[Point]) -> Vec<Pose> {
    let mut poses: Vec<Option<Pose>> = vec![None; count];
    let mut links: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (index, edge) in edges.iter().enumerate() {
        links[edge.a].push(index);
        links[edge.b].push(index);
    }
    let mut roots: Vec<usize> = (0..count).collect();
    roots.sort_by(|&i, &j| {
        links[j]
            .len()
            .cmp(&links[i].len())
            .then(areas[j].total_cmp(&areas[i]))
    });

    for root in roots {
        if poses[root].is_some() {
            continue;
        }
        // The root lies grain up, unless its first stacked partner should be the one on top.
        let flip = links[root]
            .iter()
            .map(|&e| &edges[e])
            .find_map(|edge| match edge.join {
                Join::Stacked {
                    turned_over: true, ..
                } => Some(edge.b_on_top == (edge.a == root)),
                _ => None,
            });
        let iso = if flip == Some(true) {
            Iso {
                cos: 1.0,
                sin: 0.0,
                mirror: true,
                tx: 0.0,
                ty: 2.0 * centroids[root].y,
            }
        } else {
            Iso::IDENTITY
        };
        poses[root] = Some(Pose { iso, height: 0.0 });

        let mut queue = vec![root];
        let mut next = 0;
        while next < queue.len() {
            let placed = queue[next];
            next += 1;
            let pose = poses[placed].expect("queued pieces are placed");
            for &index in &links[placed] {
                let edge = &edges[index];
                let placed_is_a = edge.a == placed;
                let other = if placed_is_a { edge.b } else { edge.a };
                if poses[other].is_some() {
                    continue;
                }
                let (own, theirs): (Vec<Point>, Vec<Point>) = edge
                    .pairs
                    .iter()
                    .map(|&(a, b)| if placed_is_a { (a, b) } else { (b, a) })
                    .unzip();
                let targets: Vec<Point> = own.iter().map(|&p| pose.iso.apply(p)).collect();
                let turned_over = matches!(
                    edge.join,
                    Join::Stacked {
                        turned_over: true,
                        ..
                    }
                );
                let mut iso = fit(&theirs, &targets, pose.iso.mirror != turned_over);
                let height = match edge.join {
                    Join::Stacked { side, gap, .. } => {
                        // B lies `side` of A's grain side, along A's normal.
                        let grain_a = if placed_is_a {
                            pose.iso.grain_up()
                        } else {
                            iso.grain_up()
                        };
                        let offset = side * grain_a * gap;
                        if placed_is_a {
                            pose.height + offset
                        } else {
                            pose.height - offset
                        }
                    }
                    Join::Beside { gap } => {
                        // Holes over each other; move the other piece off by the gap, toward
                        // its own side of the seam.
                        if let Some(d) = principal_direction(&targets) {
                            let n = Point::new(-d.y, d.x);
                            let seam = mean(&targets);
                            let body = iso.apply(centroids[other]);
                            let side = (body.x - seam.x) * n.x + (body.y - seam.y) * n.y;
                            let sign = if side >= 0.0 { 1.0 } else { -1.0 };
                            iso.tx += n.x * sign * gap;
                            iso.ty += n.y * sign * gap;
                        }
                        pose.height
                    }
                };
                poses[other] = Some(Pose { iso, height });
                queue.push(other);
            }
        }
    }
    poses
        .into_iter()
        .map(|pose| {
            pose.unwrap_or(Pose {
                iso: Iso::IDENTITY,
                height: 0.0,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_finds_a_rotation_and_a_turn_over() {
        let from = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 5.0),
        ];
        // Turned 90° and moved.
        let to: Vec<Point> = from
            .iter()
            .map(|p| Point::new(3.0 - p.y, 4.0 + p.x))
            .collect();
        let iso = fit(&from, &to, false);
        for (p, q) in from.iter().zip(&to) {
            let r = iso.apply(*p);
            assert!((r.x - q.x).abs() < 1.0e-9 && (r.y - q.y).abs() < 1.0e-9);
        }
        // Mirrored across the x axis.
        let to: Vec<Point> = from.iter().map(|p| Point::new(p.x, -p.y)).collect();
        let iso = fit(&from, &to, true);
        for (p, q) in from.iter().zip(&to) {
            let r = iso.apply(*p);
            assert!((r.x - q.x).abs() < 1.0e-9 && (r.y - q.y).abs() < 1.0e-9);
        }
    }

    #[test]
    fn prefold_turns_side_b_toward_the_flesh_side() {
        let mut local = vec![Vec3::new(-10.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)];
        let pairs = [(Point::new(-8.0, 0.0), Point::new(8.0, 0.0))];
        prefold(&mut local, &pairs, &[], 1.0);
        assert_eq!(local[0], Vec3::new(-10.0, 0.0, 0.0));
        // 10 mm out, of which a half circle of radius 0.5 goes into the bend.
        let expected = 10.0 - std::f64::consts::PI * 0.5;
        assert!(
            (local[1].x + expected).abs() < 1.0e-9 && local[1].y == -1.0,
            "{:?}",
            local[1]
        );
    }
}
