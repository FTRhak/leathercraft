use super::drag_state_struct::ShapeRef;
use super::{DragState, ShapeDraft};
use crate::panels::tools::Tool;
use bevy::prelude::*;
use bevy_egui::egui;
use leathercraft::events::ProjectChange;
use leathercraft::i18n::{Language, Message};
use leathercraft::material::Material;
use leathercraft::model::{BooleanOp, PatternPiece, Point, Project};
use std::borrow::Cow;

#[derive(Resource)]
pub struct EditorState {
    pub(super) tool: Tool,
    pub(super) draft: Vec<Point>,
    pub(super) shape_draft: Option<ShapeDraft>,
    /// The number of sides the Regular polygon tool draws.
    pub(super) polygon_sides: usize,
    /// With an operation, the drawing tools add a child with it to the selected piece instead
    /// of adding a piece.
    pub(super) op_mode: Option<BooleanOp>,
    /// The primary selected piece: the one shown in Properties and reshaped with handles.
    pub(super) selected: Option<u64>,
    /// Every selected piece, in the order selected. It contains `selected` whenever that is set,
    /// and has more than one piece only through Shift+click or Ctrl+A.
    pub(super) pieces: Vec<u64>,
    /// Piece id and child id. Read through `selected_child_id`, since it only counts
    /// while that piece is the selected one.
    pub(super) selected_child: Option<(u64, u64)>,
    /// Piece id, shape and edge index, picked with the Edge tool. Read through
    /// `selected_edge`, since it only counts while that shape is the one shown in Properties.
    pub(super) selected_edge: Option<(u64, ShapeRef, usize)>,
    pub(super) drag: Option<DragState>,
    pub(super) pan: egui::Vec2,
    pub(super) zoom: f32,
    pub(super) fit_requested: bool,
    pub(super) snap_enabled: bool,
    pub(super) snap_mm: f64,
    pub(super) rename_buffer: Option<String>,
    /// The material being chosen while the new-project dialog is open.
    pub(super) new_project_dialog: Option<Material>,
    pub(super) language: Language,
    /// Kept untranslated so that it follows a change of language.
    pub(super) status: Message,
    pub(super) status_is_error: bool,
    pub(super) cursor: Option<Point>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            tool: Tool::default(),
            draft: Vec::new(),
            shape_draft: None,
            polygon_sides: 6,
            op_mode: None,
            selected: None,
            pieces: Vec::new(),
            selected_child: None,
            selected_edge: None,
            drag: None,
            pan: egui::Vec2::ZERO,
            zoom: 2.0,
            fit_requested: false,
            snap_enabled: true,
            snap_mm: 1.0,
            rename_buffer: None,
            new_project_dialog: None,
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

    pub(super) fn report(&mut self, message: Message) {
        self.status = message;
        self.status_is_error = false;
    }

    pub fn report_error(&mut self, message: Message) {
        self.status = message;
        self.status_is_error = true;
    }

    pub(super) fn snap(&self, value: f64) -> f64 {
        if self.snap_enabled {
            (value / self.snap_mm).round() * self.snap_mm
        } else {
            value
        }
    }

    pub(super) fn snap_point(&self, point: Point) -> Point {
        Point::new(self.snap(point.x), self.snap(point.y))
    }

    fn reset_document(&mut self) {
        self.draft.clear();
        self.shape_draft = None;
        self.selected = None;
        self.pieces.clear();
        self.selected_child = None;
        self.selected_edge = None;
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
                self.select(Some(id), None);
                self.tool = Tool::Select;
                if let Some(piece) = project.pieces.iter().find(|piece| piece.id == id) {
                    self.report(Message::PieceAdded {
                        name: piece.name.clone(),
                    });
                }
            }
            ProjectChange::PieceRemoved(id) => {
                self.pieces.retain(|piece| *piece != id);
                if self.selected == Some(id) {
                    self.selected = self.pieces.last().copied();
                }
                // The id can be given to the next new piece.
                self.selected_child = None;
                self.selected_edge = None;
                if self.drag.as_ref().is_some_and(|drag| drag.piece_id == id) {
                    self.drag = None;
                }
                self.report(Message::PieceDeleted);
            }
            ProjectChange::PiecesRemoved => {
                let count = self.pieces.len();
                self.prune_selection(project);
                let count = count - self.pieces.len();
                self.selected_child = None;
                self.selected_edge = None;
                if self.drag.as_ref().is_some_and(|drag| {
                    !project.pieces.iter().any(|piece| piece.id == drag.piece_id)
                }) {
                    self.drag = None;
                }
                self.report(Message::PiecesDeleted { count });
            }
            // The tool and operation stay, so that several shapes can be added in a row.
            ProjectChange::ChildAdded { id, child } => {
                self.draft.clear();
                self.select(Some(id), Some(child));
                if let Some(piece) = project.pieces.iter().find(|piece| piece.id == id) {
                    self.report(Message::ChildAdded {
                        name: piece.name.clone(),
                    });
                }
            }
            ProjectChange::ChildRemoved { id } => {
                if self.selected_child.is_some_and(|(piece, _)| piece == id) {
                    self.selected_child = None;
                }
                self.selected_edge = None;
                self.report(Message::ChildDeleted);
            }
        }
    }

    /// Work in progress refers to the replaced content, so it is dropped; the selection stays
    /// if its piece still exists.
    pub fn on_history_restore(&mut self, status: Message, project: &Project) {
        self.draft.clear();
        self.shape_draft = None;
        self.drag = None;
        self.rename_buffer = None;
        self.prune_selection(project);
        let child_exists = |(id, child): (u64, u64)| {
            project
                .pieces
                .iter()
                .any(|piece| piece.id == id && piece.child(child).is_some())
        };
        self.selected_child = self
            .selected_child
            .filter(|selected| child_exists(*selected));
        self.report(status);
    }

    pub(super) fn selected_child_id(&self) -> Option<u64> {
        self.selected_child
            .filter(|(id, _)| self.selected == Some(*id))
            .map(|(_, child)| child)
    }

    pub(super) fn select(&mut self, piece: Option<u64>, child: Option<u64>) {
        self.selected = piece;
        self.pieces = piece.into_iter().collect();
        self.selected_child = piece.zip(child);
        self.selected_edge = None;
    }

    pub(super) fn is_selected(&self, piece: u64) -> bool {
        self.pieces.contains(&piece)
    }

    /// Adds the piece to the selection as its primary piece, or removes it, after which the
    /// piece selected before it becomes primary.
    pub(super) fn toggle_piece(&mut self, piece: u64) {
        if self.is_selected(piece) {
            self.pieces.retain(|selected| *selected != piece);
            if self.selected == Some(piece) {
                self.selected = self.pieces.last().copied();
            }
        } else {
            self.pieces.push(piece);
            self.selected = Some(piece);
        }
        self.selected_child = None;
        self.selected_edge = None;
    }

    /// Makes an already selected piece the primary one, keeping the rest of the selection.
    pub(super) fn focus(&mut self, piece: u64) {
        self.selected = Some(piece);
        self.selected_child = None;
        self.selected_edge = None;
    }

    /// The primary piece stays if there is one.
    pub(super) fn select_all(&mut self, project: &Project) {
        self.pieces = project.pieces.iter().map(|piece| piece.id).collect();
        if self.selected.is_none() {
            self.selected = self.pieces.last().copied();
        }
    }

    /// Drops pieces that no longer exist, and picks a new primary piece if it was one of them.
    fn prune_selection(&mut self, project: &Project) {
        let exists = |id: u64| project.pieces.iter().any(|piece| piece.id == id);
        self.pieces.retain(|id| exists(*id));
        if self.selected.is_some_and(|id| !exists(id)) {
            self.selected = self.pieces.last().copied();
        }
    }

    /// Selects the edge along with its piece, and its child if it belongs to one.
    pub(super) fn select_edge(&mut self, piece: u64, shape: ShapeRef, edge: usize) {
        let child = match shape {
            ShapeRef::Base => None,
            ShapeRef::Child(child) => Some(child),
        };
        self.select(Some(piece), child);
        self.selected_edge = Some((piece, shape, edge));
    }

    /// The selected edge, if its shape is the one selected. The index may be out of range
    /// after an undo, so readers check it against the shape.
    pub(super) fn selected_edge(&self) -> Option<(ShapeRef, usize)> {
        let (piece, shape, edge) = self.selected_edge?;
        let shown = match self.selected_child_id() {
            Some(child) => ShapeRef::Child(child),
            None => ShapeRef::Base,
        };
        (self.selected == Some(piece) && shape == shown).then_some((shape, edge))
    }

    pub fn clear_released_drag(&mut self) {
        if self.drag.as_ref().is_some_and(|drag| drag.released) {
            self.drag = None;
        }
    }

    pub(super) fn displayed<'a>(&self, piece: &'a PatternPiece) -> Cow<'a, PatternPiece> {
        match &self.drag {
            Some(drag) if drag.piece_id == piece.id => Cow::Owned(drag.current.clone()),
            Some(drag) if drag.group.contains(&piece.id) => {
                let mut moved = piece.clone();
                moved.translate(drag.offset.0, drag.offset.1);
                Cow::Owned(moved)
            }
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
