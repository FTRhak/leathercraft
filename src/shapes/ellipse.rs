use super::{to_local, to_world};
use crate::model::Point;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

/// Above the editor's limit for edge length labels, which would clutter an ellipse.
pub const CIRCLE_SEGMENTS: usize = 72;

/// `rotation` is in radians, around the center.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {
    pub center: Point,
    pub radius_x: f64,
    pub radius_y: f64,
    pub rotation: f64,
}

impl Ellipse {
    /// An ellipse around `center` whose radii reach `edge` along each axis, or a circle through
    /// `edge`. `None` if it has no area.
    pub fn from_center(center: Point, edge: Point, circle: bool) -> Option<Self> {
        let (radius_x, radius_y) = if circle {
            let radius = (edge.x - center.x).hypot(edge.y - center.y);
            (radius, radius)
        } else {
            ((edge.x - center.x).abs(), (edge.y - center.y).abs())
        };
        if radius_x == 0.0 || radius_y == 0.0 {
            return None;
        }
        Some(Self {
            center,
            radius_x,
            radius_y,
            rotation: 0.0,
        })
    }

    pub fn ring(&self) -> Vec<Point> {
        (0..CIRCLE_SEGMENTS)
            .map(|index| {
                let angle = TAU * index as f64 / CIRCLE_SEGMENTS as f64;
                to_world(
                    self.center,
                    self.rotation,
                    self.radius_x * angle.cos(),
                    self.radius_y * angle.sin(),
                )
            })
            .collect()
    }

    /// The ends of the axes.
    pub fn handles(&self) -> Vec<Point> {
        let (rx, ry) = (self.radius_x, self.radius_y);
        [(rx, 0.0), (0.0, ry), (-rx, 0.0), (0.0, -ry)]
            .into_iter()
            .map(|(x, y)| to_world(self.center, self.rotation, x, y))
            .collect()
    }

    /// The center stays in place; the handle's axis follows.
    pub fn with_handle_at(&self, handle: usize, to: Point) -> Self {
        let mut ellipse = self.clone();
        let (x, y) = to_local(self.center, self.rotation, to);
        if handle.is_multiple_of(2) {
            ellipse.radius_x = x.abs();
        } else {
            ellipse.radius_y = y.abs();
        }
        ellipse
    }

    pub(crate) fn has_valid_size(&self) -> bool {
        super::positive(self.radius_x)
            && super::positive(self.radius_y)
            && self.rotation.is_finite()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Shape, test_support::assert_near};
    use super::CIRCLE_SEGMENTS;
    use crate::model::Point;

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
    fn degenerate_ellipses_are_rejected() {
        let point = Point::new(3.0, 3.0);
        assert_eq!(Shape::ellipse_from_center(point, point, true), None);
        assert_eq!(
            Shape::ellipse_from_center(point, Point::new(8.0, 3.0), false),
            None
        );
    }

    #[test]
    fn dragging_an_axis_end_keeps_the_center() {
        let ellipse =
            Shape::ellipse_from_center(Point::new(0.0, 0.0), Point::new(5.0, 5.0), false).unwrap();
        let wider = ellipse.with_handle_at(2, Point::new(-8.0, 3.0));
        assert_near(wider.handles()[0], Point::new(8.0, 0.0));
        assert_near(wider.handles()[1], Point::new(0.0, 5.0));
    }
}
