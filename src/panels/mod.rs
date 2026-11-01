use bevy_egui::egui;

pub mod layout;
pub mod tools;

/// A square button with a painted icon; the closure gets the button's rect and stroke.
fn icon_button(
    ui: &mut egui::Ui,
    size: f32,
    selected: bool,
    name: &str,
    paint: impl FnOnce(&egui::Painter, egui::Rect, egui::Stroke),
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, name)
    });
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, selected);
        ui.painter().rect(
            rect,
            visuals.corner_radius,
            visuals.weak_bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        paint(ui.painter(), rect, visuals.fg_stroke);
    }
    response.on_hover_text(name)
}
