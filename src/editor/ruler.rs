//! The rulers along the top and left edges of the canvas, from the "Ruler Top", "Ruler Left" and
//! "Ruler Corner" of the Brilliant canvas design. They count in the project's display units.

use super::canvas::Viewport;
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::model::{Point, Units};

/// The thickness of a ruler strip.
pub(super) const RULER_SIZE: f32 = 20.0;
/// Numbered ticks are at least this far apart, so their numbers don't run into each other.
const MIN_LABEL_SPACING_PX: f64 = 64.0;
/// The steps between numbered ticks, before the power of ten.
const LABEL_MANTISSAS: [f64; 3] = [1.0, 2.0, 5.0];
/// Ticks between two numbered ones; every second one is longer.
const SUBDIVISIONS: i64 = 10;
const MINOR_TICK_PX: f32 = 3.0;
const MEDIUM_TICK_PX: f32 = 5.0;
const MAJOR_TICK_PX: f32 = 8.0;
/// Ticks, borders and the cursor mark.
const LINE_WIDTH: f32 = 1.0;
const LABEL_SIZE: f32 = 9.0;
/// From a numbered tick to its number.
const LABEL_GAP: f32 = 3.0;
/// From the outer edge of the strip to the numbers.
const LABEL_INSET: f32 = 2.0;

#[derive(Clone, Copy, PartialEq)]
enum Axis {
    /// The top ruler, counting x.
    Horizontal,
    /// The left ruler, counting y.
    Vertical,
}

/// The part of `rect` left for the drawing once the rulers take their strips.
pub(super) fn content_rect(rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(rect.min + egui::vec2(RULER_SIZE, RULER_SIZE), rect.max)
}

/// The top and left strips, without the corner square where they meet.
pub(super) fn strips(rect: egui::Rect) -> [egui::Rect; 2] {
    let top = egui::Rect::from_min_max(
        egui::pos2(rect.left() + RULER_SIZE, rect.top()),
        egui::pos2(rect.right(), rect.top() + RULER_SIZE),
    );
    let left = egui::Rect::from_min_max(
        egui::pos2(rect.left(), rect.top() + RULER_SIZE),
        egui::pos2(rect.left() + RULER_SIZE, rect.bottom()),
    );
    [top, left]
}

/// Both rulers and the corner square, with the cursor marked on them when it is over the canvas.
pub(super) fn draw_rulers(
    painter: &egui::Painter,
    rect: egui::Rect,
    view: Viewport,
    units: Units,
    cursor: Option<Point>,
) {
    let [top, left] = strips(rect);
    draw_strip(painter, top, Axis::Horizontal, view, units, cursor);
    draw_strip(painter, left, Axis::Vertical, view, units, cursor);

    let colors = &Palette::of(painter).ruler;
    let corner = egui::Rect::from_min_size(rect.min, egui::vec2(RULER_SIZE, RULER_SIZE));
    painter.rect_filled(corner, 0.0, colors.background);
    let border = egui::Stroke::new(LINE_WIDTH, colors.border);
    painter.vline(corner.right() - 0.5, corner.y_range(), border);
    painter.hline(corner.x_range(), corner.bottom() - 0.5, border);
}

/// The step between numbered ticks, in display units: 1, 2 or 5 times a power of ten.
fn label_step(px_per_unit: f64) -> f64 {
    (-4..=7)
        .flat_map(|exponent| {
            LABEL_MANTISSAS
                .into_iter()
                .map(move |mantissa| mantissa * 10f64.powi(exponent))
        })
        .find(|step| step * px_per_unit >= MIN_LABEL_SPACING_PX)
        .unwrap_or(1e7)
}

/// A number on the ruler, with as many decimals as the step needs and never a "-0".
fn format_label(value: f64, step: f64) -> String {
    let decimals = (-step.log10().floor()).max(0.0) as usize;
    let text = format!("{value:.decimals$}");
    if text
        .trim_start_matches('-')
        .chars()
        .all(|c| c == '0' || c == '.')
    {
        text.trim_start_matches('-').to_owned()
    } else {
        text
    }
}

fn draw_strip(
    painter: &egui::Painter,
    strip: egui::Rect,
    axis: Axis,
    view: Viewport,
    units: Units,
    cursor: Option<Point>,
) {
    if strip.width() <= 0.0 || strip.height() <= 0.0 {
        return;
    }
    let painter = painter.with_clip_rect(strip);
    let palette = Palette::of(&painter);
    let colors = &palette.ruler;
    painter.rect_filled(strip, 0.0, colors.background);

    let mm_per_unit = 1.0 / units.from_millimeters(1.0);
    let step = label_step(f64::from(view.zoom) * mm_per_unit);
    let minor = step / SUBDIVISIONS as f64;
    // The world coordinate, in mm, of a screen position along the strip, and back.
    let to_world = |position: egui::Pos2| {
        let point = view.to_world(position);
        if axis == Axis::Horizontal {
            point.x
        } else {
            point.y
        }
    };
    let to_screen = |millimeters: f64| match axis {
        Axis::Horizontal => view.to_screen(Point::new(millimeters, 0.0)).x,
        Axis::Vertical => view.to_screen(Point::new(0.0, millimeters)).y,
    };
    let start = units.from_millimeters(to_world(strip.min));
    let end = units.from_millimeters(to_world(strip.max));
    let first = (start / minor).floor() as i64;
    let last = (end / minor).ceil() as i64;

    let tick = egui::Stroke::new(LINE_WIDTH, colors.tick);
    let font = egui::FontId::monospace(LABEL_SIZE);
    for index in first..=last {
        let position = to_screen(index as f64 * minor * mm_per_unit);
        let position = painter.round_to_pixel_center(position);
        let numbered = index.rem_euclid(SUBDIVISIONS) == 0;
        let length = if numbered {
            MAJOR_TICK_PX
        } else if index.rem_euclid(2) == 0 {
            MEDIUM_TICK_PX
        } else {
            MINOR_TICK_PX
        };
        match axis {
            Axis::Horizontal => painter.vline(
                position,
                egui::Rangef::new(strip.bottom() - length, strip.bottom()),
                tick,
            ),
            Axis::Vertical => painter.hline(
                egui::Rangef::new(strip.right() - length, strip.right()),
                position,
                tick,
            ),
        };
        if !numbered {
            continue;
        }
        let text = format_label(index.div_euclid(SUBDIVISIONS) as f64 * step, step);
        let galley = painter.layout_no_wrap(text, font.clone(), colors.label);
        match axis {
            Axis::Horizontal => {
                let at = egui::pos2(position + LABEL_GAP, strip.top() + LABEL_INSET);
                painter.galley(at, galley, colors.label);
            }
            Axis::Vertical => {
                // Turned a quarter counterclockwise, reading upwards and ending just past the tick.
                let at = egui::pos2(
                    strip.left() + LABEL_INSET,
                    position + LABEL_GAP + galley.size().x,
                );
                painter.add(
                    egui::epaint::TextShape::new(at, galley, colors.label)
                        .with_angle(-std::f32::consts::FRAC_PI_2),
                );
            }
        }
    }

    let border = egui::Stroke::new(LINE_WIDTH, colors.border);
    match axis {
        Axis::Horizontal => painter.hline(strip.x_range(), strip.bottom() - 0.5, border),
        Axis::Vertical => painter.vline(strip.right() - 0.5, strip.y_range(), border),
    };
    if let Some(cursor) = cursor {
        let millimeters = if axis == Axis::Horizontal {
            cursor.x
        } else {
            cursor.y
        };
        let position = painter.round_to_pixel_center(to_screen(millimeters));
        let mark = egui::Stroke::new(LINE_WIDTH, colors.cursor);
        match axis {
            Axis::Horizontal => painter.vline(position, strip.y_range(), mark),
            Axis::Vertical => painter.hline(strip.x_range(), position, mark),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_step_keeps_numbers_apart() {
        assert_eq!(label_step(2.0), 50.0);
        assert_eq!(label_step(64.0), 1.0);
        assert_eq!(label_step(100.0), 1.0);
        assert_eq!(label_step(10.0), 10.0);
        assert_eq!(label_step(500.0), 0.2);
    }

    #[test]
    fn labels_have_the_decimals_of_their_step() {
        assert_eq!(format_label(150.0, 50.0), "150");
        assert_eq!(format_label(-0.0, 1.0), "0");
        assert_eq!(format_label(0.4, 0.2), "0.4");
        assert_eq!(format_label(-0.00000001, 0.1), "0.0");
    }
}
