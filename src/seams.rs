//! Seam links: two stitch lines, or two stretches of one, whose holes are sewn together. A link
//! keeps only its parameters (the ranges, the direction, the offset and the manual pairs); the
//! pairs themselves are computed by [`SeamLink::pairs`], so that they follow any edit of the
//! lines, as the holes do.

use crate::i18n::Message;
use crate::model::Project;
use crate::shapes::{Shape, StitchLine};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::RangeInclusive;

/// One side of a link: a stitch line and the stretch of its holes that takes part.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeamSide {
    pub piece: u64,
    /// A child of the piece whose shape is a stitch line.
    pub child: u64,
    /// The first hole of the stretch, 0-based.
    pub from: usize,
    /// The last hole of the stretch, inclusive. It may run past the line's holes after the line
    /// was shortened; the pairs are then computed with the range cut short.
    pub to: usize,
    /// Whether the holes are taken from `to` back to `from`.
    #[serde(default)]
    pub reversed: bool,
}

impl SeamSide {
    /// The whole of a line: every hole, in order.
    pub fn whole(piece: u64, child: u64, line: &StitchLine) -> Self {
        Self {
            piece,
            child,
            from: 0,
            to: line.hole_count().saturating_sub(1),
            reversed: false,
        }
    }

    /// Whether both sides are on the same stitch line.
    pub fn same_line(&self, other: &Self) -> bool {
        self.piece == other.piece && self.child == other.child
    }
}

/// A pair set by hand: hole indices on side A and side B.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualPair {
    pub a: usize,
    pub b: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeamLink {
    pub id: u64,
    pub a: SeamSide,
    pub b: SeamSide,
    /// How many holes B's sequence is shifted against A's: a positive offset skips the first
    /// holes of B, a negative one the first holes of A.
    #[serde(default)]
    pub offset: i64,
    /// Pairs set by hand, laid over the automatic ones.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub manual: Vec<ManualPair>,
}

/// Two holes sewn together.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HolePair {
    pub a: usize,
    pub b: usize,
    pub manual: bool,
    /// How far B's hole is along its stretch beyond A's hole along its own, in mm: how much the
    /// seam creeps by this pair.
    pub delta_mm: f64,
}

/// What a link comes to on the current lines.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkPairs {
    /// In the order of A's sequence.
    pub pairs: Vec<HolePair>,
    /// Holes of the ranges in no pair, ascending.
    pub free_a: Vec<usize>,
    pub free_b: Vec<usize>,
    /// The holes of each range in the order they are taken, after the direction is applied.
    pub sequence_a: Vec<usize>,
    pub sequence_b: Vec<usize>,
    /// Each hole's distance along its stretch from where the sequence starts, in mm, by hole
    /// index (holes outside the range have none).
    pub distance_a: Vec<Option<f64>>,
    pub distance_b: Vec<Option<f64>>,
    /// The path length of each used range, in mm.
    pub length_a: f64,
    pub length_b: f64,
    /// A range ran past its line's holes, or a manual pair fell outside a range.
    pub clamped: bool,
}

impl LinkPairs {
    /// Whether the ranges hold a different number of holes.
    pub fn count_mismatch(&self) -> bool {
        self.sequence_a.len() != self.sequence_b.len()
    }

    pub fn manual_count(&self) -> usize {
        self.pairs.iter().filter(|pair| pair.manual).count()
    }
}

/// The stitch line of a piece's child, if it is one.
pub fn stitch_line(project: &Project, piece: u64, child: u64) -> Option<&StitchLine> {
    let piece = project
        .pieces
        .iter()
        .find(|candidate| candidate.id == piece)?;
    match &piece.child(child)?.shape {
        Shape::StitchLine(line) => Some(line),
        _ => None,
    }
}

/// Every stitch line of the project as `(piece, child)`, in the order of pieces and children.
pub fn stitch_lines(project: &Project) -> Vec<(u64, u64)> {
    project
        .pieces
        .iter()
        .flat_map(|piece| {
            piece
                .children
                .iter()
                .filter(|child| matches!(child.shape, Shape::StitchLine(_)))
                .map(move |child| (piece.id, child.id))
        })
        .collect()
}

/// The 1-based number of a stitch line among the stitch lines of its piece.
pub fn stitch_number(project: &Project, piece: u64, child: u64) -> Option<usize> {
    let piece = project
        .pieces
        .iter()
        .find(|candidate| candidate.id == piece)?;
    piece
        .children
        .iter()
        .filter(|child| matches!(child.shape, Shape::StitchLine(_)))
        .position(|candidate| candidate.id == child)
        .map(|index| index + 1)
}

/// One side's range cut to the line's holes, with the sequence and distances along it.
struct Stretch {
    sequence: Vec<usize>,
    distance: Vec<Option<f64>>,
    length: f64,
    clamped: bool,
    /// The holes taking part, ascending; `None` when the range is past every hole.
    range: Option<RangeInclusive<usize>>,
}

fn stretch(side: &SeamSide, line: &StitchLine) -> Stretch {
    let offsets = line.hole_offsets();
    let count = offsets.len();
    let mut distance = vec![None; count];
    if count == 0 || side.from >= count {
        return Stretch {
            sequence: Vec::new(),
            distance,
            length: 0.0,
            clamped: true,
            range: None,
        };
    }
    let clamped = side.to >= count;
    let (from, to) = (side.from, side.to.min(count - 1));
    let mut sequence: Vec<usize> = (from..=to).collect();
    let start = if side.reversed {
        sequence.reverse();
        offsets[to]
    } else {
        offsets[from]
    };
    for &index in &sequence {
        distance[index] = Some((offsets[index] - start).abs());
    }
    Stretch {
        sequence,
        distance,
        length: offsets[to] - offsets[from],
        clamped,
        range: Some(from..=to),
    }
}

impl SeamLink {
    /// The stitch line on each side, or `SeamNotFound`.
    pub fn lines<'a>(
        &self,
        project: &'a Project,
    ) -> Result<(&'a StitchLine, &'a StitchLine), Message> {
        let line = |side: &SeamSide| {
            stitch_line(project, side.piece, side.child).ok_or(Message::SeamNotFound)
        };
        Ok((line(&self.a)?, line(&self.b)?))
    }

    /// Whether either side is on the child `child` of piece `piece`.
    pub fn uses_child(&self, piece: u64, child: u64) -> bool {
        [&self.a, &self.b]
            .iter()
            .any(|side| side.piece == piece && side.child == child)
    }

    pub fn uses_piece(&self, piece: u64) -> bool {
        self.a.piece == piece || self.b.piece == piece
    }

    /// Whether the ranges overlap while both sides are on one line.
    pub fn ranges_overlap(&self) -> bool {
        self.a.same_line(&self.b) && self.a.from <= self.b.to && self.b.from <= self.a.to
    }

    /// Pairs the holes of the two ranges: in step along both sequences, B shifted by `offset`,
    /// then the manual pairs laid over them, each replacing the automatic pairs that use either
    /// of its holes.
    pub fn pairs(&self, project: &Project) -> Result<LinkPairs, Message> {
        let (line_a, line_b) = self.lines(project)?;
        let a = stretch(&self.a, line_a);
        let b = stretch(&self.b, line_b);
        let mut clamped = a.clamped || b.clamped;

        let skip = self.offset.unsigned_abs() as usize;
        let (skip_a, skip_b) = if self.offset >= 0 {
            (0, skip)
        } else {
            (skip, 0)
        };
        let delta = |hole_a: usize, hole_b: usize| {
            b.distance[hole_b].unwrap_or(0.0) - a.distance[hole_a].unwrap_or(0.0)
        };
        let mut pairs: Vec<HolePair> = a
            .sequence
            .iter()
            .skip(skip_a)
            .zip(b.sequence.iter().skip(skip_b))
            .map(|(&hole_a, &hole_b)| HolePair {
                a: hole_a,
                b: hole_b,
                manual: false,
                delta_mm: delta(hole_a, hole_b),
            })
            .collect();

        for manual in &self.manual {
            let within = |stretch: &Stretch, hole: usize| {
                stretch
                    .range
                    .as_ref()
                    .is_some_and(|range| range.contains(&hole))
            };
            if !within(&a, manual.a) || !within(&b, manual.b) {
                clamped = true;
                continue;
            }
            pairs.retain(|pair| pair.a != manual.a && pair.b != manual.b);
            pairs.push(HolePair {
                a: manual.a,
                b: manual.b,
                manual: true,
                delta_mm: delta(manual.a, manual.b),
            });
        }
        // In the order of A's sequence, which is the order the seam is sewn in.
        pairs.sort_by(|x, y| {
            let place = |hole: usize| a.distance[hole].unwrap_or(f64::INFINITY);
            place(x.a).total_cmp(&place(y.a))
        });

        let used_a: HashSet<usize> = pairs.iter().map(|pair| pair.a).collect();
        let used_b: HashSet<usize> = pairs.iter().map(|pair| pair.b).collect();
        let free = |range: &Option<RangeInclusive<usize>>, used: &HashSet<usize>| {
            range
                .clone()
                .into_iter()
                .flatten()
                .filter(|hole| !used.contains(hole))
                .collect::<Vec<_>>()
        };
        Ok(LinkPairs {
            free_a: free(&a.range, &used_a),
            free_b: free(&b.range, &used_b),
            pairs,
            sequence_a: a.sequence,
            sequence_b: b.sequence,
            distance_a: a.distance,
            distance_b: b.distance,
            length_a: a.length,
            length_b: b.length,
            clamped,
        })
    }

    /// Checks the link against the project: its lines exist, its ranges are ordered and, on one
    /// line, apart, and no hole is in two manual pairs. A range past the line's holes is not an
    /// error, so that a line can be shortened; `pairs` reports it as `clamped`.
    pub(crate) fn validate(&self, project: &Project) -> Result<(), Message> {
        self.lines(project)?;
        if self.a.from > self.a.to || self.b.from > self.b.to {
            return Err(Message::InvalidSeamRange);
        }
        if self.ranges_overlap() {
            return Err(Message::SeamRangesOverlap);
        }
        let mut holes_a = HashSet::with_capacity(self.manual.len());
        let mut holes_b = HashSet::with_capacity(self.manual.len());
        for pair in &self.manual {
            if !holes_a.insert(pair.a) || !holes_b.insert(pair.b) {
                return Err(Message::DuplicateManualPair);
            }
        }
        Ok(())
    }
}

impl Project {
    pub fn seam_link(&self, id: u64) -> Option<&SeamLink> {
        self.seam_links.iter().find(|link| link.id == id)
    }

    pub fn next_seam_link_id(&self) -> u64 {
        self.seam_links
            .iter()
            .map(|link| link.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
    }

    /// The links with a side on the child `child` of piece `piece`.
    pub fn seam_links_of(&self, piece: u64, child: u64) -> impl Iterator<Item = &SeamLink> {
        self.seam_links
            .iter()
            .filter(move |link| link.uses_child(piece, child))
    }

    pub(crate) fn validate_seam_links(&self) -> Result<(), Message> {
        let mut ids = HashSet::with_capacity(self.seam_links.len());
        for link in &self.seam_links {
            if link.id == 0 || !ids.insert(link.id) {
                return Err(Message::DuplicateSeamLink);
            }
            link.validate(self)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::{BooleanOp, ChildShape, PatternPiece, Point};
    use crate::shapes::HoleKind;

    /// A straight stitch line of `holes` holes `pitch` apart, along y = `y`.
    pub(crate) fn straight(holes: usize, pitch: f64, y: f64) -> Shape {
        let length = pitch * (holes - 1) as f64;
        Shape::StitchLine(
            StitchLine::new(
                vec![Point::new(5.0, y), Point::new(5.0 + length, y)],
                HoleKind::Round,
                pitch,
                1.2,
            )
            .unwrap(),
        )
    }

    /// Two pieces, each with one stitch line: piece 1 child 1 of `a` holes, piece 2 child 1 of
    /// `b` holes, both at a 4 mm pitch unless `pitch_b` differs.
    pub(crate) fn two_lines(a: usize, b: usize, pitch_b: f64) -> Project {
        let mut project = Project::default();
        for (id, holes, pitch) in [(1, a, 4.0), (2, b, pitch_b)] {
            let base = Shape::rectangle_from_corners(Point::new(0.0, 0.0), Point::new(300.0, 40.0))
                .unwrap();
            let mut piece = PatternPiece::new(id, format!("Piece {id}"), base);
            piece.children.push(ChildShape {
                id: 1,
                op: BooleanOp::default(),
                shape: straight(holes, pitch, 20.0),
            });
            project.add_piece(piece);
        }
        project
    }

    pub(crate) fn side(piece: u64, from: usize, to: usize) -> SeamSide {
        SeamSide {
            piece,
            child: 1,
            from,
            to,
            reversed: false,
        }
    }

    pub(crate) fn link(a: SeamSide, b: SeamSide) -> SeamLink {
        SeamLink {
            id: 1,
            a,
            b,
            offset: 0,
            manual: Vec::new(),
        }
    }

    fn pairs_of(pairs: &LinkPairs) -> Vec<(usize, usize)> {
        pairs.pairs.iter().map(|pair| (pair.a, pair.b)).collect()
    }

    #[test]
    fn equal_counts_pair_every_hole() {
        let project = two_lines(10, 10, 4.0);
        let pairs = link(side(1, 0, 9), side(2, 0, 9)).pairs(&project).unwrap();
        assert_eq!(pairs.pairs.len(), 10);
        assert!(pairs.free_a.is_empty() && pairs.free_b.is_empty());
        assert!(
            pairs
                .pairs
                .iter()
                .all(|pair| pair.a == pair.b && !pair.manual)
        );
        assert!(pairs.pairs.iter().all(|pair| pair.delta_mm.abs() < 1.0e-9));
        assert!((pairs.length_a - 36.0).abs() < 1.0e-9);
        assert!(!pairs.clamped && !pairs.count_mismatch());
    }

    #[test]
    fn reversing_b_turns_its_sequence_around() {
        let project = two_lines(5, 5, 4.0);
        let mut reversed = side(2, 0, 4);
        reversed.reversed = true;
        let pairs = link(side(1, 0, 4), reversed).pairs(&project).unwrap();
        assert_eq!(pairs_of(&pairs), [(0, 4), (1, 3), (2, 2), (3, 1), (4, 0)]);
        assert!(pairs.pairs.iter().all(|pair| pair.delta_mm.abs() < 1.0e-9));
    }

    #[test]
    fn offsets_skip_the_first_holes_of_one_side() {
        let project = two_lines(5, 5, 4.0);
        let mut shifted = link(side(1, 0, 4), side(2, 0, 4));
        shifted.offset = 2;
        let pairs = shifted.pairs(&project).unwrap();
        assert_eq!(pairs_of(&pairs), [(0, 2), (1, 3), (2, 4)]);
        assert_eq!(pairs.free_a, [3, 4]);
        assert_eq!(pairs.free_b, [0, 1]);

        shifted.offset = -1;
        let pairs = shifted.pairs(&project).unwrap();
        assert_eq!(pairs_of(&pairs), [(1, 0), (2, 1), (3, 2), (4, 3)]);
        assert_eq!(pairs.free_a, [0]);
        assert_eq!(pairs.free_b, [4]);
    }

    #[test]
    fn a_manual_pair_replaces_the_automatic_pairs_of_its_holes() {
        let project = two_lines(48, 52, 4.0);
        let mut manual = link(side(1, 0, 47), side(2, 0, 51));
        manual.manual.push(ManualPair { a: 47, b: 51 });
        let pairs = manual.pairs(&project).unwrap();
        // 47–47 is gone; 51 was unpaired, so only one automatic pair is replaced here.
        assert_eq!(pairs.pairs.len(), 48);
        let last = pairs.pairs.last().unwrap();
        assert_eq!((last.a, last.b, last.manual), (47, 51, true));
        assert_eq!(pairs.free_b, [47, 48, 49, 50]);
        assert!((last.delta_mm - 16.0).abs() < 1.0e-9);

        // A pair across two automatic pairs removes both.
        let mut cross = link(side(1, 0, 9), side(2, 0, 9));
        cross.manual.push(ManualPair { a: 2, b: 5 });
        let pairs = cross.pairs(&project_with(10, 10)).unwrap();
        assert_eq!(pairs.pairs.len(), 9);
        assert_eq!(pairs.manual_count(), 1);
        assert_eq!(pairs.free_a, [5]);
        assert_eq!(pairs.free_b, [2]);
    }

    fn project_with(a: usize, b: usize) -> Project {
        two_lines(a, b, 4.0)
    }

    #[test]
    fn different_counts_leave_holes_free() {
        let project = two_lines(48, 52, 4.0);
        let pairs = link(side(1, 0, 47), side(2, 0, 51))
            .pairs(&project)
            .unwrap();
        assert_eq!(pairs.pairs.len(), 48);
        assert!(pairs.free_a.is_empty());
        assert_eq!(pairs.free_b, [48, 49, 50, 51]);
        assert!(pairs.count_mismatch());
    }

    #[test]
    fn a_creeping_seam_shows_in_the_delta() {
        let project = two_lines(10, 10, 4.5);
        let pairs = link(side(1, 0, 9), side(2, 0, 9)).pairs(&project).unwrap();
        assert!((pairs.pairs[9].delta_mm - 4.5).abs() < 1.0e-9);
        assert!((pairs.length_b - pairs.length_a - 4.5).abs() < 1.0e-9);
    }

    #[test]
    fn a_shortened_line_clamps_the_range() {
        let project = two_lines(10, 6, 4.0);
        let mut stale = link(side(1, 0, 9), side(2, 0, 9));
        let pairs = stale.pairs(&project).unwrap();
        assert!(pairs.clamped);
        assert_eq!(pairs.pairs.len(), 6);
        assert_eq!(pairs.free_a, [6, 7, 8, 9]);

        stale.b.to = 5;
        stale.manual.push(ManualPair { a: 0, b: 8 });
        assert!(stale.pairs(&project).unwrap().clamped);
    }

    #[test]
    fn one_line_links_to_itself_with_apart_ranges() {
        let project = two_lines(20, 5, 4.0);
        let mut fold = link(side(1, 0, 9), side(1, 10, 19));
        fold.b.reversed = true;
        let pairs = fold.pairs(&project).unwrap();
        assert_eq!(pairs.pairs.len(), 10);
        assert_eq!((pairs.pairs[0].a, pairs.pairs[0].b), (0, 19));
        assert_eq!((pairs.pairs[9].a, pairs.pairs[9].b), (9, 10));
        assert!(!fold.ranges_overlap());

        fold.b.from = 9;
        assert!(fold.ranges_overlap());
    }

    #[test]
    fn missing_lines_are_reported() {
        let project = two_lines(5, 5, 4.0);
        let mut missing = link(side(1, 0, 4), side(2, 0, 4));
        missing.b.child = 7;
        assert_eq!(missing.pairs(&project), Err(Message::SeamNotFound));
    }

    #[test]
    fn stitch_lines_are_listed_and_numbered() {
        let project = two_lines(5, 5, 4.0);
        assert_eq!(stitch_lines(&project), [(1, 1), (2, 1)]);
        assert_eq!(stitch_number(&project, 2, 1), Some(1));
        assert_eq!(stitch_number(&project, 2, 9), None);
    }
}
