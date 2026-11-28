//! Background images on the canvas: drawing them as textured quads, the frame of the selected
//! one with its scale and rotate handles, dragging it, and calibrating its scale from two points.
//! The placement math lives in `leathercraft::background`; see `docs/background-image.md`.

use super::canvas::Viewport;
use super::{EditorState, send};
use crate::document::ProjectMessage;
use crate::ui_components::button::Button;
use crate::ui_components::input::NumberInput;
use crate::ui_components::palette::Palette;
use bevy::prelude::*;
use bevy_egui::egui;
use leathercraft::background::{BackgroundImage, Depth};
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Message};
use leathercraft::model::{Point, Project};
use leathercraft::snap::{SnapScope, SnapTargets};
use std::borrow::Cow;

/// How far from a handle, in screen points, a press still grabs it.
const HANDLE_HIT_PX: f32 = 8.0;
const HANDLE_SIZE_PX: f32 = 8.0;
/// How far the rotate handle stands above the top edge.
const ROTATE_OFFSET_PX: f32 = 24.0;
const ROTATE_RADIUS_PX: f32 = 5.0;
/// Shift snaps rotation to this step.
const ROTATE_STEP: f64 = std::f64::consts::PI / 12.0;
/// The smallest drawn side a scale drag leaves, in mm.
const MIN_SIDE_MM: f64 = 1.0;
const FRAME_DASH_PX: f32 = 6.0;
const FRAME_GAP_PX: f32 = 4.0;
const MARK_RADIUS_PX: f32 = 4.0;

/// What a drag on the selected image changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DragKind {
    Move,
    /// Scales from the corner at this index of `BackgroundImage::corners`, keeping the opposite
    /// one in place.
    Scale(usize),
    Rotate,
}

/// A drag of a background image in progress; sent as one edit when it ends.
#[derive(Clone, Debug)]
pub(super) struct BackgroundDrag {
    pub(super) kind: DragKind,
    pub(super) start: BackgroundImage,
    pub(super) current: BackgroundImage,
    /// Where the press was, in mm.
    pub(super) anchor: Point,
}

/// Calibration in progress: two points clicked on the image, then the real distance between
/// them typed in.
#[derive(Clone, Debug)]
pub(super) struct Calibration {
    pub(super) id: u64,
    pub(super) first: Option<Point>,
    pub(super) second: Option<Point>,
    /// The distance to set, in mm; starts as the measured one.
    pub(super) distance: f64,
}

/// The image as shown: the dragged copy while a drag is on it.
pub(super) fn displayed<'a>(
    state: &EditorState,
    image: &'a BackgroundImage,
) -> Cow<'a, BackgroundImage> {
    match &state.background_drag {
        Some(drag) if drag.current.id == image.id => Cow::Owned(drag.current.clone()),
        _ => Cow::Borrowed(image),
    }
}

/// Draws the visible images of `depth`, in list order. An image whose file can't be read is
/// skipped; the panel shows why.
pub(super) fn draw_images(
    painter: &egui::Painter,
    view: Viewport,
    project: &Project,
    state: &mut EditorState,
    depth: Depth,
) {
    for image in &project.background_images {
        if !image.visible || image.depth != depth {
            continue;
        }
        let image = displayed(state, image).into_owned();
        let picture = state.background_textures.get(painter.ctx(), &image.path);
        let crate::background_textures::Picture::Loaded {
            texture, size_px, ..
        } = picture
        else {
            continue;
        };
        let quad = image.quad(*size_px);
        let tint = egui::Color32::WHITE.gamma_multiply(image.opacity);
        let mut mesh = egui::Mesh::with_texture(texture.id());
        for (corner, (u, v)) in quad.corners.iter().zip(quad.uvs) {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: view.to_screen(*corner),
                uv: egui::pos2(u as f32, v as f32),
                color: tint,
            });
        }
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        painter.add(egui::Shape::mesh(mesh));
    }
}

/// The selected image, if it is unlocked and shown, as displayed.
fn selected_image<'a>(
    project: &'a Project,
    state: &EditorState,
) -> Option<Cow<'a, BackgroundImage>> {
    let image = project.background_image(state.selected_background?)?;
    (image.visible && !image.locked).then(|| displayed(state, image))
}

/// The corners of the image and the rotate handle, on screen.
fn handles(view: Viewport, image: &BackgroundImage) -> ([egui::Pos2; 4], egui::Pos2) {
    let corners = image.corners().map(|corner| view.to_screen(corner));
    let center = view.to_screen(image.offset);
    let top = corners[0] + (corners[1] - corners[0]) * 0.5;
    let up = (top - center).normalized();
    let up = if up.is_finite() { up } else { -egui::Vec2::Y };
    (corners, top + up * ROTATE_OFFSET_PX)
}

/// The dashed frame of the selected image, with its four scale handles and its rotate handle.
pub(super) fn draw_selection(
    painter: &egui::Painter,
    view: Viewport,
    project: &Project,
    state: &EditorState,
) {
    let Some(image) = selected_image(project, state) else {
        return;
    };
    let colors = &Palette::of(painter).background;
    let stroke = egui::Stroke::new(1.0_f32, colors.frame);
    let (corners, rotate) = handles(view, &image);
    let mut ring = corners.to_vec();
    ring.push(corners[0]);
    painter.extend(egui::Shape::dashed_line(
        &ring,
        stroke,
        FRAME_DASH_PX,
        FRAME_GAP_PX,
    ));
    let top = corners[0] + (corners[1] - corners[0]) * 0.5;
    painter.line_segment([top, rotate], stroke);
    let handle_stroke = egui::Stroke::new(1.5_f32, colors.frame);
    painter.circle(rotate, ROTATE_RADIUS_PX, colors.handle_fill, handle_stroke);
    for corner in corners {
        let rect = egui::Rect::from_center_size(corner, egui::Vec2::splat(HANDLE_SIZE_PX));
        painter.rect(
            rect,
            0.0,
            colors.handle_fill,
            handle_stroke,
            egui::StrokeKind::Middle,
        );
    }
}

/// Starts a drag of the selected image if the press is on one of its handles, or inside it
/// with no piece under the pointer. Returns whether it did.
pub(super) fn begin_drag(
    origin: egui::Pos2,
    view: Viewport,
    project: &Project,
    state: &mut EditorState,
    piece_under: bool,
) -> bool {
    let Some(image) = selected_image(project, state).map(Cow::into_owned) else {
        return false;
    };
    let (corners, rotate) = handles(view, &image);
    let point = view.to_world(origin);
    let kind = if origin.distance(rotate) <= HANDLE_HIT_PX {
        DragKind::Rotate
    } else if let Some(corner) = corners
        .iter()
        .position(|corner| origin.distance(*corner) <= HANDLE_HIT_PX)
    {
        DragKind::Scale(corner)
    } else if !piece_under && image.contains(point) {
        DragKind::Move
    } else {
        return false;
    };
    state.background_drag = Some(BackgroundDrag {
        kind,
        start: image.clone(),
        current: image,
        anchor: point,
    });
    true
}

/// Follows the pointer at `point`; Shift scales freely, or turns in 15° steps.
pub(super) fn update_drag(point: Point, shift: bool, state: &mut EditorState) {
    let Some(drag) = &mut state.background_drag else {
        return;
    };
    let start = &drag.start;
    drag.current = match drag.kind {
        DragKind::Move => BackgroundImage {
            offset: Point::new(
                start.offset.x + point.x - drag.anchor.x,
                start.offset.y + point.y - drag.anchor.y,
            ),
            ..start.clone()
        },
        DragKind::Rotate => {
            let angle = |p: Point| (p.y - start.offset.y).atan2(p.x - start.offset.x);
            let mut rotation = start.rotation + angle(point) - angle(drag.anchor);
            if shift {
                rotation = (rotation / ROTATE_STEP).round() * ROTATE_STEP;
            }
            BackgroundImage {
                rotation,
                ..start.clone()
            }
        }
        DragKind::Scale(corner) => scaled_from_corner(start, corner, point, !shift),
    };
}

/// The image scaled so that `corner` follows `point` and the opposite corner stays.
fn scaled_from_corner(
    start: &BackgroundImage,
    corner: usize,
    point: Point,
    keep_aspect: bool,
) -> BackgroundImage {
    const SIGNS: [(f64, f64); 4] = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    let fixed = start.corners()[(corner + 2) % 4];
    let (sx, sy) = SIGNS[corner];
    let (w0, h0) = start.drawn_size();
    // The pointer in the image's own axes, from the fixed corner towards the dragged one.
    let (lx, ly) = start.to_local(point);
    let (fx, fy) = start.to_local(fixed);
    let (dx, dy) = ((lx - fx) * sx, (ly - fy) * sy);
    let (w, h) = if keep_aspect {
        // The pointer projected on the diagonal through the fixed corner.
        let factor = (dx * w0 + dy * h0) / (w0 * w0 + h0 * h0);
        let factor = factor.max(MIN_SIDE_MM / w0.min(h0));
        (w0 * factor, h0 * factor)
    } else {
        (dx.max(MIN_SIDE_MM), dy.max(MIN_SIDE_MM))
    };
    let (cos, sin) = (start.rotation.cos(), start.rotation.sin());
    let (cx, cy) = (sx * w * 0.5, sy * h * 0.5);
    BackgroundImage {
        width: w / start.scale,
        height: h / start.scale,
        offset: Point::new(fixed.x + cx * cos - cy * sin, fixed.y + cx * sin + cy * cos),
        ..start.clone()
    }
}

/// Sends the dragged image as one edit; a drag that changed nothing sends none.
pub(super) fn finish_drag(events: &mut MessageWriter<ProjectMessage>, state: &mut EditorState) {
    if let Some(drag) = state.background_drag.take()
        && drag.current != drag.start
    {
        send(
            events,
            ProjectEvent::UpdateBackgroundImage {
                image: drag.current,
            },
        );
    }
}

/// Esc drops a drag in progress, else the selected image.
pub(super) fn cancel(state: &mut EditorState) {
    if state.background_drag.take().is_none() {
        state.selected_background = None;
    }
}

/// While calibrating, clicks place the two points instead of selecting; they snap to the
/// vertices and guides of the pieces when snapping is on. Esc cancels.
pub(super) fn handle_calibration(
    ui: &egui::Ui,
    response: &egui::Response,
    view: Viewport,
    project: &Project,
    state: &mut EditorState,
) {
    if !ui.ctx().wants_keyboard_input() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.calibration = None;
        state.report(Message::Ready);
        return;
    }
    let Some(calibration) = &state.calibration else {
        return;
    };
    if calibration.second.is_some() {
        return;
    }
    let Some(position) = response.hover_pos() else {
        return;
    };
    let targets = SnapTargets::collect(
        project,
        &SnapScope::Project {
            exclude: Vec::new(),
        },
    );
    let tolerance = view.snap_tolerance(state.snap.distance_px);
    let point = state.snap_point(view.to_world(position), &targets, tolerance);
    state.hover_target = Some(point);
    if !response.clicked() {
        return;
    }
    let Some(calibration) = &mut state.calibration else {
        return;
    };
    match calibration.first {
        None => {
            calibration.first = Some(point);
            state.report(Message::CalibrateSecondPoint);
        }
        Some(first) => {
            calibration.second = Some(point);
            calibration.distance = distance(first, point);
        }
    }
}

fn distance(a: Point, b: Point) -> f64 {
    (b.x - a.x).hypot(b.y - a.y)
}

/// The calibration points and the line between them, to the pointer until the second is placed.
pub(super) fn draw_calibration(painter: &egui::Painter, view: Viewport, state: &EditorState) {
    let Some(calibration) = &state.calibration else {
        return;
    };
    let color = Palette::of(painter).background.frame;
    let stroke = egui::Stroke::new(1.5_f32, color);
    let end = calibration.second.or(state.hover_target);
    if let Some(first) = calibration.first {
        let a = view.to_screen(first);
        painter.circle_stroke(a, MARK_RADIUS_PX, stroke);
        if let Some(end) = end {
            let b = view.to_screen(end);
            painter.line_segment([a, b], stroke);
            painter.circle_stroke(b, MARK_RADIUS_PX, stroke);
        }
    } else if let Some(end) = end {
        painter.circle_stroke(view.to_screen(end), MARK_RADIUS_PX, stroke);
    }
}

/// Once both points are placed: a small window at the second one to type the real distance.
/// OK scales the image about the first point, so that point stays where it was clicked.
pub(super) fn calibration_window(
    ctx: &egui::Context,
    view: Viewport,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    let language = state.language;
    let units = project.units;
    let Some(calibration) = &mut state.calibration else {
        return;
    };
    let (Some(first), Some(second)) = (calibration.first, calibration.second) else {
        return;
    };
    let mut done = None;
    egui::Window::new(language.label(Label::Calibrate))
        .id(egui::Id::new("background_calibration"))
        .collapsible(false)
        .resizable(false)
        .fixed_pos(view.to_screen(second) + egui::vec2(12.0, 12.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(language.label(Label::CalibrateDistance));
                let scale = units.from_millimeters(1.0);
                let mut value = calibration.distance * scale;
                let field = NumberInput::new(&mut value)
                    .id_salt("calibration_distance")
                    .range(0.01..=1_000_000.0)
                    .decimals(2)
                    .unit(language.unit_symbol(units));
                if ui.add(field).changed() {
                    calibration.distance = value / scale;
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.add(Button::primary(language.label(Label::Ok))).clicked() {
                    done = Some(true);
                }
                if ui
                    .add(Button::secondary(language.label(Label::Cancel)))
                    .clicked()
                {
                    done = Some(false);
                }
            });
        });
    let Some(apply) = done else {
        return;
    };
    let measured = distance(first, second);
    let calibration = state.calibration.take();
    state.report(Message::Ready);
    let Some(calibration) = calibration.filter(|_| apply) else {
        return;
    };
    let Some(image) = project.background_image(calibration.id) else {
        return;
    };
    if measured > 1e-9 && calibration.distance > 0.0 {
        let image = image.scaled_about(first, calibration.distance / measured);
        send(events, ProjectEvent::UpdateBackgroundImage { image });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn image() -> BackgroundImage {
        let mut image = BackgroundImage::new(
            1,
            PathBuf::new(),
            [2, 1],
            Point::new(0.0, 0.0),
            (100.0, 100.0),
        );
        image.width = 20.0;
        image.height = 10.0;
        image
    }

    #[test]
    fn scaling_from_a_corner_keeps_the_opposite_one() {
        let start = image();
        let fixed = start.corners()[0];
        let scaled = scaled_from_corner(&start, 2, Point::new(30.0, 15.0), true);
        let corners = scaled.corners();
        assert!((corners[0].x - fixed.x).abs() < 1e-9 && (corners[0].y - fixed.y).abs() < 1e-9);
        assert!((scaled.width / scaled.height - 2.0).abs() < 1e-9);
        assert!((corners[2].x - 30.0).abs() < 1e-9);
    }
}
