use crate::align::{self, AlignTo, Arrange, Spacing};
use crate::i18n::Message;
use crate::material::Material;
use crate::model::{BooleanOp, ChildShape, PatternPiece, Project, Units};
use crate::persistence;
use crate::shapes::Shape;
use std::path::PathBuf;

/// A requested change to the project. `Project::apply` is the only intended way to change one.
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectEvent {
    New {
        material: Material,
    },
    Open {
        path: PathBuf,
    },
    Save {
        path: PathBuf,
    },
    SetUnits(Units),
    AddPiece {
        name: String,
        base: Shape,
    },
    RenamePiece {
        id: u64,
        name: String,
    },
    SetBaseShape {
        id: u64,
        shape: Shape,
    },
    /// Moves the base and every child together.
    MovePiece {
        id: u64,
        dx: f64,
        dy: f64,
    },
    /// Moves several pieces by the same offset, as one edit.
    MovePieces {
        ids: Vec<u64>,
        dx: f64,
        dy: f64,
    },
    /// Aligns or distributes several pieces, as one edit. Locked pieces and the key object stay.
    ArrangePieces {
        ids: Vec<u64>,
        arrange: Arrange,
        to: AlignTo,
        spacing: Spacing,
    },
    SetPieceLocked {
        id: u64,
        locked: bool,
    },
    SetPieceMaterial {
        id: u64,
        material: Material,
    },
    RemovePiece {
        id: u64,
    },
    /// Removes several pieces as one edit.
    RemovePieces {
        ids: Vec<u64>,
    },
    AddChild {
        id: u64,
        op: BooleanOp,
        shape: Shape,
    },
    SetChildShape {
        id: u64,
        child: u64,
        shape: Shape,
    },
    SetChildOp {
        id: u64,
        child: u64,
        op: BooleanOp,
    },
    /// Moves a child to `index` in the list, which changes the order its operation applies in.
    ReorderChild {
        id: u64,
        child: u64,
        index: usize,
    },
    RemoveChild {
        id: u64,
        child: u64,
    },
    /// Moves several children of one piece by the same offset, as one edit.
    MoveChildren {
        id: u64,
        children: Vec<u64>,
        dx: f64,
        dy: f64,
    },
    /// Aligns or distributes several children of one piece, as one edit. The key object stays.
    ArrangeChildren {
        id: u64,
        children: Vec<u64>,
        arrange: Arrange,
        to: AlignTo,
        spacing: Spacing,
    },
    /// Removes several children of one piece as one edit.
    RemoveChildren {
        id: u64,
        children: Vec<u64>,
    },
}

/// What an applied event did, so that listeners can react without inspecting the project.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectChange {
    Created,
    Opened,
    Saved,
    Edited,
    PieceAdded(u64),
    PieceRemoved(u64),
    PiecesRemoved,
    ChildAdded { id: u64, child: u64 },
    ChildRemoved { id: u64 },
    ChildrenRemoved { id: u64, count: usize },
}

impl Project {
    /// Applies an event atomically: on error the project is left exactly as it was.
    pub fn apply(&mut self, event: ProjectEvent) -> Result<ProjectChange, Message> {
        match event {
            ProjectEvent::New { material } => {
                let created = Project {
                    default_material: material,
                    ..Project::default()
                };
                created.validate()?;
                *self = created;
                Ok(ProjectChange::Created)
            }
            ProjectEvent::Open { path } => {
                let mut loaded = persistence::load_project(&path)?;
                loaded.path = Some(path);
                *self = loaded;
                Ok(ProjectChange::Opened)
            }
            ProjectEvent::Save { path } => {
                persistence::save_project(&path, self)?;
                self.path = Some(path);
                self.modified = false;
                Ok(ProjectChange::Saved)
            }
            ProjectEvent::SetUnits(units) => self.edit(|project| {
                project.units = units;
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::AddPiece { name, base } => self.edit(|project| {
                let id = project.next_piece_id();
                let mut piece = PatternPiece::new(id, name, base);
                piece.material = project.default_material;
                project.add_piece(piece);
                Ok(ProjectChange::PieceAdded(id))
            }),
            ProjectEvent::RenamePiece { id, name } => self.edit(|project| {
                project.piece_mut(id)?.name = name;
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::SetBaseShape { id, shape } => self.edit(|project| {
                project.unlocked_piece_mut(id)?.base = shape;
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::MovePiece { id, dx, dy } => self.edit(|project| {
                project.unlocked_piece_mut(id)?.translate(dx, dy);
                Ok(ProjectChange::Edited)
            }),
            // Each id is checked first, so a repeated id still moves its piece only once.
            ProjectEvent::MovePieces { ids, dx, dy } => self.edit(|project| {
                for id in &ids {
                    project.unlocked_piece_mut(*id)?;
                }
                for piece in &mut project.pieces {
                    if ids.contains(&piece.id) {
                        piece.translate(dx, dy);
                    }
                }
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::ArrangePieces {
                ids,
                arrange,
                to,
                spacing,
            } => self.edit(|project| {
                for id in &ids {
                    project.piece_mut(*id)?;
                }
                let offsets = align::offsets(project, &ids, arrange, to, spacing);
                // Rejected rather than recorded, so that the history gets no empty entry.
                if offsets.is_empty() {
                    return Err(Message::AlreadyArranged);
                }
                for (id, dx, dy) in offsets {
                    project.piece_mut(id)?.translate(dx, dy);
                }
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::SetPieceLocked { id, locked } => self.edit(|project| {
                project.piece_mut(id)?.locked = locked;
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::SetPieceMaterial { id, material } => self.edit(|project| {
                project.piece_mut(id)?.material = material;
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::RemovePiece { id } => self.edit(|project| {
                project.unlocked_piece_mut(id)?;
                project.remove_piece(id);
                Ok(ProjectChange::PieceRemoved(id))
            }),
            ProjectEvent::RemovePieces { ids } => self.edit(|project| {
                for id in &ids {
                    project.unlocked_piece_mut(*id)?;
                }
                for id in &ids {
                    project.remove_piece(*id);
                }
                Ok(ProjectChange::PiecesRemoved)
            }),
            ProjectEvent::AddChild { id, op, shape } => self.edit(|project| {
                let piece = project.unlocked_piece_mut(id)?;
                let child = piece.next_child_id();
                piece.children.push(ChildShape {
                    id: child,
                    op,
                    shape,
                });
                Ok(ProjectChange::ChildAdded { id, child })
            }),
            ProjectEvent::SetChildShape { id, child, shape } => self.edit(|project| {
                project.child_mut(id, child)?.shape = shape;
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::SetChildOp { id, child, op } => self.edit(|project| {
                project.child_mut(id, child)?.op = op;
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::ReorderChild { id, child, index } => self.edit(|project| {
                let piece = project.unlocked_piece_mut(id)?;
                let from = child_index(piece, child)?;
                let moved = piece.children.remove(from);
                piece
                    .children
                    .insert(index.min(piece.children.len()), moved);
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::RemoveChild { id, child } => self.edit(|project| {
                let piece = project.unlocked_piece_mut(id)?;
                let index = child_index(piece, child)?;
                piece.children.remove(index);
                Ok(ProjectChange::ChildRemoved { id })
            }),
            // Each id is checked first, so a repeated id still moves its child only once.
            ProjectEvent::MoveChildren {
                id,
                children,
                dx,
                dy,
            } => self.edit(|project| {
                let piece = project.unlocked_piece_mut(id)?;
                for child in &children {
                    child_index(piece, *child)?;
                }
                for child in &mut piece.children {
                    if children.contains(&child.id) {
                        child.shape.translate(dx, dy);
                    }
                }
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::ArrangeChildren {
                id,
                children,
                arrange,
                to,
                spacing,
            } => self.edit(|project| {
                let piece = project.unlocked_piece_mut(id)?;
                for child in &children {
                    child_index(piece, *child)?;
                }
                let offsets = align::child_offsets(project, id, &children, arrange, to, spacing);
                // Rejected rather than recorded, so that the history gets no empty entry.
                if offsets.is_empty() {
                    return Err(Message::ShapesAlreadyArranged);
                }
                for (child, dx, dy) in offsets {
                    project.child_mut(id, child)?.shape.translate(dx, dy);
                }
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::RemoveChildren { id, children } => self.edit(|project| {
                let piece = project.unlocked_piece_mut(id)?;
                for child in &children {
                    child_index(piece, *child)?;
                }
                let before = piece.children.len();
                piece.children.retain(|child| !children.contains(&child.id));
                let count = before - piece.children.len();
                Ok(ProjectChange::ChildrenRemoved { id, count })
            }),
        }
    }

    fn edit(
        &mut self,
        change: impl FnOnce(&mut Project) -> Result<ProjectChange, Message>,
    ) -> Result<ProjectChange, Message> {
        let mut candidate = self.clone();
        let result = change(&mut candidate)?;
        candidate.validate()?;
        candidate.modified = true;
        *self = candidate;
        Ok(result)
    }

    fn piece_mut(&mut self, id: u64) -> Result<&mut PatternPiece, Message> {
        self.pieces
            .iter_mut()
            .find(|piece| piece.id == id)
            .ok_or(Message::PieceNotFound { id })
    }

    fn unlocked_piece_mut(&mut self, id: u64) -> Result<&mut PatternPiece, Message> {
        let piece = self.piece_mut(id)?;
        if piece.locked {
            return Err(Message::PieceLocked {
                name: piece.name.clone(),
            });
        }
        Ok(piece)
    }

    fn child_mut(&mut self, id: u64, child: u64) -> Result<&mut ChildShape, Message> {
        let piece = self.unlocked_piece_mut(id)?;
        let index = child_index(piece, child)?;
        Ok(&mut piece.children[index])
    }
}

fn child_index(piece: &PatternPiece, child: u64) -> Result<usize, Message> {
    piece
        .child_index(child)
        .ok_or_else(|| Message::ChildNotFound {
            name: piece.name.clone(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{LeatherColor, Origin};
    use crate::model::Point;
    use crate::shapes::{Polygon, Rectangle};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn triangle() -> Shape {
        Shape::Polygon(Polygon {
            points: vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(0.0, 10.0),
            ],
            ..Default::default()
        })
    }

    fn small_square() -> Shape {
        Shape::rectangle_from_corners(Point::new(1.0, 1.0), Point::new(3.0, 3.0)).unwrap()
    }

    fn add_piece() -> ProjectEvent {
        ProjectEvent::AddPiece {
            name: "Деталь 1".to_owned(),
            base: triangle(),
        }
    }

    fn add_child(op: BooleanOp) -> ProjectEvent {
        ProjectEvent::AddChild {
            id: 1,
            op,
            shape: small_square(),
        }
    }

    fn project_with_piece() -> Project {
        let mut project = Project::default();
        project.apply(add_piece()).unwrap();
        project
    }

    fn locked() -> Result<ProjectChange, Message> {
        Err(Message::PieceLocked {
            name: "Деталь 1".to_owned(),
        })
    }

    #[test]
    fn edits_mark_the_project_as_modified() {
        let mut project = Project::default();
        assert_eq!(project.apply(add_piece()), Ok(ProjectChange::PieceAdded(1)));
        assert_eq!(project.pieces[0].name, "Деталь 1");
        assert!(project.modified);
    }

    #[test]
    fn rejected_events_leave_the_project_unchanged() {
        let mut project = project_with_piece();
        project.apply(add_child(BooleanOp::Subtract)).unwrap();
        project.modified = false;
        let before = project.clone();

        let invalid = [
            ProjectEvent::AddPiece {
                name: "Flap".to_owned(),
                base: Shape::Polygon(Polygon::default()),
            },
            ProjectEvent::AddPiece {
                name: " ".to_owned(),
                base: triangle(),
            },
            ProjectEvent::RenamePiece {
                id: 1,
                name: " ".to_owned(),
            },
            ProjectEvent::SetBaseShape {
                id: 1,
                shape: Shape::Polygon(Polygon {
                    points: vec![Point::new(0.0, 0.0); 3],
                    radii: Vec::new(),
                }),
            },
            ProjectEvent::SetBaseShape {
                id: 9,
                shape: triangle(),
            },
            ProjectEvent::MovePiece {
                id: 1,
                dx: f64::NAN,
                dy: 0.0,
            },
            ProjectEvent::RemovePiece { id: 9 },
            ProjectEvent::AddChild {
                id: 9,
                op: BooleanOp::Union,
                shape: small_square(),
            },
            ProjectEvent::SetChildShape {
                id: 1,
                child: 1,
                shape: Shape::Rectangle(Rectangle {
                    center: Point::new(2.0, 2.0),
                    width: -1.0,
                    height: 2.0,
                    rotation: 0.0,
                    radii: Vec::new(),
                }),
            },
            ProjectEvent::SetChildOp {
                id: 1,
                child: 9,
                op: BooleanOp::Union,
            },
            ProjectEvent::RemoveChild { id: 1, child: 9 },
        ];
        for event in invalid {
            assert!(project.apply(event.clone()).is_err(), "{event:?}");
            assert_eq!(project, before);
        }
    }

    #[test]
    fn corners_can_be_rounded_and_made_sharp_again() {
        let mut project = project_with_piece();
        let rounded = triangle().with_corner_radius(0, 2.0).unwrap();
        project
            .apply(ProjectEvent::SetBaseShape {
                id: 1,
                shape: rounded.clone(),
            })
            .unwrap();
        assert_eq!(project.pieces[0].base, rounded);
        assert!(project.pieces[0].region().is_ok());

        let mut invalid = triangle();
        if let Shape::Polygon(polygon) = &mut invalid {
            polygon.radii = vec![-1.0];
        }
        let before = project.clone();
        assert!(
            project
                .apply(ProjectEvent::SetBaseShape {
                    id: 1,
                    shape: invalid,
                })
                .is_err()
        );
        assert_eq!(project, before);

        project
            .apply(ProjectEvent::SetBaseShape {
                id: 1,
                shape: triangle(),
            })
            .unwrap();
        assert_eq!(project.pieces[0].base, triangle());
    }

    #[test]
    fn edits_and_removal_reach_the_named_piece() {
        let mut project = project_with_piece();
        project
            .apply(ProjectEvent::RenamePiece {
                id: 1,
                name: "Flap".to_owned(),
            })
            .unwrap();
        assert_eq!(project.pieces[0].name, "Flap");

        assert_eq!(
            project.apply(ProjectEvent::RemovePiece { id: 1 }),
            Ok(ProjectChange::PieceRemoved(1))
        );
        assert!(project.pieces.is_empty());
    }

    #[test]
    fn children_can_be_added_edited_reordered_and_removed() {
        let mut project = project_with_piece();
        assert_eq!(
            project.apply(add_child(BooleanOp::Subtract)),
            Ok(ProjectChange::ChildAdded { id: 1, child: 1 })
        );
        assert_eq!(
            project.apply(add_child(BooleanOp::Union)),
            Ok(ProjectChange::ChildAdded { id: 1, child: 2 })
        );

        let mut moved = small_square();
        moved.translate(1.0, 0.0);
        project
            .apply(ProjectEvent::SetChildShape {
                id: 1,
                child: 1,
                shape: moved.clone(),
            })
            .unwrap();
        project
            .apply(ProjectEvent::SetChildOp {
                id: 1,
                child: 1,
                op: BooleanOp::Intersect,
            })
            .unwrap();
        let first = &project.pieces[0].children[0];
        assert_eq!(
            (first.shape.clone(), first.op),
            (moved, BooleanOp::Intersect)
        );

        project
            .apply(ProjectEvent::ReorderChild {
                id: 1,
                child: 2,
                index: 0,
            })
            .unwrap();
        let order: Vec<u64> = project.pieces[0]
            .children
            .iter()
            .map(|child| child.id)
            .collect();
        assert_eq!(order, vec![2, 1]);

        assert_eq!(
            project.apply(ProjectEvent::RemoveChild { id: 1, child: 2 }),
            Ok(ProjectChange::ChildRemoved { id: 1 })
        );
        assert_eq!(project.pieces[0].children.len(), 1);
        assert_eq!(project.pieces[0].next_child_id(), 2);
    }

    #[test]
    fn moving_a_piece_moves_its_children() {
        let mut project = project_with_piece();
        project.apply(add_child(BooleanOp::Subtract)).unwrap();
        project
            .apply(ProjectEvent::MovePiece {
                id: 1,
                dx: 5.0,
                dy: -2.0,
            })
            .unwrap();
        let piece = &project.pieces[0];
        assert_eq!(piece.base.ring()[0], Point::new(5.0, -2.0));
        assert_eq!(piece.children[0].shape.center(), Point::new(7.0, 0.0));
    }

    fn project_with_three_pieces() -> Project {
        let mut project = Project::default();
        for _ in 0..3 {
            project.apply(add_piece()).unwrap();
        }
        project
    }

    #[test]
    fn group_edits_reach_every_named_piece_once() {
        let mut project = project_with_three_pieces();
        project
            .apply(ProjectEvent::MovePieces {
                ids: vec![1, 3, 3],
                dx: 5.0,
                dy: 0.0,
            })
            .unwrap();
        let first_points: Vec<Point> = project
            .pieces
            .iter()
            .map(|piece| piece.base.ring()[0])
            .collect();
        assert_eq!(
            first_points,
            [
                Point::new(5.0, 0.0),
                Point::new(0.0, 0.0),
                Point::new(5.0, 0.0)
            ]
        );
        assert_eq!(
            project.apply(ProjectEvent::RemovePieces { ids: vec![1, 3] }),
            Ok(ProjectChange::PiecesRemoved)
        );
        let ids: Vec<u64> = project.pieces.iter().map(|piece| piece.id).collect();
        assert_eq!(ids, [2]);
    }

    #[test]
    fn group_edits_with_a_locked_or_missing_piece_change_nothing() {
        let mut project = project_with_three_pieces();
        project
            .apply(ProjectEvent::SetPieceLocked {
                id: 2,
                locked: true,
            })
            .unwrap();
        let before = project.clone();
        assert!(
            project
                .apply(ProjectEvent::MovePieces {
                    ids: vec![1, 2],
                    dx: 5.0,
                    dy: 0.0,
                })
                .is_err()
        );
        assert_eq!(
            project.apply(ProjectEvent::RemovePieces { ids: vec![1, 9] }),
            Err(Message::PieceNotFound { id: 9 })
        );
        assert_eq!(project, before);
    }

    #[test]
    fn group_child_edits_reach_only_the_named_children() {
        let mut project = project_with_piece();
        for op in [BooleanOp::Subtract, BooleanOp::Union, BooleanOp::Subtract] {
            project.apply(add_child(op)).unwrap();
        }
        let first = small_square().ring()[0];
        project
            .apply(ProjectEvent::MoveChildren {
                id: 1,
                children: vec![1, 3, 3],
                dx: 2.0,
                dy: 1.0,
            })
            .unwrap();
        let children = &project.pieces[0].children;
        assert_eq!(
            children[0].shape.ring()[0],
            Point::new(first.x + 2.0, first.y + 1.0)
        );
        assert_eq!(children[1].shape.ring()[0], first);
        assert_eq!(
            children[2].shape.ring()[0],
            Point::new(first.x + 2.0, first.y + 1.0)
        );

        let before = project.clone();
        assert_eq!(
            project.apply(ProjectEvent::MoveChildren {
                id: 1,
                children: vec![1, 9],
                dx: 1.0,
                dy: 0.0,
            }),
            Err(Message::ChildNotFound {
                name: "Деталь 1".to_owned()
            })
        );
        assert!(
            project
                .apply(ProjectEvent::RemoveChildren {
                    id: 1,
                    children: vec![2, 9],
                })
                .is_err()
        );
        assert_eq!(project, before);

        assert_eq!(
            project.apply(ProjectEvent::RemoveChildren {
                id: 1,
                children: vec![1, 3],
            }),
            Ok(ProjectChange::ChildrenRemoved { id: 1, count: 2 })
        );
        let ids: Vec<u64> = project.pieces[0]
            .children
            .iter()
            .map(|child| child.id)
            .collect();
        assert_eq!(ids, vec![2]);
    }

    #[test]
    fn arranging_moves_pieces_or_rejects_a_no_op() {
        let mut project = project_with_three_pieces();
        let align_left = ProjectEvent::ArrangePieces {
            ids: vec![1, 2, 3],
            arrange: Arrange::Align(align::Axis::Horizontal, align::Edge::Min),
            to: AlignTo::Selection,
            spacing: Spacing::Auto,
        };
        let before = project.clone();
        assert_eq!(
            project.apply(align_left.clone()),
            Err(Message::AlreadyArranged)
        );
        assert_eq!(project, before);

        project
            .apply(ProjectEvent::MovePiece {
                id: 3,
                dx: 5.0,
                dy: 2.0,
            })
            .unwrap();
        assert_eq!(project.apply(align_left), Ok(ProjectChange::Edited));
        assert_eq!(project.pieces[2].base.ring()[0], Point::new(0.0, 2.0));
        assert_eq!(
            project.apply(ProjectEvent::ArrangePieces {
                ids: vec![1, 9],
                arrange: Arrange::Space(align::Axis::Vertical),
                to: AlignTo::Project,
                spacing: Spacing::Auto,
            }),
            Err(Message::PieceNotFound { id: 9 })
        );
    }

    #[test]
    fn arranging_children_moves_them_within_their_piece() {
        let mut project = project_with_piece();
        project.apply(add_child(BooleanOp::Subtract)).unwrap();
        let center = ProjectEvent::ArrangeChildren {
            id: 1,
            children: vec![1],
            arrange: Arrange::Align(align::Axis::Horizontal, align::Edge::Center),
            to: AlignTo::Parent,
            spacing: Spacing::Auto,
        };
        assert_eq!(project.apply(center.clone()), Ok(ProjectChange::Edited));
        assert_eq!(
            project.pieces[0].children[0].shape.ring()[0],
            Point::new(4.0, 1.0)
        );

        let before = project.clone();
        assert_eq!(
            project.apply(center.clone()),
            Err(Message::ShapesAlreadyArranged)
        );
        assert_eq!(
            project.apply(ProjectEvent::ArrangeChildren {
                id: 1,
                children: vec![1, 9],
                arrange: Arrange::Align(align::Axis::Vertical, align::Edge::Min),
                to: AlignTo::Parent,
                spacing: Spacing::Auto,
            }),
            Err(Message::ChildNotFound {
                name: "Деталь 1".to_owned()
            })
        );
        assert_eq!(project, before);

        project
            .apply(ProjectEvent::SetPieceLocked {
                id: 1,
                locked: true,
            })
            .unwrap();
        assert_eq!(project.apply(center), locked());
    }

    #[test]
    fn a_child_that_splits_the_piece_is_accepted() {
        let mut project = project_with_piece();
        let band =
            Shape::rectangle_from_corners(Point::new(-1.0, 4.0), Point::new(20.0, 5.0)).unwrap();
        project
            .apply(ProjectEvent::AddChild {
                id: 1,
                op: BooleanOp::Subtract,
                shape: band,
            })
            .unwrap();
        assert_eq!(
            project.pieces[0].region(),
            Err(Message::ResultSplit {
                name: "Деталь 1".to_owned()
            })
        );
    }

    #[test]
    fn locked_pieces_reject_shape_changes_and_removal() {
        let mut project = project_with_piece();
        project.apply(add_child(BooleanOp::Subtract)).unwrap();
        project.modified = false;
        project
            .apply(ProjectEvent::SetPieceLocked {
                id: 1,
                locked: true,
            })
            .unwrap();
        assert!(project.pieces[0].locked);
        assert!(project.modified);

        let before = project.clone();
        let shape_edits = [
            ProjectEvent::SetBaseShape {
                id: 1,
                shape: small_square(),
            },
            ProjectEvent::MovePiece {
                id: 1,
                dx: 1.0,
                dy: 0.0,
            },
            ProjectEvent::RemovePiece { id: 1 },
            add_child(BooleanOp::Union),
            ProjectEvent::SetChildShape {
                id: 1,
                child: 1,
                shape: small_square(),
            },
            ProjectEvent::SetChildOp {
                id: 1,
                child: 1,
                op: BooleanOp::Union,
            },
            ProjectEvent::ReorderChild {
                id: 1,
                child: 1,
                index: 0,
            },
            ProjectEvent::RemoveChild { id: 1, child: 1 },
            ProjectEvent::MoveChildren {
                id: 1,
                children: vec![1],
                dx: 1.0,
                dy: 0.0,
            },
            ProjectEvent::RemoveChildren {
                id: 1,
                children: vec![1],
            },
        ];
        for event in shape_edits {
            assert_eq!(project.apply(event), locked());
            assert_eq!(project, before);
        }

        project
            .apply(ProjectEvent::RenamePiece {
                id: 1,
                name: "Flap".to_owned(),
            })
            .unwrap();
        assert_eq!(project.pieces[0].name, "Flap");

        project
            .apply(ProjectEvent::SetPieceLocked {
                id: 1,
                locked: false,
            })
            .unwrap();
        assert_eq!(
            project.apply(ProjectEvent::RemovePiece { id: 1 }),
            Ok(ProjectChange::PieceRemoved(1))
        );
    }

    #[test]
    fn save_and_open_track_the_document_path() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "leathercraft-events-{}-{id}.leathercraft",
            std::process::id()
        ));
        let mut project = project_with_piece();

        project
            .apply(ProjectEvent::Save { path: path.clone() })
            .unwrap();
        assert_eq!(project.path.as_ref(), Some(&path));
        assert!(!project.modified);

        let mut reopened = Project::default();
        reopened
            .apply(ProjectEvent::Open { path: path.clone() })
            .unwrap();
        assert_eq!(reopened, project);
        fs::remove_file(path).unwrap();

        assert_eq!(
            project.apply(ProjectEvent::New {
                material: Material::default()
            }),
            Ok(ProjectChange::Created)
        );
        assert_eq!(project, Project::default());
    }

    fn black_goat() -> Material {
        Material {
            origin: Origin::Goat,
            color: LeatherColor::Black,
            thickness_mm: 0.8,
            ..Material::default()
        }
    }

    #[test]
    fn new_pieces_take_the_material_chosen_for_the_project() {
        let mut project = project_with_piece();
        project
            .apply(ProjectEvent::New {
                material: black_goat(),
            })
            .unwrap();
        assert!(project.pieces.is_empty());
        assert_eq!(project.default_material, black_goat());

        project.apply(add_piece()).unwrap();
        assert_eq!(project.pieces[0].material, black_goat());
    }

    #[test]
    fn setting_a_material_reaches_only_the_named_piece_even_when_locked() {
        let mut project = project_with_piece();
        project.apply(add_piece()).unwrap();
        project
            .apply(ProjectEvent::SetPieceLocked {
                id: 1,
                locked: true,
            })
            .unwrap();

        project
            .apply(ProjectEvent::SetPieceMaterial {
                id: 1,
                material: black_goat(),
            })
            .unwrap();
        assert_eq!(project.pieces[0].material, black_goat());
        assert_eq!(project.pieces[1].material, Material::default());
        assert_eq!(project.default_material, Material::default());
    }

    #[test]
    fn invalid_thickness_leaves_the_project_unchanged() {
        let mut project = project_with_piece();
        let before = project.clone();
        let material = Material {
            thickness_mm: 0.0,
            ..Material::default()
        };
        assert_eq!(
            project.apply(ProjectEvent::SetPieceMaterial { id: 1, material }),
            Err(Message::ThicknessOutOfRange {
                name: "Деталь 1".to_owned()
            })
        );
        assert_eq!(
            project.apply(ProjectEvent::New { material }),
            Err(Message::DefaultThicknessOutOfRange)
        );
        assert_eq!(project, before);
    }

    #[test]
    fn stitch_lines_are_added_and_an_invalid_pitch_is_rejected() {
        use crate::shapes::{HoleKind, StitchLine};
        let mut project = project_with_piece();
        let line = |pitch| {
            Shape::StitchLine(
                StitchLine::new(
                    vec![Point::new(1.0, 1.0), Point::new(9.0, 1.0)],
                    HoleKind::Round,
                    pitch,
                    1.0,
                )
                .unwrap(),
            )
        };
        project
            .apply(ProjectEvent::AddChild {
                id: 1,
                op: BooleanOp::Subtract,
                shape: line(2.0),
            })
            .unwrap();
        assert_eq!(project.pieces[0].children.len(), 1);
        let before = project.clone();
        assert!(matches!(
            project.apply(ProjectEvent::AddChild {
                id: 1,
                op: BooleanOp::Subtract,
                shape: line(0.5),
            }),
            Err(Message::InvalidHoleParameters { .. })
        ));
        assert_eq!(project, before);
    }
}
