use crate::model::Project;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum ExportError {
    EmptyProject,
    InvalidProject(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProject => {
                formatter.write_str("Add at least one pattern piece before exporting.")
            }
            Self::InvalidProject(message) => formatter.write_str(message),
        }
    }
}

impl Error for ExportError {}

pub fn to_svg(project: &Project) -> Result<String, ExportError> {
    validate_export(project)?;
    let bounds = project.bounds().ok_or(ExportError::EmptyProject)?;
    let (min_x, min_y) = (bounds.min_x, bounds.min_y);
    let width = bounds.width();
    let height = bounds.height();
    let mut svg = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}mm\" height=\"{}mm\" viewBox=\"0 0 {} {}\">\n",
        number(width),
        number(height),
        number(width),
        number(height)
    );
    svg.push_str(
        "  <g fill=\"none\" stroke=\"#111827\" stroke-width=\"0.25\" stroke-linejoin=\"round\">\n",
    );
    for piece in &project.pieces {
        svg.push_str(&format!("    <path data-piece=\"{}\" d=\"", piece.id));
        for (index, point) in piece.points.iter().enumerate() {
            let command = if index == 0 { 'M' } else { 'L' };
            svg.push_str(&format!(
                "{command} {} {} ",
                number(point.x - min_x),
                number(point.y - min_y)
            ));
        }
        svg.push_str("Z\" />\n");
    }
    svg.push_str("  </g>\n</svg>\n");
    Ok(svg)
}

pub fn to_dxf(project: &Project) -> Result<String, ExportError> {
    validate_export(project)?;
    let mut dxf = String::from(
        "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n9\n$MEASUREMENT\n70\n1\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n",
    );
    for piece in &project.pieces {
        dxf.push_str(&format!(
            "0\nLWPOLYLINE\n8\nPATTERN\n90\n{}\n70\n1\n",
            piece.points.len()
        ));
        for point in &piece.points {
            dxf.push_str(&format!(
                "10\n{}\n20\n{}\n",
                number(point.x),
                number(point.y)
            ));
        }
    }
    dxf.push_str("0\nENDSEC\n0\nEOF\n");
    Ok(dxf)
}

fn validate_export(project: &Project) -> Result<(), ExportError> {
    project.validate().map_err(ExportError::InvalidProject)?;
    if project.pieces.is_empty() {
        return Err(ExportError::EmptyProject);
    }
    Ok(())
}

fn number(value: f64) -> String {
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
