use crate::align::Arrange;
use crate::events::{ProjectChange, ProjectEvent};
use crate::i18n::Message;
use crate::model::Project;

/// The oldest entries are dropped beyond this many.
pub const MAX_HISTORY: usize = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryCommand {
    Undo,
    Redo,
    GoTo(usize),
}

/// Consecutive edits with the same key share one entry, so that typing a name
/// or dragging a value doesn't add an entry per frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MergeKey {
    Rename(u64),
    Material(u64),
    Base(u64),
    Child(u64, u64),
}

#[derive(Clone, Debug)]
struct Entry {
    action: Message,
    merge: Option<MergeKey>,
    /// The project as it was right after this action.
    snapshot: Project,
}

/// The edits of the current document, kept as snapshots so that any of them can be restored.
#[derive(Clone, Debug)]
pub struct History {
    entries: Vec<Entry>,
    position: usize,
    /// The entry that matches the file on disk, if it is still in the history.
    saved: Option<usize>,
    /// Cleared by saving and by moving through the history, so that the next edit gets its own entry.
    mergeable: bool,
}

impl Default for History {
    fn default() -> Self {
        Self::new(&Project::default(), Message::ActionNewProject)
    }
}

impl History {
    pub fn new(project: &Project, action: Message) -> Self {
        Self {
            entries: vec![Entry {
                action,
                merge: None,
                snapshot: project.clone(),
            }],
            position: 0,
            saved: Some(0),
            mergeable: false,
        }
    }

    /// Records an event that `Project::apply` accepted; `project` is the result.
    pub fn record(&mut self, event: &ProjectEvent, change: ProjectChange, project: &Project) {
        let (action, merge) = match change {
            ProjectChange::Created => {
                *self = Self::new(project, Message::ActionNewProject);
                return;
            }
            ProjectChange::Opened => {
                let name = project
                    .path
                    .as_ref()
                    .and_then(|path| path.file_name())
                    .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
                *self = Self::new(project, Message::ActionOpened { name });
                return;
            }
            ProjectChange::Saved => {
                self.saved = Some(self.position);
                self.mergeable = false;
                return;
            }
            _ => self.describe(event),
        };

        self.entries.truncate(self.position + 1);
        if self.saved.is_some_and(|saved| saved > self.position) {
            self.saved = None;
        }
        let top = &mut self.entries[self.position];
        if self.mergeable && merge.is_some() && top.merge == merge {
            top.action = action;
            top.snapshot = project.clone();
            return;
        }
        self.entries.push(Entry {
            action,
            merge,
            snapshot: project.clone(),
        });
        self.position += 1;
        self.mergeable = true;
        if self.entries.len() > MAX_HISTORY {
            self.entries.remove(0);
            self.position -= 1;
            self.saved = self.saved.and_then(|saved| saved.checked_sub(1));
        }
    }

    /// Restores an entry into `project`, returning the status to show, or `None` if nothing moved.
    pub fn apply(&mut self, command: HistoryCommand, project: &mut Project) -> Option<Message> {
        let target = match command {
            HistoryCommand::Undo => self.position.checked_sub(1)?,
            HistoryCommand::Redo => self.position + 1,
            HistoryCommand::GoTo(index) => index,
        };
        if target == self.position || target >= self.entries.len() {
            return None;
        }
        let status = match command {
            HistoryCommand::Undo => Message::Undone {
                action: Box::new(self.entries[self.position].action.clone()),
            },
            HistoryCommand::Redo => Message::Redone {
                action: Box::new(self.entries[target].action.clone()),
            },
            HistoryCommand::GoTo(_) => Message::HistoryRestored {
                action: Box::new(self.entries[target].action.clone()),
            },
        };
        self.position = target;
        self.mergeable = false;

        let path = project.path.take();
        *project = self.entries[target].snapshot.clone();
        project.path = path;
        project.modified = self.saved != Some(target);
        Some(status)
    }

    pub fn actions(&self) -> impl Iterator<Item = &Message> {
        self.entries.iter().map(|entry| &entry.action)
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    pub fn can_redo(&self) -> bool {
        self.position + 1 < self.entries.len()
    }

    /// Names come from the project before the event, so a removed piece is still found.
    fn describe(&self, event: &ProjectEvent) -> (Message, Option<MergeKey>) {
        let before = &self.entries[self.position].snapshot;
        let name = |id: u64| {
            before
                .pieces
                .iter()
                .find(|piece| piece.id == id)
                .map_or_else(|| id.to_string(), |piece| piece.name.clone())
        };
        match event {
            ProjectEvent::SetUnits(_) => (Message::ActionUnits, None),
            ProjectEvent::AddPiece { name, .. } => {
                (Message::ActionAddPiece { name: name.clone() }, None)
            }
            ProjectEvent::RenamePiece { id, name } => (
                Message::ActionRenamePiece { name: name.clone() },
                Some(MergeKey::Rename(*id)),
            ),
            ProjectEvent::SetBaseShape { id, .. } => (
                Message::ActionEditBase { name: name(*id) },
                Some(MergeKey::Base(*id)),
            ),
            ProjectEvent::MovePiece { id, .. } => {
                (Message::ActionMovePiece { name: name(*id) }, None)
            }
            ProjectEvent::MovePieces { ids, .. } => {
                (Message::ActionMovePieces { count: ids.len() }, None)
            }
            ProjectEvent::ArrangePieces {
                ids,
                arrange: Arrange::Align(..),
                ..
            } => (Message::ActionAlignPieces { count: ids.len() }, None),
            ProjectEvent::ArrangePieces { ids, .. } => {
                (Message::ActionDistributePieces { count: ids.len() }, None)
            }
            ProjectEvent::SetPieceLocked { id, locked: true } => {
                (Message::ActionLockPiece { name: name(*id) }, None)
            }
            ProjectEvent::SetPieceLocked { id, locked: false } => {
                (Message::ActionUnlockPiece { name: name(*id) }, None)
            }
            ProjectEvent::SetPieceMaterial { id, .. } => (
                Message::ActionMaterial { name: name(*id) },
                Some(MergeKey::Material(*id)),
            ),
            ProjectEvent::RemovePiece { id } => {
                (Message::ActionRemovePiece { name: name(*id) }, None)
            }
            ProjectEvent::RemovePieces { ids } => {
                (Message::ActionRemovePieces { count: ids.len() }, None)
            }
            ProjectEvent::AddChild { id, .. } => {
                (Message::ActionAddChild { name: name(*id) }, None)
            }
            ProjectEvent::SetChildShape { id, child, .. } => (
                Message::ActionEditChild { name: name(*id) },
                Some(MergeKey::Child(*id, *child)),
            ),
            ProjectEvent::SetChildOp { id, .. } => {
                (Message::ActionSetChildOp { name: name(*id) }, None)
            }
            ProjectEvent::ReorderChild { id, .. } => {
                (Message::ActionReorderChild { name: name(*id) }, None)
            }
            ProjectEvent::RemoveChild { id, .. } => {
                (Message::ActionRemoveChild { name: name(*id) }, None)
            }
            ProjectEvent::MoveChildren { id, children, .. } => (
                Message::ActionMoveChildren {
                    name: name(*id),
                    count: children.len(),
                },
                None,
            ),
            ProjectEvent::ArrangeChildren {
                id,
                children,
                arrange: Arrange::Align(..),
                ..
            } => (
                Message::ActionAlignChildren {
                    name: name(*id),
                    count: children.len(),
                },
                None,
            ),
            ProjectEvent::ArrangeChildren { id, children, .. } => (
                Message::ActionDistributeChildren {
                    name: name(*id),
                    count: children.len(),
                },
                None,
            ),
            ProjectEvent::RemoveChildren { id, children } => (
                Message::ActionRemoveChildren {
                    name: name(*id),
                    count: children.len(),
                },
                None,
            ),
            // These are handled by their `ProjectChange` before describing.
            ProjectEvent::New { .. } | ProjectEvent::Open { .. } | ProjectEvent::Save { .. } => {
                (Message::ActionNewProject, None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::align::{AlignTo, Axis, Edge, Spacing};
    use crate::model::{BooleanOp, Point};
    use crate::shapes::{Polygon, Shape};
    use std::path::PathBuf;

    fn triangle() -> Shape {
        Shape::Polygon(Polygon {
            points: vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(0.0, 10.0),
            ],
        })
    }

    fn add_piece(name: &str) -> ProjectEvent {
        ProjectEvent::AddPiece {
            name: name.to_owned(),
            base: triangle(),
        }
    }

    fn rename(id: u64, name: &str) -> ProjectEvent {
        ProjectEvent::RenamePiece {
            id,
            name: name.to_owned(),
        }
    }

    /// Applies an event and records it, as `apply_project_events` does.
    fn run(project: &mut Project, history: &mut History, event: ProjectEvent) {
        let change = project.apply(event.clone()).unwrap();
        history.record(&event, change, project);
    }

    /// Marks the project saved without touching the disk.
    fn mark_saved(project: &mut Project, history: &mut History) {
        project.modified = false;
        history.record(
            &ProjectEvent::Save {
                path: PathBuf::new(),
            },
            ProjectChange::Saved,
            project,
        );
    }

    fn actions(history: &History) -> Vec<Message> {
        history.actions().cloned().collect()
    }

    #[test]
    fn undo_and_redo_restore_the_snapshots() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        let with_piece = project.clone();

        assert!(history.can_undo());
        assert_eq!(
            history.apply(HistoryCommand::Undo, &mut project),
            Some(Message::Undone {
                action: Box::new(Message::ActionAddPiece {
                    name: "Flap".to_owned()
                })
            })
        );
        assert!(project.pieces.is_empty());
        assert!(!history.can_undo());
        assert_eq!(history.apply(HistoryCommand::Undo, &mut project), None);

        assert!(history.can_redo());
        assert!(history.apply(HistoryCommand::Redo, &mut project).is_some());
        assert_eq!(project, with_piece);
        assert_eq!(history.apply(HistoryCommand::Redo, &mut project), None);
    }

    #[test]
    fn a_new_edit_after_undo_drops_the_redo_branch() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        run(&mut project, &mut history, add_piece("Strap"));
        history.apply(HistoryCommand::Undo, &mut project);

        run(&mut project, &mut history, add_piece("Gusset"));
        assert!(!history.can_redo());
        assert_eq!(
            actions(&history),
            vec![
                Message::ActionNewProject,
                Message::ActionAddPiece {
                    name: "Flap".to_owned()
                },
                Message::ActionAddPiece {
                    name: "Gusset".to_owned()
                },
            ]
        );
    }

    #[test]
    fn consecutive_renames_share_an_entry_until_a_save() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        run(&mut project, &mut history, rename(1, "F"));
        run(&mut project, &mut history, rename(1, "Fr"));
        run(&mut project, &mut history, rename(1, "Front"));
        assert_eq!(history.actions().count(), 3);
        assert_eq!(
            history.actions().last(),
            Some(&Message::ActionRenamePiece {
                name: "Front".to_owned()
            })
        );

        mark_saved(&mut project, &mut history);
        run(&mut project, &mut history, rename(1, "Front 2"));
        assert_eq!(history.actions().count(), 4);

        history.apply(HistoryCommand::Undo, &mut project);
        assert_eq!(project.pieces[0].name, "Front");
        history.apply(HistoryCommand::Undo, &mut project);
        assert_eq!(project.pieces[0].name, "Flap");
    }

    #[test]
    fn modified_follows_the_saved_entry() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        mark_saved(&mut project, &mut history);
        run(&mut project, &mut history, add_piece("Strap"));
        assert!(project.modified);

        history.apply(HistoryCommand::Undo, &mut project);
        assert!(!project.modified);
        history.apply(HistoryCommand::Undo, &mut project);
        assert!(project.modified);
        history.apply(HistoryCommand::Redo, &mut project);
        assert!(!project.modified);

        // Branching from an earlier entry loses the saved one.
        history.apply(HistoryCommand::Undo, &mut project);
        run(&mut project, &mut history, add_piece("Gusset"));
        history.apply(HistoryCommand::Undo, &mut project);
        assert!(project.modified);
        history.apply(HistoryCommand::Redo, &mut project);
        assert!(project.modified);
    }

    #[test]
    fn restoring_keeps_the_document_path() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        project.path = Some(PathBuf::from("bag.leathercraft"));
        history.apply(HistoryCommand::Undo, &mut project);
        assert_eq!(project.path, Some(PathBuf::from("bag.leathercraft")));
    }

    #[test]
    fn the_oldest_entries_are_dropped_beyond_the_limit() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        for index in 0..MAX_HISTORY {
            let locked = index % 2 == 0;
            run(
                &mut project,
                &mut history,
                ProjectEvent::SetPieceLocked { id: 1, locked },
            );
        }
        assert_eq!(history.actions().count(), MAX_HISTORY);
        assert_eq!(history.position(), MAX_HISTORY - 1);
        assert_ne!(history.actions().next(), Some(&Message::ActionNewProject));

        history.apply(HistoryCommand::GoTo(0), &mut project);
        assert!(!history.can_undo());
        assert!(project.modified);
    }

    #[test]
    fn go_to_restores_any_entry() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        run(&mut project, &mut history, add_piece("Strap"));
        run(&mut project, &mut history, add_piece("Gusset"));

        assert!(
            history
                .apply(HistoryCommand::GoTo(1), &mut project)
                .is_some()
        );
        assert_eq!(project.pieces.len(), 1);
        assert_eq!(history.position(), 1);
        assert_eq!(history.apply(HistoryCommand::GoTo(1), &mut project), None);
        assert_eq!(history.apply(HistoryCommand::GoTo(9), &mut project), None);

        history.apply(HistoryCommand::GoTo(3), &mut project);
        assert_eq!(project.pieces.len(), 3);
    }

    #[test]
    fn a_removed_piece_is_named_in_its_entry() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        run(
            &mut project,
            &mut history,
            ProjectEvent::RemovePiece { id: 1 },
        );
        assert_eq!(
            history.actions().last(),
            Some(&Message::ActionRemovePiece {
                name: "Flap".to_owned()
            })
        );
    }

    #[test]
    fn a_group_edit_is_one_entry_counting_its_pieces() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        run(&mut project, &mut history, add_piece("Strap"));
        run(
            &mut project,
            &mut history,
            ProjectEvent::MovePieces {
                ids: vec![1, 2],
                dx: 1.0,
                dy: 1.0,
            },
        );
        run(
            &mut project,
            &mut history,
            ProjectEvent::RemovePieces { ids: vec![1, 2] },
        );
        let actions: Vec<&Message> = history.actions().skip(3).collect();
        assert_eq!(
            actions,
            [
                &Message::ActionMovePieces { count: 2 },
                &Message::ActionRemovePieces { count: 2 }
            ]
        );
        history.apply(HistoryCommand::Undo, &mut project);
        assert_eq!(project.pieces.len(), 2);
    }

    #[test]
    fn a_group_child_edit_is_one_entry_counting_its_shapes() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        for _ in 0..2 {
            let shape =
                Shape::rectangle_from_corners(Point::new(1.0, 1.0), Point::new(2.0, 2.0)).unwrap();
            run(
                &mut project,
                &mut history,
                ProjectEvent::AddChild {
                    id: 1,
                    op: BooleanOp::Subtract,
                    shape,
                },
            );
        }
        run(
            &mut project,
            &mut history,
            ProjectEvent::MoveChildren {
                id: 1,
                children: vec![1, 2],
                dx: 1.0,
                dy: 0.0,
            },
        );
        for arrange in [
            Arrange::Align(Axis::Horizontal, Edge::Min),
            Arrange::Distribute(Axis::Horizontal, Edge::Min),
        ] {
            run(
                &mut project,
                &mut history,
                ProjectEvent::ArrangeChildren {
                    id: 1,
                    children: vec![1, 2],
                    arrange,
                    to: AlignTo::Parent,
                    spacing: Spacing::Auto,
                },
            );
        }
        run(
            &mut project,
            &mut history,
            ProjectEvent::RemoveChildren {
                id: 1,
                children: vec![1, 2],
            },
        );
        let name = || "Flap".to_owned();
        let actions: Vec<&Message> = history.actions().skip(4).collect();
        assert_eq!(
            actions,
            [
                &Message::ActionMoveChildren {
                    name: name(),
                    count: 2
                },
                &Message::ActionAlignChildren {
                    name: name(),
                    count: 2
                },
                &Message::ActionDistributeChildren {
                    name: name(),
                    count: 2
                },
                &Message::ActionRemoveChildren {
                    name: name(),
                    count: 2
                }
            ]
        );
        history.apply(HistoryCommand::Undo, &mut project);
        assert_eq!(project.pieces[0].children.len(), 2);
    }

    #[test]
    fn arranging_is_named_by_its_kind() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        run(&mut project, &mut history, add_piece("Strap"));
        run(
            &mut project,
            &mut history,
            ProjectEvent::MovePiece {
                id: 2,
                dx: 30.0,
                dy: 10.0,
            },
        );
        let arrange = |arrange| ProjectEvent::ArrangePieces {
            ids: vec![1, 2],
            arrange,
            to: AlignTo::Selection,
            spacing: Spacing::Auto,
        };
        run(
            &mut project,
            &mut history,
            arrange(Arrange::Align(Axis::Vertical, Edge::Min)),
        );
        run(
            &mut project,
            &mut history,
            ProjectEvent::AddPiece {
                name: "Pocket".to_owned(),
                base: triangle(),
            },
        );
        run(
            &mut project,
            &mut history,
            ProjectEvent::ArrangePieces {
                ids: vec![1, 2, 3],
                arrange: Arrange::Distribute(Axis::Horizontal, Edge::Center),
                to: AlignTo::Selection,
                spacing: Spacing::Auto,
            },
        );
        let actions: Vec<&Message> = history.actions().skip(4).collect();
        assert_eq!(
            actions,
            [
                &Message::ActionAlignPieces { count: 2 },
                &Message::ActionAddPiece {
                    name: "Pocket".to_owned()
                },
                &Message::ActionDistributePieces { count: 3 }
            ]
        );
    }

    #[test]
    fn a_new_document_starts_a_new_history() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        run(
            &mut project,
            &mut history,
            ProjectEvent::New {
                material: Default::default(),
            },
        );
        assert_eq!(actions(&history), vec![Message::ActionNewProject]);
        assert!(!history.can_undo());
    }

    #[test]
    fn consecutive_edits_of_one_child_share_an_entry() {
        let mut project = Project::default();
        let mut history = History::default();
        run(&mut project, &mut history, add_piece("Flap"));
        run(
            &mut project,
            &mut history,
            ProjectEvent::AddChild {
                id: 1,
                op: BooleanOp::Subtract,
                shape: Shape::rectangle_from_corners(Point::new(1.0, 1.0), Point::new(2.0, 2.0))
                    .unwrap(),
            },
        );
        for size in [2.0, 3.0, 4.0] {
            let shape =
                Shape::rectangle_from_corners(Point::new(1.0, 1.0), Point::new(size, 2.0)).unwrap();
            run(
                &mut project,
                &mut history,
                ProjectEvent::SetChildShape {
                    id: 1,
                    child: 1,
                    shape,
                },
            );
        }
        assert_eq!(history.actions().count(), 4);
        assert_eq!(
            history.actions().last(),
            Some(&Message::ActionEditChild {
                name: "Flap".to_owned()
            })
        );
        history.apply(HistoryCommand::Undo, &mut project);
        assert_eq!(
            project.pieces[0].children[0]
                .shape
                .bounds()
                .map(|b| b.max_x),
            Some(2.0)
        );
    }
}
