//! The triangle mesh a piece is simulated on: its outline and holes resampled at an even
//! spacing, points along its centerlines so that the fold runs along mesh edges, and a
//! hexagonal grid inside, triangulated and cut back to the leather.

use super::delaunay::triangulate;
use crate::boolean::Region;
use crate::model::Point;
use std::collections::HashSet;

/// A ring vertex whose edges turn by more than this, in radians, is kept as a corner.
const CORNER_TURN: f64 = 0.35;
/// Grid and line points stay this many spacings away from the boundary and from centerlines.
const CLEARANCE: f64 = 0.6;
/// A centerline crossing lands on the nearest ring point within this many spacings.
const SNAP: f64 = 0.35;

pub(super) struct Sheet {
    /// Where each vertex lies in the pattern, in mm.
    pub points: Vec<Point>,
    /// Clockwise in the pattern's plane: with the pattern's y along the model's Z, their normal
    /// points up, out of the grain side.
    pub triangles: Vec<[usize; 3]>,
    /// Mesh edges lying on a centerline, as (lower index, higher index).
    pub creases: HashSet<(usize, usize)>,
}

fn sub(a: Point, b: Point) -> Point {
    Point::new(a.x - b.x, a.y - b.y)
}

fn distance(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn lerp(a: Point, b: Point, t: f64) -> Point {
    Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

pub(super) fn segment_distance(point: Point, a: Point, b: Point) -> f64 {
    let ab = sub(b, a);
    let length2 = ab.x * ab.x + ab.y * ab.y;
    let t = if length2 > 0.0 {
        (((point.x - a.x) * ab.x + (point.y - a.y) * ab.y) / length2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    distance(point, lerp(a, b, t))
}

/// Where segments `ab` and `cd` cross, as the parameters along each, if they do.
fn crossing(a: Point, b: Point, c: Point, d: Point) -> Option<(f64, f64)> {
    let r = sub(b, a);
    let s = sub(d, c);
    let denominator = r.x * s.y - r.y * s.x;
    if denominator.abs() < 1.0e-12 {
        return None;
    }
    let ca = sub(c, a);
    let t = (ca.x * s.y - ca.y * s.x) / denominator;
    let u = (ca.x * r.y - ca.y * r.x) / denominator;
    ((0.0..=1.0).contains(&t) && (0.0..1.0).contains(&u)).then_some((t, u))
}

/// Points at the given distances along an open polyline, which must be ascending.
fn sample(polyline: &[Point], distances: impl Iterator<Item = f64>) -> Vec<Point> {
    let mut out = Vec::new();
    let mut segment = 0;
    let mut walked = 0.0;
    for target in distances {
        while segment + 1 < polyline.len() - 1
            && walked + distance(polyline[segment], polyline[segment + 1]) < target
        {
            walked += distance(polyline[segment], polyline[segment + 1]);
            segment += 1;
        }
        let length = distance(polyline[segment], polyline[segment + 1]).max(1.0e-12);
        let t = ((target - walked) / length).clamp(0.0, 1.0);
        out.push(lerp(polyline[segment], polyline[segment + 1], t));
    }
    out
}

/// The ring with its points about `spacing` apart, keeping its sharp corners.
fn resample(ring: &[Point], spacing: f64) -> Vec<Point> {
    let n = ring.len();
    let turn = |i: usize| {
        let (a, b, c) = (ring[(i + n - 1) % n], ring[i], ring[(i + 1) % n]);
        let (u, v) = (sub(b, a), sub(c, b));
        (u.x * v.y - u.y * v.x).atan2(u.x * v.x + u.y * v.y).abs()
    };
    let corners: Vec<usize> = (0..n).filter(|&i| turn(i) > CORNER_TURN).collect();
    let starts = if corners.is_empty() { vec![0] } else { corners };
    let mut out = Vec::new();
    for (k, &start) in starts.iter().enumerate() {
        let end = starts[(k + 1) % starts.len()];
        let mut polyline = vec![ring[start]];
        let mut index = start;
        loop {
            index = (index + 1) % n;
            polyline.push(ring[index]);
            if index == end {
                break;
            }
        }
        let length: f64 = polyline.windows(2).map(|w| distance(w[0], w[1])).sum();
        let minimum = if starts.len() == 1 { 3 } else { 1 };
        let segments = ((length / spacing).ceil() as usize).max(minimum);
        out.extend(sample(
            &polyline,
            (0..segments).map(|j| length * j as f64 / segments as f64),
        ));
    }
    out
}

/// A small fixed offset per point, which keeps the triangulation off exact ties.
fn jitter(index: usize, spacing: f64) -> Point {
    let hash = |seed: f64| ((index as f64 * seed).sin() * 43_758.545_3).fract() - 0.5;
    Point::new(
        hash(12.9898) * 1.0e-4 * spacing,
        hash(78.233) * 1.0e-4 * spacing,
    )
}

/// The piece's mesh, or `None` when nothing is left after cutting it back to the leather.
pub(super) fn build(region: &Region, lines: &[(Point, Point)], spacing: f64) -> Option<Sheet> {
    let mut rings: Vec<Vec<Point>> = region.rings().map(|ring| resample(ring, spacing)).collect();
    // Which centerline each ring point lies on.
    let mut ring_tags: Vec<Vec<Option<usize>>> =
        rings.iter().map(|ring| vec![None; ring.len()]).collect();

    // Where a centerline crosses the boundary, so that its fold reaches the edge.
    for (line_index, &(start, end)) in lines.iter().enumerate() {
        for (ring, tags) in rings.iter_mut().zip(ring_tags.iter_mut()) {
            let n = ring.len();
            let mut hits: Vec<(usize, f64, Point)> = (0..n)
                .filter_map(|i| {
                    let (t, u) = crossing(start, end, ring[i], ring[(i + 1) % n])?;
                    Some((i, u, lerp(start, end, t)))
                })
                .collect();
            hits.sort_by(|x, y| y.0.cmp(&x.0).then(y.1.total_cmp(&x.1)));
            for (edge, _, point) in hits {
                let n = ring.len();
                let next = (edge + 1) % n;
                if distance(point, ring[edge]) < SNAP * spacing {
                    ring[edge] = point;
                    tags[edge] = Some(line_index);
                } else if distance(point, ring[next]) < SNAP * spacing {
                    ring[next] = point;
                    tags[next] = Some(line_index);
                } else {
                    ring.insert(edge + 1, point);
                    tags.insert(edge + 1, Some(line_index));
                }
            }
        }
    }

    let leather = Region {
        outline: rings[0].clone(),
        holes: rings[1..].to_vec(),
    };
    let edges: Vec<(Point, Point)> = rings
        .iter()
        .flat_map(|ring| (0..ring.len()).map(|i| (ring[i], ring[(i + 1) % ring.len()])))
        .collect();
    let clear_of_edges = |point: Point| {
        edges
            .iter()
            .all(|&(a, b)| segment_distance(point, a, b) > CLEARANCE * spacing)
    };

    let mut points: Vec<Point> = rings.iter().flatten().copied().collect();
    let mut tags: Vec<Option<usize>> = ring_tags.into_iter().flatten().collect();

    for (line_index, &(start, end)) in lines.iter().enumerate() {
        let steps = ((distance(start, end) / spacing).ceil() as usize).max(1);
        for j in 0..=steps {
            let point = lerp(start, end, j as f64 / steps as f64);
            if leather.contains(point) && clear_of_edges(point) {
                points.push(point);
                tags.push(Some(line_index));
            }
        }
    }

    let bounds = leather.bounds()?;
    let row = spacing * 3.0_f64.sqrt() * 0.5;
    let rows = ((bounds.height() / row).ceil() as usize) + 1;
    let columns = ((bounds.width() / spacing).ceil() as usize) + 2;
    for j in 0..rows {
        let y = bounds.min_y + j as f64 * row;
        let shift = if j % 2 == 0 { 0.0 } else { 0.5 * spacing };
        for i in 0..columns {
            let point = Point::new(bounds.min_x + i as f64 * spacing + shift, y);
            let clear_of_lines = lines
                .iter()
                .all(|&(a, b)| segment_distance(point, a, b) > CLEARANCE * spacing);
            if clear_of_lines && leather.contains(point) && clear_of_edges(point) {
                points.push(point);
                tags.push(None);
            }
        }
    }

    let jittered: Vec<Point> = points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let offset = jitter(index, spacing);
            Point::new(point.x + offset.x, point.y + offset.y)
        })
        .collect();
    let inside = |point: Point, toward: Point| leather.contains(lerp(point, toward, 1.0e-3));
    let min_area = 1.0e-6 * spacing * spacing;
    let kept: Vec<[usize; 3]> = triangulate(&jittered)
        .into_iter()
        .filter(|&[a, b, c]| {
            let (pa, pb, pc) = (points[a], points[b], points[c]);
            let area = ((pb.x - pa.x) * (pc.y - pa.y) - (pb.y - pa.y) * (pc.x - pa.x)) * 0.5;
            let centroid = Point::new((pa.x + pb.x + pc.x) / 3.0, (pa.y + pb.y + pc.y) / 3.0);
            area.abs() > min_area
                && leather.contains(centroid)
                && inside(lerp(pa, pb, 0.5), centroid)
                && inside(lerp(pb, pc, 0.5), centroid)
                && inside(lerp(pc, pa, 0.5), centroid)
        })
        .collect();
    if kept.is_empty() {
        return None;
    }

    // Leave out the points no triangle uses, and turn the triangles clockwise.
    let mut remap = vec![usize::MAX; points.len()];
    let mut used = Vec::new();
    let mut used_tags = Vec::new();
    let triangles: Vec<[usize; 3]> = kept
        .iter()
        .map(|triangle| {
            let [a, b, c] = triangle.map(|index| {
                if remap[index] == usize::MAX {
                    remap[index] = used.len();
                    used.push(points[index]);
                    used_tags.push(tags[index]);
                }
                remap[index]
            });
            [a, c, b]
        })
        .collect();

    let mut creases = HashSet::new();
    for triangle in &triangles {
        for k in 0..3 {
            let (a, b) = (triangle[k], triangle[(k + 1) % 3]);
            if used_tags[a].is_some() && used_tags[a] == used_tags[b] {
                creases.insert((a.min(b), a.max(b)));
            }
        }
    }
    Some(Sheet {
        points: used,
        triangles,
        creases,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rectangle(width: f64, height: f64) -> Region {
        Region {
            outline: vec![
                Point::new(0.0, 0.0),
                Point::new(width, 0.0),
                Point::new(width, height),
                Point::new(0.0, height),
            ],
            holes: Vec::new(),
        }
    }

    fn area(sheet: &Sheet) -> f64 {
        sheet
            .triangles
            .iter()
            .map(|&[a, b, c]| {
                let (pa, pb, pc) = (sheet.points[a], sheet.points[b], sheet.points[c]);
                ((pb.x - pa.x) * (pc.y - pa.y) - (pb.y - pa.y) * (pc.x - pa.x)) * 0.5
            })
            .sum()
    }

    #[test]
    fn a_rectangle_is_covered_by_clockwise_triangles() {
        let sheet = build(&rectangle(100.0, 60.0), &[], 5.0).unwrap();
        // Clockwise triangles have a negative signed area.
        assert!((area(&sheet) + 6000.0).abs() < 1.0, "{}", area(&sheet));
        assert!(sheet.creases.is_empty());
        assert!(sheet.points.len() > 200);
    }

    #[test]
    fn a_hole_stays_open() {
        let mut region = rectangle(100.0, 100.0);
        region.holes.push(vec![
            Point::new(30.0, 30.0),
            Point::new(30.0, 70.0),
            Point::new(70.0, 70.0),
            Point::new(70.0, 30.0),
        ]);
        let sheet = build(&region, &[], 5.0).unwrap();
        assert!((area(&sheet) + 8400.0).abs() < 20.0, "{}", area(&sheet));
    }

    #[test]
    fn a_centerline_runs_along_mesh_edges_from_edge_to_edge() {
        let line = (Point::new(50.0, -10.0), Point::new(50.0, 70.0));
        let sheet = build(&rectangle(100.0, 60.0), &[line], 5.0).unwrap();
        let length: f64 = sheet
            .creases
            .iter()
            .map(|&(a, b)| distance(sheet.points[a], sheet.points[b]))
            .sum();
        assert!((length - 60.0).abs() < 0.5, "crease length {length}");
    }
}
