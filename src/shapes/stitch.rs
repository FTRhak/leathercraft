//! Stitch holes: an open path along which a piece is punched for a hand stitch. The holes are
//! computed from the path, the hole kind, the pitch and the size; they are never stored.

use super::corners::{fillet, push_point, radius_at, valid_radii};
use crate::model::Point;
use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_PI_4, TAU};

/// The smallest and largest hole size, in mm.
pub const MIN_HOLE_MM: f64 = 0.3;
pub const MAX_HOLE_MM: f64 = 10.0;
/// The smallest and largest pitch, in mm.
pub const MIN_PITCH_MM: f64 = 1.0;
pub const MAX_PITCH_MM: f64 = 50.0;
/// The most holes one line may have; validation rejects more.
pub const MAX_HOLES_PER_LINE: usize = 4_096;
/// The usual iron pitches, offered as presets.
pub const PITCH_PRESETS: [f64; 6] = [2.7, 3.0, 3.38, 3.85, 4.0, 5.0];
/// The pitch of new lines, in mm.
pub const DEFAULT_PITCH_MM: f64 = 3.85;
/// The angle between a slanted hole's long axis and the path.
pub const HOLE_SLANT: f64 = FRAC_PI_4;
/// The width of a French slot, as a part of its length.
pub const FRENCH_WIDTH_RATIO: f64 = 0.25;
/// The short diagonal of a diamond hole, as a part of its long one.
pub const DIAMOND_WIDTH_RATIO: f64 = 0.45;
/// Points of a round hole's outline.
pub const HOLE_SEGMENTS: usize = 16;
const ANGLE_SNAP: f64 = TAU / 24.0;
const MERGE_DISTANCE_MM: f64 = 1.0e-6;

/// The shape of the iron's teeth.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HoleKind {
    /// A thin slanted slot.
    French,
    /// A slanted rhombus.
    #[default]
    Diamond,
    /// A circle, without slant.
    Round,
}

impl HoleKind {
    pub const ALL: [Self; 3] = [Self::French, Self::Diamond, Self::Round];

    /// The size new holes of this kind start with, in mm.
    pub const fn default_size(self) -> f64 {
        match self {
            Self::French | Self::Round => 1.2,
            Self::Diamond => 1.6,
        }
    }

    /// The hole's short side for a `size` long one.
    pub fn width(self, size: f64) -> f64 {
        match self {
            Self::French => size * FRENCH_WIDTH_RATIO,
            Self::Diamond => size * DIAMOND_WIDTH_RATIO,
            Self::Round => size,
        }
    }

    /// The angle between the hole's long axis and the path.
    pub const fn slant(self) -> f64 {
        match self {
            Self::French | Self::Diamond => HOLE_SLANT,
            Self::Round => 0.0,
        }
    }

    /// How far a `size` hole reaches along the path.
    pub fn footprint(self, size: f64) -> f64 {
        let (sin, cos) = self.slant().sin_cos();
        let width = self.width(size);
        match self {
            Self::Round => size,
            Self::French => size * cos.abs() + width * sin.abs(),
            Self::Diamond => (size * cos.abs()).max(width * sin.abs()),
        }
    }
}

/// A range of iron pitches with the thread and leather that suit it, from the usual saddle
/// stitch tables. It drives the hints in the Properties panel and never restricts an edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StitchBand {
    /// 2.7–3.0 mm: watch straps, small card holders, thin wallets.
    Fine,
    /// 3.38–3.5 mm: classic wallets, card holders, passport covers.
    Classic,
    /// 3.85–4.0 mm: long wallets, clutches, small bags, sheaths.
    Medium,
    /// 5 mm and up: large bags, backpacks, belts, footwear.
    Heavy,
}

impl StitchBand {
    pub fn from_pitch(pitch: f64) -> Self {
        if pitch < 3.2 {
            Self::Fine
        } else if pitch < 3.7 {
            Self::Classic
        } else if pitch < 4.5 {
            Self::Medium
        } else {
            Self::Heavy
        }
    }

    /// The thread diameter that suits the band, in mm; the heavy band has no upper limit.
    pub const fn thread_mm(self) -> (f64, Option<f64>) {
        match self {
            Self::Fine => (0.35, Some(0.45)),
            Self::Classic => (0.45, Some(0.55)),
            Self::Medium => (0.55, Some(0.65)),
            Self::Heavy => (0.8, None),
        }
    }

    /// The total leather thickness that suits the band, in mm; the heavy band has no upper limit.
    pub const fn leather_mm(self) -> (f64, Option<f64>) {
        match self {
            Self::Fine => (1.0, Some(2.0)),
            Self::Classic => (1.5, Some(2.5)),
            Self::Medium => (2.5, Some(4.0)),
            Self::Heavy => (4.0, None),
        }
    }

    /// Whether leather of this thickness suits the band.
    pub fn suits_leather(self, thickness: f64) -> bool {
        let (min, max) = self.leather_mm();
        thickness >= min && max.is_none_or(|max| thickness <= max)
    }
}

/// One hole: its center, and the direction of its long axis in radians.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hole {
    pub center: Point,
    pub angle: f64,
}

/// A stretch of the path between two anchors: indices into `path` and its length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Run {
    pub start: usize,
    pub end: usize,
    pub length: f64,
}

impl Run {
    /// How many intervals the run is split into for `pitch`.
    fn intervals(self, pitch: f64) -> usize {
        let count = (self.length / pitch).round();
        if count.is_finite() {
            (count as usize).max(1)
        } else {
            usize::MAX
        }
    }
}

/// An open path along which holes are punched. Its vertices are its handles; the inner ones can
/// be rounded like a polygon's corners.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StitchLine {
    pub points: Vec<Point>,
    /// Corner radii by vertex; a missing one is 0, and the ends are never rounded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub radii: Vec<f64>,
    /// Named `hole` rather than `kind`, which tags the `Shape`.
    pub hole: HoleKind,
    /// Center to center along the path, in mm.
    pub pitch: f64,
    /// The diameter of a round hole, the long side of the others, in mm.
    pub size: f64,
}

impl StitchLine {
    /// A line through `points` with sharp vertices, or `None` with fewer than two points.
    pub fn new(points: Vec<Point>, hole: HoleKind, pitch: f64, size: f64) -> Option<Self> {
        (points.len() >= 2).then_some(Self {
            points,
            radii: Vec::new(),
            hole,
            pitch,
            size,
        })
    }

    /// `to` turned around `from` in 15° steps, keeping its distance.
    pub fn snap(from: Point, to: Point) -> Point {
        let (dx, dy) = (to.x - from.x, to.y - from.y);
        let length = dx.hypot(dy);
        let angle = (dy.atan2(dx) / ANGLE_SNAP).round() * ANGLE_SNAP;
        let (sin, cos) = angle.sin_cos();
        Point::new(from.x + length * cos, from.y + length * sin)
    }

    /// The path with its anchors: the ends and the sharp vertices, as indices into the path.
    fn path_with_anchors(&self) -> (Vec<Point>, Vec<usize>) {
        let count = self.points.len();
        let mut path = Vec::with_capacity(count);
        let mut anchors = Vec::new();
        for (index, &point) in self.points.iter().enumerate() {
            let radius = radius_at(&self.radii, index);
            let arc = (index > 0 && index + 1 < count && radius > 0.0)
                .then(|| {
                    fillet(
                        self.points[index - 1],
                        point,
                        self.points[index + 1],
                        radius,
                    )
                })
                .flatten();
            match arc {
                Some(arc) => {
                    for point in arc {
                        push_point(&mut path, point);
                    }
                }
                None => {
                    push_point(&mut path, point);
                    let last = path.len() - 1;
                    if anchors.last() != Some(&last) {
                        anchors.push(last);
                    }
                }
            }
        }
        (path, anchors)
    }

    /// The path, with each rounded vertex replaced by an arc.
    pub fn path(&self) -> Vec<Point> {
        self.path_with_anchors().0
    }

    pub fn length(&self) -> f64 {
        segments_length(&self.path())
    }

    /// The stretches between anchors, each of which starts and ends with a hole.
    pub fn runs(&self) -> Vec<Run> {
        let (path, anchors) = self.path_with_anchors();
        anchors
            .windows(2)
            .map(|pair| Run {
                start: pair[0],
                end: pair[1],
                length: segments_length(&path[pair[0]..=pair[1]]),
            })
            .collect()
    }

    /// How many holes the line has, without building them.
    pub fn hole_count(&self) -> usize {
        let runs = self.runs();
        if runs.is_empty() {
            return 0;
        }
        runs.iter().fold(1usize, |sum, run| {
            sum.saturating_add(run.intervals(self.pitch))
        })
    }

    /// The smallest and largest spacing of the holes, or `None` for a line without length.
    pub fn actual_pitch(&self) -> Option<(f64, f64)> {
        self.runs()
            .iter()
            .filter(|run| run.length > 0.0)
            .map(|run| run.length / run.intervals(self.pitch) as f64)
            .fold(None, |range, pitch| match range {
                None => Some((pitch, pitch)),
                Some((min, max)) => Some((pitch.min(min), pitch.max(max))),
            })
    }

    /// The holes in order from the first point. At most `MAX_HOLES_PER_LINE` are returned.
    pub fn holes(&self) -> Vec<Hole> {
        let (path, anchors) = self.path_with_anchors();
        let slant = self.hole.slant();
        let mut holes = Vec::new();
        let Some(&first) = anchors.first() else {
            return holes;
        };
        holes.push(Hole {
            center: path[first],
            angle: anchor_direction(&path, first) + slant,
        });
        for pair in anchors.windows(2) {
            let run = &path[pair[0]..=pair[1]];
            let length = segments_length(run);
            let intervals = Run {
                start: pair[0],
                end: pair[1],
                length,
            }
            .intervals(self.pitch);
            let step = length / intervals as f64;
            let mut segment = 0;
            let mut walked = 0.0;
            for k in 1..intervals {
                if holes.len() >= MAX_HOLES_PER_LINE {
                    return holes;
                }
                let target = step * k as f64;
                while segment + 2 < run.len()
                    && walked + distance(run[segment], run[segment + 1]) < target
                {
                    walked += distance(run[segment], run[segment + 1]);
                    segment += 1;
                }
                let (a, b) = (run[segment], run[segment + 1]);
                let span = distance(a, b);
                let t = if span > 0.0 {
                    ((target - walked) / span).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                holes.push(Hole {
                    center: Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t),
                    angle: (b.y - a.y).atan2(b.x - a.x) + slant,
                });
            }
            if holes.len() >= MAX_HOLES_PER_LINE {
                return holes;
            }
            holes.push(Hole {
                center: path[pair[1]],
                angle: anchor_direction(&path, pair[1]) + slant,
            });
        }
        holes
    }

    /// The closed outline of a hole, without a repeated closing point.
    pub fn hole_outline(&self, hole: &Hole) -> Vec<Point> {
        hole_outline(hole, self.hole, self.size)
    }

    /// The distance from the point to the nearest point of the path.
    pub fn distance_to(&self, point: Point) -> f64 {
        let path = self.path();
        path.windows(2)
            .map(|pair| segment_distance(point, pair[0], pair[1]))
            .fold(f64::INFINITY, f64::min)
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        for point in &mut self.points {
            point.x += dx;
            point.y += dy;
        }
    }

    pub fn handles(&self) -> Vec<Point> {
        self.points.clone()
    }

    pub fn with_handle_at(&self, handle: usize, to: Point) -> Self {
        let mut line = self.clone();
        if let Some(point) = line.points.get_mut(handle) {
            *point = to;
        }
        line
    }

    /// Whether the line has two points and valid radii; the hole parameters are checked by
    /// `Project::validate` with messages of their own.
    pub(crate) fn has_valid_size(&self) -> bool {
        self.points.len() >= 2 && valid_radii(&self.radii)
    }

    /// Whether the size and pitch are within their limits.
    pub fn has_valid_parameters(&self) -> bool {
        (MIN_HOLE_MM..=MAX_HOLE_MM).contains(&self.size)
            && (MIN_PITCH_MM..=MAX_PITCH_MM).contains(&self.pitch)
    }

    /// Whether neighboring holes stay apart at the smallest actual pitch.
    pub fn holes_overlap(&self) -> bool {
        self.actual_pitch()
            .is_some_and(|(min, _)| self.hole.footprint(self.size) >= min)
    }
}

/// The closed outline of a `kind` hole of `size`, without a repeated closing point.
pub fn hole_outline(hole: &Hole, kind: HoleKind, size: f64) -> Vec<Point> {
    let (sin, cos) = hole.angle.sin_cos();
    let at = |x: f64, y: f64| {
        Point::new(
            hole.center.x + x * cos - y * sin,
            hole.center.y + x * sin + y * cos,
        )
    };
    let (half_length, half_width) = (size * 0.5, kind.width(size) * 0.5);
    match kind {
        HoleKind::Round => (0..HOLE_SEGMENTS)
            .map(|step| {
                let theta = TAU * step as f64 / HOLE_SEGMENTS as f64;
                at(half_length * theta.cos(), half_length * theta.sin())
            })
            .collect(),
        HoleKind::French => vec![
            at(-half_length, -half_width),
            at(half_length, -half_width),
            at(half_length, half_width),
            at(-half_length, half_width),
        ],
        HoleKind::Diamond => vec![
            at(-half_length, 0.0),
            at(0.0, -half_width),
            at(half_length, 0.0),
            at(0.0, half_width),
        ],
    }
}

fn distance(a: Point, b: Point) -> f64 {
    (b.x - a.x).hypot(b.y - a.y)
}

fn segments_length(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| distance(pair[0], pair[1]))
        .sum()
}

fn direction(a: Point, b: Point) -> Option<(f64, f64)> {
    let length = distance(a, b);
    (length >= MERGE_DISTANCE_MM).then(|| ((b.x - a.x) / length, (b.y - a.y) / length))
}

/// The direction of the path at an anchor: its segment at an end, the bisector of its two
/// segments at a vertex.
fn anchor_direction(path: &[Point], index: usize) -> f64 {
    let incoming = index
        .checked_sub(1)
        .and_then(|i| direction(path[i], path[index]));
    let outgoing = path
        .get(index + 1)
        .and_then(|&next| direction(path[index], next));
    let (x, y) = match (incoming, outgoing) {
        (Some(a), Some(b)) => {
            let sum = (a.0 + b.0, a.1 + b.1);
            if sum.0.hypot(sum.1) < 1.0e-9 { a } else { sum }
        }
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => (1.0, 0.0),
    };
    y.atan2(x)
}

fn segment_distance(point: Point, a: Point, b: Point) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length_sq = dx * dx + dy * dy;
    let t = if length_sq > 0.0 {
        (((point.x - a.x) * dx + (point.y - a.y) * dy) / length_sq).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (a.x + dx * t - point.x).hypot(a.y + dy * t - point.y)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::assert_near;
    use super::*;

    fn line(points: &[(f64, f64)], hole: HoleKind, pitch: f64) -> StitchLine {
        StitchLine::new(
            points.iter().map(|&(x, y)| Point::new(x, y)).collect(),
            hole,
            pitch,
            hole.default_size(),
        )
        .unwrap()
    }

    #[test]
    fn a_line_needs_two_points() {
        assert_eq!(
            StitchLine::new(vec![Point::new(0.0, 0.0)], HoleKind::Round, 4.0, 1.0),
            None
        );
    }

    #[test]
    fn holes_are_spread_evenly_along_a_straight_run() {
        let stitch = line(&[(0.0, 0.0), (20.0, 0.0)], HoleKind::Round, 4.0);
        let holes = stitch.holes();
        assert_eq!(holes.len(), 6);
        assert_eq!(stitch.hole_count(), 6);
        for (k, hole) in holes.iter().enumerate() {
            assert_near(hole.center, Point::new(4.0 * k as f64, 0.0));
        }
        // 21 mm at a 4 mm pitch rounds to 5 intervals of 4.2 mm.
        let stretched = line(&[(0.0, 0.0), (21.0, 0.0)], HoleKind::Round, 4.0);
        let (min, max) = stretched.actual_pitch().unwrap();
        assert!((min - 4.2).abs() < 1.0e-9 && (max - 4.2).abs() < 1.0e-9);
        assert_eq!(stretched.hole_count(), 6);
    }

    #[test]
    fn ends_and_sharp_vertices_get_a_hole() {
        // 10 mm then 7 mm at a 4.5 mm pitch: 2 intervals of 5, then 2 of 3.5.
        let stitch = line(
            &[(0.0, 0.0), (10.0, 0.0), (10.0, 7.0)],
            HoleKind::Round,
            4.5,
        );
        let holes = stitch.holes();
        assert_eq!(holes.len(), 5);
        assert_eq!(stitch.hole_count(), 5);
        assert_near(holes[2].center, Point::new(10.0, 0.0));
        assert_near(holes[3].center, Point::new(10.0, 3.5));
        assert_near(holes[4].center, Point::new(10.0, 7.0));
        let (min, max) = stitch.actual_pitch().unwrap();
        assert!((min - 3.5).abs() < 1.0e-9 && (max - 5.0).abs() < 1.0e-9);
    }

    #[test]
    fn rounded_vertices_are_not_anchors() {
        let mut stitch = line(
            &[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)],
            HoleKind::Round,
            3.0,
        );
        stitch.radii = vec![0.0, 4.0, 0.0];
        assert_eq!(stitch.runs().len(), 1);
        assert!(stitch.path().len() > 3);
        assert!(stitch.length() < 20.0);
        let holes = stitch.holes();
        assert_eq!(holes.len(), stitch.hole_count());
        assert!(
            !holes.iter().any(|hole| {
                (hole.center.x - 10.0).abs() < 1.0e-9 && hole.center.y.abs() < 1.0e-9
            })
        );
    }

    #[test]
    fn slanted_holes_turn_with_the_path() {
        let stitch = line(&[(0.0, 0.0), (0.0, 12.0)], HoleKind::Diamond, 4.0);
        for hole in stitch.holes() {
            assert!((hole.angle - (TAU / 4.0 + HOLE_SLANT)).abs() < 1.0e-9);
        }
        let round = line(&[(0.0, 0.0), (0.0, 12.0)], HoleKind::Round, 4.0);
        assert!((round.holes()[1].angle - TAU / 4.0).abs() < 1.0e-9);
        // At a right-angled vertex the hole follows the bisector.
        let corner = line(&[(0.0, 0.0), (8.0, 0.0), (8.0, 8.0)], HoleKind::Round, 4.0);
        assert!((corner.holes()[2].angle - TAU / 8.0).abs() < 1.0e-9);
    }

    #[test]
    fn outlines_have_the_kind_s_shape() {
        let hole = Hole {
            center: Point::new(1.0, 2.0),
            angle: 0.0,
        };
        let round = hole_outline(&hole, HoleKind::Round, 2.0);
        assert_eq!(round.len(), HOLE_SEGMENTS);
        assert!(
            round
                .iter()
                .all(|p| (distance(*p, hole.center) - 1.0).abs() < 1.0e-9)
        );
        let diamond = hole_outline(&hole, HoleKind::Diamond, 2.0);
        assert_eq!(diamond.len(), 4);
        assert_near(diamond[0], Point::new(0.0, 2.0));
        assert_near(diamond[1], Point::new(1.0, 2.0 - 0.45));
        let french = hole_outline(&hole, HoleKind::French, 2.0);
        assert_eq!(french.len(), 4);
        assert_near(french[2], Point::new(2.0, 2.25));
    }

    #[test]
    fn overlapping_holes_are_detected() {
        let mut stitch = line(&[(0.0, 0.0), (20.0, 0.0)], HoleKind::Round, 4.0);
        assert!(stitch.has_valid_parameters());
        assert!(!stitch.holes_overlap());
        stitch.size = 4.5;
        assert!(stitch.holes_overlap());
        stitch.size = 11.0;
        assert!(!stitch.has_valid_parameters());
    }

    #[test]
    fn distance_handles_and_translation() {
        let mut stitch = line(
            &[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)],
            HoleKind::Round,
            4.0,
        );
        assert!((stitch.distance_to(Point::new(5.0, 2.0)) - 2.0).abs() < 1.0e-9);
        assert!((stitch.distance_to(Point::new(12.0, 5.0)) - 2.0).abs() < 1.0e-9);
        let moved = stitch.with_handle_at(2, Point::new(20.0, 0.0));
        assert_eq!(moved.points[2], Point::new(20.0, 0.0));
        stitch.translate(1.0, 2.0);
        assert_eq!(stitch.handles()[0], Point::new(1.0, 2.0));
    }

    fn validate_child(stitch: StitchLine) -> Result<(), crate::i18n::Message> {
        use crate::model::{BooleanOp, ChildShape, PatternPiece, Project};
        let mut project = Project::default();
        let base = super::super::Shape::rectangle_from_corners(
            Point::new(0.0, 0.0),
            Point::new(50.0, 50.0),
        )
        .unwrap();
        let mut piece = PatternPiece::new(1, "Wallet", base);
        piece.children.push(ChildShape {
            id: 1,
            op: BooleanOp::default(),
            shape: super::super::Shape::StitchLine(stitch),
        });
        project.add_piece(piece);
        project.validate()
    }

    #[test]
    fn validation_checks_the_hole_parameters() {
        use crate::i18n::Message;
        let name = || "Wallet".to_owned();
        let good = line(&[(0.0, 0.0), (20.0, 0.0)], HoleKind::Diamond, 3.85);
        assert_eq!(validate_child(good.clone()), Ok(()));

        let mut small = good.clone();
        small.size = 0.1;
        assert_eq!(
            validate_child(small),
            Err(Message::InvalidHoleParameters { name: name() })
        );
        let mut tight = good.clone();
        tight.pitch = 1.0;
        assert_eq!(
            validate_child(tight),
            Err(Message::HolesOverlap { name: name() })
        );
        let long = line(&[(0.0, 0.0), (10_000.0, 0.0)], HoleKind::Round, 1.5);
        assert_eq!(
            validate_child(long),
            Err(Message::TooManyHoles {
                name: name(),
                max: MAX_HOLES_PER_LINE
            })
        );
        let repeated = line(&[(0.0, 0.0), (0.0, 0.0), (5.0, 0.0)], HoleKind::Round, 2.0);
        assert_eq!(
            validate_child(repeated),
            Err(Message::CoincidentPoints { name: name() })
        );
        let mut negative = good;
        negative.radii = vec![0.0, -1.0];
        assert_eq!(
            validate_child(negative),
            Err(Message::InvalidShapeSize { name: name() })
        );
    }

    #[test]
    fn a_stitch_line_cannot_be_a_base() {
        use crate::model::{PatternPiece, Project};
        let mut project = Project::default();
        let stitch = line(&[(0.0, 0.0), (20.0, 0.0)], HoleKind::Round, 4.0);
        project.add_piece(PatternPiece::new(
            1,
            "Strap",
            super::super::Shape::StitchLine(stitch),
        ));
        assert!(matches!(
            project.validate(),
            Err(crate::i18n::Message::BaseMustBeClosed { .. })
        ));
    }

    #[test]
    fn pitches_fall_into_bands() {
        assert_eq!(StitchBand::from_pitch(2.7), StitchBand::Fine);
        assert_eq!(StitchBand::from_pitch(3.38), StitchBand::Classic);
        assert_eq!(StitchBand::from_pitch(3.85), StitchBand::Medium);
        assert_eq!(StitchBand::from_pitch(5.0), StitchBand::Heavy);
        assert!(StitchBand::Medium.suits_leather(3.0));
        assert!(!StitchBand::Medium.suits_leather(1.2));
        assert!(StitchBand::Heavy.suits_leather(8.0));
    }

    #[test]
    fn snapping_keeps_the_distance() {
        let snapped = StitchLine::snap(Point::new(0.0, 0.0), Point::new(10.0, 1.0));
        assert_near(snapped, Point::new(10.0_f64.hypot(1.0), 0.0));
    }
}
