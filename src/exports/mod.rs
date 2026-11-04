use crate::boolean::Region;
use crate::i18n::Message;
use crate::model::Project;
use crate::shapes::{Line, Shape};
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

/// Checks the project and computes each piece's region, in piece order. A piece whose
/// shapes don't combine into one outline can't be cut, so it stops the export.
pub(super) fn validate_export(project: &Project) -> Result<Vec<Region>, ExportError> {
    project.validate().map_err(ExportError::InvalidProject)?;
    if project.pieces.is_empty() {
        return Err(ExportError::EmptyProject);
    }
    project
        .pieces
        .iter()
        .map(|piece| piece.region().map_err(ExportError::InvalidProject))
        .collect()
}

/// Every centerline of the project, in piece and child order, with its piece and child ids.
pub(super) fn centerlines(project: &Project) -> impl Iterator<Item = (u64, u64, &Line)> {
    project.pieces.iter().flat_map(|piece| {
        piece
            .children
            .iter()
            .filter_map(move |child| match &child.shape {
                Shape::Line(line) => Some((piece.id, child.id, line)),
                _ => None,
            })
    })
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
    use crate::model::{BooleanOp, ChildShape, PatternPiece, Point};
    use crate::shapes::Shape;

    fn project() -> Project {
        Project {
            pieces: vec![PatternPiece::new(
                1,
                "Pocket",
                Shape::rectangle_from_corners(Point::new(-10.0, 5.0), Point::new(40.0, 85.0))
                    .unwrap(),
            )],
            ..Project::default()
        }
    }

    fn project_with_child(op: BooleanOp, a: Point, b: Point) -> Project {
        let mut project = project();
        project.pieces[0].children.push(ChildShape {
            id: 1,
            op,
            shape: Shape::rectangle_from_corners(a, b).unwrap(),
        });
        project
    }

    fn project_with_hole() -> Project {
        project_with_child(
            BooleanOp::Subtract,
            Point::new(0.0, 15.0),
            Point::new(10.0, 25.0),
        )
    }

    #[test]
    fn svg_exports_true_size_and_offsets_negative_coordinates() {
        let svg = to_svg(&project()).unwrap();

        assert!(svg.contains("width=\"50mm\" height=\"80mm\""));
        assert!(svg.contains("viewBox=\"0 0 50 80\""));
        assert_eq!(svg.matches(" L ").count(), 3);
        for corner in ["0 0", "50 0", "50 80", "0 80"] {
            assert!(svg.contains(corner), "missing {corner}");
        }
    }

    #[test]
    fn dxf_exports_closed_millimeter_polylines() {
        let dxf = to_dxf(&project()).unwrap();

        assert!(dxf.contains(
            "$INSUNITS
70
4"
        ));
        assert!(dxf.contains(
            "0
LWPOLYLINE
8
PATTERN
90
4
70
1"
        ));
        assert!(dxf.ends_with(
            "0
EOF
"
        ));
    }

    #[test]
    fn svg_exports_holes_as_subpaths_of_their_piece() {
        let svg = to_svg(&project_with_hole()).unwrap();

        assert_eq!(svg.matches('M').count(), 2);
        assert_eq!(svg.matches("<path").count(), 1);
    }

    #[test]
    fn svg_size_follows_the_combined_outline() {
        let svg = to_svg(&project_with_child(
            BooleanOp::Union,
            Point::new(30.0, 15.0),
            Point::new(60.0, 25.0),
        ))
        .unwrap();
        assert!(svg.contains("width=\"70mm\" height=\"80mm\""));
    }

    #[test]
    fn dxf_exports_holes_as_separate_polylines() {
        let dxf = to_dxf(&project_with_hole()).unwrap();

        assert_eq!(dxf.matches("LWPOLYLINE").count(), 2);
    }

    #[test]
    fn pieces_that_do_not_combine_into_one_outline_are_not_exported() {
        let split = project_with_child(
            BooleanOp::Subtract,
            Point::new(-20.0, 40.0),
            Point::new(50.0, 50.0),
        );
        let error = Err(ExportError::InvalidProject(Message::ResultSplit {
            name: "Pocket".to_owned(),
        }));
        assert_eq!(to_svg(&split), error);
        assert_eq!(to_dxf(&split), error);
    }

    #[test]
    fn empty_projects_are_not_exported() {
        assert_eq!(to_svg(&Project::default()), Err(ExportError::EmptyProject));
        assert_eq!(to_dxf(&Project::default()), Err(ExportError::EmptyProject));
    }

    fn project_with_line() -> Project {
        let mut project = project();
        project.pieces[0].children.push(ChildShape {
            id: 1,
            op: BooleanOp::Subtract,
            shape: Shape::line_from_ends(
                Point::new(15.0, -5.0),
                Point::new(15.0, 95.0),
                false,
                crate::shapes::LineStyle::DashDot {
                    dash: 10.0,
                    gap: 2.0,
                },
            )
            .unwrap(),
        });
        project
    }

    #[test]
    fn svg_draws_centerlines_beyond_their_piece() {
        let svg = to_svg(&project_with_line()).unwrap();
        // The line reaches 10 mm above and below the piece.
        assert!(svg.contains("width=\"50mm\" height=\"100mm\""));
        assert!(svg.contains(
            "<line data-piece=\"1\" data-child=\"1\" x1=\"25\" y1=\"0\" x2=\"25\" y2=\"100\" stroke-dasharray=\"10 2 1 2\" />"
        ));
    }

    #[test]
    fn dxf_puts_each_dash_on_the_centerline_layer() {
        let dxf = to_dxf(&project_with_line()).unwrap();
        // 100 mm of a 15 mm pattern: 6 periods of a dash and a dot, then a last dash.
        assert_eq!(dxf.matches("LINE\n8\nCENTERLINE\n").count(), 13);
        assert!(dxf.contains("0\nLINE\n8\nCENTERLINE\n10\n15\n20\n-5\n11\n15\n21\n5\n"));
    }
}
