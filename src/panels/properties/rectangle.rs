use super::fields::{MIN_SIZE_MM, angle_field, center_fields, length_field};
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Units;
use leathercraft::shapes::Rectangle;

/// A rectangle's center, width, height and rotation.
pub(super) fn rectangle_fields(
    ui: &mut egui::Ui,
    rectangle: &mut Rectangle,
    units: Units,
    language: Language,
) {
    let Rectangle {
        center,
        width,
        height,
        rotation,
        ..
    } = rectangle;
    center_fields(ui, center, units, language);
    length_field(ui, Label::Width, width, MIN_SIZE_MM, units, language);
    length_field(ui, Label::Height, height, MIN_SIZE_MM, units, language);
    angle_field(ui, Label::Rotation, rotation, language);
}
