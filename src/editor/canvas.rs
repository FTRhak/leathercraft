use super::drag::{begin_drag, begin_handle_drag, finish_drag, update_drag};
use super::drag_state_struct::ShapeRef;
use super::grid::draw_grid;
use super::palette::Palette;
use super::{EditorState, ShapeDraft, ShapePart, format_length, send};
use crate::document::ProjectMessage;
use crate::panels::tools::{Tool, ZoomRequest};
use bevy::prelude::*;
use bevy_egui::egui;
use earcutr::earcut;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Language, Message};
use leathercraft::model::{BooleanOp, Bounds, PatternPiece, Point, Project, Units};
use leathercraft::shapes::{
    Ellipse, HoleKind, Line, Polygon, Rectangle, RegularPolygon, Shape, StitchLine,
};
use leathercraft::snap::{GuideAxis, SnapScope, SnapTargets};

const MIN_ZOOM: f32 = 0.05;
const MAX_ZOOM: f32 = 100.0;
const VERTEX_HIT_RADIUS_PX: f32 = 8.0;
const EDGE_HIT_RADIUS_PX: f32 = 6.0;
const CLOSE_HIT_RADIUS_PX: f32 = 10.0;
const MAX_POINTS_WITH_EDGE_LABELS: usize = 64;
/// Centerlines are thinner than outlines, which are 1.5 px, or 2.5 px when selected.
const CENTERLINE_WIDTH_PX: f32 = 0.8;
const SELECTED_CENTERLINE_WIDTH_PX: f32 = 1.5;
/// Below this period on screen, a dash pattern is drawn as a faded solid line.
const MIN_DASH_PERIOD_PX: f32 = 3.0;
/// Below this pitch on screen, stitch holes are drawn as a faded dotted path.
const MIN_HOLE_PITCH_PX: f32 = 3.0;
const SNAP_COLOR: egui::Color32 = egui::Color32::from_rgb(230, 90, 220);
const SELECTED_COLOR: egui::Color32 = egui::Color32::from_rgb(240, 160, 70);

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

    /// How far from a line, in mm, a click still hits it.
    pub(super) fn hit_tolerance(self) -> f64 {
        f64::from(EDGE_HIT_RADIUS_PX / self.zoom)
    }

    /// How far, in mm, snapping reaches for a distance on screen.
    pub(super) fn snap_tolerance(self, distance_px: f32) -> f64 {
        f64::from(distance_px / self.zoom)
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

    if let Some(request) = state.zoom_request.take() {
        let bounds = match request {
            ZoomRequest::All => project.bounds(),
            ZoomRequest::Selection => selection_bounds(project, state),
        };
        // Nothing selected leaves the view as it is; an empty project recenters it.
        if bounds.is_some() || request == ZoomRequest::All {
            fit_view(rect, bounds, state);
        }
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
    state.snap_suspended = ui.input(|input| input.modifiers.alt);
    state.snap_feedback = Default::default();
    state.hover_target = None;

    // The Draw and Stitch holes tools each click a draft; another tool drops it.
    if state.tool != state.draft_tool {
        state.draft.clear();
        state.draft_tool = state.tool;
    }
    match state.tool {
        Tool::Select => handle_select(ui, &response, view, project, events, state),
        Tool::Edge => handle_edge(ui, &response, view, project, events, state),
        Tool::Draw => handle_draw(ui, &response, view, project, events, state),
        Tool::StitchHoles => handle_stitch(ui, &response, view, project, events, state),
        Tool::Rectangle | Tool::Ellipse | Tool::RegularPolygon | Tool::Centerline => {
            handle_shape(ui, &response, view, project, events, state);
        }
    }
    // Shows where a drawing tool would place its next point before the click.
    let drawing = match state.tool {
        Tool::Draw | Tool::StitchHoles => true,
        Tool::Rectangle | Tool::Ellipse | Tool::RegularPolygon | Tool::Centerline => {
            state.shape_draft.is_none()
        }
        Tool::Select | Tool::Edge => false,
    };
    if drawing && let Some(position) = response.hover_pos() {
        state.hover_target = Some(snap_draw(project, view, state, view.to_world(position)));
    }

    painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
    let snap_grid = state.snap.enabled && state.snap.grid;
    draw_grid(
        &painter,
        rect,
        view,
        snap_grid.then_some(state.snap.grid_step_mm),
    );
    draw_pieces(&painter, view, project, state);
    draw_centerlines(&painter, view, project, state);
    draw_stitch_holes(&painter, ui, response.hover_pos(), view, project, state);
    draw_edges(&painter, response.hover_pos(), view, project, state);
    if state.tool == Tool::StitchHoles {
        draw_stitch_draft(&painter, ui, response.hover_pos(), view, project, state);
    } else {
        draw_draft(&painter, response.hover_pos(), view, project.units, state);
    }
    draw_shape_draft(&painter, view, project.units, state);
    draw_snap_feedback(&painter, view, state);
}

/// What a drawing tool snaps to. A shape that becomes a child of the selected piece snaps only
/// within that piece; a new piece snaps to every piece. The points of the draft count too.
fn draw_targets(project: &Project, state: &EditorState) -> SnapTargets {
    let child =
        state.op_mode.is_some() || matches!(state.tool, Tool::Centerline | Tool::StitchHoles);
    let scope = match state.selected {
        Some(id) if child => SnapScope::Piece {
            id,
            exclude: Vec::new(),
        },
        _ => SnapScope::Project {
            exclude: Vec::new(),
        },
    };
    SnapTargets::collect(project, &scope).with_points(state.draft.iter().copied())
}

/// A point placed by a drawing tool, snapped.
fn snap_draw(project: &Project, view: Viewport, state: &mut EditorState, point: Point) -> Point {
    let targets = draw_targets(project, state);
    let tolerance = view.snap_tolerance(state.snap.distance_px);
    state.snap_point(point, &targets, tolerance)
}

/// A marker on the vertex snapped to, and the guides that lined up.
fn draw_snap_feedback(painter: &egui::Painter, view: Viewport, state: &EditorState) {
    let feedback = &state.snap_feedback;
    let stroke = egui::Stroke::new(1.0, SNAP_COLOR);
    for guide in &feedback.guides {
        let (a, b) = match guide.axis {
            GuideAxis::X => (
                Point::new(guide.at, guide.from),
                Point::new(guide.at, guide.to),
            ),
            GuideAxis::Y => (
                Point::new(guide.from, guide.at),
                Point::new(guide.to, guide.at),
            ),
        };
        painter.extend(egui::Shape::dashed_line(
            &[view.to_screen(a), view.to_screen(b)],
            stroke,
            4.0,
            3.0,
        ));
    }
    if let Some(vertex) = feedback.vertex {
        let rect = egui::Rect::from_center_size(view.to_screen(vertex), egui::Vec2::splat(11.0));
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.5, SNAP_COLOR),
            egui::StrokeKind::Middle,
        );
    }
}

/// The box around the selected children, or else around every selected piece.
fn selection_bounds(project: &Project, state: &EditorState) -> Option<Bounds> {
    let children = state.selected_children();
    if let Some(piece) = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id))
        && !children.is_empty()
    {
        return children
            .iter()
            .filter_map(|id| piece.child(*id))
            .filter_map(|child| child.shape.bounds())
            .reduce(Bounds::union);
    }
    project
        .pieces
        .iter()
        .filter(|piece| state.is_selected(piece.id))
        .filter_map(PatternPiece::bounds)
        .reduce(Bounds::union)
}

fn fit_view(rect: egui::Rect, bounds: Option<Bounds>, state: &mut EditorState) {
    let Some(bounds) = bounds else {
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
            let shift = ui.input(|input| input.modifiers.shift);
            begin_drag(origin, shift, view, project, state);
        }
    }
    if response.dragged_by(egui::PointerButton::Primary)
        && let Some(position) = response.interact_pointer_pos()
    {
        let tolerance = view.snap_tolerance(state.snap.distance_px);
        update_drag(view.to_world(position), tolerance, state);
    }
    if response.drag_stopped() {
        finish_drag(events, state);
    }
    if response.clicked()
        && let Some(position) = response.interact_pointer_pos()
    {
        let point = view.to_world(position);
        let tolerance = view.hit_tolerance();
        if ui.input(|input| input.modifiers.shift) {
            match child_of_selected_at(project, point, tolerance, state) {
                Some((piece, child)) => state.toggle_child(piece, child),
                None => {
                    if let Some(piece) = piece_at(project, point, tolerance) {
                        state.toggle_piece(piece);
                    }
                }
            }
        } else {
            select_at(project, point, tolerance, state);
        }
    }
}

/// A click picks the vertex of the selected piece under the cursor, which can then be rounded,
/// or else the polygon edge under it, or else selects as the Select tool does. Vertices of the
/// selected piece can be dragged as with the Select tool.
fn handle_edge(
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
        begin_handle_drag(origin, view, project, state);
    }
    if response.dragged_by(egui::PointerButton::Primary)
        && let Some(position) = response.interact_pointer_pos()
    {
        let tolerance = view.snap_tolerance(state.snap.distance_px);
        update_drag(view.to_world(position), tolerance, state);
    }
    if response.drag_stopped() {
        finish_drag(events, state);
    }
    if response.clicked()
        && let Some(position) = response.interact_pointer_pos()
    {
        if let Some((piece, shape, vertex)) = vertex_at(project, state, position, view) {
            state.select_part(piece, shape, ShapePart::Vertex(vertex));
        } else if let Some((piece, shape, edge)) = edge_at(project, state, position, view) {
            state.select_part(piece, shape, ShapePart::Edge(edge));
        } else {
            select_at(
                project,
                view.to_world(position),
                view.hit_tolerance(),
                state,
            );
        }
    }
    if !ui.ctx().wants_keyboard_input() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.selected_part = None;
    }
}

/// The child of the selected piece under the point, with its piece id. `tolerance` is how
/// far from a line, in mm, still hits it.
fn child_of_selected_at(
    project: &Project,
    point: Point,
    tolerance: f64,
    state: &EditorState,
) -> Option<(u64, u64)> {
    state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id))
        .and_then(|piece| child_at(piece, point, tolerance).map(|child| (piece.id, child)))
}

/// A child of the selected piece takes the click; otherwise it selects a piece.
fn select_at(project: &Project, point: Point, tolerance: f64, state: &mut EditorState) {
    match child_of_selected_at(project, point, tolerance, state) {
        Some((piece, child)) => state.select(Some(piece), Some(child)),
        None => state.select(piece_at(project, point, tolerance), None),
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
        let point = snap_draw(project, view, state, view.to_world(position));
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
    state: &mut EditorState,
) {
    let points = state.draft.clone();
    commit_shape(project, events, state, Shape::Polygon(Polygon::new(points)));
}

/// Clicks place the vertices of a stitch line; a double-click or Enter adds it to the selected
/// piece, Backspace drops the last vertex and Esc the draft. Shift turns the next segment in
/// 15° steps.
fn handle_stitch(
    ui: &egui::Ui,
    response: &egui::Response,
    view: Viewport,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    if response.double_clicked() {
        // The first click of the pair already placed the last vertex.
        finish_stitch(project, events, state);
        return;
    }
    if response.clicked()
        && let Some(position) = response.interact_pointer_pos()
    {
        let shift = ui.input(|input| input.modifiers.shift);
        let point = snap_draw(project, view, state, view.to_world(position));
        let point = stitch_target(state, point, shift);
        if state.draft.last() != Some(&point) && stitch_allowed(project, state, point) {
            state.draft.push(point);
        }
    }
    if ui.ctx().wants_keyboard_input() {
        return;
    }
    if ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        finish_stitch(project, events, state);
    }
    if ui.input(|input| input.key_pressed(egui::Key::Backspace)) {
        state.draft.pop();
    }
    if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.draft.clear();
    }
}

/// Where the next vertex of a stitch line goes, from the snapped point: with Shift turned
/// around the last vertex in 15° steps.
fn stitch_target(state: &EditorState, point: Point, shift: bool) -> Point {
    match state.draft.last() {
        Some(&last) if shift => StitchLine::snap(last, point),
        _ => point,
    }
}

/// Whether the next vertex of a stitch line may go at `point`: it and the segment to it from
/// the last vertex must stay on the selected piece's leather. Without a selected piece any
/// point goes, and finishing the line reports that a piece is needed.
fn stitch_allowed(project: &Project, state: &EditorState, point: Point) -> bool {
    let Some(piece) = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id))
    else {
        return true;
    };
    let path: Vec<Point> = state
        .draft
        .last()
        .copied()
        .into_iter()
        .chain([point])
        .collect();
    piece.stitch_area().contains_path(&path)
}

/// The stitch line of the draft with the current settings, or `None` with fewer than two points.
fn stitch_line(state: &EditorState, points: Vec<Point>) -> Option<StitchLine> {
    StitchLine::new(
        points,
        state.stitch.hole,
        state.stitch.pitch,
        state.stitch.size,
    )
}

fn finish_stitch(
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    let points = std::mem::take(&mut state.draft);
    if let Some(line) = stitch_line(state, points) {
        commit_shape(project, events, state, Shape::StitchLine(line));
    }
}

/// A finished shape becomes a child of the selected piece when an operation is chosen,
/// otherwise a new piece. A line is always a child, since it can't be a piece of its own.
fn commit_shape(
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
    shape: Shape,
) {
    let op = match state.op_mode {
        Some(op) => op,
        // The operation of a line is ignored.
        None if shape.is_open() => BooleanOp::default(),
        None => {
            add_piece(project, events, state.language, shape);
            return;
        }
    };
    match state.selected {
        Some(id) => send(events, ProjectEvent::AddChild { id, op, shape }),
        None => state.report_error(Message::SelectPieceForChild),
    }
}

fn handle_shape(
    ui: &egui::Ui,
    response: &egui::Response,
    view: Viewport,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    let constrained = ui.input(|input| input.modifiers.shift);
    if response.drag_started_by(egui::PointerButton::Primary)
        && let Some(origin) = ui.input(|input| input.pointer.press_origin())
    {
        let start = snap_draw(project, view, state, view.to_world(origin));
        state.shape_draft = Some(ShapeDraft {
            start,
            end: start,
            constrained,
        });
    }
    if response.dragged_by(egui::PointerButton::Primary)
        && let Some(position) = response.interact_pointer_pos()
    {
        let end = snap_draw(project, view, state, view.to_world(position));
        if let Some(draft) = &mut state.shape_draft {
            draft.end = end;
            draft.constrained = constrained;
        }
    }
    if response.drag_stopped_by(egui::PointerButton::Primary)
        && let Some(draft) = state.shape_draft.take()
        && let Some(shape) = state.tool.shape(
            draft.start,
            draft.end,
            draft.constrained,
            state.polygon_sides,
            state.line_style,
        )
    {
        commit_shape(project, events, state, shape);
    }
    if !ui.ctx().wants_keyboard_input() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.shape_draft = None;
    }
}

fn add_piece(
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    language: Language,
    base: Shape,
) {
    // The name is document content, so it stays in the language it was created in.
    let name = language.message(&Message::DefaultPieceName {
        id: project.next_piece_id(),
    });
    send(events, ProjectEvent::AddPiece { name, base });
}

/// Whether the shape is under the point: inside an outline, or within `tolerance` mm of a line.
fn shape_hit(shape: &Shape, point: Point, tolerance: f64) -> bool {
    match shape.distance_to(point) {
        Some(distance) => distance <= tolerance,
        None => shape.contains(point),
    }
}

/// Whether the point is on the piece or one of its lines, which may reach past it. A piece
/// whose shapes don't combine is hit by its base.
fn piece_hit(piece: &PatternPiece, point: Point, tolerance: f64) -> bool {
    let on_line = piece
        .children
        .iter()
        .any(|child| child.shape.is_open() && shape_hit(&child.shape, point, tolerance));
    on_line
        || match piece.region() {
            Ok(region) => region.contains(point),
            Err(_) => piece.base.contains(point),
        }
}

pub(super) fn piece_at(project: &Project, point: Point, tolerance: f64) -> Option<u64> {
    project
        .pieces
        .iter()
        .rev()
        .find(|piece| piece_hit(piece, point, tolerance))
        .map(|piece| piece.id)
}

/// The smallest child under the point, so that a child inside a larger one stays reachable.
/// A line counts as having no area, so it wins over the outlines it crosses.
pub(super) fn child_at(piece: &PatternPiece, point: Point, tolerance: f64) -> Option<u64> {
    let area = |shape: &Shape| {
        if shape.is_open() {
            return 0.0;
        }
        shape
            .bounds()
            .map_or(0.0, |bounds| bounds.width() * bounds.height())
    };
    piece
        .children
        .iter()
        .filter(|child| shape_hit(&child.shape, point, tolerance))
        .min_by(|a, b| area(&a.shape).total_cmp(&area(&b.shape)))
        .map(|child| child.id)
}

/// A handle of the selected child, or else of the base, under the cursor.
pub(super) fn handle_at(
    piece: &PatternPiece,
    selected_child: Option<u64>,
    position: egui::Pos2,
    view: Viewport,
) -> Option<(ShapeRef, usize)> {
    selected_child
        .map(ShapeRef::Child)
        .into_iter()
        .chain([ShapeRef::Base])
        .find_map(|shape| {
            let handles = shape.get(piece)?.handles();
            handles
                .iter()
                .position(|point| view.to_screen(*point).distance(position) <= VERTEX_HIT_RADIUS_PX)
                .map(|handle| (shape, handle))
        })
}

fn segment_distance(point: egui::Pos2, a: egui::Pos2, b: egui::Pos2) -> f32 {
    let along = b - a;
    let t = if along.length_sq() > 0.0 {
        ((point - a).dot(along) / along.length_sq()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    point.distance(a + along * t)
}

/// The polygon edge nearest the cursor, within reach. The selected piece goes first, then the
/// others from the top; within a piece the selected child, then the other children, then the base.
fn edge_at(
    project: &Project,
    state: &EditorState,
    position: egui::Pos2,
    view: Viewport,
) -> Option<(u64, ShapeRef, usize)> {
    let selected = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id));
    let others = project
        .pieces
        .iter()
        .rev()
        .filter(|piece| Some(piece.id) != state.selected);
    selected.into_iter().chain(others).find_map(|piece| {
        let selected_child = state
            .selected_child_id()
            .filter(|_| state.selected == Some(piece.id))
            .map(ShapeRef::Child);
        let children = piece
            .children
            .iter()
            .rev()
            .map(|child| ShapeRef::Child(child.id))
            .filter(|shape| Some(*shape) != selected_child);
        selected_child
            .into_iter()
            .chain(children)
            .chain([ShapeRef::Base])
            .find_map(|shape| {
                let Shape::Polygon(polygon) = shape.get(piece)? else {
                    return None;
                };
                (0..polygon.points.len())
                    .filter_map(|edge| {
                        let (a, b) = polygon.edge(edge)?;
                        let distance =
                            segment_distance(position, view.to_screen(a), view.to_screen(b));
                        (distance <= EDGE_HIT_RADIUS_PX).then_some((edge, distance))
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(edge, _)| (piece.id, shape, edge))
            })
    })
}

fn op_color(op: BooleanOp) -> egui::Color32 {
    match op {
        BooleanOp::Subtract => egui::Color32::from_rgb(235, 110, 110),
        BooleanOp::Union => egui::Color32::from_rgb(110, 205, 125),
        BooleanOp::Intersect => egui::Color32::from_rgb(110, 170, 245),
    }
}

/// The color of shapes being drawn, which shows what they will do.
fn draft_color(state: &EditorState) -> egui::Color32 {
    state
        .op_mode
        .map_or(egui::Color32::from_rgb(110, 190, 240), op_color)
}

fn to_screen(view: Viewport, ring: &[Point]) -> Vec<egui::Pos2> {
    ring.iter().map(|point| view.to_screen(*point)).collect()
}

fn dashed_ring(painter: &egui::Painter, mut screen: Vec<egui::Pos2>, stroke: egui::Stroke) {
    let Some(&first) = screen.first() else {
        return;
    };
    screen.push(first);
    painter.extend(egui::Shape::dashed_line(&screen, stroke, 6.0, 4.0));
}

fn draw_handles(painter: &egui::Painter, view: Viewport, shape: &Shape, color: egui::Color32) {
    for point in shape.handles() {
        let rect = egui::Rect::from_center_size(view.to_screen(point), egui::Vec2::splat(7.0));
        painter.rect_filled(rect, 1.0, color);
    }
}

fn draw_pieces(painter: &egui::Painter, view: Viewport, project: &Project, state: &EditorState) {
    let palette = Palette::of(painter);
    let text_color = palette.label;
    let error_color = egui::Color32::from_rgb(235, 80, 80);
    let fill_color = egui::Color32::from_rgba_unmultiplied(128, 128, 128, 80);
    for piece in &project.pieces {
        let piece = state.displayed(piece);
        let selected = state.selected == Some(piece.id);
        let stroke = if state.is_selected(piece.id) {
            egui::Stroke::new(2.5, SELECTED_COLOR)
        } else {
            egui::Stroke::new(1.5, palette.outline)
        };
        let region = piece.region();
        match &region {
            Ok(region) => {
                let vertices: Vec<egui::Pos2> = region
                    .vertices()
                    .map(|point| view.to_screen(*point))
                    .collect();
                fill_polygon(painter, &vertices, &region.hole_indices(), fill_color);
                for ring in region.rings() {
                    painter.add(egui::Shape::closed_line(to_screen(view, ring), stroke));
                }
            }
            // The shapes stay visible, so that the edit that broke the piece can be undone by hand.
            Err(_) => {
                let error_stroke = egui::Stroke::new(stroke.width, error_color);
                dashed_ring(painter, to_screen(view, &piece.base.ring()), error_stroke);
                for child in piece.children.iter().filter(|child| !child.shape.is_open()) {
                    dashed_ring(painter, to_screen(view, &child.shape.ring()), error_stroke);
                }
            }
        }
        if let Some(bounds) = piece.bounds() {
            let center = view.to_screen(Point::new(
                (bounds.min_x + bounds.max_x) * 0.5,
                (bounds.min_y + bounds.max_y) * 0.5,
            ));
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                &piece.name,
                egui::FontId::proportional(13.0),
                text_color,
            );
            if let Err(error) = &region {
                painter.text(
                    center + egui::vec2(0.0, 16.0),
                    egui::Align2::CENTER_CENTER,
                    state.language.message(error),
                    egui::FontId::proportional(12.0),
                    error_color,
                );
            }
        }
        if !selected {
            continue;
        }

        let selected_child = state.selected_child_id();
        let construction = egui::Stroke::new(1.0, stroke.color.gamma_multiply(0.7));
        dashed_ring(painter, to_screen(view, &piece.base.ring()), construction);
        // Lines are drawn by `draw_centerlines`.
        for child in piece.children.iter().filter(|child| !child.shape.is_open()) {
            let screen = to_screen(view, &child.shape.ring());
            let color = op_color(child.op);
            if state.selected_children().contains(&child.id) {
                painter.add(egui::Shape::closed_line(
                    screen,
                    egui::Stroke::new(2.0, color),
                ));
            } else {
                dashed_ring(painter, screen, egui::Stroke::new(1.0, color));
            }
        }
        if !piece.locked {
            draw_handles(painter, view, &piece.base, stroke.color);
        }
        match selected_child.and_then(|id| piece.child(id)) {
            Some(child) => {
                if let Shape::Line(line) = &child.shape {
                    if !piece.locked {
                        draw_handles(painter, view, &child.shape, SELECTED_COLOR);
                    }
                    draw_line_label(painter, view, line, project.units, state);
                    continue;
                }
                if let Shape::StitchLine(_) = &child.shape {
                    if !piece.locked {
                        draw_handles(painter, view, &child.shape, SELECTED_COLOR);
                    }
                    continue;
                }
                if !piece.locked {
                    draw_handles(painter, view, &child.shape, op_color(child.op));
                }
                let ring = child.shape.corners().unwrap_or_else(|| child.shape.ring());
                draw_edge_labels(
                    painter,
                    &ring,
                    &to_screen(view, &ring),
                    project.units,
                    state,
                );
            }
            None => {
                // Without children the outline is the base, labeled by its corners so that a
                // rounded corner's arc doesn't get labels of its own.
                if piece.children.is_empty()
                    && let Some(corners) = piece.base.corners()
                {
                    let screen = to_screen(view, &corners);
                    draw_edge_labels(painter, &corners, &screen, project.units, state);
                } else if let Ok(region) = &region {
                    let outline = to_screen(view, &region.outline);
                    draw_edge_labels(painter, &region.outline, &outline, project.units, state);
                }
            }
        }
    }
}

/// The lines of every piece, on top of all outlines and never clipped by them.
fn draw_centerlines(
    painter: &egui::Painter,
    view: Viewport,
    project: &Project,
    state: &EditorState,
) {
    for piece in &project.pieces {
        let piece = state.displayed(piece);
        let primary = state.selected == Some(piece.id);
        for child in &piece.children {
            let Shape::Line(line) = &child.shape else {
                continue;
            };
            let stroke = if primary && state.selected_children().contains(&child.id) {
                egui::Stroke::new(SELECTED_CENTERLINE_WIDTH_PX, SELECTED_COLOR)
            } else if state.is_selected(piece.id) {
                egui::Stroke::new(CENTERLINE_WIDTH_PX, Palette::of(painter).centerline)
            } else {
                egui::Stroke::new(
                    CENTERLINE_WIDTH_PX,
                    Palette::of(painter).centerline.gamma_multiply(0.8),
                )
            };
            draw_line(painter, view, line, stroke);
        }
    }
}

/// The holes of every stitch line, above all outlines and never clipped by them. The path is
/// shown faintly only while its line is selected or under the cursor, so it can be grabbed.
fn draw_stitch_holes(
    painter: &egui::Painter,
    ui: &egui::Ui,
    hover: Option<egui::Pos2>,
    view: Viewport,
    project: &Project,
    state: &EditorState,
) {
    let hover = hover.map(|position| view.to_world(position));
    for piece in &project.pieces {
        let piece = state.displayed(piece);
        let primary = state.selected == Some(piece.id);
        for child in &piece.children {
            let Shape::StitchLine(line) = &child.shape else {
                continue;
            };
            let selected = primary && state.selected_children().contains(&child.id);
            let hovered =
                hover.is_some_and(|point| line.distance_to(point) <= view.hit_tolerance());
            let color = if selected {
                SELECTED_COLOR
            } else if state.is_selected(piece.id) {
                Palette::of(painter).hole
            } else {
                Palette::of(painter).hole.gamma_multiply(0.8)
            };
            if selected || hovered {
                let path = to_screen(view, &line.path());
                painter.add(egui::Shape::line(
                    path,
                    egui::Stroke::new(CENTERLINE_WIDTH_PX, color.gamma_multiply(0.5)),
                ));
            }
            draw_stitch_line(painter, ui, view, line, color);
        }
    }
}

/// The holes of a stitch line, filled with the canvas background so they read as punched. Too
/// fine to show at this zoom, the path is drawn dotted and faded instead.
fn draw_stitch_line(
    painter: &egui::Painter,
    ui: &egui::Ui,
    view: Viewport,
    line: &StitchLine,
    color: egui::Color32,
) {
    if (line.pitch as f32) * view.zoom < MIN_HOLE_PITCH_PX {
        let path = to_screen(view, &line.path());
        painter.extend(egui::Shape::dotted_line(
            &path,
            color.gamma_multiply(0.6),
            MIN_HOLE_PITCH_PX,
            0.5,
        ));
        return;
    }
    let background = ui.visuals().extreme_bg_color;
    let stroke = egui::Stroke::new(1.0, color);
    for hole in line.holes() {
        if line.hole == HoleKind::Round {
            let radius = (line.size as f32 * 0.5 * view.zoom).max(0.5);
            painter.circle(view.to_screen(hole.center), radius, background, stroke);
        } else {
            let outline = to_screen(view, &line.hole_outline(&hole));
            painter.add(egui::Shape::convex_polygon(outline, background, stroke));
        }
    }
}

/// The stitch line being drawn: its vertices, the segment to the cursor, and its holes. A
/// segment that would leave the piece is drawn alone, in the error color.
fn draw_stitch_draft(
    painter: &egui::Painter,
    ui: &egui::Ui,
    hover: Option<egui::Pos2>,
    view: Viewport,
    project: &Project,
    state: &EditorState,
) {
    let Some(&last) = state.draft.last() else {
        return;
    };
    let mut points = state.draft.clone();
    let shift = ui.input(|input| input.modifiers.shift);
    let target = hover
        .and(state.hover_target)
        .map(|point| stitch_target(state, point, shift));
    let (target, rejected) = match target {
        Some(target) if !stitch_allowed(project, state, target) => (None, Some(target)),
        target => (target, None),
    };
    if let Some(rejected) = rejected {
        painter.line_segment(
            [view.to_screen(last), view.to_screen(rejected)],
            egui::Stroke::new(1.0, ui.visuals().error_fg_color),
        );
    }
    let units = project.units;
    if let Some(target) = target
        && target != last
    {
        points.push(target);
    }
    let color = SELECTED_COLOR;
    let screen = to_screen(view, &points);
    painter.add(egui::Shape::line(
        screen,
        egui::Stroke::new(1.0, color.gamma_multiply(0.6)),
    ));
    for position in to_screen(view, &state.draft) {
        painter.circle_filled(position, 3.0, color);
    }
    if let Some(line) = stitch_line(state, points) {
        draw_stitch_line(painter, ui, view, &line, color);
        if let Some(target) = target {
            painter.text(
                view.to_screen(target) + egui::vec2(10.0, -10.0),
                egui::Align2::LEFT_BOTTOM,
                format!(
                    "{} · {}",
                    format_length(state.language, units, line.length()),
                    line.hole_count()
                ),
                egui::FontId::proportional(12.0),
                color,
            );
        }
    }
}

/// A line in its dash pattern, which is in mm and so scales with the zoom.
fn draw_line(painter: &egui::Painter, view: Viewport, line: &Line, stroke: egui::Stroke) {
    let ends = [view.to_screen(line.start), view.to_screen(line.end)];
    let period = line
        .style
        .pattern()
        .map(|pattern| pattern.iter().sum::<f64>() as f32 * view.zoom);
    match period {
        Some(period) if period >= MIN_DASH_PERIOD_PX => {
            for (a, b) in line.dashes() {
                painter.line_segment([view.to_screen(a), view.to_screen(b)], stroke);
            }
        }
        // Too fine to show its dashes at this zoom.
        Some(_) => {
            painter.line_segment(
                ends,
                egui::Stroke::new(stroke.width, stroke.color.gamma_multiply(0.6)),
            );
        }
        None => {
            painter.line_segment(ends, stroke);
        }
    }
}

fn draw_line_label(
    painter: &egui::Painter,
    view: Viewport,
    line: &Line,
    units: Units,
    state: &EditorState,
) {
    painter.text(
        view.to_screen(line.midpoint()),
        egui::Align2::CENTER_BOTTOM,
        format_length(state.language, units, line.length()),
        egui::FontId::proportional(11.0),
        Palette::of(painter).label,
    );
}

/// The selected edge, and with the Edge tool the edge under the cursor.
fn draw_edges(
    painter: &egui::Painter,
    hover: Option<egui::Pos2>,
    view: Viewport,
    project: &Project,
    state: &EditorState,
) {
    let color = Palette::of(painter).edge;
    let segment = |piece: u64, shape: ShapeRef, edge: usize| {
        let piece = project
            .pieces
            .iter()
            .find(|candidate| candidate.id == piece)?;
        let piece = state.displayed(piece);
        let Shape::Polygon(polygon) = shape.get(&piece)? else {
            return None;
        };
        let (a, b) = polygon.edge(edge)?;
        Some([view.to_screen(a), view.to_screen(b)])
    };
    // A vertex under the cursor is dragged rather than its edge picked, so it isn't highlighted.
    let on_vertex = |hover: egui::Pos2| {
        state
            .selected
            .and_then(|id| project.pieces.iter().find(|piece| piece.id == id))
            .filter(|piece| !piece.locked)
            .and_then(|piece| handle_at(piece, state.selected_child_id(), hover, view))
            .is_some()
    };
    if state.tool == Tool::Edge
        && state.drag.is_none()
        && let Some(hover) = hover
        && !on_vertex(hover)
        && let Some((piece, shape, edge)) = edge_at(project, state, hover, view)
        && let Some(points) = segment(piece, shape, edge)
    {
        painter.line_segment(points, egui::Stroke::new(3.0, color.gamma_multiply(0.5)));
    }
    let Some(piece) = state.selected else {
        return;
    };
    match state.selected_part() {
        Some((shape, ShapePart::Edge(edge))) => {
            if let Some(points) = segment(piece, shape, edge) {
                painter.line_segment(points, egui::Stroke::new(4.0, color));
                for point in points {
                    painter.circle_filled(point, 4.0, color);
                }
            }
        }
        Some((shape, ShapePart::Vertex(vertex))) => {
            let corner = project
                .pieces
                .iter()
                .find(|candidate| candidate.id == piece)
                .and_then(|piece| shape.get(&state.displayed(piece))?.corners())
                .and_then(|corners| corners.get(vertex).copied());
            if let Some(corner) = corner {
                let center = view.to_screen(corner);
                painter.circle_stroke(center, 7.0, egui::Stroke::new(2.0, color));
                painter.circle_filled(center, 3.0, color);
            }
        }
        None => {}
    }
}

/// The vertex of the selected, unlocked piece under the cursor, on a shape with corners: the
/// selected child first, then the base.
fn vertex_at(
    project: &Project,
    state: &EditorState,
    position: egui::Pos2,
    view: Viewport,
) -> Option<(u64, ShapeRef, usize)> {
    let piece = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id))
        .filter(|piece| !piece.locked)?;
    let (shape, vertex) = handle_at(piece, state.selected_child_id(), position, view)?;
    shape
        .get(piece)?
        .corners()
        .is_some()
        .then_some((piece.id, shape, vertex))
}

fn draw_edge_labels(
    painter: &egui::Painter,
    ring: &[Point],
    screen: &[egui::Pos2],
    units: Units,
    state: &EditorState,
) {
    if ring.len() > MAX_POINTS_WITH_EDGE_LABELS {
        return;
    }
    for index in 0..ring.len() {
        let a = ring[index];
        let b = ring[(index + 1) % ring.len()];
        let next = screen[(index + 1) % screen.len()];
        painter.text(
            screen[index].lerp(next, 0.5),
            egui::Align2::CENTER_BOTTOM,
            format_length(state.language, units, (b.x - a.x).hypot(b.y - a.y)),
            egui::FontId::proportional(11.0),
            Palette::of(painter).label,
        );
    }
}

// Triangulated because pieces can be concave or have holes, which `Shape::convex_polygon`
// doesn't fill correctly.
fn fill_polygon(
    painter: &egui::Painter,
    screen: &[egui::Pos2],
    hole_indices: &[usize],
    color: egui::Color32,
) {
    let flat: Vec<f64> = screen
        .iter()
        .flat_map(|position| [f64::from(position.x), f64::from(position.y)])
        .collect();
    let Ok(triangles) = earcut(&flat, hole_indices, 2) else {
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
    let color = draft_color(state);
    let stroke = egui::Stroke::new(2.0, color);
    let screen = to_screen(view, &state.draft);
    painter.add(egui::Shape::line(screen.clone(), stroke));
    for position in &screen {
        painter.circle_filled(*position, 4.0, color);
    }
    if let Some(target) = hover.and(state.hover_target) {
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
    let Some(shape) = state.tool.shape(
        draft.start,
        draft.end,
        draft.constrained,
        state.polygon_sides,
        state.line_style,
    ) else {
        return;
    };
    let color = draft_color(state);
    match &shape {
        Shape::Line(line) => draw_line(
            painter,
            view,
            line,
            egui::Stroke::new(1.5, Palette::of(painter).centerline),
        ),
        _ => {
            painter.add(egui::Shape::closed_line(
                to_screen(view, &shape.ring()),
                egui::Stroke::new(2.0, color),
            ));
        }
    }
    let length = |millimeters: f64| format_length(state.language, units, millimeters);
    let size = match shape {
        Shape::Line(line) => length(line.length()),
        Shape::Ellipse(Ellipse {
            radius_x, radius_y, ..
        }) if radius_x == radius_y => format!("Ø {}", length(radius_x * 2.0)),
        Shape::Ellipse(Ellipse {
            radius_x, radius_y, ..
        }) => format!("{} × {}", length(radius_x * 2.0), length(radius_y * 2.0)),
        Shape::Rectangle(Rectangle { width, height, .. }) => {
            format!("{} × {}", length(width), length(height))
        }
        Shape::RegularPolygon(RegularPolygon { radius, .. }) => format!("R {}", length(radius)),
        Shape::Polygon(_) | Shape::StitchLine(_) => return,
    };
    painter.text(
        view.to_screen(draft.end) + egui::vec2(10.0, -10.0),
        egui::Align2::LEFT_BOTTOM,
        size,
        egui::FontId::proportional(12.0),
        color,
    );
}
