use super::Viewport;
use bevy_egui::egui;
use leathercraft::model::Point;

const GRID_STEPS_MM: [f64; 9] = [
    1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1_000.0, 5_000.0, 10_000.0,
];
const MIN_GRID_SPACING_PX: f64 = 12.0;

pub(super) fn draw_grid(painter: &egui::Painter, rect: egui::Rect, view: Viewport) {
    let zoom = f64::from(view.zoom);
    let step = GRID_STEPS_MM
        .into_iter()
        .find(|step| step * zoom >= MIN_GRID_SPACING_PX)
        .unwrap_or(GRID_STEPS_MM[GRID_STEPS_MM.len() - 1]);
    let top_left = view.to_world(rect.left_top());
    let bottom_right = view.to_world(rect.right_bottom());
    let line = egui::Stroke::new(1.0, egui::Color32::from_gray(60));
    let axis = egui::Stroke::new(1.5, egui::Color32::from_gray(110));

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
}
