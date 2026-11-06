use crate::icons::{glyph, icon_button, icon_text, paint_glyph};
use crate::panels::properties::{line_style_fields, stitch_fields};
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::{BooleanOp, Point, Units};
use leathercraft::shapes::{DEFAULT_PITCH_MM, HoleKind, LineStyle, REGULAR_POLYGON_SIDES, Shape};

const ICON_BUTTON_SIZE: f32 = 32.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Select,
    Edge,
    Draw,
    Rectangle,
    Ellipse,
    RegularPolygon,
    Centerline,
    StitchHoles,
}

impl Tool {
    const ALL: [Self; 8] = [
        Self::Select,
        Self::Edge,
        Self::Draw,
        Self::Rectangle,
        Self::Ellipse,
        Self::RegularPolygon,
        Self::Centerline,
        Self::StitchHoles,
    ];

    fn label(self) -> Label {
        match self {
            Self::Select => Label::Select,
            Self::Edge => Label::Edge,
            Self::Draw => Label::Draw,
            Self::Rectangle => Label::Rectangle,
            Self::Ellipse => Label::Ellipse,
            Self::RegularPolygon => Label::RegularPolygon,
            Self::Centerline => Label::Centerline,
            Self::StitchHoles => Label::StitchHoles,
        }
    }

    pub fn hint(self) -> Label {
        match self {
            Self::Select => Label::SelectHint,
            Self::Edge => Label::EdgeHint,
            Self::Draw => Label::DrawHint,
            Self::Rectangle => Label::RectangleHint,
            Self::Ellipse => Label::EllipseHint,
            Self::RegularPolygon => Label::RegularPolygonHint,
            Self::Centerline => Label::CenterlineHint,
            Self::StitchHoles => Label::StitchHolesHint,
        }
    }

    /// The shape dragged from `start` to `end`, for the tools that draw by dragging.
    /// `constrained` keeps an ellipse round and turns a regular polygon or a line in steps.
    pub fn shape(
        self,
        start: Point,
        end: Point,
        constrained: bool,
        sides: usize,
        line_style: LineStyle,
    ) -> Option<Shape> {
        match self {
            Self::Centerline => Shape::line_from_ends(start, end, constrained, line_style),
            Self::Rectangle => Shape::rectangle_from_corners(start, end),
            Self::Ellipse => Shape::ellipse_from_center(start, end, constrained),
            Self::RegularPolygon => {
                Shape::regular_polygon_from_center(start, end, sides, constrained)
            }
            Self::Select | Self::Edge | Self::Draw | Self::StitchHoles => None,
        }
    }

    /// Whether the tool only adds children to the selected piece, so it is offered
    /// only while a piece is selected.
    pub fn needs_piece(self) -> bool {
        matches!(self, Self::Centerline | Self::StitchHoles)
    }

    fn glyph(self) -> char {
        match self {
            Self::Select => glyph::SELECT,
            Self::Edge => glyph::EDGE,
            Self::Draw => glyph::DRAW,
            Self::Rectangle => glyph::RECTANGLE,
            Self::Ellipse => glyph::ELLIPSE,
            Self::RegularPolygon => glyph::REGULAR_POLYGON,
            Self::Centerline => glyph::CENTERLINE,
            Self::StitchHoles => glyph::STITCH_HOLES,
        }
    }

    fn paint_icon(self, painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
        paint_glyph(painter, rect, stroke, self.glyph());
    }
}

/// What new shapes do: start a piece (`None`) or combine with the selected one.
const OP_MODES: [Option<BooleanOp>; 4] = [
    None,
    Some(BooleanOp::Union),
    Some(BooleanOp::Subtract),
    Some(BooleanOp::Intersect),
];

fn op_mode_glyph(mode: Option<BooleanOp>) -> char {
    mode.map_or(glyph::NEW_PIECE, operation_glyph)
}

fn op_mode_label(mode: Option<BooleanOp>) -> Label {
    mode.map_or(Label::NewPieceMode, Label::OperationHint)
}

/// One button showing the current mode, with a small arrow; a click lists every mode.
fn op_mode_dropdown(ui: &mut egui::Ui, op_mode: &mut Option<BooleanOp>, language: Language) {
    let current = *op_mode;
    let response = icon_button(
        ui,
        ICON_BUTTON_SIZE,
        false,
        language.label(op_mode_label(current)),
        |painter, rect, stroke| {
            paint_glyph(painter, rect, stroke, op_mode_glyph(current));
            let corner = rect.right_bottom() - egui::vec2(3.0, 3.0);
            let arrow = vec![
                corner - egui::vec2(6.0, 3.0),
                corner - egui::vec2(0.0, 3.0),
                corner - egui::vec2(3.0, 0.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                arrow,
                stroke.color,
                egui::Stroke::NONE,
            ));
        },
    );
    egui::Popup::menu(&response).show(|ui| {
        let style = ui.style().clone();
        for mode in OP_MODES {
            let text = icon_text(
                &style,
                egui::TextStyle::Button,
                op_mode_glyph(mode),
                language.label(op_mode_label(mode)),
            );
            if ui.selectable_label(*op_mode == mode, text).clicked() {
                *op_mode = mode;
            }
        }
    });
}

fn operation_glyph(op: BooleanOp) -> char {
    match op {
        BooleanOp::Subtract => glyph::SUBTRACT,
        BooleanOp::Intersect => glyph::INTERSECT,
        BooleanOp::Union => glyph::UNION,
    }
}

/// The hole kind, pitch and size the Stitch holes tool draws with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StitchSettings {
    pub hole: HoleKind,
    pub pitch: f64,
    pub size: f64,
}

impl Default for StitchSettings {
    fn default() -> Self {
        let hole = HoleKind::default();
        Self {
            hole,
            pitch: DEFAULT_PITCH_MM,
            size: hole.default_size(),
        }
    }
}

/// What the tool bar edits: the tool and the settings of the tools that have any.
pub struct ToolSettings<'a> {
    pub tool: &'a mut Tool,
    pub polygon_sides: &'a mut usize,
    pub op_mode: &'a mut Option<BooleanOp>,
    /// The style of new centerlines.
    pub line_style: &'a mut LineStyle,
    /// The holes of new stitch lines.
    pub stitch: &'a mut StitchSettings,
    /// Whether a piece is selected; the tools that need one are disabled otherwise.
    pub piece_selected: bool,
}

/// The tool bar across the top of the window: tools, then the new-piece and operation modes,
/// then the settings of the current tool, all on one line that scrolls when the window is narrow.
pub fn show_toolbar(ui: &mut egui::Ui, settings: ToolSettings, units: Units, language: Language) {
    let ToolSettings {
        tool,
        polygon_sides,
        op_mode,
        line_style,
        stitch,
        piece_selected,
    } = settings;
    if tool.needs_piece() && !piece_selected {
        *tool = Tool::Select;
    }
    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal(|ui| {
            let mut tool_button = |ui: &mut egui::Ui, option: Tool| {
                let name = language.label(option.label());
                let response = icon_button(
                    ui,
                    ICON_BUTTON_SIZE,
                    *tool == option,
                    name,
                    |painter, rect, stroke| option.paint_icon(painter, rect, stroke),
                );
                if response.clicked() {
                    *tool = option;
                }
            };
            // Tools that start a piece or combine with it, then the operation mode, then the
            // tools that only add to the selected piece.
            for option in Tool::ALL.into_iter().filter(|option| !option.needs_piece()) {
                tool_button(ui, option);
            }
            ui.separator();
            op_mode_dropdown(ui, op_mode, language);
            ui.separator();
            ui.add_enabled_ui(piece_selected, |ui| {
                for option in Tool::ALL.into_iter().filter(|option| option.needs_piece()) {
                    tool_button(ui, option);
                }
            });
            match *tool {
                Tool::RegularPolygon => {
                    ui.separator();
                    ui.label(language.label(Label::Sides));
                    ui.add(egui::DragValue::new(polygon_sides).range(REGULAR_POLYGON_SIDES));
                }
                Tool::Centerline => {
                    ui.separator();
                    line_style_fields(ui, line_style, units, language);
                }
                Tool::StitchHoles => {
                    ui.separator();
                    stitch_fields(
                        ui,
                        &mut stitch.hole,
                        &mut stitch.pitch,
                        &mut stitch.size,
                        units,
                        language,
                    );
                }
                _ => {}
            }
        });
    });
}
