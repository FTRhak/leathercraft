use crate::model::{Bounds, Point, ring_bounds, ring_contains};
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;
use std::ops::RangeInclusive;

/// Above the editor's limit for edge length labels, which would clutter an ellipse.
pub const CIRCLE_SEGMENTS: usize = 72;
/// The number of sides a regular polygon can have.
pub const REGULAR_POLYGON_SIDES: RangeInclusive<usize> = 3..=64;
/// The step that Shift snaps a regular polygon's rotation to while drawing: 15°.
const ROTATION_SNAP: f64 = TAU / 24.0;

/// A closed outline. Primitives keep their parameters, so an ellipse stays editable as one;
/// `ring` turns any shape into the polygon that geometry works with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Shape {
    Polygon {
        points: Vec<Point>,
    },
    /// `rotation` is in radians, around the center.
    Rectangle {
        center: Point,
        width: f64,
        height: f64,
        rotation: f64,
    },
    /// `rotation` is in radians, around the center.
    Ellipse {
        center: Point,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
    },
    /// `radius` reaches the vertices, and `rotation` (radians) is the direction of the first
    /// one from the center.
    RegularPolygon {
        center: Point,
        radius: f64,
        sides: usize,
        rotation: f64,
    },
}

/// Turns a point in a primitive's own axes into world coordinates.
fn to_world(center: Point, rotation: f64, x: f64, y: f64) -> Point {
    let (sin, cos) = rotation.sin_cos();
    Point::new(center.x + x * cos - y * sin, center.y + x * sin + y * cos)
}

/// Turns a world point into a primitive's own axes.
fn to_local(center: Point, rotation: f64, point: Point) -> (f64, f64) {
    let (sin, cos) = rotation.sin_cos();
    let (dx, dy) = (point.x - center.x, point.y - center.y);
    (dx * cos + dy * sin, -dx * sin + dy * cos)
}

impl Shape {
    /// An axis-aligned rectangle with `a` and `b` as opposite corners, or `None` if it has no area.
    pub fn rectangle_from_corners(a: Point, b: Point) -> Option<Self> {
        if a.x == b.x || a.y == b.y {
            return None;
        }
        Some(Self::Rectangle {
            center: Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5),
            width: (b.x - a.x).abs(),
            height: (b.y - a.y).abs(),
            rotation: 0.0,
        })
    }

    /// An ellipse around `center` whose radii reach `edge` along each axis, or a circle through
    /// `edge`. `None` if it has no area.
    pub fn ellipse_from_center(center: Point, edge: Point, circle: bool) -> Option<Self> {
        let (radius_x, radius_y) = if circle {
            let radius = (edge.x - center.x).hypot(edge.y - center.y);
            (radius, radius)
        } else {
            ((edge.x - center.x).abs(), (edge.y - center.y).abs())
        };
        if radius_x == 0.0 || radius_y == 0.0 {
            return None;
        }
        Some(Self::Ellipse {
            center,
            radius_x,
            radius_y,
            rotation: 0.0,
        })
    }

    /// A regular polygon around `center` with its first vertex at `vertex`, or `None` if it has
    /// no area. `snap` rounds the rotation to 15°.
    pub fn regular_polygon_from_center(
        center: Point,
        vertex: Point,
        sides: usize,
        snap: bool,
    ) -> Option<Self> {
        let (dx, dy) = (vertex.x - center.x, vertex.y - center.y);
        let radius = dx.hypot(dy);
        if radius == 0.0 {
            return None;
        }
        let mut rotation = dy.atan2(dx);
        if snap {
            rotation = (rotation / ROTATION_SNAP).round() * ROTATION_SNAP;
        }
        Some(Self::RegularPolygon {
            center,
            radius,
            sides: sides.clamp(*REGULAR_POLYGON_SIDES.start(), *REGULAR_POLYGON_SIDES.end()),
            rotation,
        })
    }

    /// The closed polygon of the shape, without a repeated closing point.
    pub fn ring(&self) -> Vec<Point> {
        match self {
            Self::Polygon { points } => points.clone(),
            Self::Rectangle {
                center,
                width,
                height,
                rotation,
            } => {
                let (x, y) = (width * 0.5, height * 0.5);
                [(-x, -y), (x, -y), (x, y), (-x, y)]
                    .into_iter()
                    .map(|(px, py)| to_world(*center, *rotation, px, py))
                    .collect()
            }
            Self::Ellipse {
                center,
                radius_x,
                radius_y,
                rotation,
            } => (0..CIRCLE_SEGMENTS)
                .map(|index| {
                    let angle = TAU * index as f64 / CIRCLE_SEGMENTS as f64;
                    to_world(
                        *center,
                        *rotation,
                        radius_x * angle.cos(),
                        radius_y * angle.sin(),
                    )
                })
                .collect(),
            Self::RegularPolygon {
                center,
                radius,
                sides,
                rotation,
            } => (0..*sides)
                .map(|index| {
                    let angle = TAU * index as f64 / *sides as f64;
                    to_world(
                        *center,
                        *rotation,
                        radius * angle.cos(),
                        radius * angle.sin(),
                    )
                })
                .collect(),
        }
    }

    pub fn bounds(&self) -> Option<Bounds> {
        ring_bounds(&self.ring())
    }

    pub fn contains(&self, point: Point) -> bool {
        ring_contains(&self.ring(), point)
    }

    pub fn center(&self) -> Point {
        match self {
            Self::Rectangle { center, .. }
            | Self::Ellipse { center, .. }
            | Self::RegularPolygon { center, .. } => *center,
            Self::Polygon { .. } => self.bounds().map_or_else(Point::default, |bounds| {
                Point::new(
                    (bounds.min_x + bounds.max_x) * 0.5,
                    (bounds.min_y + bounds.max_y) * 0.5,
                )
            }),
        }
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        match self {
            Self::Polygon { points } => {
                for point in points {
                    point.x += dx;
                    point.y += dy;
                }
            }
            Self::Rectangle { center, .. }
            | Self::Ellipse { center, .. }
            | Self::RegularPolygon { center, .. } => {
                center.x += dx;
                center.y += dy;
            }
        }
    }

    /// Points that can be dragged to reshape the shape: the vertices of a polygon, the corners
    /// of a rectangle or a regular polygon, and the ends of an ellipse's axes.
    pub fn handles(&self) -> Vec<Point> {
        match self {
            Self::Polygon { points } => points.clone(),
            Self::Rectangle { .. } | Self::RegularPolygon { .. } => self.ring(),
            Self::Ellipse {
                center,
                radius_x,
                radius_y,
                rotation,
            } => [
                (*radius_x, 0.0),
                (0.0, *radius_y),
                (-radius_x, 0.0),
                (0.0, -radius_y),
            ]
            .into_iter()
            .map(|(x, y)| to_world(*center, *rotation, x, y))
            .collect(),
        }
    }

    /// The shape with a handle from `handles` moved to `to`. A rectangle keeps the opposite
    /// corner in place, and an ellipse keeps its center. A regular polygon keeps its center too,
    /// and turns and scales so that the vertex follows.
    pub fn with_handle_at(&self, handle: usize, to: Point) -> Self {
        let mut shape = self.clone();
        match &mut shape {
            Self::Polygon { points } => {
                if let Some(point) = points.get_mut(handle) {
                    *point = to;
                }
            }
            Self::Rectangle {
                center,
                width,
                height,
                rotation,
            } => {
                let Some(&fixed) = self.ring().get((handle + 2) % 4) else {
                    return shape;
                };
                let (x, y) = to_local(fixed, *rotation, to);
                *center = to_world(fixed, *rotation, x * 0.5, y * 0.5);
                *width = x.abs();
                *height = y.abs();
            }
            Self::Ellipse {
                center,
                radius_x,
                radius_y,
                rotation,
            } => {
                let (x, y) = to_local(*center, *rotation, to);
                if handle.is_multiple_of(2) {
                    *radius_x = x.abs();
                } else {
                    *radius_y = y.abs();
                }
            }
            Self::RegularPolygon {
                center,
                radius,
                sides,
                rotation,
            } => {
                let (dx, dy) = (to.x - center.x, to.y - center.y);
                *radius = dx.hypot(dy);
                *rotation = dy.atan2(dx) - TAU * (handle % *sides) as f64 / *sides as f64;
            }
        }
        shape
    }

    /// Whether the parameters describe a shape with an area. Polygons are checked by their ring.
    pub(crate) fn has_valid_size(&self) -> bool {
        let positive = |value: f64| value.is_finite() && value > 0.0;
        match self {
            Self::Polygon { .. } => true,
            Self::Rectangle {
                width,
                height,
                rotation,
                ..
            } => positive(*width) && positive(*height) && rotation.is_finite(),
            Self::Ellipse {
                radius_x,
                radius_y,
                rotation,
                ..
            } => positive(*radius_x) && positive(*radius_y) && rotation.is_finite(),
            Self::RegularPolygon {
                radius,
                sides,
                rotation,
                ..
            } => positive(*radius) && REGULAR_POLYGON_SIDES.contains(sides) && rotation.is_finite(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PatternPiece, Project};

    fn validate(shape: Shape) -> Result<(), crate::i18n::Message> {
        let mut project = Project::default();
        project.add_piece(PatternPiece::new(1, "Shape", shape));
        project.validate()
    }

    fn assert_near(a: Point, b: Point) {
        assert!(
            (a.x - b.x).abs() < 1.0e-9 && (a.y - b.y).abs() < 1.0e-9,
            "{a:?} != {b:?}"
        );
    }

    #[test]
    fn rectangle_is_the_same_for_any_drag_direction() {
        let expected = Shape::rectangle_from_corners(Point::new(1.0, 2.0), Point::new(4.0, 6.0));
        assert_eq!(
            expected.as_ref().map(Shape::ring),
            Some(vec![
                Point::new(1.0, 2.0),
                Point::new(4.0, 2.0),
                Point::new(4.0, 6.0),
                Point::new(1.0, 6.0),
            ])
        );
        for (a, b) in [((4.0, 6.0), (1.0, 2.0)), ((1.0, 6.0), (4.0, 2.0))] {
            assert_eq!(
                Shape::rectangle_from_corners(Point::new(a.0, a.1), Point::new(b.0, b.1)),
                expected
            );
        }
    }

    #[test]
    fn ellipse_points_are_at_the_radii() {
        let center = Point::new(10.0, -5.0);
        let circle = Shape::ellipse_from_center(center, Point::new(13.0, -1.0), true)
            .expect("the radius is not zero");
        let points = circle.ring();
        assert_eq!(points.len(), CIRCLE_SEGMENTS);
        for point in points {
            let radius = (point.x - center.x).hypot(point.y - center.y);
            assert!((radius - 5.0).abs() < 1.0e-9);
        }

        let ellipse = Shape::ellipse_from_center(center, Point::new(13.0, -1.0), false).unwrap();
        let bounds = ellipse.bounds().unwrap();
        assert!((bounds.width() - 6.0).abs() < 1.0e-9);
        assert!((bounds.height() - 8.0).abs() < 1.0e-9);
    }

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
        let too_many = Shape::RegularPolygon {
            center: a,
            radius: 10.0,
            sides: 65,
            rotation: 0.0,
        };
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

    #[test]
    fn rotation_turns_a_primitive_around_its_center() {
        let shape = Shape::Rectangle {
            center: Point::new(0.0, 0.0),
            width: 4.0,
            height: 2.0,
            rotation: std::f64::consts::FRAC_PI_2,
        };
        let bounds = shape.bounds().unwrap();
        assert!((bounds.width() - 2.0).abs() < 1.0e-9);
        assert!((bounds.height() - 4.0).abs() < 1.0e-9);
    }

    #[test]
    fn degenerate_shapes_are_rejected() {
        let point = Point::new(3.0, 3.0);
        assert_eq!(Shape::rectangle_from_corners(point, point), None);
        assert_eq!(
            Shape::rectangle_from_corners(point, Point::new(3.0, 8.0)),
            None
        );
        assert_eq!(Shape::ellipse_from_center(point, point, true), None);
        assert_eq!(
            Shape::ellipse_from_center(point, Point::new(8.0, 3.0), false),
            None
        );
    }

    #[test]
    fn dragging_a_corner_keeps_the_opposite_one() {
        let shape =
            Shape::rectangle_from_corners(Point::new(0.0, 0.0), Point::new(10.0, 10.0)).unwrap();
        let resized = shape.with_handle_at(2, Point::new(20.0, 5.0));
        assert_eq!(
            resized,
            Shape::rectangle_from_corners(Point::new(0.0, 0.0), Point::new(20.0, 5.0)).unwrap()
        );

        let ellipse =
            Shape::ellipse_from_center(Point::new(0.0, 0.0), Point::new(5.0, 5.0), false).unwrap();
        let wider = ellipse.with_handle_at(2, Point::new(-8.0, 3.0));
        assert_near(wider.handles()[0], Point::new(8.0, 0.0));
        assert_near(wider.handles()[1], Point::new(0.0, 5.0));
    }

    #[test]
    fn translating_moves_every_kind_of_shape() {
        let mut polygon = Shape::Polygon {
            points: vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)],
        };
        polygon.translate(2.0, 3.0);
        assert_eq!(
            polygon.handles(),
            vec![Point::new(2.0, 3.0), Point::new(3.0, 3.0)]
        );
        let mut ellipse =
            Shape::ellipse_from_center(Point::new(0.0, 0.0), Point::new(1.0, 1.0), false).unwrap();
        ellipse.translate(2.0, 3.0);
        assert_eq!(ellipse.center(), Point::new(2.0, 3.0));
    }

    #[test]
    fn shapes_are_valid_pieces() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(30.0, 20.0);
        assert_eq!(
            validate(Shape::rectangle_from_corners(a, b).unwrap()),
            Ok(())
        );
        assert_eq!(
            validate(Shape::ellipse_from_center(a, b, false).unwrap()),
            Ok(())
        );
    }
}
