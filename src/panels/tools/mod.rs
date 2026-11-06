use crate::icons::{glyph, icon_button, paint_glyph};
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

/// What the Tools panel edits: the tool and the settings of the tools that have any.
pub struct ToolSettings<'a> {
    pub tool: &'a mut Tool,
    pub polygon_sides: &'a mut usize,
    pub op_mode: &'a mut Option<BooleanOp>,
    /// The style of new centerlines.
    pub line_style: &'a mut LineStyle,
    /// The holes of new stitch lines.
    pub stitch: &'a mut StitchSettings,
}

pub fn show_panel(ui: &mut egui::Ui, settings: ToolSettings, units: Units, language: Language) {
    let ToolSettings {
        tool,
        polygon_sides,
        op_mode,
        line_style,
        stitch,
    } = settings;
    ui.horizontal_wrapped(|ui| {
        for option in Tool::ALL {
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
        }
        ui.separator();
        let response = icon_button(
            ui,
            ICON_BUTTON_SIZE,
            op_mode.is_none(),
            language.label(Label::NewPieceMode),
            |painter, rect, stroke| paint_glyph(painter, rect, stroke, glyph::NEW_PIECE),
        );
        if response.clicked() {
            *op_mode = None;
        }
        for op in BooleanOp::ALL {
            let response = icon_button(
                ui,
                ICON_BUTTON_SIZE,
                *op_mode == Some(op),
                language.label(Label::OperationHint(op)),
                |painter, rect, stroke| paint_glyph(painter, rect, stroke, operation_glyph(op)),
            );
            if response.clicked() {
                *op_mode = Some(op);
            }
        }
    });
    if *tool == Tool::RegularPolygon {
        ui.horizontal(|ui| {
            ui.label(language.label(Label::Sides));
            ui.add(egui::DragValue::new(polygon_sides).range(REGULAR_POLYGON_SIDES));
        });
    }
    if *tool == Tool::Centerline {
        line_style_fields(ui, line_style, units, language);
    }
    if *tool == Tool::StitchHoles {
        stitch_fields(
            ui,
            &mut stitch.hole,
            &mut stitch.pitch,
            &mut stitch.size,
            units,
            language,
        );
    }
}
