use super::{DragState, ShapeDraft};
use crate::panels::tools::Tool;
use bevy::prelude::*;
use bevy_egui::egui;
use leathercraft::events::ProjectChange;
use leathercraft::i18n::{Language, Message};
use leathercraft::material::Material;
use leathercraft::model::{PatternPiece, Point, Project};
use std::borrow::Cow;

#[derive(Resource)]
pub struct EditorState {
    pub(super) tool: Tool,
    pub(super) draft: Vec<Point>,
    pub(super) shape_draft: Option<ShapeDraft>,
    /// Draw, Rectangle and Circle cut a hole in the selected piece instead of adding a piece.
    pub(super) hole_mode: bool,
    pub(super) selected: Option<u64>,
    /// Piece id and hole index. Read through `selected_hole_index`, since it only counts
    /// while that piece is the selected one.
    pub(super) selected_hole: Option<(u64, usize)>,
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
            hole_mode: false,
            selected: None,
            selected_hole: None,
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
        self.selected_hole = None;
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
                // The id can be given to the next new piece.
                self.selected_hole = None;
                if self.drag.as_ref().is_some_and(|drag| drag.piece_id == id) {
                    self.drag = None;
                }
                self.report(Message::PieceDeleted);
            }
            // The tool and hole mode stay, so that several holes can be cut in a row.
            ProjectChange::HoleAdded { id, index } => {
                self.draft.clear();
                self.selected_hole = Some((id, index));
                if let Some(piece) = project.pieces.iter().find(|piece| piece.id == id) {
                    self.report(Message::HoleAdded {
                        name: piece.name.clone(),
                    });
                }
            }
            ProjectChange::HoleRemoved { id } => {
                if self.selected_hole.is_some_and(|(piece, _)| piece == id) {
                    self.selected_hole = None;
                }
                self.report(Message::HoleDeleted);
            }
        }
    }

    pub(super) fn selected_hole_index(&self) -> Option<usize> {
        self.selected_hole
            .filter(|(id, _)| self.selected == Some(*id))
            .map(|(_, index)| index)
    }

    pub fn clear_released_drag(&mut self) {
        if self.drag.as_ref().is_some_and(|drag| drag.released) {
            self.drag = None;
        }
    }

    pub(super) fn displayed<'a>(&self, piece: &'a PatternPiece) -> Cow<'a, PatternPiece> {
        match &self.drag {
            Some(drag) if drag.piece_id == piece.id => Cow::Owned(PatternPiece {
                points: drag.points.clone(),
                holes: drag.holes.clone(),
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
