use crate::model::Point;
use serde::{Deserialize, Serialize};

/// A free outline; its vertices are its handles.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    pub points: Vec<Point>,
}

impl Polygon {
    pub fn translate(&mut self, dx: f64, dy: f64) {
        for point in &mut self.points {
            point.x += dx;
            point.y += dy;
        }
    }

    pub fn with_handle_at(&self, handle: usize, to: Point) -> Self {
        let mut polygon = self.clone();
        if let Some(point) = polygon.points.get_mut(handle) {
            *point = to;
        }
        polygon
    }

    /// The ends of an edge: edge `i` runs from vertex `i` to the next one, wrapping around.
    pub fn edge(&self, edge: usize) -> Option<(Point, Point)> {
        let count = self.points.len();
        (count >= 2 && edge < count).then(|| (self.points[edge], self.points[(edge + 1) % count]))
    }

    pub fn edge_length(&self, edge: usize) -> Option<f64> {
        self.edge(edge).map(|(a, b)| (b.x - a.x).hypot(b.y - a.y))
    }

    /// The polygon with an edge resized to `length` along its own direction. The edge's start
    /// stays in place and its end moves, so the next edge changes too. `None` for an edge
    /// without a direction or a length that isn't positive.
    pub fn with_edge_length(&self, edge: usize, length: f64) -> Option<Self> {
        let (a, b) = self.edge(edge)?;
        let current = (b.x - a.x).hypot(b.y - a.y);
        if !(current > 0.0 && length.is_finite() && length > 0.0) {
            return None;
        }
        let scale = length / current;
        let end = Point::new(a.x + (b.x - a.x) * scale, a.y + (b.y - a.y) * scale);
        Some(self.with_handle_at((edge + 1) % self.points.len(), end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::test_support::assert_near;

    fn square() -> Polygon {
        Polygon {
            points: vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(10.0, 10.0),
                Point::new(0.0, 10.0),
            ],
        }
    }

    #[test]
    fn edges_wrap_around() {
        let polygon = square();
        assert_eq!(polygon.edge_length(0), Some(10.0));
        assert_eq!(
            polygon.edge(3),
            Some((Point::new(0.0, 10.0), Point::new(0.0, 0.0)))
        );
        assert_eq!(polygon.edge(4), None);
    }

    #[test]
    fn resizing_an_edge_moves_its_end() {
        let resized = square().with_edge_length(0, 25.0).unwrap();
        assert_near(resized.points[0], Point::new(0.0, 0.0));
        assert_near(resized.points[1], Point::new(25.0, 0.0));
        assert_eq!(resized.edge_length(0), Some(25.0));
        // The closing edge moves the first vertex.
        let resized = square().with_edge_length(3, 4.0).unwrap();
        assert_near(resized.points[0], Point::new(0.0, 6.0));
    }

    #[test]
    fn resizing_rejects_degenerate_input() {
        assert_eq!(square().with_edge_length(0, 0.0), None);
        assert_eq!(square().with_edge_length(0, f64::NAN), None);
        assert_eq!(square().with_edge_length(9, 5.0), None);
        let mut polygon = square();
        polygon.points[1] = polygon.points[0];
        assert_eq!(polygon.with_edge_length(0, 5.0), None);
    }
}
