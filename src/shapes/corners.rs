//! Rounded corners shared by the shapes that have vertices: each vertex can have a radius, and
//! the ring replaces that corner with an arc tangent to both of its edges.

use super::CIRCLE_SEGMENTS;
use crate::model::Point;
use std::f64::consts::{PI, TAU};

/// Points closer than this are merged, so fillets that meet at an edge's midpoint don't leave a
/// repeated point.
const MERGE_DISTANCE_MM: f64 = 1.0e-6;
/// Corners closer than this to flat or to folding back are left sharp.
const MIN_ANGLE: f64 = 1.0e-6;

/// The radius of vertex `index`; a missing radius is 0.
pub(crate) fn radius_at(radii: &[f64], index: usize) -> f64 {
    radii.get(index).copied().unwrap_or(0.0)
}

/// Sets the radius of vertex `index` of a shape with `count` vertices. The list is kept to
/// `count` entries, and emptied when no vertex is rounded so that sharp shapes save as before.
pub(crate) fn set_radius(radii: &mut Vec<f64>, index: usize, count: usize, radius: f64) {
    radii.resize(count, 0.0);
    if let Some(slot) = radii.get_mut(index) {
        *slot = radius;
    }
    if radii.iter().all(|radius| *radius == 0.0) {
        radii.clear();
    }
}

pub(crate) fn valid_radii(radii: &[f64]) -> bool {
    radii
        .iter()
        .all(|radius| radius.is_finite() && *radius >= 0.0)
}

/// The closed ring through `corners`, with each corner that has a positive radius replaced by an
/// arc. A radius too large for the corner's edges is reduced so that the arc reaches at most
/// halfway along each of them, which keeps neighboring arcs apart.
pub(crate) fn round_corners(corners: &[Point], radii: &[f64]) -> Vec<Point> {
    let count = corners.len();
    if count < 3 || !radii.iter().any(|radius| *radius > 0.0) {
        return corners.to_vec();
    }
    let mut ring = Vec::with_capacity(count);
    for (index, &corner) in corners.iter().enumerate() {
        let previous = corners[(index + count - 1) % count];
        let next = corners[(index + 1) % count];
        let radius = radius_at(radii, index);
        if radius > 0.0
            && let Some(arc) = fillet(previous, corner, next, radius)
        {
            for point in arc {
                push_point(&mut ring, point);
            }
        } else {
            push_point(&mut ring, corner);
        }
    }
    while ring.len() > 1 && distance(ring[0], ring[ring.len() - 1]) < MERGE_DISTANCE_MM {
        ring.pop();
    }
    ring
}

fn push_point(ring: &mut Vec<Point>, point: Point) {
    if ring
        .last()
        .is_none_or(|last| distance(*last, point) >= MERGE_DISTANCE_MM)
    {
        ring.push(point);
    }
}

fn distance(a: Point, b: Point) -> f64 {
    (b.x - a.x).hypot(b.y - a.y)
}

/// The arc that rounds `corner` between the edges to `previous` and to `next`, from the point
/// where it leaves the first edge to the point where it meets the second. `None` for a corner
/// that can't be rounded: an edge without length, or a flat or folded-back corner.
fn fillet(previous: Point, corner: Point, next: Point, radius: f64) -> Option<Vec<Point>> {
    let to_previous = distance(corner, previous);
    let to_next = distance(corner, next);
    if to_previous < MERGE_DISTANCE_MM || to_next < MERGE_DISTANCE_MM {
        return None;
    }
    let u = (
        (previous.x - corner.x) / to_previous,
        (previous.y - corner.y) / to_previous,
    );
    let v = ((next.x - corner.x) / to_next, (next.y - corner.y) / to_next);
    let angle = (u.0 * v.0 + u.1 * v.1).clamp(-1.0, 1.0).acos();
    if angle < MIN_ANGLE || PI - angle < MIN_ANGLE {
        return None;
    }
    let half = angle * 0.5;
    let tangent = (radius / half.tan()).min(to_previous.min(to_next) * 0.5);
    let radius = tangent * half.tan();
    let start = Point::new(corner.x + u.0 * tangent, corner.y + u.1 * tangent);
    let end = Point::new(corner.x + v.0 * tangent, corner.y + v.1 * tangent);
    let bisector = (u.0 + v.0, u.1 + v.1);
    let length = bisector.0.hypot(bisector.1);
    let offset = radius / half.sin() / length;
    let center = Point::new(
        corner.x + bisector.0 * offset,
        corner.y + bisector.1 * offset,
    );

    let (sx, sy) = (start.x - center.x, start.y - center.y);
    let (ex, ey) = (end.x - center.x, end.y - center.y);
    let direction = if sx * ey - sy * ex >= 0.0 { 1.0 } else { -1.0 };
    let sweep = PI - angle;
    let segments = ((sweep / (TAU / CIRCLE_SEGMENTS as f64)).ceil() as usize).max(1);
    let first = sy.atan2(sx);
    let mut arc = Vec::with_capacity(segments + 1);
    arc.push(start);
    for step in 1..segments {
        let theta = first + direction * sweep * step as f64 / segments as f64;
        arc.push(Point::new(
            center.x + radius * theta.cos(),
            center.y + radius * theta.sin(),
        ));
    }
    arc.push(end);
    Some(arc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::signed_area;

    fn square() -> Vec<Point> {
        vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]
    }

    #[test]
    fn sharp_corners_stay_as_they_are() {
        assert_eq!(round_corners(&square(), &[]), square());
        assert_eq!(round_corners(&square(), &[0.0, 0.0, 0.0, 0.0]), square());
    }

    #[test]
    fn a_rounded_corner_is_an_arc_tangent_to_its_edges() {
        let ring = round_corners(&square(), &[0.0, 0.0, 2.0]);
        assert!(!ring.contains(&Point::new(10.0, 10.0)));
        assert!(ring.contains(&Point::new(0.0, 0.0)));
        let center = Point::new(8.0, 8.0);
        let arc: Vec<_> = ring
            .iter()
            .filter(|point| point.x > 8.0 - 1.0e-9 && point.y > 8.0 - 1.0e-9)
            .collect();
        assert!(arc.len() > 3);
        for point in &arc {
            assert!(
                (distance(**point, center) - 2.0).abs() < 1.0e-9,
                "{point:?}"
            );
        }
        assert!(
            arc.iter()
                .any(|point| distance(**point, Point::new(10.0, 8.0)) < 1.0e-9)
        );
        assert!(
            arc.iter()
                .any(|point| distance(**point, Point::new(8.0, 10.0)) < 1.0e-9)
        );
        // The rounded square loses the corner's area outside the arc.
        let lost = 4.0 - PI;
        assert!((signed_area(&ring).abs() - (100.0 - lost)).abs() < 0.05);
    }

    #[test]
    fn a_large_radius_reaches_halfway_along_the_edges() {
        let ring = round_corners(&square(), &[100.0; 4]);
        // Every corner reaches the middle of both of its edges, which gives a circle.
        for point in &ring {
            assert!((distance(*point, Point::new(5.0, 5.0)) - 5.0).abs() < 1.0e-9);
        }
        for (index, point) in ring.iter().enumerate() {
            let next = ring[(index + 1) % ring.len()];
            assert!(distance(*point, next) >= MERGE_DISTANCE_MM);
        }
    }

    #[test]
    fn flat_corners_stay_sharp() {
        let corners = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(5.0, 5.0),
        ];
        let ring = round_corners(&corners, &[0.0, 3.0]);
        assert_eq!(ring, corners);
    }

    #[test]
    fn concave_corners_round_inward() {
        // An L shape with its inner corner at (5, 5).
        let corners = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 5.0),
            Point::new(5.0, 5.0),
            Point::new(5.0, 10.0),
            Point::new(0.0, 10.0),
        ];
        let ring = round_corners(&corners, &[0.0, 0.0, 0.0, 1.0]);
        let center = Point::new(6.0, 6.0);
        let arc: Vec<_> = ring
            .iter()
            .filter(|point| point.x > 5.0 - 1.0e-9 && point.x < 6.0 + 1.0e-9)
            .filter(|point| point.y > 5.0 - 1.0e-9 && point.y < 6.0 + 1.0e-9)
            .collect();
        assert!(arc.len() > 3);
        for point in arc {
            assert!((distance(*point, center) - 1.0).abs() < 1.0e-9);
        }
        assert!(signed_area(&ring).abs() > 75.0);
    }

    #[test]
    fn setting_a_radius_keeps_the_list_to_the_vertex_count() {
        let mut radii = Vec::new();
        set_radius(&mut radii, 2, 4, 3.0);
        assert_eq!(radii, vec![0.0, 0.0, 3.0, 0.0]);
        set_radius(&mut radii, 2, 4, 0.0);
        assert!(radii.is_empty());
        let mut radii = vec![1.0; 6];
        set_radius(&mut radii, 0, 4, 2.0);
        assert_eq!(radii, vec![2.0, 1.0, 1.0, 1.0]);
        assert!(valid_radii(&radii));
        assert!(!valid_radii(&[-1.0]));
        assert!(!valid_radii(&[f64::NAN]));
    }
}
