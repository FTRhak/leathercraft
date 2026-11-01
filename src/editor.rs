use crate::document::{ActiveProject, ProjectMessage};
use crate::panels::layout;
use crate::panels::tools::{self, Tool};
use crate::preview::{self, PreviewSettings};
use crate::settings;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use leathercraft::events::{ProjectChange, ProjectEvent};
use leathercraft::exports::{self, ExportError};
use leathercraft::i18n::{Label, Language, Message};
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

#[derive(Clone, Copy)]
struct ShapeDraft {
    start: Point,
    end: Point,
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
    shape_draft: Option<ShapeDraft>,
    selected: Option<u64>,
    drag: Option<DragState>,
    pan: egui::Vec2,
    zoom: f32,
    fit_requested: bool,
    snap_enabled: bool,
    snap_mm: f64,
    rename_buffer: Option<String>,
    language: Language,
    /// Kept untranslated so that it follows a change of language.
    status: Message,
    status_is_error: bool,
    cursor: Option<Point>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            tool: Tool::default(),
            draft: Vec::new(),
            shape_draft: None,
            selected: None,
            drag: None,
            pan: egui::Vec2::ZERO,
            zoom: 2.0,
            fit_requested: false,
            snap_enabled: true,
            snap_mm: 1.0,
            rename_buffer: None,
            language: Language::default(),
            status: Message::Ready,
            status_is_error: false,
            cursor: None,
        }
    }
}

impl EditorState {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            ..Self::default()
        }
    }

    fn report(&mut self, message: Message) {
        self.status = message;
        self.status_is_error = false;
    }

    pub fn report_error(&mut self, message: Message) {
        self.status = message;
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
        self.shape_draft = None;
        self.selected = None;
        self.drag = None;
        self.rename_buffer = None;
        self.fit_requested = true;
    }

    pub fn on_project_change(&mut self, change: ProjectChange, project: &Project) {
        match change {
            ProjectChange::Created => {
                self.reset_document();
                self.report(Message::NewProject);
            }
            ProjectChange::Opened => {
                self.reset_document();
                self.report(Message::Opened {
                    path: document_name(project),
                });
            }
            ProjectChange::Saved => self.report(Message::Saved {
                path: document_name(project),
            }),
            ProjectChange::Edited => {}
            ProjectChange::PieceAdded(id) => {
                self.draft.clear();
                self.selected = Some(id);
                self.tool = Tool::Select;
                if let Some(piece) = project.pieces.iter().find(|piece| piece.id == id) {
                    self.report(Message::PieceAdded {
                        name: piece.name.clone(),
                    });
                }
            }
            ProjectChange::PieceRemoved(id) => {
                if self.selected == Some(id) {
                    self.selected = None;
                }
                if self.drag.as_ref().is_some_and(|drag| drag.piece_id == id) {
                    self.drag = None;
                }
                self.report(Message::PieceDeleted);
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
            Some(drag) if drag.piece_id == piece.id => Cow::Owned(PatternPiece {
                points: drag.points.clone(),
                ..piece.clone()
            }),
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
            let language = state.language;
            ui.selectable_value(
                &mut *view_mode,
                ViewMode::Editor2d,
                language.label(Label::Editor2d),
            );
            ui.selectable_value(
                &mut *view_mode,
                ViewMode::Preview3d,
                language.label(Label::Preview3d),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                language_control(ui, state);
            });
        });
    });

    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let language = state.language;
            let status = language.message(&state.status);
            if state.status_is_error {
                ui.colored_label(ui.visuals().error_fg_color, status);
            } else {
                ui.label(status);
            }
            if let Some(cursor) = state.cursor {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "{}, {}",
                        format_length(language, project.units, cursor.x),
                        format_length(language, project.units, cursor.y)
                    ));
                });
            }
        });
    });

    if *view_mode == ViewMode::Editor2d {
        egui::SidePanel::left("tools")
            .default_width(110.0)
            .show(ctx, |ui| {
                tools::show_panel(ui, &mut state.tool, state.language);
            });
    }

    egui::SidePanel::left("sidebar")
        .default_width(220.0)
        .show(ctx, |ui| {
            if *view_mode == ViewMode::Editor2d {
                view_controls(ui, state);
                ui.separator();
            } else {
                preview::show_controls(ui, &mut settings, state.language);
                ui.separator();
            }
            units_control(ui, project, &mut events, state.language);
            ui.separator();
            selected_piece_details(ui, project, &mut events, state);
        });

    egui::SidePanel::right("layout")
        .default_width(200.0)
        .show(ctx, |ui| {
            let event = layout::show_panel(ui, project, &mut state.selected, state.language);
            if let Some(event) = event {
                send(&mut events, event);
            }
        });

    handle_delete_key(ctx, &mut events, state);

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

fn handle_delete_key(
    ctx: &egui::Context,
    events: &mut MessageWriter<ProjectMessage>,
    state: &EditorState,
) {
    let drafting = !state.draft.is_empty() || state.shape_draft.is_some();
    if drafting || ctx.wants_keyboard_input() {
        return;
    }
    let delete_pressed = ctx.input(|input| {
        input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)
    });
    if delete_pressed && let Some(id) = state.selected {
        send(events, ProjectEvent::RemovePiece { id });
    }
}

fn file_buttons(
    ui: &mut egui::Ui,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    let language = state.language;
    if ui.button(language.label(Label::New)).clicked() && confirm_discard(project, language) {
        send(events, ProjectEvent::New);
    }
    if ui.button(language.label(Label::Open)).clicked()
        && confirm_discard(project, language)
        && let Some(path) = project_dialog(language).pick_file()
    {
        send(events, ProjectEvent::Open { path });
    }
    if ui.button(language.label(Label::Save)).clicked() {
        match project.path.clone() {
            Some(path) => send(events, ProjectEvent::Save { path }),
            None => save_as(events, language),
        }
    }
    if ui.button(language.label(Label::SaveAs)).clicked() {
        save_as(events, language);
    }
    ui.separator();
    if ui.button(language.label(Label::ExportSvg)).clicked() {
        export(project, state, "SVG", "svg", exports::to_svg);
    }
    if ui.button(language.label(Label::ExportDxf)).clicked() {
        export(project, state, "DXF", "dxf", exports::to_dxf);
    }
}

fn language_control(ui: &mut egui::Ui, state: &mut EditorState) {
    let mut language = state.language;
    egui::ComboBox::from_label(language.label(Label::Language))
        .selected_text(language.native_name())
        .show_ui(ui, |ui| {
            for option in Language::ALL {
                ui.selectable_value(&mut language, option, option.native_name());
            }
        });
    if language != state.language {
        state.language = language;
        settings::save_language(language);
    }
}

fn confirm_discard(project: &Project, language: Language) -> bool {
    if !project.modified {
        return true;
    }
    rfd::MessageDialog::new()
        .set_title(language.label(Label::UnsavedChanges))
        .set_description(language.label(Label::DiscardChanges))
        .set_buttons(rfd::MessageButtons::YesNo)
        .show()
        == rfd::MessageDialogResult::Yes
}

fn save_as(events: &mut MessageWriter<ProjectMessage>, language: Language) {
    if let Some(mut path) = project_dialog(language).save_file() {
        if path.extension().is_none() {
            path.set_extension(PROJECT_EXTENSION);
        }
        send(events, ProjectEvent::Save { path });
    }
}

fn project_dialog(language: Language) -> rfd::FileDialog {
    rfd::FileDialog::new().add_filter(language.label(Label::ProjectFileType), &[PROJECT_EXTENSION])
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
            state.report_error(error.into());
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
        Ok(()) => state.report(Message::Exported {
            path: path.display().to_string(),
        }),
        Err(error) => state.report_error(Message::CouldNotWrite {
            path: path.display().to_string(),
            detail: error.to_string(),
        }),
    }
}

fn view_controls(ui: &mut egui::Ui, state: &mut EditorState) {
    let language = state.language;
    ui.small(language.label(state.tool.hint()));
    ui.small(language.label(Label::NavigationHint));
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut state.snap_enabled,
            language.label(Label::SnapMillimeters),
        );
        ui.add_enabled(
            state.snap_enabled,
            egui::DragValue::new(&mut state.snap_mm)
                .range(0.1..=100.0)
                .speed(0.1),
        );
    });
    if ui.button(language.label(Label::ZoomToFit)).clicked() {
        state.fit_requested = true;
    }
}

fn units_control(
    ui: &mut egui::Ui,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    language: Language,
) {
    let mut units = project.units;
    egui::ComboBox::from_label(language.label(Label::DisplayUnits))
        .selected_text(language.unit_symbol(units))
        .show_ui(ui, |ui| {
            for option in [Units::Millimeters, Units::Inches] {
                ui.selectable_value(&mut units, option, language.unit_symbol(option));
            }
        });
    if units != project.units {
        send(events, ProjectEvent::SetUnits(units));
    }
}

fn selected_piece_details(
    ui: &mut egui::Ui,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    let language = state.language;
    let units = project.units;
    let Some(piece) = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id))
    else {
        state.rename_buffer = None;
        return;
    };

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
    ui.label(language.message(&Message::PointCount {
        count: shown.points.len(),
    }));
    if let Some(bounds) = shown.bounds() {
        ui.label(format!(
            "{} × {}",
            format_length(language, units, bounds.width()),
            format_length(language, units, bounds.height())
        ));
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
    if piece.locked {
        return;
    }
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

fn format_length(language: Language, units: Units, millimeters: f64) -> String {
    format!(
        "{:.2} {}",
        units.from_millimeters(millimeters),
        language.unit_symbol(units)
    )
}
