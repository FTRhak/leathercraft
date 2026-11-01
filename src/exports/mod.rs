use crate::i18n::Message;
use crate::model::Project;
use std::error::Error;
use std::fmt;

mod dxf;
mod svg;

pub use dxf::to_dxf;
pub use svg::to_svg;

#[derive(Debug, PartialEq, Eq)]
pub enum ExportError {
    EmptyProject,
    InvalidProject(Message),
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProject => Message::ExportEmpty.fmt(formatter),
            Self::InvalidProject(message) => message.fmt(formatter),
        }
    }
}

impl Error for ExportError {}

impl From<ExportError> for Message {
    fn from(error: ExportError) -> Self {
        match error {
            ExportError::EmptyProject => Self::ExportEmpty,
            ExportError::InvalidProject(message) => message,
        }
    }
}

pub(super) fn validate_export(project: &Project) -> Result<(), ExportError> {
    project.validate().map_err(ExportError::InvalidProject)?;
    if project.pieces.is_empty() {
        return Err(ExportError::EmptyProject);
    }
    Ok(())
}

pub(super) fn number(value: f64) -> String {
    let mut formatted = format!("{value:.6}");
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    if formatted == "-0" {
        "0".to_owned()
    } else {
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PatternPiece, Point};

    fn project() -> Project {
        Project {
            pieces: vec![PatternPiece::new(
                1,
                "Pocket",
                vec![
                    Point::new(-10.0, 5.0),
                    Point::new(40.0, 5.0),
                    Point::new(40.0, 85.0),
                    Point::new(-10.0, 85.0),
                ],
            )],
            ..Project::default()
        }
    }

    #[test]
    fn svg_exports_true_size_and_offsets_negative_coordinates() {
        let svg = to_svg(&project()).unwrap();

        assert!(svg.contains("width=\"50mm\" height=\"80mm\""));
        assert!(svg.contains("viewBox=\"0 0 50 80\""));
        assert!(svg.contains("M 0 0 L 50 0 L 50 80 L 0 80 Z"));
    }

    #[test]
    fn dxf_exports_closed_millimeter_polylines() {
        let dxf = to_dxf(&project()).unwrap();

        assert!(dxf.contains("$INSUNITS\n70\n4"));
        assert!(dxf.contains("0\nLWPOLYLINE\n8\nPATTERN\n90\n4\n70\n1"));
        assert!(dxf.ends_with("0\nEOF\n"));
    }

    #[test]
    fn empty_projects_are_not_exported() {
        assert_eq!(to_svg(&Project::default()), Err(ExportError::EmptyProject));
        assert_eq!(to_dxf(&Project::default()), Err(ExportError::EmptyProject));
    }
}
