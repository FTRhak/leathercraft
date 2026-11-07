use super::fields::{
    FIELD_WIDTH, MIN_SIZE_MM, angle_field, center_fields, length_field, param_row,
};
use crate::ui_components::input::NumberInput;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Units;
use leathercraft::shapes::{REGULAR_POLYGON_SIDES, RegularPolygon};

/// A regular polygon's center, radius, number of sides and rotation.
pub(super) fn regular_polygon_fields(
    ui: &mut egui::Ui,
    polygon: &mut RegularPolygon,
    units: Units,
    language: Language,
) {
    let RegularPolygon {
        center,
        radius,
        sides,
        rotation,
        ..
    } = polygon;
    center_fields(ui, center, units, language);
    length_field(ui, Label::Radius, radius, MIN_SIZE_MM, units, language);
    param_row(ui, language.label(Label::Sides), None, |ui| {
        let field = NumberInput::new(sides)
            .id_salt("property_sides")
            .range(REGULAR_POLYGON_SIDES)
            .width(FIELD_WIDTH);
        ui.add(field);
    });
    angle_field(ui, Label::Rotation, rotation, language);
}
