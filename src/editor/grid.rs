use super::canvas::Viewport;
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::model::Point;

const GRID_STEPS_MM: [f64; 9] = [
    1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1_000.0, 5_000.0, 10_000.0,
];
const MIN_GRID_SPACING_PX: f64 = 12.0;
/// The snap grid is drawn as dots once they are at least this far apart.
const MIN_SNAP_DOT_SPACING_PX: f64 = 8.0;
/// Past this many dots the snap grid isn't drawn, however far apart they are.
const MAX_SNAP_DOTS: i64 = 40_000;

/// The grid lines and axes, then the dots of the snap grid when `snap_step` is given.
pub(super) fn draw_grid(
    painter: &egui::Painter,
    rect: egui::Rect,
    view: Viewport,
    snap_step: Option<f64>,
) {
    let zoom = f64::from(view.zoom);
    let step = GRID_STEPS_MM
        .into_iter()
        .find(|step| step * zoom >= MIN_GRID_SPACING_PX)
        .unwrap_or(GRID_STEPS_MM[GRID_STEPS_MM.len() - 1]);
    let top_left = view.to_world(rect.left_top());
    let bottom_right = view.to_world(rect.right_bottom());
    let palette = Palette::of(painter);
    let line = egui::Stroke::new(1.0, palette.grid_line);
    let axis = egui::Stroke::new(1.5, palette.grid_axis);

    let first_x = (top_left.x / step).floor() as i64;
    let last_x = (bottom_right.x / step).ceil() as i64;
    for index in first_x..=last_x {
        let x = view.to_screen(Point::new(index as f64 * step, 0.0)).x;
        painter.vline(x, rect.y_range(), if index == 0 { axis } else { line });
    }
    let first_y = (top_left.y / step).floor() as i64;
    let last_y = (bottom_right.y / step).ceil() as i64;
    for index in first_y..=last_y {
        let y = view.to_screen(Point::new(0.0, index as f64 * step)).y;
        painter.hline(rect.x_range(), y, if index == 0 { axis } else { line });
    }
    if let Some(step) = snap_step {
        draw_snap_dots(painter, view, top_left, bottom_right, step);
    }
}

fn draw_snap_dots(
    painter: &egui::Painter,
    view: Viewport,
    top_left: Point,
    bottom_right: Point,
    step: f64,
) {
    if step <= 0.0 || step * f64::from(view.zoom) < MIN_SNAP_DOT_SPACING_PX {
        return;
    }
    let first_x = (top_left.x / step).floor() as i64;
    let last_x = (bottom_right.x / step).ceil() as i64;
    let first_y = (top_left.y / step).floor() as i64;
    let last_y = (bottom_right.y / step).ceil() as i64;
    if (last_x - first_x + 1) * (last_y - first_y + 1) > MAX_SNAP_DOTS {
        return;
    }
    let color = Palette::of(painter).snap_dot;
    for x in first_x..=last_x {
        for y in first_y..=last_y {
            let position = view.to_screen(Point::new(x as f64 * step, y as f64 * step));
            painter.circle_filled(position, 1.0, color);
        }
    }
}
