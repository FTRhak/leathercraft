use super::{Label, Message};
use crate::material::{
    ColorGroup, Flexibility, HeatResistance, LeatherColor, MAX_THICKNESS_MM, MIN_THICKNESS_MM,
    Origin, Strength, Texture, ThicknessCategory, Treatment, WaterResistance,
};
use crate::model::{BooleanOp, Units};

pub(super) fn label(label: Label) -> &'static str {
    match label {
        Label::File => "File",
        Label::View => "View",
        Label::Edit => "Edit",
        Label::Window => "Window",
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
        Label::Ellipse => "Ellipse",
        Label::Polygon => "Polygon",
        Label::RegularPolygon => "Regular polygon",
        Label::RectangleHint => "Drag from corner to corner. Esc cancels.",
        Label::EllipseHint => "Drag from the center outward. Hold Shift for a circle. Esc cancels.",
        Label::RegularPolygonHint => {
            "Drag from the center to a vertex. Hold Shift to turn in 15° steps. Esc cancels."
        }
        Label::NewPieceMode => "New piece: drawing tools create a piece",
        Label::Operation(op) => operation(op),
        Label::OperationHint(op) => operation_hint(op),
        Label::Base => "Base",
        Label::CenterX => "Center X",
        Label::CenterY => "Center Y",
        Label::Width => "Width",
        Label::Height => "Height",
        Label::RadiusX => "Radius X",
        Label::RadiusY => "Radius Y",
        Label::Radius => "Radius",
        Label::Sides => "Sides",
        Label::Rotation => "Rotation, °",
        Label::MoveUp => "Apply earlier",
        Label::MoveDown => "Apply later",
        Label::DeleteChild => "Delete shape",
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
        Label::Material => "Material",
        Label::Canvas => "Canvas",
        Label::Properties => "Properties",
        Label::ClosePanel => "Close",
        Label::FloatPanel => "Float",
        Label::ResetPanels => "Reset panels",
        Label::History => "History",
        Label::Undo => "Undo",
        Label::Redo => "Redo",
        Label::NoPieceSelected => "No piece selected",
        Label::Camera => "Camera",
        Label::Orbit => "Orbit",
        Label::Tilt => "Tilt",
        Label::Zoom => "Zoom",
        Label::NewProjectTitle => "New project: default material",
        Label::Create => "Create",
        Label::Cancel => "Cancel",
        Label::BasicProperties => "Basic properties",
        Label::PhysicalProperties => "Physical properties",
        Label::OriginField => "Origin",
        Label::TreatmentField => "Treatment",
        Label::ThicknessField => "Thickness",
        Label::ColorField => "Color",
        Label::TextureField => "Texture",
        Label::StrengthField => "Strength",
        Label::FlexibilityField => "Flexibility",
        Label::WaterResistanceField => "Water resistance",
        Label::HeatResistanceField => "Heat resistance",
        Label::Origin(value) => origin(value),
        Label::OriginHint(value) => origin_hint(value),
        Label::Treatment(value) => treatment(value),
        Label::TreatmentHint(value) => treatment_hint(value),
        Label::Thickness(value) => thickness(value),
        Label::ColorGroup(value) => color_group(value),
        Label::Color(value) => color(value),
        Label::Texture(value) => texture(value),
        Label::Strength(value) => strength(value),
        Label::Flexibility(value) => flexibility(value),
        Label::FlexibilityHint(value) => flexibility_hint(value),
        Label::WaterResistance(value) => water_resistance(value),
        Label::WaterResistanceHint(value) => water_resistance_hint(value),
        Label::HeatResistance(value) => heat_resistance(value),
    }
}

fn origin(value: Origin) -> &'static str {
    match value {
        Origin::Calf => "Calf",
        Origin::Cow => "Cow",
        Origin::Bull => "Bull",
        Origin::Goat => "Goat",
        Origin::Sheep => "Sheep",
        Origin::Ram => "Ram",
        Origin::Pig => "Pig",
        Origin::Horse => "Horse",
        Origin::Deer => "Deer",
        Origin::Elk => "Elk",
        Origin::Crocodile => "Crocodile",
        Origin::Snake => "Snake",
        Origin::Ostrich => "Ostrich",
    }
}

fn origin_hint(value: Origin) -> &'static str {
    match value {
        Origin::Calf => "Soft, thin, elastic",
        Origin::Cow => "Strong, versatile",
        Origin::Bull => "Dense, coarse, durable",
        Origin::Goat => "Light, flexible",
        Origin::Sheep => "Soft, delicate",
        Origin::Ram => "Thin, elastic",
        Origin::Pig => "Porous, affordable",
        Origin::Horse => "Dense, wear-resistant",
        Origin::Deer => "Soft, strong",
        Origin::Elk => "Elastic, durable",
        Origin::Crocodile => "Expensive, decorative",
        Origin::Snake => "Thin, patterned",
        Origin::Ostrich => "Textured, with characteristic follicles",
    }
}

fn treatment(value: Treatment) -> &'static str {
    match value {
        Treatment::Raw => "Raw",
        Treatment::Dried => "Dried",
        Treatment::Soaked => "Soaked",
        Treatment::VegetableTanned => "Vegetable-tanned",
        Treatment::ChromeTanned => "Chrome-tanned",
        Treatment::AldehydeTanned => "Aldehyde-tanned",
        Treatment::OilTanned => "Oil-tanned",
        Treatment::CombinationTanned => "Combination-tanned",
        Treatment::Suede => "Suede",
        Treatment::Nubuck => "Nubuck",
        Treatment::Patent => "Patent",
        Treatment::Impregnated => "Impregnated",
    }
}

fn treatment_hint(value: Treatment) -> &'static str {
    match value {
        Treatment::Raw => "Untreated hide",
        Treatment::Dried => "Dehydrated",
        Treatment::Soaked => "Treated with water",
        Treatment::VegetableTanned => "Tanned with vegetable tannins",
        Treatment::ChromeTanned => "Tanned with chromium salts",
        Treatment::AldehydeTanned => "Special chemical treatment",
        Treatment::OilTanned => "Saturated with fats",
        Treatment::CombinationTanned => "A combination of methods",
        Treatment::Suede => "Treated to a napped surface",
        Treatment::Nubuck => "Sanded surface with a fine nap",
        Treatment::Patent => "Coated with a glossy lacquer",
        Treatment::Impregnated => "Saturated with protective substances",
    }
}

fn thickness(value: ThicknessCategory) -> &'static str {
    match value {
        ThicknessCategory::ExtraThin => "Extra thin",
        ThicknessCategory::Thin => "Thin",
        ThicknessCategory::Medium => "Medium",
        ThicknessCategory::Dense => "Dense",
        ThicknessCategory::Thick => "Thick",
        ThicknessCategory::ExtraThick => "Extra thick",
        ThicknessCategory::Heavy => "Heavy duty",
    }
}

fn color_group(value: ColorGroup) -> &'static str {
    match value {
        ColorGroup::White => "Whites",
        ColorGroup::Black => "Blacks",
        ColorGroup::Brown => "Browns",
        ColorGroup::Ginger => "Gingers",
        ColorGroup::Red => "Reds",
        ColorGroup::Yellow => "Yellows",
        ColorGroup::Green => "Greens",
        ColorGroup::Blue => "Blues",
        ColorGroup::Purple => "Purples",
        ColorGroup::Unusual => "Unusual",
    }
}

fn color(value: LeatherColor) -> &'static str {
    match value {
        LeatherColor::White => "White",
        LeatherColor::Milky => "Milky",
        LeatherColor::Cream => "Cream",
        LeatherColor::Black => "Black",
        LeatherColor::Charcoal => "Charcoal",
        LeatherColor::Graphite => "Graphite",
        LeatherColor::LightBrown => "Light brown",
        LeatherColor::Chocolate => "Chocolate",
        LeatherColor::Chestnut => "Chestnut",
        LeatherColor::Ginger => "Ginger",
        LeatherColor::Copper => "Copper",
        LeatherColor::Terracotta => "Terracotta",
        LeatherColor::Red => "Red",
        LeatherColor::Burgundy => "Burgundy",
        LeatherColor::Wine => "Wine",
        LeatherColor::Yellow => "Yellow",
        LeatherColor::Golden => "Golden",
        LeatherColor::Amber => "Amber",
        LeatherColor::Olive => "Olive",
        LeatherColor::Emerald => "Emerald",
        LeatherColor::Malachite => "Malachite",
        LeatherColor::LightBlue => "Light blue",
        LeatherColor::Blue => "Blue",
        LeatherColor::DarkBlue => "Dark blue",
        LeatherColor::Lavender => "Lavender",
        LeatherColor::Magenta => "Magenta",
        LeatherColor::Violet => "Violet",
        LeatherColor::Pearl => "Pearl",
        LeatherColor::Iridescent => "Iridescent",
        LeatherColor::Metallic => "Metallic",
    }
}

fn texture(value: Texture) -> &'static str {
    match value {
        Texture::Matte => "Matte",
        Texture::SemiMatte => "Semi-matte",
        Texture::Rough => "Rough",
        Texture::Porous => "Porous",
        Texture::Sanded => "Sanded",
        Texture::Embossed => "Embossed",
        Texture::Perforated => "Perforated",
        Texture::Scaly => "Scaly",
        Texture::Fibrous => "Fibrous",
        Texture::Marbled => "Marbled",
        Texture::Cracked => "Cracked",
        Texture::NaturalScars => "Natural scars",
    }
}

fn strength(value: Strength) -> &'static str {
    match value {
        Strength::VeryLow => "Very low",
        Strength::Low => "Low",
        Strength::Medium => "Medium",
        Strength::High => "High",
        Strength::VeryHigh => "Very high",
    }
}

fn flexibility(value: Flexibility) -> &'static str {
    match value {
        Flexibility::VerySoft => "Very soft",
        Flexibility::Soft => "Soft",
        Flexibility::SemiSoft => "Semi-soft",
        Flexibility::Medium => "Medium",
        Flexibility::Stiff => "Stiff",
        Flexibility::VeryStiff => "Very stiff",
        Flexibility::Rigid => "Rigid",
    }
}

fn flexibility_hint(value: Flexibility) -> &'static str {
    match value {
        Flexibility::VerySoft => "Bends easily",
        Flexibility::Soft => "Flexible, pleasant to the touch",
        Flexibility::SemiSoft => "Partly keeps its shape",
        Flexibility::Medium => "Moderately flexible",
        Flexibility::Stiff => "Bends poorly",
        Flexibility::VeryStiff => "Holds a given shape",
        Flexibility::Rigid => "Extremely hard",
    }
}

fn water_resistance(value: WaterResistance) -> &'static str {
    match value {
        WaterResistance::None => "None",
        WaterResistance::Low => "Low",
        WaterResistance::Moderate => "Moderate",
        WaterResistance::High => "High",
        WaterResistance::Waterproof => "Waterproof",
        WaterResistance::Absolute => "Absolute",
    }
}

fn water_resistance_hint(value: WaterResistance) -> &'static str {
    match value {
        WaterResistance::None => "Absorbs water quickly",
        WaterResistance::Low => "Weak protection",
        WaterResistance::Moderate => "Withstands slight moisture",
        WaterResistance::High => "Repels water",
        WaterResistance::Waterproof => "Keeps water out under specified conditions",
        WaterResistance::Absolute => "A fantasy property",
    }
}

fn heat_resistance(value: HeatResistance) -> &'static str {
    match value {
        HeatResistance::None => "None",
        HeatResistance::Low => "Low",
        HeatResistance::Moderate => "Moderate",
        HeatResistance::High => "High",
        HeatResistance::VeryHigh => "Very high",
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
        Message::ChildAdded { name } => format!("Added a shape to '{name}'."),
        Message::ChildDeleted => "Shape deleted.".to_owned(),
        Message::SelectPieceForChild => "Select a piece to add the shape to.".to_owned(),
        Message::ChildNotFound { name } => format!("Piece '{name}' has no such shape."),
        Message::TooManyChildren { name, max } => {
            format!("Piece '{name}' can have at most {max} shapes.")
        }
        Message::InvalidChildId { name } => {
            format!("A shape of piece '{name}' has an invalid or duplicate ID.")
        }
        Message::InvalidShapeSize { name } => {
            format!("Shapes of piece '{name}' must have a positive size.")
        }
        Message::ResultEmpty { name } => format!("Nothing is left of piece '{name}'."),
        Message::ResultSplit { name } => {
            format!("Piece '{name}' falls apart into several parts.")
        }
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
        Message::ThicknessOutOfRange { name } => format!(
            "Piece '{name}' must have a thickness between {MIN_THICKNESS_MM} and {MAX_THICKNESS_MM} mm."
        ),
        Message::DefaultThicknessOutOfRange => format!(
            "The default material must have a thickness between {MIN_THICKNESS_MM} and {MAX_THICKNESS_MM} mm."
        ),
        Message::CouldNotEncode { detail } => format!("Could not encode project: {detail}"),
        Message::CouldNotRead { path, detail } => format!("Could not read '{path}': {detail}"),
        Message::CouldNotParse { path, detail } => format!("Could not parse '{path}': {detail}"),
        Message::CouldNotWrite { path, detail } => format!("Could not write '{path}': {detail}"),
        Message::ExportEmpty => "Add at least one pattern piece before exporting.".to_owned(),
        Message::Undone { action } => format!("Undone: {}.", self::message(action)),
        Message::Redone { action } => format!("Redone: {}.", self::message(action)),
        Message::HistoryRestored { action } => format!("Returned to: {}.", self::message(action)),
        Message::ActionNewProject => "New project".to_owned(),
        Message::ActionOpened { name } => format!("Opened {name}"),
        Message::ActionUnits => "Changed display units".to_owned(),
        Message::ActionAddPiece { name } => format!("Added '{name}'"),
        Message::ActionRenamePiece { name } => format!("Renamed to '{name}'"),
        Message::ActionEditBase { name } => format!("Changed the base of '{name}'"),
        Message::ActionMovePiece { name } => format!("Moved '{name}'"),
        Message::ActionLockPiece { name } => format!("Locked '{name}'"),
        Message::ActionUnlockPiece { name } => format!("Unlocked '{name}'"),
        Message::ActionMaterial { name } => format!("Changed the material of '{name}'"),
        Message::ActionRemovePiece { name } => format!("Deleted '{name}'"),
        Message::ActionAddChild { name } => format!("Added a shape to '{name}'"),
        Message::ActionEditChild { name } => format!("Changed a shape of '{name}'"),
        Message::ActionSetChildOp { name } => format!("Changed an operation of '{name}'"),
        Message::ActionReorderChild { name } => format!("Reordered the shapes of '{name}'"),
        Message::ActionRemoveChild { name } => format!("Deleted a shape from '{name}'"),
    }
}

fn operation(op: BooleanOp) -> &'static str {
    match op {
        BooleanOp::Subtract => "Subtract",
        BooleanOp::Union => "Union",
        BooleanOp::Intersect => "Intersect",
    }
}

fn operation_hint(op: BooleanOp) -> &'static str {
    match op {
        BooleanOp::Subtract => "Subtract: new shapes cut into the selected piece",
        BooleanOp::Union => "Union: new shapes add to the selected piece",
        BooleanOp::Intersect => {
            "Intersect: only the part of the selected piece inside new shapes is kept"
        }
    }
}

pub(super) fn unit_symbol(units: Units) -> &'static str {
    match units {
        Units::Millimeters => "mm",
        Units::Inches => "in",
    }
}
