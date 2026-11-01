use crate::document::{ActiveProject, ProjectMessage};
use crate::preview::{self, PreviewSettings};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use leathercraft::events::{ProjectChange, ProjectEvent};
use leathercraft::exports::{self, ExportError};
use leathercraft::model::{PatternPiece, Point, Project, Units};
use leathercraft::persistence;
use std::borrow::Cow;

const PROJECT_EXTENSION: &str = "leathercraft";
const MIN_ZOOM: f32 = 0.05;
const MAX_ZOOM: f32 = 100.0;
const GRID_STEPS_MM: [f64; 9] = [
    1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1_000.0, 5_000.0, 10_000.0,
];
const MIN_GRID_SPACING_PX: f64 = 12.0;
const VERTEX_HIT_RADIUS_PX: f32 = 8.0;
const CLOSE_HIT_RADIUS_PX: f32 = 10.0;
const MAX_POINTS_WITH_EDGE_LABELS: usize = 64;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Editor2d,
    Preview3d,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Tool {
    #[default]
    Select,
    Draw,
}

struct DragState {
    piece_id: u64,
    vertex: Option<usize>,
    start: Point,
    original: Vec<Point>,
    /// The in-progress shape, drawn instead of the project's until the drag is committed.
    points: Vec<Point>,
    /// Keeps the drag drawn for the frame in which its event is still being applied.
    released: bool,
}

#[derive(Resource)]
pub struct EditorState {
    tool: Tool,
    draft: Vec<Point>,
    selected: Option<u64>,
    drag: Option<DragState>,
    pan: egui::Vec2,
    zoom: f32,
    fit_requested: bool,
    snap_enabled: bool,
    snap_mm: f64,
    rename_buffer: Option<String>,
    status: String,
    status_is_error: bool,
    cursor: Option<Point>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            tool: Tool::default(),
            draft: Vec::new(),
            selected: None,
            drag: None,
            pan: egui::Vec2::ZERO,
            zoom: 2.0,
            fit_requested: false,
            snap_enabled: true,
            snap_mm: 1.0,
            rename_buffer: None,
            status: "Ready.".to_owned(),
            status_is_error: false,
            cursor: None,
        }
    }
}

impl EditorState {
    fn report(&mut self, message: impl Into<String>) {
        self.status = message.into();
        self.status_is_error = false;
    }

    pub fn report_error(&mut self, message: impl Into<String>) {
        self.status = message.into();
        self.status_is_error = true;
    }

    fn snap(&self, value: f64) -> f64 {
        if self.snap_enabled {
            (value / self.snap_mm).round() * self.snap_mm
        } else {
            value
        }
    }

    fn snap_point(&self, point: Point) -> Point {
        Point::new(self.snap(point.x), self.snap(point.y))
    }

    fn reset_document(&mut self) {
        self.draft.clear();
        self.selected = None;
        self.drag = None;
        self.rename_buffer = None;
        self.fit_requested = true;
    }

    pub fn on_project_change(&mut self, change: ProjectChange, project: &Project) {
        match change {
            ProjectChange::Created => {
                self.reset_document();
                self.report("New project.");
            }
            ProjectChange::Opened => {
                self.reset_document();
                self.report(format!("Opened {}.", document_name(project)));
            }
            ProjectChange::Saved => self.report(format!("Saved {}.", document_name(project))),
            ProjectChange::Edited => {}
            ProjectChange::PieceAdded(id) => {
                self.draft.clear();
                self.selected = Some(id);
                self.tool = Tool::Select;
                self.report(format!("Added Piece {id}."));
            }
            ProjectChange::PieceRemoved(_) => {
                self.selected = None;
                self.drag = None;
                self.report("Piece deleted.");
            }
        }
    }

    pub fn clear_released_drag(&mut self) {
        if self.drag.as_ref().is_some_and(|drag| drag.released) {
            self.drag = None;
        }
    }

    fn displayed<'a>(&self, piece: &'a PatternPiece) -> Cow<'a, PatternPiece> {
        match &self.drag {
            Some(drag) if drag.piece_id == piece.id => Cow::Owned(PatternPiece::new(
                piece.id,
                piece.name.clone(),
                drag.points.clone(),
            )),
            _ => Cow::Borrowed(piece),
        }
    }
}

fn document_name(project: &Project) -> String {
    project
        .path
        .as_ref()
        .map_or_else(String::new, |path| path.display().to_string())
}

#[derive(Clone, Copy)]
struct Viewport {
    origin: egui::Pos2,
    zoom: f32,
}

impl Viewport {
    fn to_screen(self, point: Point) -> egui::Pos2 {
        self.origin + egui::vec2(point.x as f32, point.y as f32) * self.zoom
    }

    fn to_world(self, position: egui::Pos2) -> Point {
        let offset = (position - self.origin) / self.zoom;
        Point::new(f64::from(offset.x), f64::from(offset.y))
    }
}

pub fn editor_ui(
    mut contexts: EguiContexts,
    project: Res<ActiveProject>,
    mut events: MessageWriter<ProjectMessage>,
    mut state: ResMut<EditorState>,
    mut view_mode: ResMut<ViewMode>,
    mut settings: ResMut<PreviewSettings>,
) -> Result {
    let ctx: &egui::Context = contexts.ctx_mut()?;
    let project: &Project = &project;
    let state = &mut *state;

    egui::TopBottomPanel::top("file_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            file_buttons(ui, project, &mut events, state);
            ui.separator();
            ui.selectable_value(&mut *view_mode, ViewMode::Editor2d, "2D editor");
            ui.selectable_value(&mut *view_mode, ViewMode::Preview3d, "3D preview");
        });
    });

    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if state.status_is_error {
                ui.colored_label(ui.visuals().error_fg_color, &state.status);
            } else {
                ui.label(&state.status);
            }
            if let Some(cursor) = state.cursor {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "{}, {}",
                        format_length(project.units, cursor.x),
                        format_length(project.units, cursor.y)
                    ));
                });
            }
        });
    });

    egui::SidePanel::left("tools")
        .default_width(220.0)
        .show(ctx, |ui| {
            if *view_mode == ViewMode::Editor2d {
                tool_controls(ui, state);
                ui.separator();
            } else {
                preview::show_controls(ui, &mut settings);
                ui.separator();
            }
            units_control(ui, project, &mut events);
            ui.separator();
            pieces_list(ui, project, &mut events, state);
        });

    // In preview mode no central panel is drawn, so the 3D scene shows through.
    if *view_mode == ViewMode::Editor2d {
        egui::CentralPanel::default().show(ctx, |ui| canvas(ui, project, &mut events, state));
    } else {
        state.cursor = None;
    }
    Ok(())
}

fn send(events: &mut MessageWriter<ProjectMessage>, event: ProjectEvent) {
    events.write(ProjectMessage(event));
}

fn file_buttons(
    ui: &mut egui::Ui,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    if ui.button("New").clicked() && confirm_discard(project) {
        send(events, ProjectEvent::New);
    }
    if ui.button("Open…").clicked()
        && confirm_discard(project)
        && let Some(path) = project_dialog().pick_file()
    {
        send(events, ProjectEvent::Open { path });
    }
    if ui.button("Save").clicked() {
        match project.path.clone() {
            Some(path) => send(events, ProjectEvent::Save { path }),
            None => save_as(events),
        }
    }
    if ui.button("Save as…").clicked() {
        save_as(events);
    }
    ui.separator();
    if ui.button("Export SVG…").clicked() {
        export(project, state, "SVG", "svg", exports::to_svg);
    }
    if ui.button("Export DXF…").clicked() {
        export(project, state, "DXF", "dxf", exports::to_dxf);
    }
}

fn confirm_discard(project: &Project) -> bool {
    if !project.modified {
        return true;
    }
    rfd::MessageDialog::new()
        .set_title("Unsaved changes")
        .set_description("Discard the unsaved changes to this project?")
        .set_buttons(rfd::MessageButtons::YesNo)
        .show()
        == rfd::MessageDialogResult::Yes
}

fn save_as(events: &mut MessageWriter<ProjectMessage>) {
    if let Some(mut path) = project_dialog().save_file() {
        if path.extension().is_none() {
            path.set_extension(PROJECT_EXTENSION);
        }
        send(events, ProjectEvent::Save { path });
    }
}

fn project_dialog() -> rfd::FileDialog {
    rfd::FileDialog::new().add_filter("Leathercraft project", &[PROJECT_EXTENSION])
}

fn export(
    project: &Project,
    state: &mut EditorState,
    label: &str,
    extension: &str,
    encode: fn(&Project) -> std::result::Result<String, ExportError>,
) {
    let contents = match encode(project) {
        Ok(contents) => contents,
        Err(error) => {
            state.report_error(error.to_string());
            return;
        }
    };
    let Some(mut path) = rfd::FileDialog::new()
        .add_filter(label, &[extension])
        .save_file()
    else {
        return;
    };
    if path.extension().is_none() {
        path.set_extension(extension);
    }
    match persistence::write_export(&path, &contents) {
        Ok(()) => state.report(format!("Exported {}.", path.display())),
        Err(error) => state.report_error(format!("Could not write '{}': {error}", path.display())),
    }
}

fn tool_controls(ui: &mut egui::Ui, state: &mut EditorState) {
    ui.label("Tool");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.tool, Tool::Select, "Select");
        ui.selectable_value(&mut state.tool, Tool::Draw, "Draw");
    });
    if state.tool == Tool::Draw {
        ui.small(
            "Click to add points. Click the first point or press Enter to close; Esc cancels.",
        );
    } else {
        ui.small("Drag a piece or a vertex of the selected piece. Del removes the selected piece.");
    }
    ui.small("Right or middle drag pans; scroll zooms.");
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.snap_enabled, "Snap (mm)");
        ui.add_enabled(
            state.snap_enabled,
            egui::DragValue::new(&mut state.snap_mm)
                .range(0.1..=100.0)
                .speed(0.1),
        );
    });
    if ui.button("Zoom to fit").clicked() {
        state.fit_requested = true;
    }
}

fn units_control(ui: &mut egui::Ui, project: &Project, events: &mut MessageWriter<ProjectMessage>) {
    let mut units = project.units;
    egui::ComboBox::from_label("Display units")
        .selected_text(units.symbol())
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut units, Units::Millimeters, Units::Millimeters.symbol());
            ui.selectable_value(&mut units, Units::Inches, Units::Inches.symbol());
        });
    if units != project.units {
        send(events, ProjectEvent::SetUnits(units));
    }
}

fn pieces_list(
    ui: &mut egui::Ui,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    ui.label("Pieces");
    egui::ScrollArea::vertical()
        .max_height(220.0)
        .show(ui, |ui| {
            for piece in &project.pieces {
                let selected = state.selected == Some(piece.id);
                if ui.selectable_label(selected, &piece.name).clicked() {
                    state.selected = Some(piece.id);
                }
            }
        });

    let units = project.units;
    let Some(piece) = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id))
    else {
        state.rename_buffer = None;
        return;
    };

    ui.separator();
    // The buffer lets the box be empty mid-edit, which the project itself would reject.
    let mut name = state
        .rename_buffer
        .clone()
        .unwrap_or_else(|| piece.name.clone());
    let response = ui.text_edit_singleline(&mut name);
    if response.changed() && !name.trim().is_empty() {
        send(
            events,
            ProjectEvent::RenamePiece {
                id: piece.id,
                name: name.clone(),
            },
        );
    }
    state.rename_buffer = response.has_focus().then_some(name);

    let shown = state.displayed(piece);
    ui.label(format!("{} points", shown.points.len()));
    if let Some(bounds) = shown.bounds() {
        ui.label(format!(
            "{} × {}",
            format_length(units, bounds.width()),
            format_length(units, bounds.height())
        ));
    }
    if ui.button("Delete piece").clicked() {
        send(events, ProjectEvent::RemovePiece { id: piece.id });
    }
}

fn canvas(
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

    match state.tool {
        Tool::Select => {
            state.draft.clear();
            handle_select(ui, &response, view, project, events, state);
        }
        Tool::Draw => handle_draw(ui, &response, view, events, state),
    }

    painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
    draw_grid(&painter, rect, view);
    draw_pieces(&painter, view, project, state);
    draw_draft(&painter, response.hover_pos(), view, project.units, state);
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
    let delete_pressed = ui.input(|input| {
        input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)
    });
    if delete_pressed
        && !ui.ctx().wants_keyboard_input()
        && let Some(id) = state.selected
    {
        send(events, ProjectEvent::RemovePiece { id });
    }
}

fn begin_drag(origin: egui::Pos2, view: Viewport, project: &Project, state: &mut EditorState) {
    let selected = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id));
    let vertex = selected.and_then(|piece| vertex_at(piece, origin, view));
    let start = view.to_world(origin);
    let piece_id = match (vertex, selected) {
        (Some(_), Some(piece)) => Some(piece.id),
        _ => piece_at(project, start),
    };
    let Some(piece_id) = piece_id else {
        return;
    };
    let Some(piece) = project.pieces.iter().find(|piece| piece.id == piece_id) else {
        return;
    };
    state.selected = Some(piece_id);
    state.drag = Some(DragState {
        piece_id,
        vertex,
        start,
        original: piece.points.clone(),
        points: piece.points.clone(),
        released: false,
    });
}

fn update_drag(cursor: Point, state: &mut EditorState) {
    let Some(drag) = &state.drag else {
        return;
    };
    let mut points = drag.original.clone();
    match drag.vertex {
        Some(vertex) => points[vertex] = state.snap_point(cursor),
        None => {
            // Snap the offset, not the points, so the piece keeps its own shape.
            let dx = state.snap(cursor.x - drag.start.x);
            let dy = state.snap(cursor.y - drag.start.y);
            for point in &mut points {
                point.x += dx;
                point.y += dy;
            }
        }
    }
    if let Some(drag) = &mut state.drag {
        drag.points = points;
    }
}

fn finish_drag(events: &mut MessageWriter<ProjectMessage>, state: &mut EditorState) {
    let Some(drag) = &mut state.drag else {
        return;
    };
    drag.released = true;
    if drag.points != drag.original {
        send(
            events,
            ProjectEvent::SetPiecePoints {
                id: drag.piece_id,
                points: drag.points.clone(),
            },
        );
    }
}

fn handle_draw(
    ui: &egui::Ui,
    response: &egui::Response,
    view: Viewport,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    if response.clicked()
        && let Some(position) = response.interact_pointer_pos()
    {
        let closes = state.draft.len() >= 3
            && view.to_screen(state.draft[0]).distance(position) <= CLOSE_HIT_RADIUS_PX;
        if closes {
            finish_draft(events, state);
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
        finish_draft(events, state);
    }
    if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.draft.clear();
    }
}

fn finish_draft(events: &mut MessageWriter<ProjectMessage>, state: &EditorState) {
    send(
        events,
        ProjectEvent::AddPiece {
            points: state.draft.clone(),
        },
    );
}

fn piece_at(project: &Project, point: Point) -> Option<u64> {
    project
        .pieces
        .iter()
        .rev()
        .find(|piece| piece.contains(point))
        .map(|piece| piece.id)
}

fn vertex_at(piece: &PatternPiece, position: egui::Pos2, view: Viewport) -> Option<usize> {
    piece
        .points
        .iter()
        .position(|point| view.to_screen(*point).distance(position) <= VERTEX_HIT_RADIUS_PX)
}

fn draw_grid(painter: &egui::Painter, rect: egui::Rect, view: Viewport) {
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

fn draw_pieces(painter: &egui::Painter, view: Viewport, project: &Project, state: &EditorState) {
    let text_color = egui::Color32::from_gray(220);
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
        for position in &screen {
            painter.circle_filled(*position, 4.0, stroke.color);
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
                format_length(project.units, (b.x - a.x).hypot(b.y - a.y)),
                egui::FontId::proportional(11.0),
                text_color,
            );
        }
    }
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
            format_length(units, (target.x - last.x).hypot(target.y - last.y)),
            egui::FontId::proportional(12.0),
            color,
        );
    }
}

fn format_length(units: Units, millimeters: f64) -> String {
    format!(
        "{:.2} {}",
        units.from_millimeters(millimeters),
        units.symbol()
    )
}
