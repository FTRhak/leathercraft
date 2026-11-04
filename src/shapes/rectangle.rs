use super::{to_local, to_world};
use crate::model::Point;
use serde::{Deserialize, Serialize};

/// `rotation` is in radians, around the center.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rectangle {
    pub center: Point,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    /// Corner radii by vertex; a missing one is 0. See `corners::round_corners`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub radii: Vec<f64>,
}

impl Rectangle {
    /// An axis-aligned rectangle with `a` and `b` as opposite corners, or `None` if it has no area.
    pub fn from_corners(a: Point, b: Point) -> Option<Self> {
        if a.x == b.x || a.y == b.y {
            return None;
        }
        Some(Self {
            center: Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5),
            width: (b.x - a.x).abs(),
            height: (b.y - a.y).abs(),
            rotation: 0.0,
            radii: Vec::new(),
        })
    }

    /// The outline, with rounded corners.
    pub fn ring(&self) -> Vec<Point> {
        super::corners::round_corners(&self.corners(), &self.radii)
    }

    /// The four corners, before rounding.
    pub fn corners(&self) -> Vec<Point> {
        let (x, y) = (self.width * 0.5, self.height * 0.5);
        [(-x, -y), (x, -y), (x, y), (-x, y)]
            .into_iter()
            .map(|(px, py)| to_world(self.center, self.rotation, px, py))
            .collect()
    }

    /// The handles are the corners; the opposite corner stays in place.
    pub fn with_handle_at(&self, handle: usize, to: Point) -> Self {
        let Some(&fixed) = self.corners().get((handle + 2) % 4) else {
            return self.clone();
        };
        let (x, y) = to_local(fixed, self.rotation, to);
        Self {
            center: to_world(fixed, self.rotation, x * 0.5, y * 0.5),
            width: x.abs(),
            height: y.abs(),
            rotation: self.rotation,
            radii: self.radii.clone(),
        }
    }

    pub(crate) fn has_valid_size(&self) -> bool {
        super::positive(self.width)
            && super::positive(self.height)
            && self.rotation.is_finite()
            && super::corners::valid_radii(&self.radii)
    }
}

#[cfg(test)]
mod tests {
    use super::super::Shape;
    use crate::model::Point;

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
    fn rotation_turns_a_primitive_around_its_center() {
        let shape = Shape::Rectangle(super::Rectangle {
            center: Point::new(0.0, 0.0),
            width: 4.0,
            height: 2.0,
            rotation: std::f64::consts::FRAC_PI_2,
            radii: Vec::new(),
        });
        let bounds = shape.bounds().unwrap();
        assert!((bounds.width() - 2.0).abs() < 1.0e-9);
        assert!((bounds.height() - 4.0).abs() < 1.0e-9);
    }

    #[test]
    fn degenerate_rectangles_are_rejected() {
        let point = Point::new(3.0, 3.0);
        assert_eq!(Shape::rectangle_from_corners(point, point), None);
        assert_eq!(
            Shape::rectangle_from_corners(point, Point::new(3.0, 8.0)),
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
    }
}
