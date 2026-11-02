use crate::i18n::Message;
use crate::material::Material;
use crate::model::{Cut, PatternPiece, Point, Project, Units};
use crate::persistence;
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
        points: Vec<Point>,
    },
    RenamePiece {
        id: u64,
        name: String,
    },
    /// Replaces the outline and holes together, since moving them apart would fail validation.
    SetPieceGeometry {
        id: u64,
        points: Vec<Point>,
        holes: Vec<Vec<Point>>,
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
    AddHole {
        id: u64,
        points: Vec<Point>,
    },
    RemoveHole {
        id: u64,
        index: usize,
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
    HoleAdded { id: u64, index: usize },
    HoleRemoved { id: u64 },
    EdgeCut { id: u64 },
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
            ProjectEvent::AddPiece { name, points } => self.edit(|project| {
                let id = project.next_piece_id();
                let mut piece = PatternPiece::new(id, name, points);
                piece.material = project.default_material;
                project.add_piece(piece);
                Ok(ProjectChange::PieceAdded(id))
            }),
            ProjectEvent::RenamePiece { id, name } => self.edit(|project| {
                project.piece_mut(id)?.name = name;
                Ok(ProjectChange::Edited)
            }),
            ProjectEvent::SetPieceGeometry { id, points, holes } => self.edit(|project| {
                let piece = project.unlocked_piece_mut(id)?;
                piece.points = points;
                piece.holes = holes;
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
            ProjectEvent::AddHole { id, points } => self.edit(|project| {
                Ok(match project.unlocked_piece_mut(id)?.cut(points)? {
                    Cut::Hole(index) => ProjectChange::HoleAdded { id, index },
                    Cut::Edge => ProjectChange::EdgeCut { id },
                })
            }),
            ProjectEvent::RemoveHole { id, index } => self.edit(|project| {
                let piece = project.unlocked_piece_mut(id)?;
                if index >= piece.holes.len() {
                    return Err(Message::HoleNotFound {
                        name: piece.name.clone(),
                    });
                }
                piece.holes.remove(index);
                Ok(ProjectChange::HoleRemoved { id })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{LeatherColor, Origin};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn triangle() -> Vec<Point> {
        vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 10.0),
        ]
    }

    fn small_hole() -> Vec<Point> {
        vec![
            Point::new(1.0, 1.0),
            Point::new(3.0, 1.0),
            Point::new(1.0, 3.0),
        ]
    }

    fn add_piece() -> ProjectEvent {
        ProjectEvent::AddPiece {
            name: "Деталь 1".to_owned(),
            points: triangle(),
        }
    }

    fn project_with_piece() -> Project {
        let mut project = Project::default();
        project.apply(add_piece()).unwrap();
        project
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
        project.modified = false;
        let before = project.clone();

        let invalid = [
            ProjectEvent::AddPiece {
                name: "Flap".to_owned(),
                points: Vec::new(),
            },
            ProjectEvent::AddPiece {
                name: " ".to_owned(),
                points: triangle(),
            },
            ProjectEvent::RenamePiece {
                id: 1,
                name: " ".to_owned(),
            },
            ProjectEvent::SetPieceGeometry {
                id: 1,
                points: vec![Point::new(0.0, 0.0); 3],
                holes: Vec::new(),
            },
            ProjectEvent::SetPieceGeometry {
                id: 9,
                points: triangle(),
                holes: Vec::new(),
            },
            ProjectEvent::RemovePiece { id: 9 },
            ProjectEvent::AddHole {
                id: 9,
                points: small_hole(),
            },
            ProjectEvent::AddHole {
                id: 1,
                points: vec![
                    Point::new(50.0, 50.0),
                    Point::new(51.0, 50.0),
                    Point::new(50.0, 51.0),
                ],
            },
            ProjectEvent::RemoveHole { id: 1, index: 0 },
        ];
        for event in invalid {
            assert!(project.apply(event).is_err());
            assert_eq!(project, before);
        }
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
    fn locked_pieces_reject_shape_changes_and_removal() {
        let mut project = project_with_piece();
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
        let locked = Err(Message::PieceLocked {
            name: "Деталь 1".to_owned(),
        });
        let moved = vec![
            Point::new(1.0, 0.0),
            Point::new(11.0, 0.0),
            Point::new(1.0, 10.0),
        ];
        let set_points = ProjectEvent::SetPieceGeometry {
            id: 1,
            points: moved.clone(),
            holes: Vec::new(),
        };
        assert_eq!(project.apply(set_points.clone()), locked);
        assert_eq!(project.apply(ProjectEvent::RemovePiece { id: 1 }), locked);
        assert_eq!(project, before);

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
        project.apply(set_points).unwrap();
        assert_eq!(project.pieces[0].points, moved);
        assert_eq!(
            project.apply(ProjectEvent::RemovePiece { id: 1 }),
            Ok(ProjectChange::PieceRemoved(1))
        );
    }

    #[test]
    fn holes_can_be_added_and_removed() {
        let mut project = project_with_piece();
        assert_eq!(
            project.apply(ProjectEvent::AddHole {
                id: 1,
                points: small_hole(),
            }),
            Ok(ProjectChange::HoleAdded { id: 1, index: 0 })
        );
        assert_eq!(project.pieces[0].holes, vec![small_hole()]);

        assert_eq!(
            project.apply(ProjectEvent::RemoveHole { id: 1, index: 0 }),
            Ok(ProjectChange::HoleRemoved { id: 1 })
        );
        assert!(project.pieces[0].holes.is_empty());
    }

    #[test]
    fn a_hole_drawn_across_the_edge_reshapes_the_piece() {
        let mut project = project_with_piece();
        let area = project.pieces[0].signed_area().abs();
        let across = vec![
            Point::new(-1.0, 1.0),
            Point::new(2.0, 1.0),
            Point::new(2.0, 3.0),
            Point::new(-1.0, 3.0),
        ];
        assert_eq!(
            project.apply(ProjectEvent::AddHole {
                id: 1,
                points: across,
            }),
            Ok(ProjectChange::EdgeCut { id: 1 })
        );
        assert!(project.pieces[0].holes.is_empty());
        assert!(project.pieces[0].signed_area().abs() < area);
    }

    #[test]
    fn an_edge_cut_through_an_existing_hole_is_rejected() {
        let mut project = project_with_piece();
        project
            .apply(ProjectEvent::AddHole {
                id: 1,
                points: small_hole(),
            })
            .unwrap();
        let before = project.clone();
        let across = vec![
            Point::new(-1.0, 1.5),
            Point::new(2.0, 1.5),
            Point::new(2.0, 2.0),
            Point::new(-1.0, 2.0),
        ];
        assert_eq!(
            project.apply(ProjectEvent::AddHole {
                id: 1,
                points: across,
            }),
            Err(Message::HoleOutsidePiece {
                name: "Деталь 1".to_owned()
            })
        );
        assert_eq!(project, before);
    }

    #[test]
    fn locked_pieces_reject_hole_changes() {
        let mut project = project_with_piece();
        project
            .apply(ProjectEvent::AddHole {
                id: 1,
                points: small_hole(),
            })
            .unwrap();
        project
            .apply(ProjectEvent::SetPieceLocked {
                id: 1,
                locked: true,
            })
            .unwrap();

        let before = project.clone();
        let locked = Err(Message::PieceLocked {
            name: "Деталь 1".to_owned(),
        });
        assert_eq!(
            project.apply(ProjectEvent::RemoveHole { id: 1, index: 0 }),
            locked
        );
        assert_eq!(
            project.apply(ProjectEvent::AddHole {
                id: 1,
                points: small_hole(),
            }),
            locked
        );
        assert_eq!(project, before);
    }

    #[test]
    fn geometry_that_leaves_a_hole_outside_is_rejected() {
        let mut project = project_with_piece();
        project
            .apply(ProjectEvent::AddHole {
                id: 1,
                points: small_hole(),
            })
            .unwrap();
        let before = project.clone();

        let shifted: Vec<Point> = triangle()
            .iter()
            .map(|point| Point::new(point.x + 100.0, point.y))
            .collect();
        assert_eq!(
            project.apply(ProjectEvent::SetPieceGeometry {
                id: 1,
                points: shifted,
                holes: vec![small_hole()],
            }),
            Err(Message::HoleOutsidePiece {
                name: "Деталь 1".to_owned()
            })
        );
        assert_eq!(project, before);
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
}
