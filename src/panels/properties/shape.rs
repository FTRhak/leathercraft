use super::ellipse::ellipse_fields;
use super::fields::length_field;
use super::line::line_fields;
use super::polygon::polygon_fields;
use super::rectangle::rectangle_fields;
use super::regular_polygon::regular_polygon_fields;
use super::stitch::stitch_line_fields;
use crate::editor::ShapePart;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::Units;
use leathercraft::shapes::Shape;

/// Fields for the parameters of a shape, in display units, from the file of its type. A
/// selected vertex shows the radius of its corner.
pub(super) fn shape_fields(
    ui: &mut egui::Ui,
    shape: &mut Shape,
    part: Option<ShapePart>,
    units: Units,
    language: Language,
) {
    match shape {
        Shape::Polygon(polygon) => polygon_fields(ui, polygon, part, units, language),
        Shape::Rectangle(rectangle) => rectangle_fields(ui, rectangle, units, language),
        Shape::Ellipse(ellipse) => ellipse_fields(ui, ellipse, units, language),
        Shape::RegularPolygon(polygon) => regular_polygon_fields(ui, polygon, units, language),
        Shape::Line(line) => line_fields(ui, line, units, language),
        Shape::StitchLine(line) => stitch_line_fields(ui, line, units, language),
    }
    corner_radius_field(ui, shape, part, units, language);
}

/// The radius of the selected vertex's corner. It is kept as typed; the ring reduces it where
/// the corner's edges are too short.
fn corner_radius_field(
    ui: &mut egui::Ui,
    shape: &mut Shape,
    part: Option<ShapePart>,
    units: Units,
    language: Language,
) {
    if let Some(ShapePart::Vertex(vertex)) = part
        && let Some(current) = shape.corner_radius(vertex)
    {
        ui.separator();
        ui.label(language.message(&Message::SelectedVertex { number: vertex + 1 }));
        let mut value = current;
        length_field(ui, Label::CornerRadius, &mut value, 0.0, units, language);
        if value != current
            && let Some(rounded) = shape.with_corner_radius(vertex, value)
        {
            *shape = rounded;
        }
    }
}
