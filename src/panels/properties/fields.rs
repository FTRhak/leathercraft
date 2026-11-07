//! The building blocks of the panel, from the "Properties Body" of the Brilliant design: section
//! headers, the divider, and parameter rows with their name at the left, an optional marker in
//! the color of the canvas layer they belong to, and the control at the right.

use crate::ui_components::input::NumberInput;
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::{Point, Units};

/// Sizes stay above zero, which the project would reject.
pub(super) const MIN_SIZE_MM: f64 = 0.01;

/// Between the rows of a section.
pub(super) const ROW_GAP: f32 = 8.0;
/// The width of the number fields and of the dropdowns of the rows.
pub(super) const FIELD_WIDTH: f32 = 96.0;
const ROW_HEIGHT: f32 = 26.0;
const LABEL_SIZE: f32 = 12.0;
const HEADER_SIZE: f32 = 10.0;
const MARKER_SIZE: f32 = 7.0;
const MARKER_GAP: f32 = 6.0;

/// A dot before a row's name, in the color of the canvas layer the value belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Marker {
    /// The selected edge or vertex.
    Edge,
    /// The pitch of the stitch holes.
    Holes,
    /// The size of a stitch hole, as a ring.
    HoleSize,
    Centerline,
}

impl Marker {
    fn paint(self, painter: &egui::Painter, center: egui::Pos2) {
        let palette = Palette::of(painter);
        let radius = MARKER_SIZE / 2.0;
        match self {
            Self::Edge => painter.circle_filled(center, radius, palette.edge),
            Self::Holes => painter.circle_filled(center, radius, palette.hole),
            Self::Centerline => painter.circle_filled(center, radius, palette.centerline),
            Self::HoleSize => {
                painter.circle_stroke(center, radius - 0.75, egui::Stroke::new(1.5, palette.hole))
            }
        };
    }

    /// The inputs of the selected edge are drawn in the accent, as the edge is on the canvas.
    fn accent(self) -> bool {
        self == Self::Edge
    }
}

/// A row: the name, with `marker` before it, at the left, and `add_control` at the right.
pub(super) fn param_row<R>(
    ui: &mut egui::Ui,
    name: &str,
    marker: Option<Marker>,
    add_control: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, ROW_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            if let Some(marker) = marker {
                let (rect, _) =
                    ui.allocate_exact_size(egui::Vec2::splat(MARKER_SIZE), egui::Sense::hover());
                marker.paint(ui.painter(), rect.center());
                ui.add_space(MARKER_GAP - ui.spacing().item_spacing.x);
            }
            let color = Palette::of(ui.painter()).properties.param_label;
            let text = egui::RichText::new(name).size(LABEL_SIZE).color(color);
            ui.add(egui::Label::new(text).truncate());
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                add_control,
            )
            .inner
        },
    )
    .inner
}

/// Whether labelled fields are laid out as the panel's rows, or inline, as in the tool bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FieldStyle {
    Rows,
    Inline,
}

/// `add_control` labelled with `name`: a row of the panel, or the name and the control side by
/// side; an inline field has no marker.
pub(super) fn labeled<R>(
    ui: &mut egui::Ui,
    style: FieldStyle,
    name: &str,
    marker: Option<Marker>,
    add_control: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    match style {
        FieldStyle::Rows => param_row(ui, name, marker, add_control),
        FieldStyle::Inline => {
            ui.horizontal(|ui| {
                ui.label(name);
                add_control(ui)
            })
            .inner
        }
    }
}

/// A plain row: the name at the left and `value` at the right, in a monospaced font.
pub(super) fn value_row(ui: &mut egui::Ui, name: &str, value: impl Into<String>) {
    param_row(ui, name, None, |ui| {
        let color = Palette::of(ui.painter()).text.strong;
        ui.label(egui::RichText::new(value.into()).monospace().color(color));
    });
}

/// A small secondary text, such as a count or a hint.
pub(super) fn note(ui: &mut egui::Ui, text: &str) {
    let color = Palette::of(ui.painter()).properties.hint_text;
    ui.label(egui::RichText::new(text).size(11.0).color(color));
}

/// A section header: a title in capitals at the left and an optional accent text at the right.
pub(super) fn section_header(ui: &mut egui::Ui, title: &str, trailing: Option<&str>) {
    let palette = Palette::of(ui.painter());
    ui.horizontal(|ui| {
        let title = egui::RichText::new(title.to_uppercase())
            .size(HEADER_SIZE)
            .strong()
            .extra_letter_spacing(0.6)
            .color(palette.text.section_header);
        ui.add(egui::Label::new(title).truncate());
        if let Some(trailing) = trailing {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let text = egui::RichText::new(trailing)
                    .size(11.0)
                    .color(palette.accent.base);
                ui.add(egui::Label::new(text).truncate());
            });
        }
    });
}

/// A 1 px line across the panel, between its sections.
pub(super) fn divider(ui: &mut egui::Ui) {
    let color = Palette::of(ui.painter()).border.divider;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, color);
}

/// The step and decimals of a length field in `units`; `fine` for small sizes, such as holes.
fn length_precision(units: Units, fine: bool) -> (f64, usize) {
    match (units, fine) {
        (Units::Millimeters, false) => (1.0, 2),
        (Units::Millimeters, true) => (0.1, 2),
        (Units::Inches, false) => (0.05, 3),
        (Units::Inches, true) => (0.005, 3),
    }
}

/// A number field in display units for `millimeters`, within `min..=max` mm; writes the value
/// back only when it was changed, so that an untouched value isn't rounded through the units.
#[allow(clippy::too_many_arguments)]
fn length_input(
    ui: &mut egui::Ui,
    id_salt: Label,
    millimeters: &mut f64,
    (min, max): (f64, f64),
    fine: bool,
    marker: Option<Marker>,
    units: Units,
    language: Language,
) {
    let scale = units.from_millimeters(1.0);
    let current = *millimeters * scale;
    let mut value = current;
    let (step, decimals) = length_precision(units, fine);
    let field = NumberInput::new(&mut value)
        .id_salt(("property", format!("{id_salt:?}")))
        .range(min * scale..=max * scale)
        .step(step)
        .decimals(decimals)
        .unit(language.unit_symbol(units))
        .accent(marker.is_some_and(Marker::accent))
        .width(FIELD_WIDTH);
    ui.add(field);
    if value != current {
        *millimeters = value / scale;
    }
}

/// A length row in display units, at least `min` mm.
pub(super) fn length_field(
    ui: &mut egui::Ui,
    label: Label,
    millimeters: &mut f64,
    min: f64,
    units: Units,
    language: Language,
) {
    let name = language.label(label);
    marked_length_field(ui, name, label, None, millimeters, min, units, language);
}

/// A length row named `name`, with a marker before it; `label` keeps the field's identity.
#[allow(clippy::too_many_arguments)]
pub(super) fn marked_length_field(
    ui: &mut egui::Ui,
    name: &str,
    label: Label,
    marker: Option<Marker>,
    millimeters: &mut f64,
    min: f64,
    units: Units,
    language: Language,
) {
    param_row(ui, name, marker, |ui| {
        let bounds = (min, f64::MAX);
        length_input(
            ui,
            label,
            millimeters,
            bounds,
            false,
            marker,
            units,
            language,
        );
    });
}

/// A length field within `min..=max` mm, stepped finely, for small sizes such as holes.
#[allow(clippy::too_many_arguments)]
pub(super) fn bounded_length_field(
    ui: &mut egui::Ui,
    style: FieldStyle,
    label: Label,
    marker: Option<Marker>,
    millimeters: &mut f64,
    bounds: (f64, f64),
    units: Units,
    language: Language,
) {
    labeled(ui, style, language.label(label), marker, |ui| {
        length_input(
            ui,
            label,
            millimeters,
            bounds,
            true,
            marker,
            units,
            language,
        );
    });
}

/// An angle row, shown in degrees and kept in radians.
pub(super) fn angle_field(ui: &mut egui::Ui, label: Label, radians: &mut f64, language: Language) {
    param_row(ui, language.label(label), None, |ui| {
        let current = radians.to_degrees();
        let mut degrees = current;
        let field = NumberInput::new(&mut degrees)
            .id_salt(("property", format!("{label:?}")))
            .range(-360.0..=360.0)
            .step(1.0)
            .decimals(2)
            .width(FIELD_WIDTH);
        ui.add(field);
        if degrees != current {
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
