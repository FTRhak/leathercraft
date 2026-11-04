use super::fields::{MIN_SIZE_MM, length_field};
use crate::editor::ShapePart;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::Units;
use leathercraft::shapes::Polygon;

/// A polygon's point count and, with an edge selected, that edge's length.
pub(super) fn polygon_fields(
    ui: &mut egui::Ui,
    polygon: &mut Polygon,
    part: Option<ShapePart>,
    units: Units,
    language: Language,
) {
    ui.label(language.message(&Message::PointCount {
        count: polygon.points.len(),
    }));
    if let Some(ShapePart::Edge(edge)) = part
        && let Some(current) = polygon.edge_length(edge)
    {
        ui.label(language.message(&Message::SelectedEdge { number: edge + 1 }));
        let mut value = current;
        length_field(
            ui,
            Label::EdgeLength,
            &mut value,
            MIN_SIZE_MM,
            units,
            language,
        );
        if value != current
            && let Some(resized) = polygon.with_edge_length(edge, value)
        {
            *polygon = resized;
        }
    }
}
