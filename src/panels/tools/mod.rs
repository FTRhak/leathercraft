use crate::icons::icon_button;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::{BooleanOp, Point};
use leathercraft::shapes::{REGULAR_POLYGON_SIDES, Shape};
use std::f32::consts::{PI, TAU};

const ICON_BUTTON_SIZE: f32 = 32.0;
const ICON_MARGIN: f32 = 8.0;
const ICON_VERTEX_RADIUS: f32 = 2.0;
const ICON_ARC_SEGMENTS: usize = 48;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Select,
    Edge,
    Draw,
    Rectangle,
    Ellipse,
    RegularPolygon,
}

impl Tool {
    const ALL: [Self; 6] = [
        Self::Select,
        Self::Edge,
        Self::Draw,
        Self::Rectangle,
        Self::Ellipse,
        Self::RegularPolygon,
    ];

    fn label(self) -> Label {
        match self {
            Self::Select => Label::Select,
            Self::Edge => Label::Edge,
            Self::Draw => Label::Draw,
            Self::Rectangle => Label::Rectangle,
            Self::Ellipse => Label::Ellipse,
            Self::RegularPolygon => Label::RegularPolygon,
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
        }
    }

    /// The shape dragged from `start` to `end`, for the tools that draw by dragging.
    /// `constrained` keeps an ellipse round and turns a regular polygon in steps.
    pub fn shape(self, start: Point, end: Point, constrained: bool, sides: usize) -> Option<Shape> {
        match self {
            Self::Rectangle => Shape::rectangle_from_corners(start, end),
            Self::Ellipse => Shape::ellipse_from_center(start, end, constrained),
            Self::RegularPolygon => {
                Shape::regular_polygon_from_center(start, end, sides, constrained)
            }
            Self::Select | Self::Edge | Self::Draw => None,
        }
    }

    fn paint_icon(self, painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
        let rect = rect.shrink(ICON_MARGIN);
        let at = |x: f32, y: f32| rect.lerp_inside(egui::vec2(x, y));
        match self {
            Self::Select => {
                let arrow = vec![
                    at(0.1, 0.0),
                    at(0.1, 0.8),
                    at(0.35, 0.6),
                    at(0.55, 1.0),
                    at(0.7, 0.92),
                    at(0.5, 0.52),
                    at(0.85, 0.52),
                ];
                painter.add(egui::Shape::closed_line(arrow, stroke));
            }
            Self::Edge => {
                let outline = vec![at(0.0, 0.85), at(0.15, 0.1), at(0.85, 0.0), at(1.0, 0.75)];
                let faded = egui::Stroke::new(1.0, stroke.color.gamma_multiply(0.35));
                painter.add(egui::Shape::closed_line(outline.clone(), faded));
                let edge = [outline[3], outline[0]];
                painter.line_segment(edge, egui::Stroke::new(stroke.width * 2.0, stroke.color));
                for vertex in edge {
                    painter.circle_filled(vertex, ICON_VERTEX_RADIUS, stroke.color);
                }
            }
            Self::Draw => {
                let vertices = vec![at(0.0, 1.0), at(0.3, 0.2), at(0.7, 0.7), at(1.0, 0.0)];
                for vertex in &vertices {
                    painter.circle_filled(*vertex, ICON_VERTEX_RADIUS, stroke.color);
                }
                painter.add(egui::Shape::line(vertices, stroke));
            }
            Self::Rectangle => {
                painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Middle);
            }
            Self::Ellipse => {
                let center = rect.center();
                let points = (0..ICON_ARC_SEGMENTS)
                    .map(|index| {
                        let angle = TAU * index as f32 / ICON_ARC_SEGMENTS as f32;
                        center
                            + egui::vec2(
                                angle.cos() * rect.width() / 2.0,
                                angle.sin() * rect.height() / 3.0,
                            )
                    })
                    .collect();
                painter.add(egui::Shape::closed_line(points, stroke));
            }
            Self::RegularPolygon => {
                let center = rect.center();
                let radius = rect.width() / 2.0;
                let points = (0..6)
                    .map(|index| center + egui::Vec2::angled(TAU * index as f32 / 6.0) * radius)
                    .collect();
                painter.add(egui::Shape::closed_line(points, stroke));
            }
        }
    }
}

/// The arc of a circle that lies inside, or outside, a second circle of the same radius.
/// It starts opposite the part it drops, so that the points it keeps are contiguous.
fn arc(center: egui::Pos2, other: egui::Pos2, radius: f32, inside: bool) -> Vec<egui::Pos2> {
    let toward = (other - center).angle();
    let start = if inside { toward + PI } else { toward };
    (0..=ICON_ARC_SEGMENTS)
        .map(|index| {
            let angle = start + TAU * index as f32 / ICON_ARC_SEGMENTS as f32;
            center + egui::Vec2::angled(angle) * radius
        })
        .filter(|point| (point.distance(other) < radius) == inside)
        .collect()
}

/// Two overlapping circles with the outline of the operation's result drawn in full and the
/// rest faded: the left circle is the piece and the right one the new shape.
fn paint_operation_icon(
    painter: &egui::Painter,
    rect: egui::Rect,
    stroke: egui::Stroke,
    op: BooleanOp,
) {
    let rect = rect.shrink(ICON_MARGIN - 2.0);
    let radius = rect.width() * 0.32;
    let piece = rect.center() - egui::vec2(rect.width() * 0.18, 0.0);
    let shape = rect.center() + egui::vec2(rect.width() * 0.18, 0.0);
    // Whether the result keeps the arcs of each circle that lie inside the other one.
    let (piece_inside, shape_inside) = match op {
        BooleanOp::Subtract => (false, true),
        BooleanOp::Union => (false, false),
        BooleanOp::Intersect => (true, true),
    };
    let faded = egui::Stroke::new(1.0, stroke.color.gamma_multiply(0.35));
    for (center, other, inside) in [(piece, shape, piece_inside), (shape, piece, shape_inside)] {
        painter.add(egui::Shape::line(
            arc(center, other, radius, !inside),
            faded,
        ));
        painter.add(egui::Shape::line(
            arc(center, other, radius, inside),
            stroke,
        ));
    }
}

fn paint_new_piece_icon(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let rect = rect.shrink(ICON_MARGIN);
    painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Middle);
    let center = rect.center();
    let arm = rect.width() * 0.25;
    painter.line_segment(
        [center - egui::vec2(arm, 0.0), center + egui::vec2(arm, 0.0)],
        stroke,
    );
    painter.line_segment(
        [center - egui::vec2(0.0, arm), center + egui::vec2(0.0, arm)],
        stroke,
    );
}

pub fn show_panel(
    ui: &mut egui::Ui,
    tool: &mut Tool,
    polygon_sides: &mut usize,
    op_mode: &mut Option<BooleanOp>,
    language: Language,
) {
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
            paint_new_piece_icon,
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
                |painter, rect, stroke| paint_operation_icon(painter, rect, stroke, op),
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
}
