use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::{Point, Units};

/// Sizes stay above zero, which the project would reject.
pub(super) const MIN_SIZE_MM: f64 = 0.01;

/// A length in display units, at least `min` mm.
pub(super) fn length_field(
    ui: &mut egui::Ui,
    label: Label,
    millimeters: &mut f64,
    min: f64,
    units: Units,
    language: Language,
) {
    let scale = units.from_millimeters(1.0);
    let mut value = *millimeters * scale;
    ui.horizontal(|ui| {
        ui.label(language.label(label));
        let field = egui::DragValue::new(&mut value)
            .speed(0.1 * scale)
            .max_decimals(3)
            .range(min * scale..=f64::MAX)
            .suffix(format!(" {}", language.unit_symbol(units)));
        if ui.add(field).changed() {
            *millimeters = value / scale;
        }
    });
}

/// A length in display units within `min..=max` mm.
pub(super) fn bounded_length_field(
    ui: &mut egui::Ui,
    label: Label,
    millimeters: &mut f64,
    (min, max): (f64, f64),
    units: Units,
    language: Language,
) {
    let scale = units.from_millimeters(1.0);
    let mut value = *millimeters * scale;
    ui.horizontal(|ui| {
        ui.label(language.label(label));
        let field = egui::DragValue::new(&mut value)
            .speed(0.01 * scale)
            .max_decimals(3)
            .range(min * scale..=max * scale)
            .suffix(format!(" {}", language.unit_symbol(units)));
        if ui.add(field).changed() {
            *millimeters = value / scale;
        }
    });
}

/// An angle shown in degrees and kept in radians.
pub(super) fn angle_field(ui: &mut egui::Ui, label: Label, radians: &mut f64, language: Language) {
    let mut degrees = radians.to_degrees();
    ui.horizontal(|ui| {
        ui.label(language.label(label));
        let field = egui::DragValue::new(&mut degrees)
            .speed(1.0)
            .max_decimals(2)
            .range(-360.0..=360.0);
        if ui.add(field).changed() {
            *radians = degrees.to_radians();
        }
    });
}

/// The center of a primitive in display units.
pub(super) fn center_fields(
    ui: &mut egui::Ui,
    center: &mut Point,
    units: Units,
    language: Language,
) {
    length_field(ui, Label::CenterX, &mut center.x, f64::MIN, units, language);
    length_field(ui, Label::CenterY, &mut center.y, f64::MIN, units, language);
}
