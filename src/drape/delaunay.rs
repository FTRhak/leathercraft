//! Delaunay triangulation of a point set (Bowyer–Watson, with neighbor links so that each
//! insertion only visits the triangles around the new point).

use crate::model::Point;
use std::collections::HashMap;

struct Triangle {
    /// Counterclockwise.
    vertices: [usize; 3],
    /// The neighbor across the edge opposite each vertex.
    neighbors: [Option<usize>; 3],
    center: Point,
    radius2: f64,
    alive: bool,
}

fn orient(a: Point, b: Point, c: Point) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn circumcircle(a: Point, b: Point, c: Point) -> (Point, f64) {
    let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
    if d.abs() < 1.0e-300 {
        return (a, f64::INFINITY);
    }
    let (a2, b2, c2) = (
        a.x * a.x + a.y * a.y,
        b.x * b.x + b.y * b.y,
        c.x * c.x + c.y * c.y,
    );
    let center = Point::new(
        (a2 * (b.y - c.y) + b2 * (c.y - a.y) + c2 * (a.y - b.y)) / d,
        (a2 * (c.x - b.x) + b2 * (a.x - c.x) + c2 * (b.x - a.x)) / d,
    );
    let radius2 = (a.x - center.x).powi(2) + (a.y - center.y).powi(2);
    (center, radius2)
}

struct Mesh {
    points: Vec<Point>,
    triangles: Vec<Triangle>,
}

impl Mesh {
    fn add(&mut self, vertices: [usize; 3], neighbors: [Option<usize>; 3]) -> usize {
        let [a, b, c] = vertices.map(|index| self.points[index]);
        let (center, radius2) = circumcircle(a, b, c);
        self.triangles.push(Triangle {
            vertices,
            neighbors,
            center,
            radius2,
            alive: true,
        });
        self.triangles.len() - 1
    }

    fn in_circle(&self, triangle: usize, point: Point) -> bool {
        let triangle = &self.triangles[triangle];
        (point.x - triangle.center.x).powi(2) + (point.y - triangle.center.y).powi(2)
            < triangle.radius2 * (1.0 - 1.0e-12)
    }

    /// The live triangle holding the point, walking from `start`; falls back to a scan when the
    /// walk goes astray.
    fn locate(&self, start: usize, point: Point) -> Option<usize> {
        let mut current = start;
        'walk: for _ in 0..self.triangles.len() {
            let triangle = &self.triangles[current];
            for k in 0..3 {
                let a = self.points[triangle.vertices[(k + 1) % 3]];
                let b = self.points[triangle.vertices[(k + 2) % 3]];
                if orient(a, b, point) < 0.0 {
                    match triangle.neighbors[k] {
                        Some(next) => {
                            current = next;
                            continue 'walk;
                        }
                        None => break 'walk,
                    }
                }
            }
            return Some(current);
        }
        (0..self.triangles.len())
            .find(|&index| self.triangles[index].alive && self.in_circle(index, point))
    }

    fn insert(&mut self, index: usize, start: usize) -> Option<usize> {
        let point = self.points[index];
        let first = self.locate(start, point)?;
        let duplicate = self.triangles[first].vertices.iter().any(|&vertex| {
            let other = self.points[vertex];
            (other.x - point.x).abs() < 1.0e-9 && (other.y - point.y).abs() < 1.0e-9
        });
        if duplicate {
            return None;
        }

        // The cavity: every triangle around the point whose circumcircle holds it.
        let mut cavity = vec![first];
        let mut in_cavity = HashMap::from([(first, true)]);
        let mut boundary = Vec::new();
        let mut next = 0;
        while next < cavity.len() {
            let triangle = cavity[next];
            next += 1;
            for k in 0..3 {
                let edge = (
                    self.triangles[triangle].vertices[(k + 1) % 3],
                    self.triangles[triangle].vertices[(k + 2) % 3],
                );
                let neighbor = self.triangles[triangle].neighbors[k];
                if let Some(neighbor) = neighbor {
                    match in_cavity.get(&neighbor) {
                        Some(true) => continue,
                        Some(false) => {}
                        None => {
                            let inside = self.in_circle(neighbor, point);
                            in_cavity.insert(neighbor, inside);
                            if inside {
                                cavity.push(neighbor);
                                continue;
                            }
                        }
                    }
                }
                boundary.push((edge, neighbor, triangle));
            }
        }

        for &triangle in &cavity {
            self.triangles[triangle].alive = false;
        }
        // Each boundary edge (a, b) with the point makes a new triangle (a, b, point).
        let mut starting = HashMap::with_capacity(boundary.len());
        let mut ending = HashMap::with_capacity(boundary.len());
        let mut created = Vec::with_capacity(boundary.len());
        for &((a, b), outer, old) in &boundary {
            let new = self.add([a, b, index], [None, None, outer]);
            if let Some(outer) = outer {
                let neighbors = &mut self.triangles[outer].neighbors;
                if let Some(slot) = neighbors.iter_mut().find(|slot| **slot == Some(old)) {
                    *slot = Some(new);
                }
            }
            starting.insert(a, new);
            ending.insert(b, new);
            created.push((new, a, b));
        }
        for (new, a, b) in created {
            // Across (b, point) lies the triangle starting at b; across (point, a) the one
            // ending at a.
            self.triangles[new].neighbors[0] = starting.get(&b).copied();
            self.triangles[new].neighbors[1] = ending.get(&a).copied();
        }
        Some(self.triangles.len() - 1)
    }
}

/// Counterclockwise triangles over `points`, by index. Points that coincide with an earlier
/// one are left out.
pub(super) fn triangulate(points: &[Point]) -> Vec<[usize; 3]> {
    if points.len() < 3 {
        return Vec::new();
    }
    let (mut min, mut max) = (points[0], points[0]);
    for point in points {
        min = Point::new(min.x.min(point.x), min.y.min(point.y));
        max = Point::new(max.x.max(point.x), max.y.max(point.y));
    }
    let size = (max.x - min.x).max(max.y - min.y).max(1.0e-9);
    let middle = Point::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5);
    let count = points.len();
    let mut mesh = Mesh {
        points: points.to_vec(),
        triangles: Vec::with_capacity(count * 6),
    };
    // A triangle around everything, removed at the end.
    mesh.points.extend([
        Point::new(middle.x - 40.0 * size, middle.y - 30.0 * size),
        Point::new(middle.x + 40.0 * size, middle.y - 30.0 * size),
        Point::new(middle.x, middle.y + 40.0 * size),
    ]);
    let mut last = mesh.add([count, count + 1, count + 2], [None; 3]);

    // Inserting along a snake through rows keeps each walk short.
    let row = (size / (count as f64).sqrt()).max(1.0e-9);
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by(|&i, &j| {
        let key = |index: usize| {
            let point = points[index];
            let band = ((point.y - min.y) / row).floor() as i64;
            let along = if band % 2 == 0 { point.x } else { -point.x };
            (band, along)
        };
        let (band_i, along_i) = key(i);
        let (band_j, along_j) = key(j);
        band_i.cmp(&band_j).then(along_i.total_cmp(&along_j))
    });
    for index in order {
        if let Some(created) = mesh.insert(index, last) {
            last = created;
        }
    }

    mesh.triangles
        .iter()
        .filter(|triangle| triangle.alive && triangle.vertices.iter().all(|&v| v < count))
        .map(|triangle| triangle.vertices)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_square_grid_is_covered_once() {
        let mut points = Vec::new();
        for row in 0..10 {
            for column in 0..10 {
                // A little skew keeps the test off cocircular ties.
                points.push(Point::new(
                    f64::from(column) + f64::from(row) * 1.0e-3,
                    f64::from(row) + f64::from(column) * 1.3e-3,
                ));
            }
        }
        let triangles = triangulate(&points);
        assert_eq!(triangles.len(), 2 * 9 * 9);
        let area: f64 = triangles
            .iter()
            .map(|t| orient(points[t[0]], points[t[1]], points[t[2]]) * 0.5)
            .sum();
        assert!(
            triangles
                .iter()
                .all(|t| orient(points[t[0]], points[t[1]], points[t[2]]) > 0.0)
        );
        assert!((area - 81.0).abs() < 0.1, "area {area}");
    }

    #[test]
    fn every_triangle_is_delaunay() {
        let points: Vec<Point> = (0..200)
            .map(|i| {
                let t = f64::from(i);
                Point::new((t * 12.9898).sin() * 50.0, (t * 78.233).cos() * 30.0)
            })
            .collect();
        let triangles = triangulate(&points);
        for triangle in &triangles {
            let (center, radius2) = circumcircle(
                points[triangle[0]],
                points[triangle[1]],
                points[triangle[2]],
            );
            for (index, point) in points.iter().enumerate() {
                if triangle.contains(&index) {
                    continue;
                }
                let d2 = (point.x - center.x).powi(2) + (point.y - center.y).powi(2);
                assert!(d2 >= radius2 * (1.0 - 1.0e-9));
            }
        }
    }
}
