use super::fields::{MIN_SIZE_MM, angle_field, length_field};
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Units;
use leathercraft::shapes::{Line, LineStyle, MIN_DASH_MM};

/// A centerline's ends, length, angle and style.
pub(super) fn line_fields(ui: &mut egui::Ui, line: &mut Line, units: Units, language: Language) {
    let length = |ui: &mut egui::Ui, label: Label, millimeters: &mut f64, min: f64| {
        length_field(ui, label, millimeters, min, units, language);
    };
    length(ui, Label::StartX, &mut line.start.x, f64::MIN);
    length(ui, Label::StartY, &mut line.start.y, f64::MIN);
    length(ui, Label::EndX, &mut line.end.x, f64::MIN);
    length(ui, Label::EndY, &mut line.end.y, f64::MIN);
    // Length and angle turn and stretch the line around its start.
    let current = line.length();
    let mut value = current;
    length(ui, Label::EdgeLength, &mut value, MIN_SIZE_MM);
    if value != current {
        *line = line.with_length(value);
    }
    let current = line.angle();
    let mut value = current;
    angle_field(ui, Label::Angle, &mut value, language);
    if value != current {
        *line = line.with_angle(value);
    }
    ui.separator();
    line_style_fields(ui, &mut line.style, units, language);
}

/// The type of a line and, for a dashed one, its dash and gap in display units. Choosing
/// another type starts from that type's default lengths.
pub(crate) fn line_style_fields(
    ui: &mut egui::Ui,
    style: &mut LineStyle,
    units: Units,
    language: Language,
) {
    let kinds = [
        (LineStyle::Solid, Label::LineSolid),
        (LineStyle::dashed_default(), Label::LineDashed),
        (LineStyle::dash_dot_default(), Label::LineDashDot),
    ];
    let same_kind =
        |a: &LineStyle, b: &LineStyle| std::mem::discriminant(a) == std::mem::discriminant(b);
    let current = kinds
        .iter()
        .find(|(kind, _)| same_kind(kind, style))
        .map_or(Label::LineSolid, |(_, label)| *label);
    ui.horizontal(|ui| {
        ui.label(language.label(Label::LineType));
        egui::ComboBox::from_id_salt("line_style")
            .selected_text(language.label(current))
            .show_ui(ui, |ui| {
                for (kind, label) in kinds {
                    let chosen = same_kind(&kind, style);
                    if ui.selectable_label(chosen, language.label(label)).clicked() && !chosen {
                        *style = kind;
                    }
                }
            });
    });
    if let LineStyle::Dashed { dash, gap } | LineStyle::DashDot { dash, gap } = style {
        length_field(ui, Label::DashLength, dash, MIN_DASH_MM, units, language);
        length_field(ui, Label::GapLength, gap, MIN_DASH_MM, units, language);
    }
}
