use super::icon_button;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Point;
use leathercraft::shapes;

const ICON_BUTTON_SIZE: f32 = 32.0;
const ICON_MARGIN: f32 = 8.0;
const ICON_VERTEX_RADIUS: f32 = 2.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Select,
    Draw,
    Rectangle,
    Circle,
}

impl Tool {
    const ALL: [Self; 4] = [Self::Select, Self::Draw, Self::Rectangle, Self::Circle];

    fn label(self) -> Label {
        match self {
            Self::Select => Label::Select,
            Self::Draw => Label::Draw,
            Self::Rectangle => Label::Rectangle,
            Self::Circle => Label::Circle,
        }
    }

    pub fn hint(self) -> Label {
        match self {
            Self::Select => Label::SelectHint,
            Self::Draw => Label::DrawHint,
            Self::Rectangle => Label::RectangleHint,
            Self::Circle => Label::CircleHint,
        }
    }

    /// The shape dragged from `start` to `end`, for the tools that draw by dragging.
    pub fn shape_points(self, start: Point, end: Point) -> Option<Vec<Point>> {
        match self {
            Self::Rectangle => shapes::rectangle(start, end),
            Self::Circle => shapes::circle(start, end),
            Self::Select | Self::Draw => None,
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
            Self::Circle => {
                painter.circle_stroke(rect.center(), rect.width() / 2.0, stroke);
            }
        }
    }
}

pub fn show_panel(ui: &mut egui::Ui, tool: &mut Tool, language: Language) {
    ui.label(language.label(Label::Tools));
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
    });
}
