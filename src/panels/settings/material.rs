//! The Material tab of the Settings window: the default material of new projects.

use super::SettingsDraft;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::material::{
    ColorGroup, Flexibility, HeatResistance, LeatherColor, MAX_THICKNESS_MM, MIN_THICKNESS_MM,
    Material, Origin, Strength, Texture, Treatment, WaterResistance,
};
use leathercraft::model::Units;

const COMBO_WIDTH: f32 = 200.0;
const COMBO_POPUP_HEIGHT: f32 = 360.0;
const SWATCH_SIZE: f32 = 14.0;

pub(super) fn material_tab(ui: &mut egui::Ui, draft: &mut SettingsDraft, language: Language) {
    let material = &mut draft.preferences.default_material;
    ui.push_id("settings_material", |ui| {
        ui.strong(language.label(Label::BasicProperties));
        egui::Grid::new("basic")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| basic_rows(ui, material, language));
        ui.add_space(6.0);
        ui.strong(language.label(Label::PhysicalProperties));
        egui::Grid::new("physical")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| physical_rows(ui, material, language));
    });
    ui.add_space(4.0);
    if ui.button(language.label(Label::ResetToDefaults)).clicked() {
        *material = Material::default();
    }
}

fn basic_rows(ui: &mut egui::Ui, material: &mut Material, language: Language) {
    combo(
        ui,
        language,
        Label::OriginField,
        &mut material.origin,
        &Origin::ALL,
        Label::Origin,
        |value| Some(Label::OriginHint(value)),
    );
    combo(
        ui,
        language,
        Label::TreatmentField,
        &mut material.treatment,
        &Treatment::ALL,
        Label::Treatment,
        |value| Some(Label::TreatmentHint(value)),
    );
    thickness_row(ui, language, material);
    color_row(ui, language, &mut material.color);
    combo(
        ui,
        language,
        Label::TextureField,
        &mut material.texture,
        &Texture::ALL,
        Label::Texture,
        |_| None,
    );
}

fn physical_rows(ui: &mut egui::Ui, material: &mut Material, language: Language) {
    combo(
        ui,
        language,
        Label::StrengthField,
        &mut material.strength,
        &Strength::ALL,
        Label::Strength,
        |_| None,
    );
    combo(
        ui,
        language,
        Label::FlexibilityField,
        &mut material.flexibility,
        &Flexibility::ALL,
        Label::Flexibility,
        |value| Some(Label::FlexibilityHint(value)),
    );
    combo(
        ui,
        language,
        Label::WaterResistanceField,
        &mut material.water_resistance,
        &WaterResistance::ALL,
        Label::WaterResistance,
        |value| Some(Label::WaterResistanceHint(value)),
    );
    combo(
        ui,
        language,
        Label::HeatResistanceField,
        &mut material.heat_resistance,
        &HeatResistance::ALL,
        Label::HeatResistance,
        |_| None,
    );
}

/// A labelled combo box over `options`; `hint` describes a value when the catalog has a description.
fn combo<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    language: Language,
    field: Label,
    value: &mut T,
    options: &[T],
    name: impl Fn(T) -> Label,
    hint: impl Fn(T) -> Option<Label>,
) {
    ui.label(language.label(field));
    let response = egui::ComboBox::from_id_salt(language.label(field))
        .width(COMBO_WIDTH)
        .height(COMBO_POPUP_HEIGHT)
        .selected_text(language.label(name(*value)))
        .show_ui(ui, |ui| {
            for &option in options {
                let item = ui.selectable_value(value, option, language.label(name(option)));
                if let Some(text) = hint(option) {
                    item.on_hover_text(language.label(text));
                }
            }
        })
        .response;
    if let Some(text) = hint(*value) {
        response.on_hover_text(language.label(text));
    }
    ui.end_row();
}

fn thickness_row(ui: &mut egui::Ui, language: Language, material: &mut Material) {
    ui.label(language.label(Label::ThicknessField));
    ui.horizontal(|ui| {
        // Leather is always specified in millimeters, whatever the display units are.
        ui.add(
            egui::DragValue::new(&mut material.thickness_mm)
                .range(MIN_THICKNESS_MM..=MAX_THICKNESS_MM)
                .speed(0.05)
                .max_decimals(2)
                .suffix(format!(" {}", language.unit_symbol(Units::Millimeters))),
        );
        ui.label(language.label(Label::Thickness(material.thickness_category())));
    });
    ui.end_row();
}

fn color_row(ui: &mut egui::Ui, language: Language, value: &mut LeatherColor) {
    let field = language.label(Label::ColorField);
    ui.label(field);
    ui.horizontal(|ui| {
        swatch(ui, *value);
        egui::ComboBox::from_id_salt(field)
            .width(COMBO_WIDTH - SWATCH_SIZE - ui.spacing().item_spacing.x)
            .height(COMBO_POPUP_HEIGHT)
            .selected_text(language.label(Label::Color(*value)))
            .show_ui(ui, |ui| {
                for group in ColorGroup::ALL {
                    ui.weak(language.label(Label::ColorGroup(group)));
                    for option in LeatherColor::ALL.into_iter().filter(|c| c.group() == group) {
                        ui.horizontal(|ui| {
                            swatch(ui, option);
                            ui.selectable_value(
                                value,
                                option,
                                language.label(Label::Color(option)),
                            );
                        });
                    }
                }
            });
    });
    ui.end_row();
}

fn swatch(ui: &mut egui::Ui, color: LeatherColor) {
    let [red, green, blue] = color.rgb();
    let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(SWATCH_SIZE), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, 2.0, egui::Color32::from_rgb(red, green, blue));
}
