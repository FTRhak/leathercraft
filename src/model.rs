use crate::i18n::Message;
use crate::material::Material;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

pub const PROJECT_VERSION: u32 = 1;
const MAX_PIECES: usize = 10_000;
const MAX_POINTS_PER_PIECE: usize = 2_048;
const MAX_HOLES_PER_PIECE: usize = 512;
const MAX_COORDINATE_MM: f64 = 1_000_000.0;
const MIN_POLYGON_AREA_MM2: f64 = 1.0e-6;
/// Removes the float noise that polygon subtraction leaves in coordinates.
const CUT_PRECISION_MM: f64 = 1.0e-6;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Units {
    #[default]
    Millimeters,
    Inches,
}

impl Units {
    pub fn from_millimeters(self, value: f64) -> f64 {
        match self {
            Self::Millimeters => value,
            Self::Inches => value / 25.4,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Bounds {
    pub fn width(self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(self) -> f64 {
        self.max_y - self.min_y
    }

    fn overlaps(self, other: Self) -> bool {
        self.min_x <= other.max_x
            && other.min_x <= self.max_x
            && self.min_y <= other.max_y
            && other.min_y <= self.max_y
    }
}

fn ring_bounds(ring: &[Point]) -> Option<Bounds> {
    let first = *ring.first()?;
    let mut bounds = Bounds {
        min_x: first.x,
        min_y: first.y,
        max_x: first.x,
        max_y: first.y,
    };
    for point in &ring[1..] {
        bounds.min_x = bounds.min_x.min(point.x);
        bounds.min_y = bounds.min_y.min(point.y);
        bounds.max_x = bounds.max_x.max(point.x);
        bounds.max_y = bounds.max_y.max(point.y);
    }
    Some(bounds)
}

fn ring_contains(ring: &[Point], point: Point) -> bool {
    if ring.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut previous = ring.len() - 1;
    for current in 0..ring.len() {
        let a = ring[current];
        let b = ring[previous];
        let crosses = (a.y > point.y) != (b.y > point.y)
            && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x;
        if crosses {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

/// Positive when `c` lies to one side of the line through `a` and `b`, zero when on it.
fn orientation(a: Point, b: Point, c: Point) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn within_segment(a: Point, b: Point, c: Point) -> bool {
    c.x >= a.x.min(b.x) && c.x <= a.x.max(b.x) && c.y >= a.y.min(b.y) && c.y <= a.y.max(b.y)
}

fn segments_touch(a: Point, b: Point, c: Point, d: Point) -> bool {
    let (ab_c, ab_d) = (orientation(a, b, c), orientation(a, b, d));
    let (cd_a, cd_b) = (orientation(c, d, a), orientation(c, d, b));
    if ab_c * ab_d < 0.0 && cd_a * cd_b < 0.0 {
        return true;
    }
    (ab_c == 0.0 && within_segment(a, b, c))
        || (ab_d == 0.0 && within_segment(a, b, d))
        || (cd_a == 0.0 && within_segment(c, d, a))
        || (cd_b == 0.0 && within_segment(c, d, b))
}

/// Whether the outlines of two closed rings cross or touch.
fn rings_cross(first: &[Point], second: &[Point]) -> bool {
    (0..first.len()).any(|i| {
        let (a, b) = (first[i], first[(i + 1) % first.len()]);
        (0..second.len()).any(|j| segments_touch(a, b, second[j], second[(j + 1) % second.len()]))
    })
}

pub fn signed_area(ring: &[Point]) -> f64 {
    let mut area = 0.0;
    for index in 0..ring.len() {
        let a = ring[index];
        let b = ring[(index + 1) % ring.len()];
        area += a.x * b.y - b.x * a.y;
    }
    area * 0.5
}

/// What cutting a shape out of a piece did to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cut {
    Hole(usize),
    Edge,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatternPiece {
    pub id: u64,
    pub name: String,
    pub points: Vec<Point>,
    /// A locked piece cannot be moved, reshaped or deleted. Older files lack the field.
    #[serde(default)]
    pub locked: bool,
    /// Older files lack the field.
    #[serde(default)]
    pub material: Material,
    /// Closed rings cut out of the piece, stored like `points`. Older files lack the field.
    #[serde(default)]
    pub holes: Vec<Vec<Point>>,
}

impl PatternPiece {
    pub fn new(id: u64, name: impl Into<String>, points: Vec<Point>) -> Self {
        Self {
            id,
            name: name.into(),
            points,
            locked: false,
            material: Material::default(),
            holes: Vec::new(),
        }
    }

    pub fn bounds(&self) -> Option<Bounds> {
        ring_bounds(&self.points)
    }

    /// Whether the point is on the leather: inside the outline and not inside a hole.
    pub fn contains(&self, point: Point) -> bool {
        ring_contains(&self.points, point) && self.hole_at(point).is_none()
    }

    pub fn hole_at(&self, point: Point) -> Option<usize> {
        self.holes
            .iter()
            .position(|hole| ring_contains(hole, point))
    }

    pub fn signed_area(&self) -> f64 {
        signed_area(&self.points)
    }

    /// The outline followed by each hole.
    pub fn rings(&self) -> impl Iterator<Item = &[Point]> {
        std::iter::once(self.points.as_slice()).chain(self.holes.iter().map(Vec::as_slice))
    }

    /// Every point of `rings`, in order, as triangulation expects them.
    pub fn vertices(&self) -> impl Iterator<Item = &Point> {
        self.rings().flatten()
    }

    /// Where each hole starts within `vertices`.
    pub fn hole_indices(&self) -> Vec<usize> {
        let mut start = self.points.len();
        self.holes
            .iter()
            .map(|hole| {
                let index = start;
                start += hole.len();
                index
            })
            .collect()
    }

    /// Cuts a shape out of the piece: inside the outline it becomes a hole, and across the
    /// outline it changes the outline. The caller still validates the result, which rejects
    /// a new outline that crosses an existing hole.
    pub(crate) fn cut(&mut self, ring: Vec<Point>) -> Result<Cut, Message> {
        self.validate_ring(&ring)?;
        if rings_cross(&self.points, &ring) {
            self.points = self.outline_without(&ring)?;
            return Ok(Cut::Edge);
        }
        if ring_contains(&ring, self.points[0]) {
            return Err(Message::HoleCoversPiece {
                name: self.name.clone(),
            });
        }
        if !ring_contains(&self.points, ring[0]) {
            return Err(Message::HoleOutsidePiece {
                name: self.name.clone(),
            });
        }
        self.holes.push(ring);
        Ok(Cut::Hole(self.holes.len() - 1))
    }

    fn outline_without(&self, ring: &[Point]) -> Result<Vec<Point>, Message> {
        let contour = |points: &[Point]| -> Vec<[f64; 2]> {
            points.iter().map(|point| [point.x, point.y]).collect()
        };
        let shapes = contour(&self.points).overlay(
            &contour(ring),
            OverlayRule::Difference,
            FillRule::NonZero,
        );
        let name = self.name.clone();
        let outline = match shapes.as_slice() {
            [] => return Err(Message::HoleCoversPiece { name }),
            // A second contour means the shape touches the outline from inside at a single point.
            [shape] => match shape.as_slice() {
                [outline] => outline,
                _ => return Err(Message::HoleOutsidePiece { name }),
            },
            _ => return Err(Message::HoleSplitsPiece { name }),
        };
        let round = |value: f64| (value / CUT_PRECISION_MM).round() * CUT_PRECISION_MM;
        let points: Vec<Point> = outline
            .iter()
            .map(|[x, y]| Point::new(round(*x), round(*y)))
            .collect();
        // A shape that only touches the outline from outside removes nothing.
        let removed = self.signed_area().abs() - signed_area(&points).abs();
        if removed < MIN_POLYGON_AREA_MM2 {
            return Err(Message::HoleOutsidePiece { name });
        }
        Ok(points)
    }

    fn validate_ring(&self, ring: &[Point]) -> Result<(), Message> {
        if !(3..=MAX_POINTS_PER_PIECE).contains(&ring.len()) {
            return Err(Message::PointCountOutOfRange {
                name: self.name.clone(),
                max: MAX_POINTS_PER_PIECE,
            });
        }
        for (index, point) in ring.iter().enumerate() {
            if !point.x.is_finite()
                || !point.y.is_finite()
                || point.x.abs() > MAX_COORDINATE_MM
                || point.y.abs() > MAX_COORDINATE_MM
            {
                return Err(Message::CoordinateOutOfRange {
                    name: self.name.clone(),
                });
            }
            let next = ring[(index + 1) % ring.len()];
            if (point.x - next.x).hypot(point.y - next.y) < 1.0e-8 {
                return Err(Message::CoincidentPoints {
                    name: self.name.clone(),
                });
            }
        }
        if signed_area(ring).abs() < MIN_POLYGON_AREA_MM2 {
            return Err(Message::ZeroArea {
                name: self.name.clone(),
            });
        }
        Ok(())
    }

    fn validate_holes(&self) -> Result<(), Message> {
        if self.holes.len() > MAX_HOLES_PER_PIECE {
            return Err(Message::TooManyHoles {
                name: self.name.clone(),
                max: MAX_HOLES_PER_PIECE,
            });
        }
        let mut checked: Vec<(&[Point], Option<Bounds>)> = Vec::with_capacity(self.holes.len());
        for hole in &self.holes {
            self.validate_ring(hole)?;
            if !ring_contains(&self.points, hole[0]) || rings_cross(&self.points, hole) {
                return Err(Message::HoleOutsidePiece {
                    name: self.name.clone(),
                });
            }
            let bounds = ring_bounds(hole);
            for (other, other_bounds) in &checked {
                let near = matches!(
                    (bounds, *other_bounds),
                    (Some(a), Some(b)) if a.overlaps(b)
                );
                if near
                    && (rings_cross(hole, other)
                        || ring_contains(hole, other[0])
                        || ring_contains(other, hole[0]))
                {
                    return Err(Message::HolesOverlap {
                        name: self.name.clone(),
                    });
                }
            }
            checked.push((hole, bounds));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub units: Units,
    pub pieces: Vec<PatternPiece>,
    /// The material given to each new piece. Older files lack the field.
    #[serde(default)]
    pub default_material: Material,
    /// Where the document lives on disk; not part of the file format.
    #[serde(skip)]
    pub path: Option<PathBuf>,
    /// Whether there are edits since the last save or load; not part of the file format.
    #[serde(skip)]
    pub modified: bool,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: PROJECT_VERSION,
            units: Units::Millimeters,
            pieces: Vec::new(),
            default_material: Material::default(),
            path: None,
            modified: false,
        }
    }
}

impl Project {
    pub fn bounds(&self) -> Option<Bounds> {
        let mut pieces = self.pieces.iter().filter_map(PatternPiece::bounds);
        let first = pieces.next()?;
        Some(pieces.fold(first, |total, bounds| Bounds {
            min_x: total.min_x.min(bounds.min_x),
            min_y: total.min_y.min(bounds.min_y),
            max_x: total.max_x.max(bounds.max_x),
            max_y: total.max_y.max(bounds.max_y),
        }))
    }

    pub fn validate(&self) -> Result<(), Message> {
        if self.version != PROJECT_VERSION {
            return Err(Message::UnsupportedVersion {
                found: self.version,
                expected: PROJECT_VERSION,
            });
        }
        if self.pieces.len() > MAX_PIECES {
            return Err(Message::TooManyPieces { max: MAX_PIECES });
        }
        if !self.default_material.has_valid_thickness() {
            return Err(Message::DefaultThicknessOutOfRange);
        }

        let mut ids = HashSet::with_capacity(self.pieces.len());
        for piece in &self.pieces {
            if piece.id == 0 || !ids.insert(piece.id) {
                return Err(Message::InvalidPieceId {
                    name: piece.name.clone(),
                });
            }
            if piece.name.trim().is_empty() {
                return Err(Message::PieceNameRequired { id: piece.id });
            }
            if !piece.material.has_valid_thickness() {
                return Err(Message::ThicknessOutOfRange {
                    name: piece.name.clone(),
                });
            }
            piece.validate_ring(&piece.points)?;
            piece.validate_holes()?;
        }
        Ok(())
    }

    pub fn next_piece_id(&self) -> u64 {
        self.pieces
            .iter()
            .map(|piece| piece.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
    }

    pub fn add_piece(&mut self, piece: PatternPiece) {
        self.pieces.push(piece);
    }

    pub fn remove_piece(&mut self, id: u64) -> bool {
        let original_len = self.pieces.len();
        self.pieces.retain(|piece| piece.id != id);
        self.pieces.len() != original_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rectangle(id: u64) -> PatternPiece {
        PatternPiece::new(
            id,
            "Pocket",
            vec![
                Point::new(0.0, 0.0),
                Point::new(80.0, 0.0),
                Point::new(80.0, 120.0),
                Point::new(0.0, 120.0),
            ],
        )
    }

    #[test]
    fn accepts_a_valid_pattern_and_reports_bounds() {
        let piece = rectangle(1);
        let project = Project {
            pieces: vec![piece.clone()],
            ..Project::default()
        };

        assert_eq!(project.validate(), Ok(()));
        assert_eq!(
            piece.bounds(),
            Some(Bounds {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 80.0,
                max_y: 120.0,
            })
        );
    }

    #[test]
    fn rejects_invalid_polygons_and_duplicate_ids() {
        let mut project = Project {
            pieces: vec![rectangle(1), rectangle(1)],
            ..Project::default()
        };
        assert_eq!(
            project.validate(),
            Err(Message::InvalidPieceId {
                name: "Pocket".to_owned()
            })
        );

        project.pieces = vec![PatternPiece::new(
            2,
            "Bad",
            vec![
                Point::new(0.0, 0.0),
                Point::new(2.0, 2.0),
                Point::new(4.0, 4.0),
            ],
        )];
        assert_eq!(
            project.validate(),
            Err(Message::ZeroArea {
                name: "Bad".to_owned()
            })
        );
    }

    #[test]
    fn hit_testing_handles_inside_and_outside_points() {
        let piece = rectangle(1);
        assert!(piece.contains(Point::new(40.0, 60.0)));
        assert!(!piece.contains(Point::new(100.0, 60.0)));
    }

    fn square(x: f64, y: f64, size: f64) -> Vec<Point> {
        vec![
            Point::new(x, y),
            Point::new(x + size, y),
            Point::new(x + size, y + size),
            Point::new(x, y + size),
        ]
    }

    fn project_with_holes(holes: Vec<Vec<Point>>) -> Project {
        let mut piece = rectangle(1);
        piece.holes = holes;
        Project {
            pieces: vec![piece],
            ..Project::default()
        }
    }

    #[test]
    fn points_inside_a_hole_are_not_on_the_piece() {
        let mut piece = rectangle(1);
        piece.holes = vec![square(10.0, 10.0, 10.0), square(40.0, 40.0, 10.0)];

        assert!(!piece.contains(Point::new(45.0, 45.0)));
        assert!(piece.contains(Point::new(30.0, 30.0)));
        assert_eq!(piece.hole_at(Point::new(45.0, 45.0)), Some(1));
        assert_eq!(piece.hole_at(Point::new(30.0, 30.0)), None);
        assert_eq!(piece.hole_indices(), vec![4, 8]);
        assert_eq!(piece.vertices().count(), 12);
    }

    #[test]
    fn accepts_separate_holes_inside_the_piece() {
        let holes = vec![square(10.0, 10.0, 10.0), square(40.0, 40.0, 10.0)];
        assert_eq!(project_with_holes(holes).validate(), Ok(()));
    }

    #[test]
    fn rejects_holes_outside_or_crossing_the_outline() {
        let outside = Err(Message::HoleOutsidePiece {
            name: "Pocket".to_owned(),
        });
        let beyond = vec![square(200.0, 200.0, 10.0)];
        assert_eq!(project_with_holes(beyond).validate(), outside);
        let crossing = vec![square(75.0, 10.0, 10.0)];
        assert_eq!(project_with_holes(crossing).validate(), outside);
        let touching = vec![square(70.0, 10.0, 10.0)];
        assert_eq!(project_with_holes(touching).validate(), outside);
    }

    #[test]
    fn rejects_overlapping_and_nested_holes() {
        let overlap = Err(Message::HolesOverlap {
            name: "Pocket".to_owned(),
        });
        let crossing = vec![square(10.0, 10.0, 10.0), square(15.0, 15.0, 10.0)];
        assert_eq!(project_with_holes(crossing).validate(), overlap);
        let nested = vec![square(10.0, 10.0, 30.0), square(15.0, 15.0, 5.0)];
        assert_eq!(project_with_holes(nested).validate(), overlap);
        let enclosing = vec![square(15.0, 15.0, 5.0), square(10.0, 10.0, 30.0)];
        assert_eq!(project_with_holes(enclosing).validate(), overlap);
    }

    #[test]
    fn holes_follow_the_same_ring_rules_as_outlines() {
        let flat = vec![
            Point::new(10.0, 10.0),
            Point::new(20.0, 20.0),
            Point::new(30.0, 30.0),
        ];
        assert_eq!(
            project_with_holes(vec![flat]).validate(),
            Err(Message::ZeroArea {
                name: "Pocket".to_owned()
            })
        );
    }

    fn assert_same_outline(piece: &PatternPiece, expected: &[(f64, f64)]) {
        assert_eq!(piece.points.len(), expected.len());
        for (x, y) in expected {
            assert!(
                piece
                    .points
                    .iter()
                    .any(|point| { (point.x - x).abs() < 1.0e-5 && (point.y - y).abs() < 1.0e-5 }),
                "missing ({x}, {y}) in {:?}",
                piece.points
            );
        }
    }

    #[test]
    fn a_shape_inside_the_outline_becomes_a_hole() {
        let mut piece = rectangle(1);
        let outline = piece.points.clone();
        assert_eq!(piece.cut(square(10.0, 10.0, 10.0)), Ok(Cut::Hole(0)));
        assert_eq!(piece.points, outline);
        assert_eq!(piece.holes.len(), 1);
    }

    #[test]
    fn a_shape_across_the_outline_notches_the_edge() {
        let mut piece = rectangle(1);
        assert_eq!(piece.cut(square(70.0, 50.0, 20.0)), Ok(Cut::Edge));
        assert!(piece.holes.is_empty());
        assert_same_outline(
            &piece,
            &[
                (0.0, 0.0),
                (80.0, 0.0),
                (80.0, 50.0),
                (70.0, 50.0),
                (70.0, 70.0),
                (80.0, 70.0),
                (80.0, 120.0),
                (0.0, 120.0),
            ],
        );
        assert!((piece.signed_area().abs() - 9400.0).abs() < 1.0e-3);
    }

    #[test]
    fn a_shape_flush_with_the_outline_notches_the_edge() {
        let mut piece = rectangle(1);
        assert_eq!(piece.cut(square(70.0, 50.0, 10.0)), Ok(Cut::Edge));
        assert!(piece.holes.is_empty());
        assert!((piece.signed_area().abs() - 9500.0).abs() < 1.0e-3);
    }

    #[test]
    fn a_corner_can_be_cut_off() {
        let mut piece = rectangle(1);
        assert_eq!(piece.cut(square(70.0, -10.0, 20.0)), Ok(Cut::Edge));
        assert_same_outline(
            &piece,
            &[
                (0.0, 0.0),
                (70.0, 0.0),
                (70.0, 10.0),
                (80.0, 10.0),
                (80.0, 120.0),
                (0.0, 120.0),
            ],
        );
    }

    #[test]
    fn cuts_that_do_not_leave_one_outline_are_rejected() {
        let name = "Pocket".to_owned();
        let splitting = vec![
            Point::new(-10.0, 50.0),
            Point::new(90.0, 50.0),
            Point::new(90.0, 60.0),
            Point::new(-10.0, 60.0),
        ];
        assert_eq!(
            rectangle(1).cut(splitting),
            Err(Message::HoleSplitsPiece { name: name.clone() })
        );
        assert_eq!(
            rectangle(1).cut(square(-10.0, -10.0, 200.0)),
            Err(Message::HoleCoversPiece { name: name.clone() })
        );
        assert_eq!(
            rectangle(1).cut(square(200.0, 200.0, 10.0)),
            Err(Message::HoleOutsidePiece { name: name.clone() })
        );
        assert_eq!(
            rectangle(1).cut(square(80.0, 50.0, 10.0)),
            Err(Message::HoleOutsidePiece { name })
        );
    }

    #[test]
    fn converts_display_measurements_without_changing_internal_geometry() {
        assert_eq!(Units::Inches.from_millimeters(25.4), 1.0);
        assert_eq!(Units::Millimeters.from_millimeters(25.4), 25.4);
    }
}
