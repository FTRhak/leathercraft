mod en;
mod uk;

use crate::align::Arrange;
use crate::background::{Depth, FrameMethod};
use crate::material::{
    ColorGroup, Flexibility, HeatResistance, LeatherColor, Origin, Strength, Texture,
    ThicknessCategory, Treatment, WaterResistance,
};
use crate::model::{BooleanOp, Units};
use crate::sewing::{Assembly, StitchMethod, ThreadKind, ThreadShade};
use crate::shapes::{HoleKind, StitchBand};
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
    File,
    View,
    Edit,
    Window,
    /// The menu of pattern commands in the title bar.
    Pattern,
    MinimizeWindow,
    MaximizeWindow,
    RestoreWindow,
    CloseWindow,
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
    Select,
    Draw,
    Rectangle,
    Ellipse,
    Polygon,
    RegularPolygon,
    Edge,
    DrawHint,
    SelectHint,
    RectangleHint,
    EllipseHint,
    RegularPolygonHint,
    EdgeHint,
    Centerline,
    CenterlineHint,
    LineType,
    LineSolid,
    LineDashed,
    LineDashDot,
    DashLength,
    GapLength,
    StitchHoles,
    StitchHolesHint,
    HoleType,
    HoleKind(HoleKind),
    Pitch,
    PitchPresets,
    HoleSize,
    StitchLength,
    HoleCount,
    ActualPitch,
    StartX,
    StartY,
    EndX,
    EndY,
    Angle,
    /// Drawing tools create a new piece.
    NewPieceMode,
    /// Drawing tools add a child with this operation to the selected piece.
    Operation(BooleanOp),
    OperationHint(BooleanOp),
    Base,
    CenterX,
    CenterY,
    Width,
    Height,
    RadiusX,
    RadiusY,
    Radius,
    Sides,
    Rotation,
    EdgeLength,
    CornerRadius,
    ToPolygon,
    MoveUp,
    MoveDown,
    DeleteChild,
    NavigationHint,
    /// The title of the snap popover on the canvas.
    Snap,
    SnapEnabled,
    SnapGrid,
    /// The caption of the target list in the snap popover, and the tooltip of its button.
    SnapTargets,
    SnapGridStep,
    SnapVertices,
    SnapVerticesHint,
    SnapGuides,
    SnapGuidesHint,
    SnapDistance,
    SnapAltHint,
    /// The hotkey action that turns snapping on or off.
    ToggleSnap,
    ZoomToFit,
    ZoomToSelection,
    /// The label of the Zoom to selection button of the canvas view controls.
    FitSelection,
    ZoomIn,
    ZoomOut,
    /// The tooltip of the zoom percentage, which sets the actual size when clicked.
    ZoomActualSize,
    DisplayUnits,
    /// The title of the hint box at the top of the Properties panel.
    CanvasControls,
    /// The section header of the selected piece in the Properties panel.
    PieceParameters,
    /// The hint of the piece name field.
    PieceName,
    /// One-letter prefixes of the piece's width and height boxes.
    WidthShort,
    HeightShort,
    Pieces,
    DeletePiece,
    Layout,
    LockPiece,
    UnlockPiece,
    Material,
    Canvas,
    Properties,
    ClosePanel,
    FloatPanel,
    ResetPanels,
    History,
    /// Beside the current entry of the History panel.
    HistoryCurrent,
    /// Beside an undone entry of the History panel.
    HistoryUndone,
    Align,
    AlignObjects,
    DistributeObjects,
    DistributeSpacing,
    /// An Align panel button.
    Arrange(Arrange),
    AlignTo,
    AlignToSelection,
    AlignToKeyObject,
    AlignToProject,
    AlignToParent,
    KeyObjectHint,
    KeyShapeHint,
    ParentHint,
    Spacing,
    SpacingAuto,
    Undo,
    Redo,
    SelectAll,
    /// The Edit menu item that opens the Settings window.
    Settings,
    SettingsTitle,
    DefaultMaterial,
    Hotkeys,
    /// The Settings tab with the theme.
    Appearance,
    ThemeField,
    ThemeDark,
    ThemeLight,
    /// Shown on a shortcut button while it waits for keys.
    PressKeys,
    /// The hint of the search field above the Hotkeys list.
    SearchHotkeys,
    AddShortcut,
    RemoveShortcut,
    ResetToDefaults,
    ShortcutConflict,
    Ok,
    DeleteSelection,
    ToggleView,
    NoPieceSelected,
    Camera,
    Orbit,
    Tilt,
    Zoom,
    PreviewNavigationHint,
    /// The title above the preview's Sewn / Flat switch.
    PreviewShows,
    PreviewSewn,
    PreviewFlat,
    /// Starts the sewn preview's simulation over.
    SewAgain,
    SewAgainHint,
    NewProjectTitle,
    /// What the default material is for, under the material card of the new-project dialog.
    DefaultMaterialHint,
    Create,
    Cancel,
    BasicProperties,
    PhysicalProperties,
    OriginField,
    TreatmentField,
    ThicknessField,
    ColorField,
    TextureField,
    StrengthField,
    FlexibilityField,
    WaterResistanceField,
    HeatResistanceField,
    Origin(Origin),
    OriginHint(Origin),
    Treatment(Treatment),
    TreatmentHint(Treatment),
    Thickness(ThicknessCategory),
    ColorGroup(ColorGroup),
    Color(LeatherColor),
    Texture(Texture),
    Strength(Strength),
    Flexibility(Flexibility),
    FlexibilityHint(Flexibility),
    WaterResistance(WaterResistance),
    WaterResistanceHint(WaterResistance),
    HeatResistance(HeatResistance),
    /// The Pattern menu item and Properties button that open the Link seams dialog.
    LinkSeams,
    LinkSeamsTitle,
    LinkSeamsSubtitle,
    SeamA,
    SeamB,
    /// Before the first hole of a seam's range.
    HolesFrom,
    /// Before the last hole of a seam's range.
    HolesTo,
    Reverse,
    StartOffset,
    /// The unit after the start offset: holes, shortened.
    HolesShort,
    LinkSchematic,
    HolePairs,
    /// The legend of automatic pairs on the schematic.
    PairAuto,
    /// The badge of a manual pair in the table.
    PairManual,
    /// The legend of manual pairs on the schematic.
    PairManualLegend,
    /// Holes without a pair: the legend, the table badge and the summary tile.
    PairFree,
    LinkHint,
    ResetManualPairs,
    /// The summary tile with the number of pairs.
    Pairs,
    LengthDifference,
    /// The section of a stitch line's links in Properties.
    SeamLinks,
    EditLink,
    RemoveLink,
    /// A link whose range runs past its line's holes, which were cut short.
    SeamLinkStale,
    /// The card of the stitch in the Link seams dialog.
    StitchMethodTitle,
    StitchMethod(StitchMethod),
    /// Under a stitch's name on its tile, such as how many needles it takes.
    StitchMethodNote(StitchMethod),
    /// Beside the card's title: what the chosen stitch is good for.
    StitchMethodHint(StitchMethod),
    /// The card of how the pieces are laid together.
    AssemblyTitle,
    Assembly(Assembly),
    AssemblyNote(Assembly),
    /// The holes sewn back over at each end of the seam.
    LockHoles,
    Overlap,
    /// Which side lies on top.
    OnTop,
    /// The card of the thread in the Link seams dialog, and its fields.
    ThreadTitle,
    ThreadKindCaption,
    ThreadKind(ThreadKind),
    ThreadDiameter,
    ThreadColorCaption,
    ThreadShade(ThreadShade),
    /// A thread color that isn't one of the named shades.
    ThreadCustomColor,
    /// The Background images panel: its title and the Window menu item.
    BackgroundImages,
    AddBackgroundImage,
    /// Section headers of a background image's entry.
    ImageSource,
    ImageDisplay,
    ImagePlacement,
    Opacity,
    ImageDepth,
    ImageDepthOption(Depth),
    FrameMethodField,
    FrameMethod(FrameMethod),
    Offset,
    Scale,
    Flip,
    /// Starts calibration: two points on the image and the real distance between them.
    Calibrate,
    CalibrateHint,
    /// The field of the real distance in the calibration window.
    CalibrateDistance,
    /// The file dialog filter of image files.
    ImageFileType,
    /// After the name of an image whose file can't be read.
    ImageNotFound,
    BackgroundEmptyHint,
    ShowImage,
    HideImage,
    LockImage,
    UnlockImage,
    RemoveImage,
    BrowseImage,
    ReloadImage,
    /// The toggle that keeps the width and height in proportion.
    KeepAspect,
}

/// Text with arguments, kept structured so it is rendered in the language current when shown.
/// `detail` arguments come from the OS or serde and are not translated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Ready,
    NewProject,
    Opened {
        path: String,
    },
    Saved {
        path: String,
    },
    Exported {
        path: String,
    },
    PieceAdded {
        name: String,
    },
    PieceDeleted,
    PiecesDeleted {
        count: usize,
    },
    ChildAdded {
        name: String,
    },
    ChildDeleted,
    ChildrenDeleted {
        count: usize,
    },
    SelectPieceForChild,
    ChildNotFound {
        name: String,
    },
    TooManyChildren {
        name: String,
        max: usize,
    },
    InvalidChildId {
        name: String,
    },
    InvalidShapeSize {
        name: String,
    },
    BaseMustBeClosed {
        name: String,
    },
    TooManyDashes {
        name: String,
        max: usize,
    },
    InvalidHoleParameters {
        name: String,
    },
    HolesOverlap {
        name: String,
    },
    TooManyHoles {
        name: String,
        max: usize,
    },
    StitchHolesRow {
        count: usize,
    },
    StitchVertices {
        count: usize,
    },
    ThreadHint {
        band: StitchBand,
    },
    LeatherOutsideBand {
        band: StitchBand,
    },
    ResultEmpty {
        name: String,
    },
    ResultSplit {
        name: String,
    },
    PointCount {
        count: usize,
    },
    /// The number of pieces, beside the root of the Layout tree.
    PieceCount {
        count: usize,
    },
    SelectedEdge {
        number: usize,
    },
    /// The selected piece in the status bar: its shapes, base included, and stitch holes.
    SelectedPiece {
        name: String,
        shapes: usize,
        holes: usize,
    },
    /// Several selected pieces in the status bar.
    SelectedPieces {
        count: usize,
    },
    /// Several selected shapes of one piece in the status bar.
    SelectedShapes {
        name: String,
        count: usize,
    },
    SelectedVertex {
        number: usize,
    },
    /// Under the material card of the Material panel.
    PieceMaterial {
        name: String,
    },
    DefaultPieceName {
        id: u64,
    },
    PieceNotFound {
        id: u64,
    },
    PieceLocked {
        name: String,
    },
    AlreadyArranged,
    ShapesAlreadyArranged,
    UnsupportedVersion {
        found: u32,
        expected: u32,
    },
    TooManyPieces {
        max: usize,
    },
    InvalidPieceId {
        name: String,
    },
    PieceNameRequired {
        id: u64,
    },
    PointCountOutOfRange {
        name: String,
        max: usize,
    },
    CoordinateOutOfRange {
        name: String,
    },
    CoincidentPoints {
        name: String,
    },
    ZeroArea {
        name: String,
    },
    ThicknessOutOfRange {
        name: String,
    },
    DefaultThicknessOutOfRange,
    CouldNotEncode {
        detail: String,
    },
    CouldNotRead {
        path: String,
        detail: String,
    },
    CouldNotParse {
        path: String,
        detail: String,
    },
    CouldNotWrite {
        path: String,
        detail: String,
    },
    ExportEmpty,
    Undone {
        action: Box<Message>,
    },
    Redone {
        action: Box<Message>,
    },
    HistoryRestored {
        action: Box<Message>,
    },
    // Entries of the History panel.
    ActionNewProject,
    ActionOpened {
        name: String,
    },
    ActionUnits,
    ActionAddPiece {
        name: String,
    },
    ActionRenamePiece {
        name: String,
    },
    ActionEditBase {
        name: String,
    },
    ActionMovePiece {
        name: String,
    },
    ActionMovePieces {
        count: usize,
    },
    ActionAlignPieces {
        count: usize,
    },
    ActionDistributePieces {
        count: usize,
    },
    ActionLockPiece {
        name: String,
    },
    ActionUnlockPiece {
        name: String,
    },
    ActionMaterial {
        name: String,
    },
    ActionRemovePiece {
        name: String,
    },
    ActionRemovePieces {
        count: usize,
    },
    ActionAddChild {
        name: String,
    },
    ActionEditChild {
        name: String,
    },
    ActionSetChildOp {
        name: String,
    },
    ActionReorderChild {
        name: String,
    },
    ActionRemoveChild {
        name: String,
    },
    ActionMoveChildren {
        name: String,
        count: usize,
    },
    ActionAlignChildren {
        name: String,
        count: usize,
    },
    ActionDistributeChildren {
        name: String,
        count: usize,
    },
    ActionRemoveChildren {
        name: String,
        count: usize,
    },
    ActionLinkSeams {
        a: String,
        b: String,
    },
    ActionEditSeamLink {
        a: String,
        b: String,
    },
    ActionRemoveSeamLink {
        a: String,
        b: String,
    },
    // Background images.
    ActionAddBackgroundImage {
        name: String,
    },
    ActionEditBackgroundImage {
        name: String,
    },
    ActionRemoveBackgroundImage {
        name: String,
    },
    ActionReorderBackgroundImages,
    BackgroundImageNotFound,
    InvalidBackgroundImage {
        name: String,
    },
    InvalidBackgroundImageId {
        name: String,
    },
    /// `detail` comes from the image decoder and is not translated.
    ImageLoadFailed {
        path: String,
        detail: String,
    },
    /// The pixel size and file size of an image, under its path.
    ImageInfo {
        width: u32,
        height: u32,
        kilobytes: u64,
    },
    CalibrateFirstPoint,
    CalibrateSecondPoint,
    // Seam links.
    SeamNotFound,
    InvalidSeamRange,
    SeamRangesOverlap,
    DuplicateManualPair,
    DuplicateSeamLink,
    /// The title of the warning when the ranges hold different numbers of holes.
    HoleCountMismatch {
        a: usize,
        b: usize,
    },
    /// Under that title: how many holes are left without a pair, on one side or on both.
    UnpairedHoles {
        side: Option<char>,
        count: usize,
    },
    /// A button that sets a side's pitch so that the counts match; `pitch` is formatted.
    FitPitch {
        side: char,
        pitch: String,
    },
    LinkPairsButton {
        count: usize,
    },
    /// A stitch line by its piece and its number among the piece's stitch lines.
    SeamName {
        piece: String,
        index: usize,
    },
    /// A stitch line in the seam lists, with its hole count.
    SeamOption {
        piece: String,
        index: usize,
        holes: usize,
    },
    /// The hole count beside a seam's title.
    HoleCountBadge {
        count: usize,
    },
    /// The number of pairs beside the table title.
    PairsCount {
        count: usize,
    },
    /// The summary tile of unpaired holes: on one side, or on both.
    FreeIn {
        side: Option<char>,
    },
    /// A link in the Properties of one of its seams: the other seam and the number of pairs.
    SeamLinkRow {
        other: String,
        pairs: usize,
    },
    LockHolesOutOfRange,
    OverlapOutOfRange,
    ThreadDiameterOutOfRange,
    /// Beside the assembly card's title: how the chosen assembly lays the sides, `top` being
    /// the side on top for those that stack.
    AssemblyHint {
        assembly: Assembly,
        top: char,
        under: char,
    },
    /// The thread a seam takes: its formatted length and the stitch's factor, in tenths.
    ThreadEstimate {
        length: String,
        factor_tenths: u32,
    },
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
    fn piece_counts_use_the_right_plural_form() {
        let uk = |count| Language::Ukrainian.message(&Message::PieceCount { count });
        assert_eq!(uk(1), "1 деталь");
        assert_eq!(uk(2), "2 деталі");
        assert_eq!(uk(5), "5 деталей");
        let en = |count| Language::English.message(&Message::PieceCount { count });
        assert_eq!(en(1), "1 piece");
        assert_eq!(en(3), "3 pieces");
    }

    #[test]
    fn selected_piece_lists_its_shapes_and_holes() {
        let piece = |shapes, holes| Message::SelectedPiece {
            name: "Wallet".to_owned(),
            shapes,
            holes,
        };
        let en = |message| Language::English.message(&message);
        assert_eq!(en(piece(1, 0)), "Selected: Wallet (1 shape)");
        assert_eq!(
            en(piece(2, 48)),
            "Selected: Wallet (2 shapes · 48 stitch holes)"
        );
        let uk = |message| Language::Ukrainian.message(&message);
        assert_eq!(uk(piece(1, 0)), "Обрано: Wallet (1 фігура)");
        assert_eq!(
            uk(piece(2, 48)),
            "Обрано: Wallet (2 фігури · 48 отворів шва)"
        );
    }

    #[test]
    fn seam_link_counts_use_the_right_plural_form() {
        let uk = |message| Language::Ukrainian.message(&message);
        assert_eq!(
            uk(Message::LinkPairsButton { count: 48 }),
            "Зв’язати 48 пар"
        );
        assert_eq!(uk(Message::LinkPairsButton { count: 2 }), "Зв’язати 2 пари");
        assert_eq!(uk(Message::HoleCountBadge { count: 52 }), "52 отвори");
        assert_eq!(
            uk(Message::UnpairedHoles {
                side: Some('B'),
                count: 3
            }),
            "3 отвори шва B лишаються без пари. Вирівняйте крок, щоб кількості збіглися."
        );
        assert!(
            uk(Message::UnpairedHoles {
                side: None,
                count: 1
            })
            .starts_with("1 отвір лишається")
        );
        let en = |message| Language::English.message(&message);
        assert_eq!(en(Message::LinkPairsButton { count: 1 }), "Link 1 pair");
        assert_eq!(
            en(Message::ActionLinkSeams {
                a: "Flap".to_owned(),
                b: "Flap".to_owned()
            }),
            "Linked seams of 'Flap'"
        );
    }

    #[test]
    fn thickness_limits_use_the_decimal_separator_of_the_language() {
        let message = Message::DefaultThicknessOutOfRange;
        assert!(Language::English.message(&message).contains("0.3"));
        assert!(Language::Ukrainian.message(&message).contains("0,3"));
    }

    #[test]
    fn messages_display_in_english_and_render_in_the_chosen_language() {
        let message = Message::PieceNotFound { id: 7 };
        assert_eq!(message.to_string(), "Piece 7 does not exist.");
        assert_eq!(Language::Ukrainian.message(&message), "Деталь 7 не існує.");
    }
}
