use crate::i18n::Message;
use crate::model::Project;
use std::fs;
use std::io;
use std::path::Path;

pub fn save_project(path: &Path, project: &Project) -> Result<(), Message> {
    project.validate()?;
    let json = serde_json::to_string_pretty(project).map_err(|error| Message::CouldNotEncode {
        detail: error.to_string(),
    })?;
    fs::write(path, format!("{json}\n")).map_err(|error| Message::CouldNotWrite {
        path: path.display().to_string(),
        detail: error.to_string(),
    })
}

pub fn load_project(path: &Path) -> Result<Project, Message> {
    let contents = fs::read_to_string(path).map_err(|error| Message::CouldNotRead {
        path: path.display().to_string(),
        detail: error.to_string(),
    })?;
    let project: Project =
        serde_json::from_str(&contents).map_err(|error| Message::CouldNotParse {
            path: path.display().to_string(),
            detail: error.to_string(),
        })?;
    project.validate()?;
    Ok(project)
}

pub fn write_export(path: &Path, contents: &str) -> io::Result<()> {
    fs::write(path, contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{LeatherColor, Material, Origin, Treatment};
    use crate::model::{PatternPiece, Point, Project};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn project_save_and_load_round_trip() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "leathercraft-project-{}-{id}.leathercraft",
            std::process::id()
        ));
        let project = Project {
            pieces: vec![PatternPiece::new(
                1,
                "Flap",
                vec![
                    Point::new(-2.0, 0.0),
                    Point::new(50.0, 0.0),
                    Point::new(50.0, 70.0),
                    Point::new(-2.0, 70.0),
                ],
            )],
            ..Project::default()
        };

        save_project(&path, &project).unwrap();
        assert_eq!(load_project(&path).unwrap(), project);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn holes_round_trip_and_default_to_none_in_older_files() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "leathercraft-holes-{}-{id}.leathercraft",
            std::process::id()
        ));
        let mut piece = PatternPiece::new(
            1,
            "Flap",
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(0.0, 10.0),
            ],
        );
        piece.holes = vec![vec![
            Point::new(1.0, 1.0),
            Point::new(3.0, 1.0),
            Point::new(1.0, 3.0),
        ]];
        let project = Project {
            pieces: vec![piece],
            ..Project::default()
        };
        save_project(&path, &project).unwrap();
        assert_eq!(load_project(&path).unwrap(), project);

        fs::write(
            &path,
            r#"{"version":1,"units":"millimeters","pieces":[{"id":1,"name":"Flap","points":[{"x":0.0,"y":0.0},{"x":10.0,"y":0.0},{"x":0.0,"y":10.0}]}]}"#,
        )
        .unwrap();
        assert!(load_project(&path).unwrap().pieces[0].holes.is_empty());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn piece_lock_round_trips_and_defaults_to_unlocked() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "leathercraft-lock-{}-{id}.leathercraft",
            std::process::id()
        ));
        let mut piece = PatternPiece::new(
            1,
            "Flap",
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(0.0, 10.0),
            ],
        );
        piece.locked = true;
        let project = Project {
            pieces: vec![piece],
            ..Project::default()
        };
        save_project(&path, &project).unwrap();
        assert!(load_project(&path).unwrap().pieces[0].locked);

        fs::write(
            &path,
            r#"{"version":1,"units":"millimeters","pieces":[{"id":1,"name":"Flap","points":[{"x":0.0,"y":0.0},{"x":10.0,"y":0.0},{"x":0.0,"y":10.0}]}]}"#,
        )
        .unwrap();
        assert!(!load_project(&path).unwrap().pieces[0].locked);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn materials_round_trip_and_default_in_older_files() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "leathercraft-material-{}-{id}.leathercraft",
            std::process::id()
        ));
        let material = Material {
            origin: Origin::Ostrich,
            treatment: Treatment::Nubuck,
            thickness_mm: 2.5,
            color: LeatherColor::Emerald,
            ..Material::default()
        };
        let mut piece = PatternPiece::new(
            1,
            "Flap",
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(0.0, 10.0),
            ],
        );
        piece.material = material;
        let project = Project {
            pieces: vec![piece],
            default_material: material,
            ..Project::default()
        };
        save_project(&path, &project).unwrap();
        assert_eq!(load_project(&path).unwrap(), project);

        fs::write(
            &path,
            r#"{"version":1,"units":"millimeters","pieces":[{"id":1,"name":"Flap","points":[{"x":0.0,"y":0.0},{"x":10.0,"y":0.0},{"x":0.0,"y":10.0}]}]}"#,
        )
        .unwrap();
        let older = load_project(&path).unwrap();
        assert_eq!(older.default_material, Material::default());
        assert_eq!(older.pieces[0].material, Material::default());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_unknown_project_versions_on_load() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "leathercraft-invalid-{}-{id}.leathercraft",
            std::process::id()
        ));
        fs::write(&path, r#"{"version":99,"units":"millimeters","pieces":[]}"#).unwrap();

        assert_eq!(
            load_project(&path),
            Err(Message::UnsupportedVersion {
                found: 99,
                expected: crate::model::PROJECT_VERSION
            })
        );
        fs::remove_file(path).unwrap();
    }
}
