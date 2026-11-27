//! The schematic of a link: each seam's range unrolled into a row of diamond holes, A above B,
//! placed by their distance along the seam, so that a creeping seam shows as slanting lines.
//! The rows are lined up at the first automatic pair. Clicks on holes set manual pairs.

use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::seams::LinkPairs;
use std::collections::HashSet;

/// Beside the rows, for the A and B letters and the direction arrows.
const LEFT_MARGIN: f32 = 32.0;
const RIGHT_MARGIN: f32 = 28.0;
/// The gap between the rows: 100 px in the design's 292 px tall box, and its share of a taller one.
const MIN_ROW_GAP: f32 = 100.0;
const ROW_GAP_SHARE: f32 = 0.34;
const HOLE_RADIUS: f32 = 5.0;
/// Holes closer than this are thinned out to every 2nd, 5th, 10th… one.
const MIN_HOLE_SPACING: f32 = 24.0;
/// Shown holes never come closer than this, so that their diamonds don't overlap.
const MIN_HOLE_GAP: f32 = 2.4 * HOLE_RADIUS;
const NUMBER_GAP: f32 = 16.0;
const MIN_NUMBER_SPACING: f32 = 18.0;
const NUMBER_SIZE: f32 = 10.0;
const HIT_RADIUS: f32 = 7.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Side {
    A,
    B,
}

impl Side {
    pub(super) fn letter(self) -> char {
        match self {
            Self::A => 'A',
            Self::B => 'B',
        }
    }
}

/// A click on a hole, with Alt held or not.
pub(super) struct Click {
    pub(super) side: Side,
    pub(super) hole: usize,
    pub(super) alt: bool,
}

/// How a hole is drawn.
#[derive(Clone, Copy, PartialEq, Eq)]
enum HoleState {
    Auto,
    Manual,
    Free,
}

/// One row of holes: each with its x on the screen and its state.
struct Row {
    side: Side,
    y: f32,
    holes: Vec<(usize, f32, HoleState)>,
    shown: HashSet<usize>,
}

impl Row {
    fn x_of(&self, hole: usize) -> Option<f32> {
        self.holes
            .iter()
            .find(|(index, ..)| *index == hole)
            .map(|&(_, x, _)| x)
    }

    fn hole_at(&self, pointer: egui::Pos2) -> Option<usize> {
        if (pointer.y - self.y).abs() > HIT_RADIUS {
            return None;
        }
        self.holes
            .iter()
            .filter(|(index, ..)| self.shown.contains(index))
            .map(|&(index, x, _)| (index, (pointer.x - x).abs()))
            .filter(|&(_, distance)| distance <= HIT_RADIUS)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }
}

/// Draws the schematic in a framed box of `size` and returns a click on a hole.
pub(super) fn schematic(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    pairs: &LinkPairs,
    offset: i64,
    pending_a: Option<usize>,
) -> Option<Click> {
    let palette = Palette::of(ui.painter());
    let paint = &palette.seam_link;
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        8,
        palette.surface.input,
        egui::Stroke::new(1.0, palette.border.divider),
        egui::StrokeKind::Inside,
    );

    let left = rect.left() + LEFT_MARGIN;
    let right = rect.right() - RIGHT_MARGIN;
    let row_gap = (rect.height() * ROW_GAP_SHARE).max(MIN_ROW_GAP);
    let y_a = rect.center().y - row_gap / 2.0;
    let y_b = rect.center().y + row_gap / 2.0;
    let (rows, rows_stride) = layout_rows(pairs, offset, left, right, (y_a, y_b), pending_a);

    for row in &rows {
        dashed_guide(&painter, left - 8.0, right + 4.0, row.y, paint.guide);
        let tip = egui::pos2(right + 14.0, row.y);
        painter.add(egui::Shape::convex_polygon(
            vec![
                tip,
                tip + egui::vec2(-7.0, -4.0),
                tip + egui::vec2(-7.0, 4.0),
            ],
            paint.arrow,
            egui::Stroke::NONE,
        ));
        painter.text(
            egui::pos2(rect.left() + 12.0, row.y),
            egui::Align2::LEFT_CENTER,
            row.side.letter(),
            egui::FontId::proportional(11.0),
            palette.text.primary,
        );
    }
    let [row_a, row_b] = &rows;

    // The lines of the holes of A that show, under the holes.
    for pair in &pairs.pairs {
        if !row_a.shown.contains(&pair.a) {
            continue;
        }
        let (Some(x_a), Some(x_b)) = (row_a.x_of(pair.a), row_b.x_of(pair.b)) else {
            continue;
        };
        let stroke = if pair.manual {
            egui::Stroke::new(1.5, paint.link_manual)
        } else {
            egui::Stroke::new(1.0, paint.link_auto.gamma_multiply(0.75))
        };
        painter.line_segment(
            [
                egui::pos2(x_a, y_a + HOLE_RADIUS),
                egui::pos2(x_b, y_b - HOLE_RADIUS),
            ],
            stroke,
        );
    }

    let hovered = response
        .hover_pos()
        .and_then(|pointer| hole_under(&rows, pointer));
    let label_step = rows_stride.max(5);
    for row in &rows {
        let mut last_number = f32::NEG_INFINITY;
        let count = row.holes.len();
        for (position, &(hole, x, state)) in row.holes.iter().enumerate() {
            if !row.shown.contains(&hole) {
                continue;
            }
            let (color, fill) = match state {
                HoleState::Auto => (paint.link_auto, paint.hole_fill),
                HoleState::Manual => (paint.link_manual, paint.hole_fill),
                HoleState::Free => (paint.link_free, paint.free_fill),
            };
            let pending = row.side == Side::A && pending_a == Some(hole);
            let fill = if pending { paint.link_manual } else { fill };
            let radius = if hovered == Some((row.side, hole)) {
                HOLE_RADIUS + 1.5
            } else {
                HOLE_RADIUS
            };
            paint_diamond(&painter, egui::pos2(x, row.y), radius, Some(fill), color);

            // The ends, every fifth (or n-th) hole and the holes without a pair are numbered.
            let numbered = position == 0
                || position + 1 == count
                || (position + 1) % label_step == 0
                || state == HoleState::Free;
            if numbered && x - last_number >= MIN_NUMBER_SPACING {
                last_number = x;
                let y = match row.side {
                    Side::A => row.y - NUMBER_GAP,
                    Side::B => row.y + NUMBER_GAP,
                };
                let color = if state == HoleState::Free {
                    paint.link_free
                } else {
                    palette.text.secondary
                };
                painter.text(
                    egui::pos2(x, y),
                    egui::Align2::CENTER_CENTER,
                    (hole + 1).to_string(),
                    egui::FontId::proportional(NUMBER_SIZE),
                    color,
                );
            }
        }
    }

    if hovered.is_some() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.clicked()
        && let Some((side, hole)) = response
            .interact_pointer_pos()
            .and_then(|pointer| hole_under(&rows, pointer))
    {
        let alt = ui.input(|input| input.modifiers.alt);
        return Some(Click { side, hole, alt });
    }
    None
}

fn hole_under(rows: &[Row; 2], pointer: egui::Pos2) -> Option<(Side, usize)> {
    rows.iter()
        .find_map(|row| row.hole_at(pointer).map(|hole| (row.side, hole)))
}

/// Places both rows. Each hole sits at its distance along its own range; B is shifted so that
/// the first automatic pair lines up, and both are scaled to fit between `left` and `right`.
fn layout_rows(
    pairs: &LinkPairs,
    offset: i64,
    left: f32,
    right: f32,
    (y_a, y_b): (f32, f32),
    pending_a: Option<usize>,
) -> ([Row; 2], usize) {
    let distance = |distances: &[Option<f64>], hole: usize| {
        distances.get(hole).copied().flatten().unwrap_or(0.0)
    };
    let skip = offset.unsigned_abs() as usize;
    let shift = |sequence: &[usize], distances: &[Option<f64>]| {
        sequence
            .get(skip)
            .map_or(0.0, |&hole| distance(distances, hole))
    };
    let (shift_a, shift_b) = if offset >= 0 {
        (0.0, shift(&pairs.sequence_b, &pairs.distance_b))
    } else {
        (shift(&pairs.sequence_a, &pairs.distance_a), 0.0)
    };
    let place_a = |hole: usize| distance(&pairs.distance_a, hole) - shift_a;
    let place_b = |hole: usize| distance(&pairs.distance_b, hole) - shift_b;
    let all = pairs
        .sequence_a
        .iter()
        .map(|&hole| place_a(hole))
        .chain(pairs.sequence_b.iter().map(|&hole| place_b(hole)));
    let (min, max) = all.fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), value| {
        (min.min(value), max.max(value))
    });
    let span = if max - min > 1.0e-9 { max - min } else { 1.0 };
    let width = right - left;
    let to_x = |value: f64| left + ((value - min) / span) as f32 * width;

    let manual_a: HashSet<usize> = pairs
        .pairs
        .iter()
        .filter(|pair| pair.manual)
        .map(|pair| pair.a)
        .collect();
    let manual_b: HashSet<usize> = pairs
        .pairs
        .iter()
        .filter(|pair| pair.manual)
        .map(|pair| pair.b)
        .collect();
    let free_a: HashSet<usize> = pairs.free_a.iter().copied().collect();
    let free_b: HashSet<usize> = pairs.free_b.iter().copied().collect();
    let longest = pairs.sequence_a.len().max(pairs.sequence_b.len()).max(1);
    let spacing = width / longest as f32;
    let stride = stride(spacing);

    let row = |side: Side,
               y: f32,
               sequence: &[usize],
               place: &dyn Fn(usize) -> f64,
               manual: &HashSet<usize>,
               free: &HashSet<usize>| {
        let state = |hole: usize| {
            if manual.contains(&hole) {
                HoleState::Manual
            } else if free.contains(&hole) {
                HoleState::Free
            } else {
                HoleState::Auto
            }
        };
        let holes: Vec<_> = sequence
            .iter()
            .map(|&hole| (hole, to_x(place(hole)), state(hole)))
            .collect();
        // The ends, manual pairs and the hole waiting for its pair always show; then the
        // unpaired holes and every n-th hole, where they leave room.
        let last = holes.len().saturating_sub(1);
        let must = |position: usize, hole: usize, state: HoleState| {
            position == 0
                || position == last
                || state == HoleState::Manual
                || (side == Side::A && pending_a == Some(hole))
        };
        let mut kept: Vec<(usize, f32)> = holes
            .iter()
            .enumerate()
            .filter(|&(position, &(hole, _, state))| must(position, hole, state))
            .map(|(_, &(hole, x, _))| (hole, x))
            .collect();
        let candidates = holes
            .iter()
            .enumerate()
            .filter(|&(position, &(hole, _, state))| {
                !must(position, hole, state) && state == HoleState::Free
            });
        let regular = holes
            .iter()
            .enumerate()
            .filter(|&(position, &(_, _, state))| {
                state == HoleState::Auto && (position + 1) % stride == 0
            });
        // Unpaired holes only need room for their diamonds; every n-th hole keeps its spacing.
        let candidates = candidates.map(|item| (item, MIN_HOLE_GAP));
        let regular = regular.map(|item| (item, MIN_HOLE_SPACING * 0.75));
        for ((_, &(hole, x, _)), gap) in candidates.chain(regular) {
            if kept.iter().all(|&(_, other)| (other - x).abs() >= gap) {
                kept.push((hole, x));
            }
        }
        let shown = kept.into_iter().map(|(hole, _)| hole).collect();
        Row {
            side,
            y,
            holes,
            shown,
        }
    };
    let rows = [
        row(
            Side::A,
            y_a,
            &pairs.sequence_a,
            &place_a,
            &manual_a,
            &free_a,
        ),
        row(
            Side::B,
            y_b,
            &pairs.sequence_b,
            &place_b,
            &manual_b,
            &free_b,
        ),
    ];
    (rows, stride)
}

/// Every how many holes one shows when they are `spacing` apart: 1, 2, 5, 10, 20, 50…
fn stride(spacing: f32) -> usize {
    let mut stride = 1;
    for factor in [2, 5, 2].into_iter().cycle() {
        if spacing * stride as f32 >= MIN_HOLE_SPACING || stride > 100_000 {
            break;
        }
        stride = if factor == 5 {
            stride / 2 * 5
        } else {
            stride * 2
        };
    }
    stride
}

fn dashed_guide(painter: &egui::Painter, from: f32, to: f32, y: f32, color: egui::Color32) {
    painter.extend(egui::Shape::dashed_line(
        &[egui::pos2(from, y), egui::pos2(to, y)],
        egui::Stroke::new(1.0, color),
        4.0,
        3.0,
    ));
}

/// A diamond hole of `radius` around `center`, filled or only outlined.
pub(super) fn paint_diamond(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    fill: Option<egui::Color32>,
    stroke: egui::Color32,
) {
    let points = vec![
        center + egui::vec2(0.0, -radius),
        center + egui::vec2(radius, 0.0),
        center + egui::vec2(0.0, radius),
        center + egui::vec2(-radius, 0.0),
    ];
    painter.add(egui::Shape::convex_polygon(
        points,
        fill.unwrap_or(egui::Color32::TRANSPARENT),
        egui::Stroke::new(1.5, stroke),
    ));
}
