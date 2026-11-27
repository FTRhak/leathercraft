//! The table of hole pairs: number, hole of A, hole of B and how far the seam creeps, with the
//! manual pairs marked and the holes without a pair collapsed into ranges at the end.

use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Units;
use leathercraft::seams::LinkPairs;

const ROW_HEIGHT: f32 = 28.0;
const TABLE_HEIGHT: f32 = 230.0;
const PADDING: f32 = 10.0;
/// The x of each column after the left padding, from the design.
const COLUMNS: [f32; 4] = [0.0, 34.0, 80.0, 140.0];
const TEXT: f32 = 12.0;
const HEADER_TEXT: f32 = 11.0;
const BADGE_TEXT: f32 = 10.0;

enum Row {
    Pair {
        number: usize,
        a: usize,
        b: usize,
        delta: String,
        manual: bool,
    },
    /// Holes of one side without a pair, as ranges such as `49–51`.
    Free {
        a: Option<String>,
        b: Option<String>,
    },
}

pub(super) fn pair_table(ui: &mut egui::Ui, pairs: &LinkPairs, units: Units, language: Language) {
    let palette = Palette::of(ui.painter());
    let rows = rows(pairs, units);
    let width = ui.available_width();
    egui::Frame::new()
        .fill(palette.surface.input)
        .stroke(egui::Stroke::new(1.0, palette.border.divider))
        .corner_radius(8)
        .show(ui, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            let unit = language.unit_symbol(units);
            let header = ["#", "A", "B", &format!("Δ {unit}")].map(str::to_owned);
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(width, ROW_HEIGHT), egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                egui::CornerRadius {
                    nw: 8,
                    ne: 8,
                    sw: 0,
                    se: 0,
                },
                palette.surface.ruler,
            );
            for (text, x) in header.iter().zip(COLUMNS) {
                cell(
                    ui.painter(),
                    rect,
                    x,
                    text,
                    egui::FontId::proportional(HEADER_TEXT),
                    palette.text.secondary,
                );
            }
            divider(ui.painter(), rect, palette.border.divider);

            egui::ScrollArea::vertical()
                .max_height(TABLE_HEIGHT)
                .auto_shrink([false, true])
                .show_rows(ui, ROW_HEIGHT, rows.len(), |ui, range| {
                    for row in &rows[range] {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), ROW_HEIGHT),
                            egui::Sense::hover(),
                        );
                        paint_row(ui.painter(), rect, row, language);
                    }
                });
        });
}

fn rows(pairs: &LinkPairs, units: Units) -> Vec<Row> {
    let decimals = match units {
        Units::Millimeters => 1,
        Units::Inches => 3,
    };
    let mut rows: Vec<Row> = pairs
        .pairs
        .iter()
        .enumerate()
        .map(|(index, pair)| Row::Pair {
            number: index + 1,
            a: pair.a + 1,
            b: pair.b + 1,
            delta: format!("{:.*}", decimals, units.from_millimeters(pair.delta_mm)),
            manual: pair.manual,
        })
        .collect();
    rows.extend(ranges(&pairs.free_a).into_iter().map(|a| Row::Free {
        a: Some(a),
        b: None,
    }));
    rows.extend(ranges(&pairs.free_b).into_iter().map(|b| Row::Free {
        a: None,
        b: Some(b),
    }));
    rows
}

/// Ascending hole indices as 1-based ranges: `[48, 49, 50, 52]` is `49–51` and `53`.
fn ranges(holes: &[usize]) -> Vec<String> {
    let mut ranges = Vec::new();
    let mut iter = holes.iter().copied().peekable();
    while let Some(start) = iter.next() {
        let mut end = start;
        while iter.peek() == Some(&(end + 1)) {
            end += 1;
            iter.next();
        }
        ranges.push(if start == end {
            (start + 1).to_string()
        } else {
            format!("{}–{}", start + 1, end + 1)
        });
    }
    ranges
}

fn paint_row(painter: &egui::Painter, rect: egui::Rect, row: &Row, language: Language) {
    let palette = Palette::of(painter);
    let paint = &palette.seam_link;
    let mono = egui::FontId::monospace(TEXT);
    match row {
        Row::Pair {
            number,
            a,
            b,
            delta,
            manual,
        } => {
            if *manual {
                painter.rect_filled(rect, 0, paint.row_manual);
            }
            let cells = [
                (number.to_string(), palette.text.muted),
                (a.to_string(), palette.text.primary),
                (b.to_string(), palette.text.primary),
                (delta.clone(), palette.text.label),
            ];
            for ((text, color), x) in cells.into_iter().zip(COLUMNS) {
                cell(painter, rect, x, &text, mono.clone(), color);
            }
            if *manual {
                badge(
                    painter,
                    rect,
                    language.label(Label::PairManual),
                    paint.badge_manual,
                    paint.link_manual,
                );
            }
        }
        Row::Free { a, b } => {
            painter.rect_filled(rect, 0, paint.row_free);
            let side = |text: &Option<String>| match text {
                Some(text) => (text.clone(), paint.link_free),
                None => ("—".to_owned(), palette.text.primary),
            };
            let cells = [("—".to_owned(), palette.text.muted), side(a), side(b)];
            for ((text, color), x) in cells.into_iter().zip(COLUMNS) {
                cell(painter, rect, x, &text, mono.clone(), color);
            }
            badge(
                painter,
                rect,
                language.label(Label::PairFree),
                paint.badge_free,
                paint.link_free,
            );
        }
    }
    divider(painter, rect, paint.row_divider);
}

fn cell(
    painter: &egui::Painter,
    rect: egui::Rect,
    x: f32,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
) {
    painter.text(
        egui::pos2(rect.left() + PADDING + x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        font,
        color,
    );
}

/// A small badge at the right end of a row.
fn badge(
    painter: &egui::Painter,
    rect: egui::Rect,
    text: &str,
    fill: egui::Color32,
    color: egui::Color32,
) {
    let galley = painter.layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(BADGE_TEXT),
        color,
    );
    let size = galley.size() + egui::vec2(14.0, 6.0);
    let badge = egui::Rect::from_min_size(
        egui::pos2(
            rect.right() - PADDING - size.x,
            rect.center().y - size.y / 2.0,
        ),
        size,
    );
    painter.rect_filled(badge, 4, fill);
    painter.galley(badge.center() - galley.size() / 2.0, galley, color);
}

fn divider(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        egui::Stroke::new(1.0, color),
    );
}

#[cfg(test)]
mod tests {
    use super::ranges;

    #[test]
    fn holes_collapse_into_ranges() {
        assert_eq!(ranges(&[48, 49, 50, 52]), ["49–51", "53"]);
        assert!(ranges(&[]).is_empty());
    }
}
