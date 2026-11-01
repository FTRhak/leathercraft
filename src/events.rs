use crate::i18n::Message;
use crate::material::Material;
use crate::model::{PatternPiece, Point, Project, Units};
use crate::persistence;
use std::path::PathBuf;

/// A requested change to the project. `Project::apply` is the only intended way to change one.
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectEvent {
    New { material: Material },
    Open { path: PathBuf },
    Save { path: PathBuf },
    SetUnits(Units),
    AddPiece { name: String, points: Vec<Point> },
    RenamePiece { id: u64, name: String },
    SetPiecePoints { id: u64, points: Vec<Point> },
    SetPieceLocked { id: u64, locked: bool },
    SetPieceMaterial { id: u64, material: Material },
    RemovePiece { id: u64 },
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
            ProjectEvent::SetPiecePoints { id, points } => self.edit(|project| {
                project.unlocked_piece_mut(id)?.points = points;
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
            ProjectEvent::SetPiecePoints {
                id: 1,
                points: vec![Point::new(0.0, 0.0); 3],
            },
            ProjectEvent::SetPiecePoints {
                id: 9,
                points: triangle(),
            },
            ProjectEvent::RemovePiece { id: 9 },
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
        let set_points = ProjectEvent::SetPiecePoints {
            id: 1,
            points: moved.clone(),
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
