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
