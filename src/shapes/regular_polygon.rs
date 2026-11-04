use super::to_world;
use crate::model::Point;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;
use std::ops::RangeInclusive;

/// The number of sides a regular polygon can have.
pub const REGULAR_POLYGON_SIDES: RangeInclusive<usize> = 3..=64;
/// The step that Shift snaps a regular polygon's rotation to while drawing: 15°.
const ROTATION_SNAP: f64 = TAU / 24.0;

/// `radius` reaches the vertices, and `rotation` (radians) is the direction of the first one
/// from the center.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegularPolygon {
    pub center: Point,
    pub radius: f64,
    pub sides: usize,
    pub rotation: f64,
    /// Corner radii by vertex; a missing one is 0. See `corners::round_corners`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub radii: Vec<f64>,
}

impl RegularPolygon {
    /// A regular polygon around `center` with its first vertex at `vertex`, or `None` if it has
    /// no area. `snap` rounds the rotation to 15°.
    pub fn from_center(center: Point, vertex: Point, sides: usize, snap: bool) -> Option<Self> {
        let (dx, dy) = (vertex.x - center.x, vertex.y - center.y);
        let radius = dx.hypot(dy);
        if radius == 0.0 {
            return None;
        }
        let mut rotation = dy.atan2(dx);
        if snap {
            rotation = (rotation / ROTATION_SNAP).round() * ROTATION_SNAP;
        }
        Some(Self {
            center,
            radius,
            sides: sides.clamp(*REGULAR_POLYGON_SIDES.start(), *REGULAR_POLYGON_SIDES.end()),
            rotation,
            radii: Vec::new(),
        })
    }

    /// The outline, with rounded corners.
    pub fn ring(&self) -> Vec<Point> {
        super::corners::round_corners(&self.corners(), &self.radii)
    }

    /// The vertices, before rounding.
    pub fn corners(&self) -> Vec<Point> {
        (0..self.sides)
            .map(|index| {
                let angle = TAU * index as f64 / self.sides as f64;
                to_world(
                    self.center,
                    self.rotation,
                    self.radius * angle.cos(),
                    self.radius * angle.sin(),
                )
            })
            .collect()
    }

    /// The handles are the vertices. The center stays in place, and the polygon turns and scales
    /// so that the vertex follows.
    pub fn with_handle_at(&self, handle: usize, to: Point) -> Self {
        let (dx, dy) = (to.x - self.center.x, to.y - self.center.y);
        Self {
            radius: dx.hypot(dy),
            rotation: dy.atan2(dx) - TAU * (handle % self.sides) as f64 / self.sides as f64,
            ..self.clone()
        }
    }

    pub(crate) fn has_valid_size(&self) -> bool {
        super::positive(self.radius)
            && REGULAR_POLYGON_SIDES.contains(&self.sides)
            && self.rotation.is_finite()
            && super::corners::valid_radii(&self.radii)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        Shape,
        test_support::{assert_near, validate},
    };
    use super::RegularPolygon;
    use crate::model::Point;

    #[test]
    fn regular_polygon_starts_at_the_dragged_vertex() {
        let center = Point::new(5.0, 5.0);
        let vertex = Point::new(5.0, 15.0);
        let hexagon = Shape::regular_polygon_from_center(center, vertex, 6, false).unwrap();
        let points = hexagon.ring();
        assert_eq!(points.len(), 6);
        assert_near(points[0], vertex);
        for (index, point) in points.iter().enumerate() {
            assert!(((point.x - center.x).hypot(point.y - center.y) - 10.0).abs() < 1.0e-9);
            let next = points[(index + 1) % points.len()];
            assert!(((next.x - point.x).hypot(next.y - point.y) - 10.0).abs() < 1.0e-9);
        }

        let snapped =
            Shape::regular_polygon_from_center(center, Point::new(15.0, 6.0), 3, true).unwrap();
        assert_near(
            snapped.ring()[0],
            Point::new(5.0 + 1.0_f64.hypot(10.0), 5.0),
        );
        assert_eq!(
            Shape::regular_polygon_from_center(center, center, 5, false),
            None
        );
    }

    #[test]
    fn regular_polygon_sides_stay_in_range() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 0.0);
        for (sides, expected) in [(1, 3), (64, 64), (100, 64)] {
            let shape = Shape::regular_polygon_from_center(a, b, sides, false).unwrap();
            assert_eq!(shape.ring().len(), expected);
            assert_eq!(validate(shape), Ok(()));
        }
        let too_many = Shape::RegularPolygon(RegularPolygon {
            center: a,
            radius: 10.0,
            sides: 65,
            rotation: 0.0,
            radii: Vec::new(),
        });
        assert!(validate(too_many).is_err());
    }

    #[test]
    fn dragging_a_regular_polygon_vertex_keeps_the_center() {
        let square = Shape::regular_polygon_from_center(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            4,
            false,
        )
        .unwrap();
        let turned = square.with_handle_at(1, Point::new(-20.0, 0.0));
        assert_eq!(turned.center(), Point::new(0.0, 0.0));
        assert_near(turned.handles()[1], Point::new(-20.0, 0.0));
        assert_near(turned.handles()[0], Point::new(0.0, 20.0));
    }
}
