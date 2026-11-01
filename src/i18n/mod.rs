mod en;
mod uk;

use crate::model::Units;
use std::error::Error;
use std::fmt;

/// The interface language. A preference of the application, not part of the file format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    Ukrainian,
}

impl Language {
    pub const ALL: [Self; 2] = [Self::English, Self::Ukrainian];

    pub fn tag(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Ukrainian => "uk",
        }
    }

    /// Matches on the primary subtag, so OS locales such as `uk-UA` are accepted.
    pub fn from_tag(tag: &str) -> Option<Self> {
        let primary = tag.split(['-', '_']).next()?;
        Self::ALL
            .into_iter()
            .find(|language| primary.eq_ignore_ascii_case(language.tag()))
    }

    /// The language's name in that language, as shown in the language selector.
    pub fn native_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Ukrainian => "Українська",
        }
    }

    pub fn label(self, label: Label) -> &'static str {
        match self {
            Self::English => en::label(label),
            Self::Ukrainian => uk::label(label),
        }
    }

    pub fn message(self, message: &Message) -> String {
        match self {
            Self::English => en::message(message),
            Self::Ukrainian => uk::message(message),
        }
    }

    pub fn unit_symbol(self, units: Units) -> &'static str {
        match self {
            Self::English => en::unit_symbol(units),
            Self::Ukrainian => uk::unit_symbol(units),
        }
    }
}

/// Fixed interface text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Label {
    New,
    Open,
    Save,
    SaveAs,
    ExportSvg,
    ExportDxf,
    Editor2d,
    Preview3d,
    Language,
    UnsavedChanges,
    DiscardChanges,
    ProjectFileType,
    Tools,
    Select,
    Draw,
    Rectangle,
    Circle,
    DrawHint,
    SelectHint,
    RectangleHint,
    CircleHint,
    NavigationHint,
    SnapMillimeters,
    ZoomToFit,
    DisplayUnits,
    Pieces,
    DeletePiece,
    Layout,
    LockPiece,
    UnlockPiece,
    Camera,
    Orbit,
    Tilt,
    Zoom,
}

/// Text with arguments, kept structured so it is rendered in the language current when shown.
/// `detail` arguments come from the OS or serde and are not translated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Ready,
    NewProject,
    Opened { path: String },
    Saved { path: String },
    Exported { path: String },
    PieceAdded { name: String },
    PieceDeleted,
    PointCount { count: usize },
    DefaultPieceName { id: u64 },
    PieceNotFound { id: u64 },
    PieceLocked { name: String },
    UnsupportedVersion { found: u32, expected: u32 },
    TooManyPieces { max: usize },
    InvalidPieceId { name: String },
    PieceNameRequired { id: u64 },
    PointCountOutOfRange { name: String, max: usize },
    CoordinateOutOfRange { name: String },
    CoincidentPoints { name: String },
    ZeroArea { name: String },
    CouldNotEncode { detail: String },
    CouldNotRead { path: String, detail: String },
    CouldNotParse { path: String, detail: String },
    CouldNotWrite { path: String, detail: String },
    ExportEmpty,
}

impl fmt::Display for Message {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&en::message(self))
    }
}

impl Error for Message {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_tags_round_trip() {
        for language in Language::ALL {
            assert_eq!(Language::from_tag(language.tag()), Some(language));
        }
    }

    #[test]
    fn language_tags_match_os_locales() {
        assert_eq!(Language::from_tag("uk-UA"), Some(Language::Ukrainian));
        assert_eq!(Language::from_tag("UK_ua"), Some(Language::Ukrainian));
        assert_eq!(Language::from_tag("en-US"), Some(Language::English));
        assert_eq!(Language::from_tag("fr-FR"), None);
        assert_eq!(Language::from_tag(""), None);
    }

    #[test]
    fn ukrainian_point_counts_use_the_right_plural_form() {
        let text = |count| Language::Ukrainian.message(&Message::PointCount { count });
        assert_eq!(text(1), "1 точка");
        assert_eq!(text(2), "2 точки");
        assert_eq!(text(5), "5 точок");
        assert_eq!(text(11), "11 точок");
        assert_eq!(text(21), "21 точка");
        assert_eq!(text(22), "22 точки");
    }

    #[test]
    fn messages_display_in_english_and_render_in_the_chosen_language() {
        let message = Message::PieceNotFound { id: 7 };
        assert_eq!(message.to_string(), "Piece 7 does not exist.");
        assert_eq!(Language::Ukrainian.message(&message), "Деталь 7 не існує.");
    }
}
