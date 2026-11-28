use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

const ICON_FONT: &str = "leathercraft-icons";

/// Glyphs of the icon font, in its private use area.
pub mod glyph {
    pub const TRASH: char = '\u{E110}';
    pub const LOCK: char = '\u{E115}';
    pub const UNLOCK: char = '\u{E116}';
    pub const HISTORY: char = '\u{E121}';
    pub const PROPERTIES: char = '\u{E122}';
    pub const LAYOUT: char = '\u{E123}';
    pub const SELECT: char = '\u{E130}';
    pub const DRAW: char = '\u{E132}';
    pub const RECTANGLE: char = '\u{E133}';
    pub const ELLIPSE: char = '\u{E134}';
    pub const EDGE: char = '\u{E131}';
    pub const STITCH_HOLES: char = '\u{E136}';
    pub const CENTERLINE: char = '\u{E137}';
    pub const REGULAR_POLYGON: char = '\u{E138}';
    pub const NEW_PIECE: char = '\u{E13C}';
    pub const SUBTRACT: char = '\u{E13D}';
    pub const INTERSECT: char = '\u{E13F}';
    pub const UNION: char = '\u{E13E}';
    pub const ALIGN_LEFT: char = '\u{E140}';
    pub const ALIGN_CENTER_X: char = '\u{E141}';
    pub const ALIGN_RIGHT: char = '\u{E142}';
    pub const ALIGN_TOP: char = '\u{E143}';
    pub const ALIGN_CENTER_Y: char = '\u{E144}';
    pub const ALIGN_BOTTOM: char = '\u{E145}';
    pub const DISTRIBUTE_TOP: char = '\u{E146}';
    pub const DISTRIBUTE_CENTER_Y: char = '\u{E147}';
    pub const DISTRIBUTE_BOTTOM: char = '\u{E148}';
    pub const DISTRIBUTE_LEFT: char = '\u{E149}';
    pub const DISTRIBUTE_CENTER_X: char = '\u{E14A}';
    pub const DISTRIBUTE_RIGHT: char = '\u{E14B}';
    pub const SPACE_VERTICAL: char = '\u{E14C}';
    pub const SPACE_HORIZONTAL: char = '\u{E14D}';
}

fn icon_font() -> egui::FontFamily {
    egui::FontFamily::Name(ICON_FONT.into())
}

/// Registers the icon font as its own family; run once, it takes effect from the next pass,
/// so nothing may use the family in the pass that installs it.
pub fn install_icon_font(mut contexts: EguiContexts) -> Result {
    let data = egui::FontData::from_static(include_bytes!("../../res/leathercrafticons.ttf"));
    contexts
        .ctx_mut()?
        .add_font(egui::epaint::text::FontInsert::new(
            ICON_FONT,
            data,
            vec![egui::epaint::text::InsertFontFamily {
                family: icon_font(),
                priority: egui::epaint::text::FontPriority::Highest,
            }],
        ));
    Ok(())
}

/// Paints a glyph of the icon font centered in `rect`, in the stroke's color.
pub fn paint_glyph(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke, glyph: char) {
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::new(rect.height() * 0.75, icon_font()),
        stroke.color,
    );
}

/// Two chain links on the diagonal: linked seams. Painted, as the icon font has no such glyph.
pub fn paint_link(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let side = rect.width().min(rect.height());
    let center = rect.center();
    let along = egui::vec2(1.0, -1.0).normalized();
    let (half_length, radius) = (side * 0.2, side * 0.16);
    for sign in [-1.0, 1.0] {
        let link_center = center + along * (sign * side * 0.17);
        painter.add(egui::Shape::closed_line(
            capsule(link_center, along, half_length, radius),
            stroke,
        ));
    }
}

/// The outline of a stadium around `center`, long along the unit vector `along`.
fn capsule(
    center: egui::Pos2,
    along: egui::Vec2,
    half_length: f32,
    radius: f32,
) -> Vec<egui::Pos2> {
    const ARC_STEPS: usize = 8;
    let across = egui::vec2(-along.y, along.x);
    let mut points = Vec::with_capacity(2 * (ARC_STEPS + 1));
    for (end, start_angle) in [(1.0, -90.0_f32), (-1.0, 90.0)] {
        let cap = center + along * (end * half_length);
        for step in 0..=ARC_STEPS {
            let angle = (start_angle + 180.0 * step as f32 / ARC_STEPS as f32).to_radians();
            points.push(cap + (along * angle.cos() + across * angle.sin()) * radius);
        }
    }
    points
}

/// Two opposite arrows, one above the other: reverse the direction.
pub fn paint_reverse(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let rect = rect.shrink(rect.width() * 0.1);
    let head = rect.height() * 0.22;
    let rows = [
        (rect.top() + rect.height() * 0.3, 1.0),
        (rect.top() + rect.height() * 0.7, -1.0),
    ];
    for (y, direction) in rows {
        let (from, to) = if direction > 0.0 {
            (rect.left(), rect.right())
        } else {
            (rect.right(), rect.left())
        };
        let tip = egui::pos2(to, y);
        painter.line_segment([egui::pos2(from, y), tip], stroke);
        let back = -direction * head;
        painter.line_segment([tip, egui::pos2(to + back, y - head)], stroke);
        painter.line_segment([tip, egui::pos2(to + back, y + head)], stroke);
    }
}

/// A triangle with an exclamation mark: a warning.
pub fn paint_warning(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let side = rect.width().min(rect.height());
    let center = rect.center();
    let top = egui::pos2(center.x, center.y - side * 0.45);
    let left = egui::pos2(center.x - side * 0.5, center.y + side * 0.42);
    let right = egui::pos2(center.x + side * 0.5, center.y + side * 0.42);
    painter.add(egui::Shape::closed_line(vec![top, right, left], stroke));
    painter.line_segment(
        [
            egui::pos2(center.x, center.y - side * 0.12),
            egui::pos2(center.x, center.y + side * 0.14),
        ],
        stroke,
    );
    painter.circle_filled(
        egui::pos2(center.x, center.y + side * 0.28),
        stroke.width * 0.7,
        stroke.color,
    );
}

/// A cross: close.
pub fn paint_close(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let rect = rect.shrink(rect.width() * 0.25);
    painter.line_segment([rect.left_top(), rect.right_bottom()], stroke);
    painter.line_segment([rect.right_top(), rect.left_bottom()], stroke);
}

/// An eye, or an eye struck through when `open` is false: shown or hidden.
pub fn paint_eye(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke, open: bool) {
    const STEPS: usize = 10;
    let center = rect.center();
    let half_width = rect.width().min(rect.height()) * 0.42;
    let half_height = half_width * 0.55;
    // Two arcs meeting at the corners of the eye.
    let mut outline = Vec::with_capacity(2 * STEPS);
    for (sign, range) in [(-1.0_f32, 0..STEPS), (1.0, 0..STEPS)] {
        for step in range {
            let t = step as f32 / STEPS as f32;
            let x = if sign < 0.0 {
                -1.0 + 2.0 * t
            } else {
                1.0 - 2.0 * t
            };
            let y = sign * (1.0 - x * x).max(0.0).sqrt();
            outline.push(center + egui::vec2(x * half_width, y * half_height));
        }
    }
    painter.add(egui::Shape::closed_line(outline, stroke));
    painter.circle_stroke(center, half_height * 0.55, stroke);
    if !open {
        let reach = half_width * 0.95;
        painter.line_segment(
            [
                center + egui::vec2(-reach, reach * 0.8),
                center + egui::vec2(reach, -reach * 0.8),
            ],
            stroke,
        );
    }
}

/// A framed landscape, a hill under a sun: an image.
pub fn paint_image(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let side = rect.width().min(rect.height());
    let frame = egui::Rect::from_center_size(rect.center(), egui::vec2(side * 0.84, side * 0.68));
    painter.rect_stroke(frame, 1.5, stroke, egui::StrokeKind::Middle);
    let hill = vec![
        egui::pos2(frame.left(), frame.bottom() - frame.height() * 0.2),
        egui::pos2(
            frame.left() + frame.width() * 0.4,
            frame.top() + frame.height() * 0.45,
        ),
        egui::pos2(
            frame.left() + frame.width() * 0.65,
            frame.bottom() - frame.height() * 0.3,
        ),
        egui::pos2(frame.right(), frame.top() + frame.height() * 0.55),
    ];
    painter.add(egui::Shape::line(hill, stroke));
    painter.circle_stroke(
        egui::pos2(
            frame.right() - frame.width() * 0.25,
            frame.top() + frame.height() * 0.28,
        ),
        side * 0.07,
        stroke,
    );
}

/// An open folder: choose a file.
pub fn paint_folder(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let side = rect.width().min(rect.height());
    let body = egui::Rect::from_center_size(rect.center(), egui::vec2(side * 0.84, side * 0.62));
    let tab = body.left() + body.width() * 0.38;
    let points = vec![
        body.left_bottom(),
        body.left_top(),
        egui::pos2(tab, body.top()),
        egui::pos2(tab + side * 0.08, body.top() + side * 0.1),
        egui::pos2(body.right(), body.top() + side * 0.1),
        body.right_bottom(),
    ];
    painter.add(egui::Shape::closed_line(points, stroke));
}

/// A counter-clockwise arrow around a circle: reset.
pub fn paint_reset(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    const STEPS: usize = 16;
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.36;
    // From the top, counter-clockwise round to the left side, leaving a gap at the top left.
    let start = -60.0_f32;
    let sweep = 290.0_f32;
    let points: Vec<egui::Pos2> = (0..=STEPS)
        .map(|step| {
            let angle = (start + sweep * step as f32 / STEPS as f32).to_radians();
            center + egui::vec2(angle.cos(), angle.sin()) * radius
        })
        .collect();
    let tip = points[0];
    painter.add(egui::Shape::line(points, stroke));
    let head = radius * 0.6;
    painter.line_segment([tip, tip + egui::vec2(-head, -head * 0.1)], stroke);
    painter.line_segment([tip, tip + egui::vec2(head * 0.15, -head)], stroke);
}

/// The icon font at `size`, for glyphs laid out with text.
pub fn icon_font_id(size: f32) -> egui::FontId {
    egui::FontId::new(size, icon_font())
}

/// A square button with a painted icon; the closure gets the button's rect and stroke.
pub fn icon_button(
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
