use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Point;
use leathercraft::shapes;

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
}

pub fn show_panel(ui: &mut egui::Ui, tool: &mut Tool, language: Language) {
    ui.label(language.label(Label::Tools));
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
        for option in Tool::ALL {
            ui.selectable_value(tool, option, language.label(option.label()));
        }
    });
}
