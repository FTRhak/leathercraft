use super::{Label, Message};
use crate::model::Units;

pub(super) fn label(label: Label) -> &'static str {
    match label {
        Label::New => "New",
        Label::Open => "Open…",
        Label::Save => "Save",
        Label::SaveAs => "Save as…",
        Label::ExportSvg => "Export SVG…",
        Label::ExportDxf => "Export DXF…",
        Label::Editor2d => "2D editor",
        Label::Preview3d => "3D preview",
        Label::Language => "Language",
        Label::UnsavedChanges => "Unsaved changes",
        Label::DiscardChanges => "Discard the unsaved changes to this project?",
        Label::ProjectFileType => "Leathercraft project",
        Label::Tools => "Tools",
        Label::Select => "Select",
        Label::Draw => "Draw",
        Label::Rectangle => "Rectangle",
        Label::Circle => "Circle",
        Label::RectangleHint => "Drag from corner to corner. Esc cancels.",
        Label::CircleHint => "Drag from the center outward. Esc cancels.",
        Label::DrawHint => {
            "Click to add points. Click the first point or press Enter to close; Esc cancels."
        }
        Label::SelectHint => {
            "Drag a piece or a vertex of the selected piece. Del removes the selected piece."
        }
        Label::NavigationHint => "Right or middle drag pans; scroll zooms.",
        Label::SnapMillimeters => "Snap (mm)",
        Label::ZoomToFit => "Zoom to fit",
        Label::DisplayUnits => "Display units",
        Label::Pieces => "Pieces",
        Label::DeletePiece => "Delete piece",
        Label::Layout => "Layout",
        Label::LockPiece => "Lock piece",
        Label::UnlockPiece => "Unlock piece",
        Label::Camera => "Camera",
        Label::Orbit => "Orbit",
        Label::Tilt => "Tilt",
        Label::Zoom => "Zoom",
    }
}

pub(super) fn message(message: &Message) -> String {
    match message {
        Message::Ready => "Ready.".to_owned(),
        Message::NewProject => "New project.".to_owned(),
        Message::Opened { path } => format!("Opened {path}."),
        Message::Saved { path } => format!("Saved {path}."),
        Message::Exported { path } => format!("Exported {path}."),
        Message::PieceAdded { name } => format!("Added {name}."),
        Message::PieceDeleted => "Piece deleted.".to_owned(),
        Message::PointCount { count: 1 } => "1 point".to_owned(),
        Message::PointCount { count } => format!("{count} points"),
        Message::DefaultPieceName { id } => format!("Piece {id}"),
        Message::PieceNotFound { id } => format!("Piece {id} does not exist."),
        Message::PieceLocked { name } => format!("Piece '{name}' is locked."),
        Message::UnsupportedVersion { found, expected } => {
            format!("Project version {found} is not supported (expected {expected}).")
        }
        Message::TooManyPieces { max } => format!("A project can contain at most {max} pieces."),
        Message::InvalidPieceId { name } => {
            format!("Piece '{name}' has an invalid or duplicate ID.")
        }
        Message::PieceNameRequired { id } => format!("Piece {id} must have a name."),
        Message::PointCountOutOfRange { name, max } => {
            format!("Piece '{name}' must have between 3 and {max} points.")
        }
        Message::CoordinateOutOfRange { name } => {
            format!("Piece '{name}' has a point outside the supported coordinate range.")
        }
        Message::CoincidentPoints { name } => {
            format!("Piece '{name}' has two consecutive points in the same position.")
        }
        Message::ZeroArea { name } => format!("Piece '{name}' must enclose a non-zero area."),
        Message::CouldNotEncode { detail } => format!("Could not encode project: {detail}"),
        Message::CouldNotRead { path, detail } => format!("Could not read '{path}': {detail}"),
        Message::CouldNotParse { path, detail } => format!("Could not parse '{path}': {detail}"),
        Message::CouldNotWrite { path, detail } => format!("Could not write '{path}': {detail}"),
        Message::ExportEmpty => "Add at least one pattern piece before exporting.".to_owned(),
    }
}

pub(super) fn unit_symbol(units: Units) -> &'static str {
    match units {
        Units::Millimeters => "mm",
        Units::Inches => "in",
    }
}
