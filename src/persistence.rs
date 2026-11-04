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
    use crate::model::{BooleanOp, ChildShape, PatternPiece, Point, Project};
    use crate::shapes::{Ellipse, Polygon, Rectangle, Shape};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> std::path::PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "leathercraft-{label}-{}-{id}.leathercraft",
            std::process::id()
        ))
    }

    fn flap() -> PatternPiece {
        PatternPiece::new(
            1,
            "Flap",
            Shape::Polygon(Polygon {
                points: vec![
                    Point::new(-2.0, 0.0),
                    Point::new(50.0, 0.0),
                    Point::new(50.0, 70.0),
                    Point::new(-2.0, 70.0),
                ],
                ..Default::default()
            }),
        )
    }

    #[test]
    fn project_save_and_load_round_trip() {
        let path = temp_path("project");
        let project = Project {
            pieces: vec![flap()],
            ..Project::default()
        };

        save_project(&path, &project).unwrap();
        assert_eq!(load_project(&path).unwrap(), project);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn corner_radii_round_trip_and_sharp_shapes_save_without_them() {
        let path = temp_path("radii");
        let mut piece = flap();
        piece.base = piece.base.with_corner_radius(1, 4.0).unwrap();
        let project = Project {
            pieces: vec![piece],
            ..Project::default()
        };
        save_project(&path, &project).unwrap();
        assert!(fs::read_to_string(&path).unwrap().contains("\"radii\""));
        assert_eq!(load_project(&path).unwrap(), project);

        let sharp = Project {
            pieces: vec![flap()],
            ..Project::default()
        };
        save_project(&path, &sharp).unwrap();
        assert!(!fs::read_to_string(&path).unwrap().contains("radii"));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn children_round_trip_with_their_operations_and_order() {
        let path = temp_path("children");
        let mut piece = flap();
        piece.children = vec![
            ChildShape {
                id: 2,
                op: BooleanOp::Subtract,
                shape: Shape::Ellipse(Ellipse {
                    center: Point::new(20.0, 20.0),
                    radius_x: 5.0,
                    radius_y: 3.0,
                    rotation: 0.5,
                }),
            },
            ChildShape {
                id: 1,
                op: BooleanOp::Union,
                shape: Shape::Rectangle(Rectangle {
                    center: Point::new(50.0, 30.0),
                    width: 10.0,
                    height: 8.0,
                    rotation: 0.0,
                    radii: Vec::new(),
                }),
            },
            ChildShape {
                id: 3,
                op: BooleanOp::Intersect,
                shape: Shape::rectangle_from_corners(
                    Point::new(-5.0, -5.0),
                    Point::new(60.0, 60.0),
                )
                .unwrap(),
            },
        ];
        piece.locked = true;
        let project = Project {
            pieces: vec![piece],
            ..Project::default()
        };
        save_project(&path, &project).unwrap();
        assert_eq!(load_project(&path).unwrap(), project);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn materials_round_trip() {
        let path = temp_path("material");
        let material = Material {
            origin: Origin::Ostrich,
            treatment: Treatment::Nubuck,
            thickness_mm: 2.5,
            color: LeatherColor::Emerald,
            ..Material::default()
        };
        let mut piece = flap();
        piece.material = material;
        let project = Project {
            pieces: vec![piece],
            default_material: material,
            ..Project::default()
        };
        save_project(&path, &project).unwrap();
        assert_eq!(load_project(&path).unwrap(), project);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_other_project_versions_on_load() {
        let path = temp_path("invalid");
        for found in [1, 99] {
            fs::write(
                &path,
                format!(r#"{{"version":{found},"units":"millimeters","pieces":[]}}"#),
            )
            .unwrap();
            assert_eq!(
                load_project(&path),
                Err(Message::UnsupportedVersion {
                    found,
                    expected: crate::model::PROJECT_VERSION
                })
            );
        }
        fs::remove_file(path).unwrap();
    }
}
