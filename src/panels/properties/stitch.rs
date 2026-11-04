use super::fields::bounded_length_field;
use crate::editor::format_length;
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
    stitch_fields(
        ui,
        &mut line.hole,
        &mut line.pitch,
        &mut line.size,
        units,
        language,
    );
}

/// The hole type, the pitch with the usual iron pitches as presets, and the hole size, in
/// display units. Choosing another type starts from that type's default size.
pub(crate) fn stitch_fields(
    ui: &mut egui::Ui,
    hole: &mut HoleKind,
    pitch: &mut f64,
    size: &mut f64,
    units: Units,
    language: Language,
) {
    ui.horizontal(|ui| {
        ui.label(language.label(Label::HoleType));
        egui::ComboBox::from_id_salt("hole_kind")
            .selected_text(language.label(Label::HoleKind(*hole)))
            .show_ui(ui, |ui| {
                for kind in HoleKind::ALL {
                    let chosen = kind == *hole;
                    if ui
                        .selectable_label(chosen, language.label(Label::HoleKind(kind)))
                        .clicked()
                        && !chosen
                    {
                        *hole = kind;
                        *size = kind.default_size();
                    }
                }
            });
    });
    bounded_length_field(
        ui,
        Label::Pitch,
        pitch,
        (MIN_PITCH_MM, MAX_PITCH_MM),
        units,
        language,
    );
    // Wrapped, so that every preset stays reachable in a narrow panel.
    ui.horizontal_wrapped(|ui| {
        ui.label(format!(
            "{}, {}",
            language.label(Label::PitchPresets),
            language.unit_symbol(Units::Millimeters)
        ));
        for preset in PITCH_PRESETS {
            let text = preset.to_string();
            if ui.selectable_label(*pitch == preset, text).clicked() {
                *pitch = preset;
            }
        }
    });
    bounded_length_field(
        ui,
        Label::HoleSize,
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
    ui.separator();
    ui.label(language.message(&Message::StitchVertices {
        count: line.points.len(),
    }));
    egui::Grid::new("stitch_details").show(ui, |ui| {
        ui.label(language.label(Label::StitchLength));
        ui.label(length(line.length()));
        ui.end_row();
        ui.label(language.label(Label::HoleCount));
        ui.label(line.hole_count().to_string());
        ui.end_row();
        if let Some((min, max)) = line.actual_pitch() {
            ui.label(language.label(Label::ActualPitch));
            // Runs that differ by less than the display precision show one value.
            let (min, max) = (length(min), length(max));
            if min == max {
                ui.label(min);
            } else {
                ui.label(format!("{min} – {max}"));
            }
            ui.end_row();
        }
    });
    let band = StitchBand::from_pitch(line.pitch);
    ui.weak(language.message(&Message::ThreadHint { band }));
    if !band.suits_leather(thickness_mm) {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            language.message(&Message::LeatherOutsideBand { band }),
        );
    }
}
