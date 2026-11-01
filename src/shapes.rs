use crate::model::Point;
use std::f64::consts::TAU;

/// Above the editor's limit for edge length labels, which would clutter a circle.
const CIRCLE_SEGMENTS: usize = 72;

/// An axis-aligned rectangle with `a` and `b` as opposite corners, or `None` if it has no area.
pub fn rectangle(a: Point, b: Point) -> Option<Vec<Point>> {
    if a.x == b.x || a.y == b.y {
        return None;
    }
    let (min_x, max_x) = (a.x.min(b.x), a.x.max(b.x));
    let (min_y, max_y) = (a.y.min(b.y), a.y.max(b.y));
    Some(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ])
}

/// A polygon approximating the circle around `center` through `edge`, or `None` if it has no area.
pub fn circle(center: Point, edge: Point) -> Option<Vec<Point>> {
    let radius = (edge.x - center.x).hypot(edge.y - center.y);
    if radius == 0.0 {
        return None;
    }
    Some(
        (0..CIRCLE_SEGMENTS)
            .map(|index| {
                let angle = TAU * index as f64 / CIRCLE_SEGMENTS as f64;
                Point::new(
                    center.x + radius * angle.cos(),
                    center.y + radius * angle.sin(),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PatternPiece, Project};

    fn validate(points: Vec<Point>) -> Result<(), crate::i18n::Message> {
        let mut project = Project::default();
        project.add_piece(PatternPiece::new(1, "Shape", points));
        project.validate()
    }

    #[test]
    fn rectangle_is_the_same_for_any_drag_direction() {
        let expected = rectangle(Point::new(1.0, 2.0), Point::new(4.0, 6.0));
        assert_eq!(
            expected.as_deref(),
            Some(
                &[
                    Point::new(1.0, 2.0),
                    Point::new(4.0, 2.0),
                    Point::new(4.0, 6.0),
                    Point::new(1.0, 6.0),
                ][..]
            )
        );
        assert_eq!(
            rectangle(Point::new(4.0, 6.0), Point::new(1.0, 2.0)),
            expected
        );
        assert_eq!(
            rectangle(Point::new(1.0, 6.0), Point::new(4.0, 2.0)),
            expected
        );
        assert_eq!(
            rectangle(Point::new(4.0, 2.0), Point::new(1.0, 6.0)),
            expected
        );
    }

    #[test]
    fn circle_points_are_at_the_radius() {
        let center = Point::new(10.0, -5.0);
        let points = circle(center, Point::new(13.0, -1.0)).expect("the radius is not zero");
        assert_eq!(points.len(), CIRCLE_SEGMENTS);
        for point in points {
            let radius = (point.x - center.x).hypot(point.y - center.y);
            assert!((radius - 5.0).abs() < 1.0e-9);
        }
    }

    #[test]
    fn degenerate_shapes_are_rejected() {
        let point = Point::new(3.0, 3.0);
        assert_eq!(rectangle(point, point), None);
        assert_eq!(rectangle(point, Point::new(3.0, 8.0)), None);
        assert_eq!(rectangle(point, Point::new(8.0, 3.0)), None);
        assert_eq!(circle(point, point), None);
    }

    #[test]
    fn shapes_are_valid_pieces() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(30.0, 20.0);
        assert_eq!(validate(rectangle(a, b).expect("it has an area")), Ok(()));
        assert_eq!(validate(circle(a, b).expect("it has an area")), Ok(()));
    }
}
