//! The Material tab of the Settings window: the default material of new projects.

use super::SettingsDraft;
use crate::ui_components::button::Button;
use crate::ui_components::dropdown::{Dropdown, DropdownItem, dropdown_value, group_label};
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::material::{
    ColorGroup, Flexibility, HeatResistance, LeatherColor, MAX_THICKNESS_MM, MIN_THICKNESS_MM,
    Material, Origin, Strength, Texture, Treatment, WaterResistance,
};
use leathercraft::model::Units;

const COMBO_WIDTH: f32 = 200.0;
const COMBO_POPUP_HEIGHT: f32 = 360.0;
const SWATCH: egui::Vec2 = egui::Vec2::splat(14.0);

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
    if ui
        .add(Button::outline(language.label(Label::ResetToDefaults)))
        .clicked()
    {
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

/// A labelled dropdown over `options`; `hint` describes a value when the catalog has a description.
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
    let response = Dropdown::new(language.label(field), language.label(name(*value)))
        .width(COMBO_WIDTH)
        .max_height(COMBO_POPUP_HEIGHT)
        .show_ui(ui, |ui| {
            for &option in options {
                let item = DropdownItem::new(language.label(name(option)));
                let item = dropdown_value(ui, value, option, item);
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
    let current = *value;
    Dropdown::new(field, language.label(Label::Color(current)))
        .width(COMBO_WIDTH)
        .max_height(COMBO_POPUP_HEIGHT)
        .leading(SWATCH, move |painter, rect| swatch(painter, rect, current))
        .show_ui(ui, |ui| {
            for group in ColorGroup::ALL {
                group_label(ui, language.label(Label::ColorGroup(group)));
                for option in LeatherColor::ALL.into_iter().filter(|c| c.group() == group) {
                    let item = DropdownItem::new(language.label(Label::Color(option)))
                        .leading(SWATCH, move |painter, rect| swatch(painter, rect, option));
                    dropdown_value(ui, value, option, item);
                }
            }
        });
    ui.end_row();
}

/// The color is document data, not a theme color, so it is painted as it is.
fn swatch(painter: &egui::Painter, rect: egui::Rect, color: LeatherColor) {
    let [red, green, blue] = color.rgb();
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(red, green, blue));
}
