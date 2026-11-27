use super::fields::{
    FIELD_WIDTH, FieldStyle, Marker, ROW_GAP, bounded_length_field, labeled, note, section_header,
    value_row,
};
use crate::editor::format_length;
use crate::panels::seam_link::{OpenLink, seam_name};
use crate::ui_components::button::Button;
use crate::ui_components::dropdown::{Dropdown, DropdownItem};
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::{Project, Units};
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

/// What a link row in Properties asks for.
pub(super) enum LinkAction {
    Open(OpenLink),
    Event(ProjectEvent),
}

/// The links of the stitch line `child` of piece `piece`: one row per link with the other seam,
/// its number of pairs, a mark when the line got shorter than its range, and Edit and Remove;
/// then the button that links the seam anew.
pub(super) fn seam_links_section(
    ui: &mut egui::Ui,
    project: &Project,
    piece: u64,
    child: u64,
    language: Language,
) -> Option<LinkAction> {
    let palette = Palette::of(ui.painter());
    let links: Vec<_> = project.seam_links_of(piece, child).collect();
    let mut action = None;
    ui.add_space(ROW_GAP / 2.0);
    section_header(
        ui,
        language.label(Label::SeamLinks),
        (!links.is_empty())
            .then(|| links.len().to_string())
            .as_deref(),
    );
    for link in links {
        // The other side; both sides when the link is on this one seam.
        let other = if link.a.piece == piece && link.a.child == child {
            &link.b
        } else {
            &link.a
        };
        let pairs = link.pairs(project);
        let text = language.message(&Message::SeamLinkRow {
            other: seam_name(project, other.piece, other.child, language),
            pairs: pairs.as_ref().map_or(0, |pairs| pairs.pairs.len()),
        });
        let stale = pairs.as_ref().map_or(true, |pairs| pairs.clamped);
        ui.horizontal(|ui| {
            let color = if stale {
                palette.status.warning
            } else {
                palette.text.strong
            };
            let label = ui.add(
                egui::Label::new(egui::RichText::new(text).size(11.0).color(color)).truncate(),
            );
            if stale {
                label.on_hover_text(language.label(Label::SeamLinkStale));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button(language.label(Label::RemoveLink)).clicked() {
                    action = Some(LinkAction::Event(ProjectEvent::RemoveSeamLink {
                        id: link.id,
                    }));
                }
                if ui.small_button(language.label(Label::EditLink)).clicked() {
                    action = Some(LinkAction::Open(OpenLink::Edit(link.id)));
                }
            });
        });
    }
    if ui
        .add(Button::outline(language.label(Label::LinkSeams)))
        .clicked()
    {
        action = Some(LinkAction::Open(OpenLink::New(Some((piece, child)))));
    }
    action
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
