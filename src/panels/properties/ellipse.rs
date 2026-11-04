use super::fields::{MIN_SIZE_MM, angle_field, center_fields, length_field};
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Units;
use leathercraft::shapes::Ellipse;

/// An ellipse's center, radii and rotation.
pub(super) fn ellipse_fields(
    ui: &mut egui::Ui,
    ellipse: &mut Ellipse,
    units: Units,
    language: Language,
) {
    let Ellipse {
        center,
        radius_x,
        radius_y,
        rotation,
    } = ellipse;
    center_fields(ui, center, units, language);
    length_field(ui, Label::RadiusX, radius_x, MIN_SIZE_MM, units, language);
    length_field(ui, Label::RadiusY, radius_y, MIN_SIZE_MM, units, language);
    angle_field(ui, Label::Rotation, rotation, language);
}
