use super::drag::{begin_drag, finish_drag, update_drag};
use super::grid::draw_grid;
use super::{EditorState, ShapeDraft, format_length, send};
use crate::document::ProjectMessage;
use crate::panels::tools::Tool;
use bevy::prelude::*;
use bevy_egui::egui;
use earcutr::earcut;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Language, Message};
use leathercraft::model::{PatternPiece, Point, Project, Units};

const MIN_ZOOM: f32 = 0.05;
const MAX_ZOOM: f32 = 100.0;
const VERTEX_HIT_RADIUS_PX: f32 = 8.0;
const CLOSE_HIT_RADIUS_PX: f32 = 10.0;
const MAX_POINTS_WITH_EDGE_LABELS: usize = 64;

#[derive(Clone, Copy)]
pub(super) struct Viewport {
    origin: egui::Pos2,
    pub(super) zoom: f32,
}

impl Viewport {
    pub(super) fn to_screen(self, point: Point) -> egui::Pos2 {
        self.origin + egui::vec2(point.x as f32, point.y as f32) * self.zoom
    }

    pub(super) fn to_world(self, position: egui::Pos2) -> Point {
        let offset = (position - self.origin) / self.zoom;
        Point::new(f64::from(offset.x), f64::from(offset.y))
    }
}

pub(super) fn canvas(
    ui: &mut egui::Ui,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    let (response, painter) =
        ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
    let rect = response.rect;

    if state.fit_requested {
        state.fit_requested = false;
        fit_view(rect, project, state);
    }
    if response.dragged_by(egui::PointerButton::Secondary)
        || response.dragged_by(egui::PointerButton::Middle)
    {
        state.pan += response.drag_delta();
    }
    if let Some(position) = response.hover_pos() {
        let scroll = ui.input(|input| input.smooth_scroll_delta.y);
        if scroll != 0.0 {
            let zoom = (state.zoom * (scroll * 0.002).exp()).clamp(MIN_ZOOM, MAX_ZOOM);
            // Keep the world point under the cursor fixed while zooming.
            let anchor = position - rect.center() - state.pan;
            state.pan -= anchor * (zoom / state.zoom - 1.0);
            state.zoom = zoom;
        }
    }

    let view = Viewport {
        origin: rect.center() + state.pan,
        zoom: state.zoom,
    };
    state.cursor = response
        .hover_pos()
        .or_else(|| response.interact_pointer_pos())
        .map(|position| view.to_world(position));

    if state.tool != Tool::Draw {
        state.draft.clear();
    }
    match state.tool {
        Tool::Select => handle_select(ui, &response, view, project, events, state),
        Tool::Draw => handle_draw(ui, &response, view, project, events, state),
        Tool::Rectangle | Tool::Circle => {
            handle_shape(ui, &response, view, project, events, state);
        }
    }

    painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
    draw_grid(&painter, rect, view);
    draw_pieces(&painter, view, project, state);
    draw_draft(&painter, response.hover_pos(), view, project.units, state);
    draw_shape_draft(&painter, view, project.units, state);
}

fn fit_view(rect: egui::Rect, project: &Project, state: &mut EditorState) {
    let Some(bounds) = project.bounds() else {
        state.pan = egui::Vec2::ZERO;
        return;
    };
    let width = bounds.width().max(1.0) as f32;
    let height = bounds.height().max(1.0) as f32;
    state.zoom = (rect.width() / width)
        .min(rect.height() / height)
        .clamp(MIN_ZOOM, MAX_ZOOM)
        * 0.85;
    let center_x = (bounds.min_x + bounds.max_x) * 0.5;
    let center_y = (bounds.min_y + bounds.max_y) * 0.5;
    state.pan = -egui::vec2(center_x as f32, center_y as f32) * state.zoom;
}

fn handle_select(
    ui: &egui::Ui,
    response: &egui::Response,
    view: Viewport,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    if response.drag_started_by(egui::PointerButton::Primary) {
        // The drag only registers past a movement threshold, so hit test where it was pressed.
        if let Some(origin) = ui.input(|input| input.pointer.press_origin()) {
            begin_drag(origin, view, project, state);
        }
    }
    if response.dragged_by(egui::PointerButton::Primary)
        && let Some(position) = response.interact_pointer_pos()
    {
        update_drag(view.to_world(position), state);
    }
    if response.drag_stopped() {
        finish_drag(events, state);
    }
    if response.clicked()
        && let Some(position) = response.interact_pointer_pos()
    {
        state.selected = piece_at(project, view.to_world(position));
    }
}

fn handle_draw(
    ui: &egui::Ui,
    response: &egui::Response,
    view: Viewport,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    if response.clicked()
        && let Some(position) = response.interact_pointer_pos()
    {
        let closes = state.draft.len() >= 3
            && view.to_screen(state.draft[0]).distance(position) <= CLOSE_HIT_RADIUS_PX;
        if closes {
            finish_draft(project, events, state);
            return;
        }
        let point = state.snap_point(view.to_world(position));
        if state.draft.last() != Some(&point) {
            state.draft.push(point);
        }
    }
    if ui.ctx().wants_keyboard_input() {
        return;
    }
    if ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        finish_draft(project, events, state);
    }
    if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.draft.clear();
    }
}

fn finish_draft(
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &EditorState,
) {
    add_piece(project, events, state.language, state.draft.clone());
}

fn handle_shape(
    ui: &egui::Ui,
    response: &egui::Response,
    view: Viewport,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    if response.drag_started_by(egui::PointerButton::Primary)
        && let Some(origin) = ui.input(|input| input.pointer.press_origin())
    {
        let start = state.snap_point(view.to_world(origin));
        state.shape_draft = Some(ShapeDraft { start, end: start });
    }
    if response.dragged_by(egui::PointerButton::Primary)
        && let Some(position) = response.interact_pointer_pos()
    {
        let end = state.snap_point(view.to_world(position));
        if let Some(draft) = &mut state.shape_draft {
            draft.end = end;
        }
    }
    if response.drag_stopped_by(egui::PointerButton::Primary)
        && let Some(draft) = state.shape_draft.take()
        && let Some(points) = state.tool.shape_points(draft.start, draft.end)
    {
        add_piece(project, events, state.language, points);
    }
    if !ui.ctx().wants_keyboard_input() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.shape_draft = None;
    }
}

fn add_piece(
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    language: Language,
    points: Vec<Point>,
) {
    // The name is document content, so it stays in the language it was created in.
    let name = language.message(&Message::DefaultPieceName {
        id: project.next_piece_id(),
    });
    send(events, ProjectEvent::AddPiece { name, points });
}

pub(super) fn piece_at(project: &Project, point: Point) -> Option<u64> {
    project
        .pieces
        .iter()
        .rev()
        .find(|piece| piece.contains(point))
        .map(|piece| piece.id)
}

pub(super) fn vertex_at(
    piece: &PatternPiece,
    position: egui::Pos2,
    view: Viewport,
) -> Option<usize> {
    piece
        .points
        .iter()
        .position(|point| view.to_screen(*point).distance(position) <= VERTEX_HIT_RADIUS_PX)
}

fn draw_pieces(painter: &egui::Painter, view: Viewport, project: &Project, state: &EditorState) {
    let text_color = egui::Color32::from_gray(220);
    let fill_color = egui::Color32::from_rgba_unmultiplied(128, 128, 128, 80);
    for piece in &project.pieces {
        let piece = state.displayed(piece);
        let selected = state.selected == Some(piece.id);
        let stroke = if selected {
            egui::Stroke::new(2.5, egui::Color32::from_rgb(240, 160, 70))
        } else {
            egui::Stroke::new(1.5, egui::Color32::from_rgb(190, 190, 200))
        };
        let screen: Vec<egui::Pos2> = piece
            .points
            .iter()
            .map(|point| view.to_screen(*point))
            .collect();
        fill_polygon(painter, &screen, fill_color);
        painter.add(egui::Shape::closed_line(screen.clone(), stroke));
        if let Some(bounds) = piece.bounds() {
            let center = Point::new(
                (bounds.min_x + bounds.max_x) * 0.5,
                (bounds.min_y + bounds.max_y) * 0.5,
            );
            painter.text(
                view.to_screen(center),
                egui::Align2::CENTER_CENTER,
                &piece.name,
                egui::FontId::proportional(13.0),
                text_color,
            );
        }
        if !selected {
            continue;
        }
        if !piece.locked {
            for position in &screen {
                painter.circle_filled(*position, 4.0, stroke.color);
            }
        }
        if piece.points.len() > MAX_POINTS_WITH_EDGE_LABELS {
            continue;
        }
        for index in 0..piece.points.len() {
            let a = piece.points[index];
            let b = piece.points[(index + 1) % piece.points.len()];
            let next = screen[(index + 1) % screen.len()];
            painter.text(
                screen[index].lerp(next, 0.5),
                egui::Align2::CENTER_BOTTOM,
                format_length(state.language, project.units, (b.x - a.x).hypot(b.y - a.y)),
                egui::FontId::proportional(11.0),
                text_color,
            );
        }
    }
}

// Triangulated because pieces can be concave, which `Shape::convex_polygon` doesn't fill correctly.
fn fill_polygon(painter: &egui::Painter, screen: &[egui::Pos2], color: egui::Color32) {
    let flat: Vec<f64> = screen
        .iter()
        .flat_map(|position| [f64::from(position.x), f64::from(position.y)])
        .collect();
    let Ok(triangles) = earcut(&flat, &[], 2) else {
        return;
    };
    let mut mesh = egui::Mesh::default();
    for position in screen {
        mesh.colored_vertex(*position, color);
    }
    for triangle in triangles.chunks_exact(3) {
        mesh.add_triangle(triangle[0] as u32, triangle[1] as u32, triangle[2] as u32);
    }
    painter.add(mesh);
}

fn draw_draft(
    painter: &egui::Painter,
    hover: Option<egui::Pos2>,
    view: Viewport,
    units: Units,
    state: &EditorState,
) {
    let Some(&last) = state.draft.last() else {
        return;
    };
    let color = egui::Color32::from_rgb(110, 190, 240);
    let stroke = egui::Stroke::new(2.0, color);
    let screen: Vec<egui::Pos2> = state
        .draft
        .iter()
        .map(|point| view.to_screen(*point))
        .collect();
    painter.add(egui::Shape::line(screen.clone(), stroke));
    for position in &screen {
        painter.circle_filled(*position, 4.0, color);
    }
    if let Some(hover) = hover {
        let target = state.snap_point(view.to_world(hover));
        let target_screen = view.to_screen(target);
        painter.line_segment(
            [view.to_screen(last), target_screen],
            egui::Stroke::new(1.0, color),
        );
        painter.text(
            target_screen + egui::vec2(10.0, -10.0),
            egui::Align2::LEFT_BOTTOM,
            format_length(
                state.language,
                units,
                (target.x - last.x).hypot(target.y - last.y),
            ),
            egui::FontId::proportional(12.0),
            color,
        );
    }
}

fn draw_shape_draft(painter: &egui::Painter, view: Viewport, units: Units, state: &EditorState) {
    let Some(draft) = state.shape_draft else {
        return;
    };
    let Some(points) = state.tool.shape_points(draft.start, draft.end) else {
        return;
    };
    let color = egui::Color32::from_rgb(110, 190, 240);
    let screen: Vec<egui::Pos2> = points.iter().map(|point| view.to_screen(*point)).collect();
    painter.add(egui::Shape::closed_line(
        screen,
        egui::Stroke::new(2.0, color),
    ));
    let language = state.language;
    let dx = draft.end.x - draft.start.x;
    let dy = draft.end.y - draft.start.y;
    let size = if state.tool == Tool::Circle {
        format!("Ø {}", format_length(language, units, dx.hypot(dy) * 2.0))
    } else {
        format!(
            "{} × {}",
            format_length(language, units, dx.abs()),
            format_length(language, units, dy.abs())
        )
    };
    painter.text(
        view.to_screen(draft.end) + egui::vec2(10.0, -10.0),
        egui::Align2::LEFT_BOTTOM,
        size,
        egui::FontId::proportional(12.0),
        color,
    );
}
