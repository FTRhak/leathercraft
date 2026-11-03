mod ellipse;
mod polygon;
mod rectangle;
mod regular_polygon;

pub use ellipse::{CIRCLE_SEGMENTS, Ellipse};
pub use polygon::Polygon;
pub use rectangle::Rectangle;
pub use regular_polygon::{REGULAR_POLYGON_SIDES, RegularPolygon};

use crate::model::{Bounds, Point, ring_bounds, ring_contains};
use serde::{Deserialize, Serialize};

/// A closed outline. Primitives keep their parameters, so an ellipse stays editable as one;
/// `ring` turns any shape into the polygon that geometry works with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Shape {
    Polygon(Polygon),
    Rectangle(Rectangle),
    Ellipse(Ellipse),
    RegularPolygon(RegularPolygon),
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

fn positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

impl Shape {
    /// See `Rectangle::from_corners`.
    pub fn rectangle_from_corners(a: Point, b: Point) -> Option<Self> {
        Rectangle::from_corners(a, b).map(Self::Rectangle)
    }

    /// See `Ellipse::from_center`.
    pub fn ellipse_from_center(center: Point, edge: Point, circle: bool) -> Option<Self> {
        Ellipse::from_center(center, edge, circle).map(Self::Ellipse)
    }

    /// See `RegularPolygon::from_center`.
    pub fn regular_polygon_from_center(
        center: Point,
        vertex: Point,
        sides: usize,
        snap: bool,
    ) -> Option<Self> {
        RegularPolygon::from_center(center, vertex, sides, snap).map(Self::RegularPolygon)
    }

    /// The closed polygon of the shape, without a repeated closing point.
    pub fn ring(&self) -> Vec<Point> {
        match self {
            Self::Polygon(polygon) => polygon.points.clone(),
            Self::Rectangle(rectangle) => rectangle.ring(),
            Self::Ellipse(ellipse) => ellipse.ring(),
            Self::RegularPolygon(polygon) => polygon.ring(),
        }
    }

    /// The shape as a free polygon through the points of its ring; a polygon stays as it is.
    pub fn to_polygon(&self) -> Self {
        Self::Polygon(Polygon {
            points: self.ring(),
        })
    }

    pub fn bounds(&self) -> Option<Bounds> {
        ring_bounds(&self.ring())
    }

    pub fn contains(&self, point: Point) -> bool {
        ring_contains(&self.ring(), point)
    }

    pub fn center(&self) -> Point {
        match self {
            Self::Rectangle(Rectangle { center, .. })
            | Self::Ellipse(Ellipse { center, .. })
            | Self::RegularPolygon(RegularPolygon { center, .. }) => *center,
            Self::Polygon(_) => self.bounds().map_or_else(Point::default, |bounds| {
                Point::new(
                    (bounds.min_x + bounds.max_x) * 0.5,
                    (bounds.min_y + bounds.max_y) * 0.5,
                )
            }),
        }
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        match self {
            Self::Polygon(polygon) => polygon.translate(dx, dy),
            Self::Rectangle(Rectangle { center, .. })
            | Self::Ellipse(Ellipse { center, .. })
            | Self::RegularPolygon(RegularPolygon { center, .. }) => {
                center.x += dx;
                center.y += dy;
            }
        }
    }

    /// Points that can be dragged to reshape the shape: the vertices of a polygon, the corners
    /// of a rectangle or a regular polygon, and the ends of an ellipse's axes.
    pub fn handles(&self) -> Vec<Point> {
        match self {
            Self::Polygon(polygon) => polygon.points.clone(),
            Self::Rectangle(_) | Self::RegularPolygon(_) => self.ring(),
            Self::Ellipse(ellipse) => ellipse.handles(),
        }
    }

    /// The shape with a handle from `handles` moved to `to`. A rectangle keeps the opposite
    /// corner in place, and an ellipse keeps its center. A regular polygon keeps its center too,
    /// and turns and scales so that the vertex follows.
    pub fn with_handle_at(&self, handle: usize, to: Point) -> Self {
        match self {
            Self::Polygon(polygon) => Self::Polygon(polygon.with_handle_at(handle, to)),
            Self::Rectangle(rectangle) => Self::Rectangle(rectangle.with_handle_at(handle, to)),
            Self::Ellipse(ellipse) => Self::Ellipse(ellipse.with_handle_at(handle, to)),
            Self::RegularPolygon(polygon) => {
                Self::RegularPolygon(polygon.with_handle_at(handle, to))
            }
        }
    }

    /// Whether the parameters describe a shape with an area. Polygons are checked by their ring.
    pub(crate) fn has_valid_size(&self) -> bool {
        match self {
            Self::Polygon(_) => true,
            Self::Rectangle(rectangle) => rectangle.has_valid_size(),
            Self::Ellipse(ellipse) => ellipse.has_valid_size(),
            Self::RegularPolygon(polygon) => polygon.has_valid_size(),
        }
    }
}

#[cfg(test)]
mod test_support {
    use super::Shape;
    use crate::model::{PatternPiece, Point, Project};

    pub fn validate(shape: Shape) -> Result<(), crate::i18n::Message> {
        let mut project = Project::default();
        project.add_piece(PatternPiece::new(1, "Shape", shape));
        project.validate()
    }

    pub fn assert_near(a: Point, b: Point) {
        assert!(
            (a.x - b.x).abs() < 1.0e-9 && (a.y - b.y).abs() < 1.0e-9,
            "{a:?} != {b:?}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::validate;
    use super::*;

    #[test]
    fn translating_moves_every_kind_of_shape() {
        let mut polygon = Shape::Polygon(Polygon {
            points: vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)],
        });
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
    fn converting_to_a_polygon_keeps_the_ring() {
        let ellipse =
            Shape::ellipse_from_center(Point::new(0.0, 0.0), Point::new(30.0, 20.0), false)
                .unwrap();
        let polygon = ellipse.to_polygon();
        assert!(matches!(polygon, Shape::Polygon(_)));
        assert_eq!(polygon.ring(), ellipse.ring());
        assert_eq!(validate(polygon.clone()), Ok(()));
        assert_eq!(polygon.to_polygon(), polygon);
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
