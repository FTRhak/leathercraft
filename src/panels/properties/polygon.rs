use super::fields::{MIN_SIZE_MM, Marker, marked_length_field, note};
use crate::editor::ShapePart;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::Units;
use leathercraft::shapes::Polygon;

/// A polygon's point count and, with an edge selected, that edge's length, marked in the color
/// of the selected edge on the canvas.
pub(super) fn polygon_fields(
    ui: &mut egui::Ui,
    polygon: &mut Polygon,
    part: Option<ShapePart>,
    units: Units,
    language: Language,
) {
    note(
        ui,
        &language.message(&Message::PointCount {
            count: polygon.points.len(),
        }),
    );
    if let Some(ShapePart::Edge(edge)) = part
        && let Some(current) = polygon.edge_length(edge)
    {
        let name = format!(
            "{} · {}",
            language.message(&Message::SelectedEdge { number: edge + 1 }),
            language.label(Label::EdgeLength)
        );
        let mut value = current;
        marked_length_field(
            ui,
            &name,
            Label::EdgeLength,
            Some(Marker::Edge),
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
