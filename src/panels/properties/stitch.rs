use super::fields::{
    FIELD_WIDTH, FieldStyle, Marker, bounded_length_field, labeled, note, value_row,
};
use crate::editor::format_length;
use crate::ui_components::dropdown::{Dropdown, DropdownItem};
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::Units;
use leathercraft::shapes::{
    HoleKind, MAX_HOLE_MM, MAX_PITCH_MM, MIN_HOLE_MM, MIN_PITCH_MM, PITCH_PRESETS, StitchBand,
    StitchLine,
};

/// A stitch line's hole type, pitch and hole size.
pub(super) fn stitch_line_fields(
    ui: &mut egui::Ui,
    line: &mut StitchLine,
    units: Units,
    language: Language,
) {
    hole_fields(
        ui,
        FieldStyle::Rows,
        &mut line.hole,
        &mut line.pitch,
        &mut line.size,
        units,
        language,
    );
}

/// The stitch fields laid out inline, for the tool bar.
pub(crate) fn stitch_fields(
    ui: &mut egui::Ui,
    hole: &mut HoleKind,
    pitch: &mut f64,
    size: &mut f64,
    units: Units,
    language: Language,
) {
    hole_fields(ui, FieldStyle::Inline, hole, pitch, size, units, language);
}

/// The hole type, the pitch with the usual iron pitches as presets, and the hole size, in
/// display units. Choosing another type starts from that type's default size.
fn hole_fields(
    ui: &mut egui::Ui,
    style: FieldStyle,
    hole: &mut HoleKind,
    pitch: &mut f64,
    size: &mut f64,
    units: Units,
    language: Language,
) {
    labeled(ui, style, language.label(Label::HoleType), None, |ui| {
        Dropdown::new("hole_kind", language.label(Label::HoleKind(*hole)))
            .width(FIELD_WIDTH)
            .show_ui(ui, |ui| {
                for kind in HoleKind::ALL {
                    let chosen = kind == *hole;
                    let item =
                        DropdownItem::new(language.label(Label::HoleKind(kind))).selected(chosen);
                    if ui.add(item).clicked() && !chosen {
                        *hole = kind;
                        *size = kind.default_size();
                    }
                }
            });
    });
    bounded_length_field(
        ui,
        style,
        Label::Pitch,
        Some(Marker::Holes),
        pitch,
        (MIN_PITCH_MM, MAX_PITCH_MM),
        units,
        language,
    );
    // Wrapped, so that every preset stays reachable in a narrow panel.
    ui.horizontal_wrapped(|ui| {
        let presets = format!(
            "{}, {}",
            language.label(Label::PitchPresets),
            language.unit_symbol(Units::Millimeters)
        );
        match style {
            FieldStyle::Rows => note(ui, &presets),
            FieldStyle::Inline => {
                ui.label(presets);
            }
        }
        for preset in PITCH_PRESETS {
            let text = egui::RichText::new(preset.to_string()).monospace();
            if ui.selectable_label(*pitch == preset, text).clicked() {
                *pitch = preset;
            }
        }
    });
    bounded_length_field(
        ui,
        style,
        Label::HoleSize,
        Some(Marker::HoleSize),
        size,
        (MIN_HOLE_MM, MAX_HOLE_MM),
        units,
        language,
    );
}

/// What a stitch line comes to: its vertices, length, hole count and actual pitch, the thread
/// that suits its pitch, and a warning when the piece's leather doesn't.
pub(super) fn stitch_details(
    ui: &mut egui::Ui,
    line: &StitchLine,
    thickness_mm: f64,
    units: Units,
    language: Language,
) {
    let length = |millimeters: f64| format_length(language, units, millimeters);
    note(
        ui,
        &language.message(&Message::StitchVertices {
            count: line.points.len(),
        }),
    );
    value_row(
        ui,
        language.label(Label::StitchLength),
        length(line.length()),
    );
    value_row(
        ui,
        language.label(Label::HoleCount),
        line.hole_count().to_string(),
    );
    if let Some((min, max)) = line.actual_pitch() {
        // Runs that differ by less than the display precision show one value.
        let (min, max) = (length(min), length(max));
        let value = if min == max {
            min
        } else {
            format!("{min} – {max}")
        };
        value_row(ui, language.label(Label::ActualPitch), value);
    }
    let band = StitchBand::from_pitch(line.pitch);
    note(ui, &language.message(&Message::ThreadHint { band }));
    if !band.suits_leather(thickness_mm) {
        let color = Palette::of(ui.painter()).status.warning;
        ui.label(
            egui::RichText::new(language.message(&Message::LeatherOutsideBand { band }))
                .size(11.0)
                .color(color),
        );
    }
}
