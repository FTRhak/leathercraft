use super::ShapePart;
use super::drag_state_struct::ShapeRef;
use super::{DragState, ShapeDraft};
use crate::panels::align::AlignOptions;
use crate::panels::settings::SettingsDraft;
use crate::panels::tools::{StitchSettings, Tool, ZoomRequest};
use bevy::prelude::*;
use bevy_egui::egui;
use leathercraft::events::ProjectChange;
use leathercraft::i18n::{Language, Message};
use leathercraft::material::Material;
use leathercraft::model::{BooleanOp, PatternPiece, Point, Project};
use leathercraft::shapes::LineStyle;
use leathercraft::snap::{self, SnapFeedback, SnapSettings, SnapTargets};
use std::borrow::Cow;

#[derive(Resource)]
pub struct EditorState {
    pub(super) tool: Tool,
    pub(super) draft: Vec<Point>,
    pub(super) shape_draft: Option<ShapeDraft>,
    /// The number of sides the Regular polygon tool draws.
    pub(super) polygon_sides: usize,
    /// The style the Centerline tool draws with.
    pub(super) line_style: LineStyle,
    /// The holes the Stitch holes tool draws with.
    pub(super) stitch: StitchSettings,
    /// The tool that `draft` belongs to, so switching tools drops it.
    pub(super) draft_tool: Tool,
    /// With an operation, the drawing tools add a child with it to the selected piece instead
    /// of adding a piece.
    pub(super) op_mode: Option<BooleanOp>,
    /// The settings of the Align panel.
    pub(super) align: AlignOptions,
    /// The primary selected piece: the one shown in Properties and reshaped with handles.
    pub(super) selected: Option<u64>,
    /// Every selected piece, in the order selected. It contains `selected` whenever that is set,
    /// and has more than one piece only through Shift+click or Ctrl+A.
    pub(super) pieces: Vec<u64>,
    /// Piece id and child id. Read through `selected_child_id`, since it only counts
    /// while that piece is the selected one.
    pub(super) selected_child: Option<(u64, u64)>,
    /// Every selected child of the piece in `selected_child`, in the order selected. It contains
    /// that child, and has more than one only through Shift+click or Ctrl+A. Read through
    /// `selected_children`.
    pub(super) children: Vec<u64>,
    /// Piece id, shape, and the edge or vertex picked with the Edge tool. Read through
    /// `selected_part`, since it only counts while that shape is the one shown in Properties.
    pub(super) selected_part: Option<(u64, ShapeRef, ShapePart)>,
    pub(super) drag: Option<DragState>,
    pub(super) pan: egui::Vec2,
    pub(super) zoom: f32,
    /// A zoom asked for by the tool bar, or by opening a project; done on the next canvas frame.
    pub(super) zoom_request: Option<ZoomRequest>,
    /// The snapping settings of the Snap panel.
    pub(super) snap: SnapSettings,
    /// The settings last written to `settings.json`, so that only a change is saved.
    pub(super) snap_saved: SnapSettings,
    /// Snapping is suspended while Alt is held.
    pub(super) snap_suspended: bool,
    /// What the last snap of this frame lined up with, drawn on the canvas.
    pub(super) snap_feedback: SnapFeedback,
    /// Where a drawing tool would place a point under the cursor, snapped.
    pub(super) hover_target: Option<Point>,
    pub(super) rename_buffer: Option<String>,
    /// The material being chosen while the new-project dialog is open.
    pub(super) new_project_dialog: Option<Material>,
    /// The preferences being edited while the Settings window is open.
    pub(super) settings_dialog: Option<SettingsDraft>,
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
            line_style: LineStyle::dash_dot_default(),
            stitch: StitchSettings::default(),
            draft_tool: Tool::Draw,
            op_mode: None,
            align: AlignOptions::default(),
            selected: None,
            pieces: Vec::new(),
            selected_child: None,
            children: Vec::new(),
            selected_part: None,
            drag: None,
            pan: egui::Vec2::ZERO,
            zoom: 2.0,
            zoom_request: None,
            snap: SnapSettings::default(),
            snap_saved: SnapSettings::default(),
            snap_suspended: false,
            snap_feedback: SnapFeedback::default(),
            hover_target: None,
            rename_buffer: None,
            new_project_dialog: None,
            settings_dialog: None,
            language: Language::default(),
            status: Message::Ready,
            status_is_error: false,
            cursor: None,
        }
    }
}

impl EditorState {
    pub fn new(language: Language, snap: SnapSettings) -> Self {
        Self {
            language,
            snap,
            snap_saved: snap,
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

    /// The snapping settings in effect: off while Alt is held.
    pub(super) fn active_snap(&self) -> SnapSettings {
        SnapSettings {
            enabled: self.snap.enabled && !self.snap_suspended,
            ..self.snap
        }
    }

    /// The point snapped to the targets, remembering what it lined up with for drawing.
    /// `tolerance` is in mm.
    pub(super) fn snap_point(
        &mut self,
        point: Point,
        targets: &SnapTargets,
        tolerance: f64,
    ) -> Point {
        let (point, feedback) = snap::snap_point(point, targets, &self.active_snap(), tolerance);
        self.snap_feedback = feedback;
        point
    }

    /// Writes the snapping settings once they changed and no button is held, so that dragging
    /// a value doesn't write the file on every frame.
    pub fn save_snap_when_settled(&mut self, pointer_down: bool) {
        if !pointer_down && self.snap != self.snap_saved {
            crate::settings::save_snap(&self.snap);
            self.snap_saved = self.snap;
        }
    }

    fn reset_document(&mut self) {
        self.draft.clear();
        self.shape_draft = None;
        self.selected = None;
        self.pieces.clear();
        self.clear_children();
        self.selected_part = None;
        self.drag = None;
        self.rename_buffer = None;
        self.zoom_request = Some(ZoomRequest::All);
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
                self.clear_children();
                self.selected_part = None;
                if self.drag.as_ref().is_some_and(|drag| drag.piece_id == id) {
                    self.drag = None;
                }
                self.report(Message::PieceDeleted);
            }
            ProjectChange::PiecesRemoved => {
                let count = self.pieces.len();
                self.prune_selection(project);
                let count = count - self.pieces.len();
                self.clear_children();
                self.selected_part = None;
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
                    self.clear_children();
                }
                self.selected_part = None;
                self.report(Message::ChildDeleted);
            }
            ProjectChange::ChildrenRemoved { id, count } => {
                if self.selected_child.is_some_and(|(piece, _)| piece == id) {
                    self.clear_children();
                }
                self.selected_part = None;
                self.report(Message::ChildrenDeleted { count });
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
        let piece = self
            .selected_child
            .and_then(|(id, _)| project.pieces.iter().find(|piece| piece.id == id));
        match piece {
            Some(piece) => {
                self.children.retain(|child| piece.child(*child).is_some());
                if self
                    .selected_child
                    .is_some_and(|(_, child)| piece.child(child).is_none())
                {
                    self.selected_child = self.children.last().map(|child| (piece.id, *child));
                }
            }
            None => self.clear_children(),
        }
        self.report(status);
    }

    pub(super) fn selected_child_id(&self) -> Option<u64> {
        self.selected_child
            .filter(|(id, _)| self.selected == Some(*id))
            .map(|(_, child)| child)
    }

    /// The selected children, if their piece is the selected one.
    pub(super) fn selected_children(&self) -> &[u64] {
        match self.selected_child {
            Some((id, _)) if self.selected == Some(id) => &self.children,
            _ => &[],
        }
    }

    fn clear_children(&mut self) {
        self.selected_child = None;
        self.children.clear();
    }

    pub(super) fn select(&mut self, piece: Option<u64>, child: Option<u64>) {
        self.selected = piece;
        self.pieces = piece.into_iter().collect();
        self.selected_child = piece.zip(child);
        self.children = child.into_iter().collect();
        self.selected_part = None;
    }

    /// Adds a child of the selected piece to the selection as its primary child, or removes it,
    /// after which the child selected before it becomes primary. A child of another piece starts
    /// a new selection. Either way the selection narrows to the child's piece.
    pub(super) fn toggle_child(&mut self, piece: u64, child: u64) {
        if self.selected_children().is_empty() || self.selected != Some(piece) {
            self.select(Some(piece), Some(child));
            return;
        }
        if self.children.contains(&child) {
            self.children.retain(|selected| *selected != child);
            if self.selected_child == Some((piece, child)) {
                self.selected_child = self.children.last().map(|child| (piece, *child));
            }
        } else {
            self.children.push(child);
            self.selected_child = Some((piece, child));
        }
        self.pieces = vec![piece];
        self.selected_part = None;
    }

    /// Makes an already selected child the primary one, keeping the rest of the selection.
    pub(super) fn focus_child(&mut self, child: u64) {
        if let Some((piece, _)) = self.selected_child {
            self.selected_child = Some((piece, child));
        }
        self.selected_part = None;
    }

    /// Selects every child of the selected piece; the primary child stays.
    pub(super) fn select_all_children(&mut self, project: &Project) {
        let Some((id, _)) = self
            .selected_child
            .filter(|(id, _)| self.selected == Some(*id))
        else {
            return;
        };
        if let Some(piece) = project.pieces.iter().find(|piece| piece.id == id) {
            self.children = piece.children.iter().map(|child| child.id).collect();
            self.pieces = vec![id];
        }
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
        self.clear_children();
        self.selected_part = None;
    }

    /// Makes an already selected piece the primary one, keeping the rest of the selection.
    pub(super) fn focus(&mut self, piece: u64) {
        self.selected = Some(piece);
        self.clear_children();
        self.selected_part = None;
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

    /// Selects the edge or vertex along with its piece, and its child if it belongs to one.
    pub(super) fn select_part(&mut self, piece: u64, shape: ShapeRef, part: ShapePart) {
        let child = match shape {
            ShapeRef::Base => None,
            ShapeRef::Child(child) => Some(child),
        };
        self.select(Some(piece), child);
        self.selected_part = Some((piece, shape, part));
    }

    /// The selected edge or vertex, if its shape is the one selected. The index may be out of
    /// range after an undo, so readers check it against the shape.
    pub(super) fn selected_part(&self) -> Option<(ShapeRef, ShapePart)> {
        let (piece, shape, part) = self.selected_part?;
        let shown = match self.selected_child_id() {
            Some(child) => ShapeRef::Child(child),
            None => ShapeRef::Base,
        };
        (self.selected == Some(piece) && shape == shown).then_some((shape, part))
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
