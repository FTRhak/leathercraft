mod corners;
mod ellipse;
mod line;
mod polygon;
mod rectangle;
mod regular_polygon;
mod stitch;

pub use ellipse::{CIRCLE_SEGMENTS, Ellipse};
pub use line::{DOT_MM, Line, LineStyle, MAX_DASHES_PER_LINE, MIN_DASH_MM};
pub use polygon::Polygon;
pub use rectangle::Rectangle;
pub use regular_polygon::{REGULAR_POLYGON_SIDES, RegularPolygon};
pub use stitch::{
    DEFAULT_PITCH_MM, HOLE_SEGMENTS, HOLE_SLANT, Hole, HoleKind, MAX_HOLE_MM, MAX_HOLES_PER_LINE,
    MAX_PITCH_MM, MIN_HOLE_MM, MIN_PITCH_MM, PITCH_PRESETS, Run, StitchBand, StitchLine,
    hole_outline,
};

use crate::model::{Bounds, Point, ring_bounds, ring_contains};
use serde::{Deserialize, Serialize};

/// A closed outline, or an open line that only marks a piece. Primitives keep their parameters,
/// so an ellipse stays editable as one; `ring` turns any closed shape into the polygon that
/// geometry works with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Shape {
    Polygon(Polygon),
    Rectangle(Rectangle),
    Ellipse(Ellipse),
    RegularPolygon(RegularPolygon),
    /// Open, so it is never a base and takes no part in a piece's outline.
    Line(Line),
    /// Open, like a line: a path of stitch holes.
    StitchLine(StitchLine),
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

    /// See `Line::from_ends`.
    pub fn line_from_ends(start: Point, end: Point, snap: bool, style: LineStyle) -> Option<Self> {
        Line::from_ends(start, end, snap, style).map(Self::Line)
    }

    /// Whether the shape is an open line rather than an outline.
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Line(_) | Self::StitchLine(_))
    }

    /// The distance from the point to an open shape, or `None` for an outline.
    pub fn distance_to(&self, point: Point) -> Option<f64> {
        match self {
            Self::Line(line) => Some(line.distance_to(point)),
            Self::StitchLine(line) => Some(line.distance_to(point)),
            _ => None,
        }
    }

    /// The closed polygon of the shape, without a repeated closing point. For an open line it
    /// is just its two ends, and for a stitch line its path; neither is closed.
    pub fn ring(&self) -> Vec<Point> {
        match self {
            Self::Line(line) => vec![line.start, line.end],
            Self::StitchLine(line) => line.path(),
            Self::Polygon(polygon) => polygon.ring(),
            Self::Rectangle(rectangle) => rectangle.ring(),
            Self::Ellipse(ellipse) => ellipse.ring(),
            Self::RegularPolygon(polygon) => polygon.ring(),
        }
    }

    /// The shape as a free polygon; a polygon stays as it is. A rectangle or a regular polygon
    /// keeps its corners and their radii, and an ellipse becomes the points of its ring.
    pub fn to_polygon(&self) -> Self {
        match self {
            Self::Polygon(_) | Self::Line(_) | Self::StitchLine(_) => self.clone(),
            Self::Rectangle(Rectangle { radii, .. })
            | Self::RegularPolygon(RegularPolygon { radii, .. }) => Self::Polygon(Polygon {
                points: self.corners().unwrap_or_default(),
                radii: radii.clone(),
            }),
            Self::Ellipse(_) => Self::Polygon(Polygon::new(self.ring())),
        }
    }

    /// The vertices before their corners are rounded, or `None` for an ellipse or a line, which
    /// have none. A stitch line's are its path's vertices, ends included.
    pub fn corners(&self) -> Option<Vec<Point>> {
        match self {
            Self::Polygon(polygon) => Some(polygon.points.clone()),
            Self::StitchLine(line) => Some(line.points.clone()),
            Self::Rectangle(rectangle) => Some(rectangle.corners()),
            Self::RegularPolygon(polygon) => Some(polygon.corners()),
            Self::Ellipse(_) | Self::Line(_) => None,
        }
    }

    fn radii(&self) -> &[f64] {
        match self {
            Self::Polygon(Polygon { radii, .. })
            | Self::Rectangle(Rectangle { radii, .. })
            | Self::RegularPolygon(RegularPolygon { radii, .. })
            | Self::StitchLine(StitchLine { radii, .. }) => radii,
            Self::Ellipse(_) | Self::Line(_) => &[],
        }
    }

    /// The radius that rounds vertex `index` of `corners`, or `None` if there is no such vertex
    /// or it can't be rounded, as the ends of a stitch line.
    pub fn corner_radius(&self, index: usize) -> Option<f64> {
        let count = self.corners()?.len();
        (index < count && self.can_round(index, count))
            .then(|| corners::radius_at(self.radii(), index))
    }

    fn can_round(&self, index: usize, count: usize) -> bool {
        !matches!(self, Self::StitchLine(_)) || (index > 0 && index + 1 < count)
    }

    /// The shape with vertex `index` rounded by `radius`; 0 makes it sharp. `None` if there is
    /// no such vertex or the radius is negative or not finite. A radius too large for the
    /// corner's edges is kept, and reduced only where the ring is built.
    pub fn with_corner_radius(&self, index: usize, radius: f64) -> Option<Self> {
        let count = self.corners()?.len();
        if index >= count || !self.can_round(index, count) || !radius.is_finite() || radius < 0.0 {
            return None;
        }
        let mut shape = self.clone();
        match &mut shape {
            Self::Polygon(Polygon { radii, .. })
            | Self::Rectangle(Rectangle { radii, .. })
            | Self::RegularPolygon(RegularPolygon { radii, .. })
            | Self::StitchLine(StitchLine { radii, .. }) => {
                corners::set_radius(radii, index, count, radius);
            }
            Self::Ellipse(_) | Self::Line(_) => return None,
        }
        Some(shape)
    }

    pub fn bounds(&self) -> Option<Bounds> {
        ring_bounds(&self.ring())
    }

    /// Whether the point is inside the outline; always false for an open shape.
    pub fn contains(&self, point: Point) -> bool {
        !self.is_open() && ring_contains(&self.ring(), point)
    }

    pub fn center(&self) -> Point {
        match self {
            Self::Rectangle(Rectangle { center, .. })
            | Self::Ellipse(Ellipse { center, .. })
            | Self::RegularPolygon(RegularPolygon { center, .. }) => *center,
            Self::Line(line) => line.midpoint(),
            Self::Polygon(_) | Self::StitchLine(_) => {
                self.bounds().map_or_else(Point::default, |bounds| {
                    Point::new(
                        (bounds.min_x + bounds.max_x) * 0.5,
                        (bounds.min_y + bounds.max_y) * 0.5,
                    )
                })
            }
        }
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        match self {
            Self::Polygon(polygon) => polygon.translate(dx, dy),
            Self::Line(line) => line.translate(dx, dy),
            Self::StitchLine(line) => line.translate(dx, dy),
            Self::Rectangle(Rectangle { center, .. })
            | Self::Ellipse(Ellipse { center, .. })
            | Self::RegularPolygon(RegularPolygon { center, .. }) => {
                center.x += dx;
                center.y += dy;
            }
        }
    }

    /// Points that can be dragged to reshape the shape: the vertices of a polygon, the corners
    /// of a rectangle or a regular polygon, the ends of an ellipse's axes and the ends of a line.
    pub fn handles(&self) -> Vec<Point> {
        match self {
            Self::Polygon(polygon) => polygon.points.clone(),
            Self::Rectangle(rectangle) => rectangle.corners(),
            Self::RegularPolygon(polygon) => polygon.corners(),
            Self::Ellipse(ellipse) => ellipse.handles(),
            Self::Line(line) => line.handles(),
            Self::StitchLine(line) => line.handles(),
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
            Self::Line(line) => Self::Line(line.with_handle_at(handle, to)),
            Self::StitchLine(line) => Self::StitchLine(line.with_handle_at(handle, to)),
        }
    }

    /// Whether the parameters describe a shape with an area, or a line with a length and a valid
    /// style. Polygons are checked by their ring.
    pub(crate) fn has_valid_size(&self) -> bool {
        match self {
            Self::Polygon(polygon) => polygon.has_valid_size(),
            Self::Rectangle(rectangle) => rectangle.has_valid_size(),
            Self::Ellipse(ellipse) => ellipse.has_valid_size(),
            Self::RegularPolygon(polygon) => polygon.has_valid_size(),
            Self::Line(line) => line.has_valid_size(),
            Self::StitchLine(line) => line.has_valid_size(),
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
        let mut polygon = Shape::Polygon(Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
        ]));
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

    #[test]
    fn rounding_a_corner_keeps_the_handles_on_the_corners() {
        let shape =
            Shape::rectangle_from_corners(Point::new(0.0, 0.0), Point::new(20.0, 10.0)).unwrap();
        let rounded = shape.with_corner_radius(2, 3.0).unwrap();
        assert_eq!(rounded.handles(), shape.handles());
        assert_eq!(rounded.corners(), shape.corners());
        assert_eq!(rounded.corner_radius(2), Some(3.0));
        assert_eq!(rounded.corner_radius(0), Some(0.0));
        assert_eq!(rounded.corner_radius(4), None);
        assert!(rounded.ring().len() > 4);
        assert!(!rounded.ring().contains(&Point::new(20.0, 10.0)));
        assert_eq!(validate(rounded.clone()), Ok(()));
        // Resizing keeps the rounding.
        let resized = rounded.with_handle_at(2, Point::new(30.0, 15.0));
        assert_eq!(resized.corner_radius(2), Some(3.0));
        // Back to sharp, the shape saves as before.
        assert_eq!(rounded.with_corner_radius(2, 0.0), Some(shape));
    }

    #[test]
    fn invalid_corner_radii_are_rejected() {
        let shape =
            Shape::rectangle_from_corners(Point::new(0.0, 0.0), Point::new(20.0, 10.0)).unwrap();
        assert_eq!(shape.with_corner_radius(0, -1.0), None);
        assert_eq!(shape.with_corner_radius(0, f64::NAN), None);
        assert_eq!(shape.with_corner_radius(4, 1.0), None);
        let ellipse =
            Shape::ellipse_from_center(Point::new(0.0, 0.0), Point::new(5.0, 5.0), false).unwrap();
        assert_eq!(ellipse.corners(), None);
        assert_eq!(ellipse.with_corner_radius(0, 1.0), None);
        let Shape::Rectangle(mut rectangle) = shape else {
            unreachable!()
        };
        rectangle.radii = vec![0.0, -2.0];
        assert!(validate(Shape::Rectangle(rectangle)).is_err());
    }

    #[test]
    fn converting_a_rounded_shape_to_a_polygon_keeps_its_radii() {
        let shape = Shape::regular_polygon_from_center(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            5,
            false,
        )
        .unwrap()
        .with_corner_radius(1, 2.0)
        .unwrap();
        let polygon = shape.to_polygon();
        assert!(matches!(polygon, Shape::Polygon(_)));
        assert_eq!(polygon.ring(), shape.ring());
        assert_eq!(polygon.corners(), shape.corners());
        assert_eq!(polygon.corner_radius(1), Some(2.0));
        assert_eq!(polygon.to_polygon(), polygon);
    }

    #[test]
    fn a_stitch_line_is_open_and_rounds_only_inner_vertices() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
        ];
        let shape = Shape::StitchLine(StitchLine::new(points, HoleKind::Round, 4.0, 1.2).unwrap());
        assert!(shape.is_open());
        assert!(!shape.contains(Point::new(8.0, 1.0)));
        assert_eq!(shape.to_polygon(), shape);
        assert_eq!(shape.corner_radius(0), None);
        assert_eq!(shape.corner_radius(2), None);
        assert_eq!(shape.corner_radius(1), Some(0.0));
        assert_eq!(shape.with_corner_radius(0, 1.0), None);
        let rounded = shape.with_corner_radius(1, 3.0).unwrap();
        assert_eq!(rounded.corner_radius(1), Some(3.0));
        assert!(rounded.ring().len() > 3);
    }

    #[test]
    fn fewer_sides_ignore_the_extra_radii() {
        let shape = Shape::regular_polygon_from_center(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            6,
            false,
        )
        .unwrap()
        .with_corner_radius(5, 2.0)
        .unwrap();
        let Shape::RegularPolygon(mut polygon) = shape else {
            unreachable!()
        };
        polygon.sides = 4;
        let shape = Shape::RegularPolygon(polygon);
        assert_eq!(shape.ring().len(), 4);
        assert_eq!(validate(shape), Ok(()));
    }
}
