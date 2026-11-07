use crate::panels::properties::{line_style_fields, stitch_fields};
use crate::settings::{Action, Hotkeys};
use crate::ui_components::icons::glyph;
use crate::ui_components::palette::Palette;
use crate::ui_components::tool_button::ToolButton;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::{BooleanOp, Point, Units};
use leathercraft::shapes::{DEFAULT_PITCH_MM, HoleKind, LineStyle, REGULAR_POLYGON_SIDES, Shape};

/// The height of the bar, including its margins.
const HEIGHT: f32 = 44.0;
const MARGIN: egui::Margin = egui::Margin::symmetric(14, 6);
/// The height of the buttons, and so of the row inside the margins.
const ROW_HEIGHT: f32 = 32.0;
/// Between the groups of tools, on both sides of their dividers.
const GROUP_GAP: f32 = 8.0;
/// Between the buttons of one group.
const BUTTON_GAP: f32 = 4.0;
const DIVIDER_HEIGHT: f32 = 22.0;
/// The width of the zoom buttons at the end of the bar.
const ZOOM_GROUP_WIDTH: f32 = 2.0 * 32.0 + BUTTON_GAP;

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

    /// The color of the mark the tool leaves on the canvas, shown as the dot of its chip, for
    /// the tools named by their mark rather than an icon.
    fn mark(self, palette: &Palette) -> Option<egui::Color32> {
        match self {
            Self::Edge => Some(palette.edge),
            Self::StitchHoles => Some(palette.hole),
            Self::Centerline => Some(palette.centerline),
            _ => None,
        }
    }

    /// The hotkey action that switches to the tool.
    fn action(self) -> Option<Action> {
        Action::ALL
            .into_iter()
            .find(|action| action.tool() == Some(self))
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
    /// Set when a zoom button is clicked.
    pub zoom_request: &'a mut Option<ZoomRequest>,
}

/// What the canvas zooms to on its next frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoomRequest {
    /// Every piece.
    All,
    /// The selected children, or else the selected pieces.
    Selection,
}

/// Paints corner brackets around a small filled square, the icon of Zoom to selection.
fn paint_fit_selection_icon(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    paint_fit_icon(painter, rect, stroke);
    let square =
        egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(rect.width() * 0.26));
    painter.rect_filled(square, 1.0, stroke.color);
}

/// Paints four corner brackets, the icon of Zoom to fit.
fn paint_fit_icon(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let frame = rect.shrink(rect.width() * 0.14);
    let arm = frame.width() * 0.35;
    for (corner, dx, dy) in [
        (frame.left_top(), arm, arm),
        (frame.right_top(), -arm, arm),
        (frame.left_bottom(), arm, -arm),
        (frame.right_bottom(), -arm, -arm),
    ] {
        painter.line(
            vec![
                corner + egui::vec2(dx, 0.0),
                corner,
                corner + egui::vec2(0.0, dy),
            ],
            stroke,
        );
    }
}

/// The short vertical line between two groups of tools.
/// The buttons of a group are `BUTTON_GAP` apart, and the divider is `GROUP_GAP` from them.
fn divider(ui: &mut egui::Ui) {
    ui.add_space(GROUP_GAP - BUTTON_GAP);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, DIVIDER_HEIGHT), egui::Sense::hover());
    ui.add_space(GROUP_GAP - BUTTON_GAP);
    let color = Palette::of(ui.painter()).toolbar.divider;
    ui.painter().rect_filled(rect, 0.0, color);
}

/// The tool bar under the title bar, from the "CAD Toolbar" of the design: the selection and
/// drawing tools, the shape tools, the tools named by their mark (edge, stitch holes,
/// centerline), the new-piece and operation modes, and the settings of the current tool; then
/// Zoom to fit and Zoom to selection at the far end. It scrolls when the window is narrow, and
/// is disabled rather than hidden when `enabled` is false, so the layout doesn't jump.
pub fn show_toolbar(
    ctx: &egui::Context,
    enabled: bool,
    settings: ToolSettings,
    hotkeys: &Hotkeys,
    units: Units,
    language: Language,
) {
    let colors = &Palette::of(&ctx.layer_painter(egui::LayerId::background())).toolbar;
    let frame = egui::Frame::new()
        .fill(colors.background)
        .inner_margin(MARGIN);
    egui::TopBottomPanel::top("tool_bar")
        .exact_height(HEIGHT)
        .resizable(false)
        .show_separator_line(false)
        .frame(frame)
        .show(ctx, |ui| {
            let bottom = ui.max_rect().bottom() + f32::from(MARGIN.bottom);
            ui.add_enabled_ui(enabled, |ui| {
                egui::ScrollArea::horizontal()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        // One row of the buttons' height, so that the dividers and the tool
                        // settings center on the buttons.
                        let row = egui::vec2(ui.available_width(), ROW_HEIGHT);
                        let layout = egui::Layout::left_to_right(egui::Align::Center);
                        ui.allocate_ui_with_layout(row, layout, |ui| {
                            ui.set_min_height(ROW_HEIGHT);
                            toolbar_contents(ui, settings, hotkeys, units, language);
                        });
                    });
            });
            ui.painter().hline(
                ui.clip_rect().x_range(),
                bottom - 0.5,
                egui::Stroke::new(1.0, colors.border),
            );
        });
}

fn toolbar_contents(
    ui: &mut egui::Ui,
    settings: ToolSettings,
    hotkeys: &Hotkeys,
    units: Units,
    language: Language,
) {
    let ToolSettings {
        tool,
        polygon_sides,
        op_mode,
        line_style,
        stitch,
        piece_selected,
        zoom_request,
    } = settings;
    if tool.needs_piece() && !piece_selected {
        *tool = Tool::Select;
    }
    // One row for the whole bar: rows nested in it would start at the default row height and
    // grow down from there, off the center line.
    ui.spacing_mut().item_spacing.x = BUTTON_GAP;
    let palette = Palette::of(ui.painter());
    let shortcut = |ui: &egui::Ui, action: Option<Action>| {
        action
            .and_then(|action| hotkeys.first(action))
            .map(|shortcut| ui.ctx().format_shortcut(shortcut))
    };
    let tool_button = |ui: &mut egui::Ui, tool: &mut Tool, option: Tool| {
        let name = language.label(option.label());
        let button = match option.mark(palette) {
            Some(dot) => ToolButton::chip(dot, name),
            None => ToolButton::glyph(option.glyph(), name),
        };
        let button = button
            .selected(*tool == option)
            .shortcut(shortcut(ui, option.action()));
        let enabled = piece_selected || !option.needs_piece();
        if ui.add_enabled(enabled, button).clicked() {
            *tool = option;
        }
    };

    {
        for option in [Tool::Select, Tool::Draw] {
            tool_button(ui, tool, option);
        }
    }
    divider(ui);
    {
        for option in [Tool::Rectangle, Tool::Ellipse, Tool::RegularPolygon] {
            tool_button(ui, tool, option);
        }
    }
    divider(ui);
    {
        for option in [Tool::Edge, Tool::StitchHoles, Tool::Centerline] {
            tool_button(ui, tool, option);
        }
    }
    divider(ui);
    {
        for mode in OP_MODES {
            let button =
                ToolButton::glyph(op_mode_glyph(mode), language.label(op_mode_label(mode)))
                    .selected(*op_mode == mode);
            if ui.add(button).clicked() {
                *op_mode = mode;
            }
        }
    }
    match *tool {
        Tool::RegularPolygon => {
            divider(ui);
            ui.label(language.label(Label::Sides));
            ui.add(egui::DragValue::new(polygon_sides).range(REGULAR_POLYGON_SIDES));
        }
        Tool::Centerline => {
            divider(ui);
            line_style_fields(ui, line_style, units, language);
        }
        Tool::StitchHoles => {
            divider(ui);
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

    // Pushes the zoom buttons to the far end while there is room; past that they follow the
    // rest and the bar scrolls.
    let room = ui.available_width() - ZOOM_GROUP_WIDTH - BUTTON_GAP;
    if room > 0.0 {
        ui.add_space(room);
    } else {
        divider(ui);
    }
    {
        let fit = ToolButton::icon(paint_fit_icon, language.label(Label::ZoomToFit));
        if ui.add(fit).clicked() {
            *zoom_request = Some(ZoomRequest::All);
        }
        let fit_selection = ToolButton::icon(
            paint_fit_selection_icon,
            language.label(Label::ZoomToSelection),
        );
        if ui.add_enabled(piece_selected, fit_selection).clicked() {
            *zoom_request = Some(ZoomRequest::Selection);
        }
    }
}
