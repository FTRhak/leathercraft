use super::fields::{MIN_SIZE_MM, angle_field, center_fields, length_field};
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
    ui.horizontal(|ui| {
        ui.label(language.label(Label::Sides));
        ui.add(egui::DragValue::new(sides).range(REGULAR_POLYGON_SIDES));
    });
    angle_field(ui, Label::Rotation, rotation, language);
}
