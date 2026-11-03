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
}
