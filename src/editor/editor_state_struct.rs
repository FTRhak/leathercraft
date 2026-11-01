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
    pub(super) selected: Option<u64>,
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
            selected: None,
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

    pub(super) fn displayed<'a>(&self, piece: &'a PatternPiece) -> Cow<'a, PatternPiece> {
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
