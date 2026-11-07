//! The material form, from the "Default material" page of the Settings dialog in the Brilliant
//! design: a summary card with the color swatch and the thickness, then the basic and the
//! physical properties as rows of a name and a dropdown. The physical properties show their
//! level as five bars before the value.

use crate::ui_components::dropdown::{Dropdown, DropdownItem, dropdown_value, group_label};
use crate::ui_components::input::NumberInput;
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::material::{
    ColorGroup, Flexibility, HeatResistance, LeatherColor, MAX_THICKNESS_MM, MIN_THICKNESS_MM,
    Material, Origin, Strength, Texture, Treatment, WaterResistance,
};
use leathercraft::model::Units;

const POPUP_HEIGHT: f32 = 360.0;
const THICKNESS_STEP_MM: f64 = 0.1;

const SECTION_GAP: f32 = 14.0;
const ROW_GAP: f32 = 8.0;
const ROW_HEIGHT: f32 = 28.0;
/// Between a row's name and its control.
const ROW_SPACING: f32 = 12.0;
/// The name column takes this share of the row, up to `MAX_LABEL_WIDTH`.
const LABEL_SHARE: f32 = 0.4;
const MAX_LABEL_WIDTH: f32 = 132.0;
const LABEL_SIZE: f32 = 12.0;
const HEADER_SIZE: f32 = 10.0;
const HEADER_LETTER_SPACING: f32 = 0.6;

const CARD_PADDING: egui::Margin = egui::Margin::symmetric(12, 10);
const CARD_RADIUS: u8 = 7;
const CARD_GAP: f32 = 12.0;
const CARD_SWATCH: egui::Vec2 = egui::Vec2::splat(40.0);
const CARD_SWATCH_RADIUS: f32 = 6.0;
const CARD_TITLE_SIZE: f32 = 12.0;
const CARD_HINT_SIZE: f32 = 11.0;
const CARD_LINE_GAP: f32 = 3.0;

const CHIP_PADDING: egui::Margin = egui::Margin::symmetric(8, 3);
const CHIP_RADIUS: u8 = 4;
const CHIP_TEXT_SIZE: f32 = 11.0;

const SWATCH: egui::Vec2 = egui::Vec2::splat(14.0);
const SWATCH_RADIUS: f32 = 3.0;

const THICKNESS_WIDTH: f32 = 110.0;
const MIN_THICKNESS_WIDTH: f32 = 72.0;
/// Between the thickness field and its class chip.
const THICKNESS_GAP: f32 = 10.0;

const LEVELS: usize = 5;
const LEVEL_BAR: egui::Vec2 = egui::vec2(4.0, 10.0);
const LEVEL_GAP: f32 = 2.0;
const LEVEL_SIZE: egui::Vec2 = egui::vec2(
    LEVELS as f32 * LEVEL_BAR.x + (LEVELS - 1) as f32 * LEVEL_GAP,
    LEVEL_BAR.y,
);

/// The whole form on `material`, with `hint` under the summary card's title.
/// Returns whether the material changed.
pub(super) fn material_form(
    ui: &mut egui::Ui,
    material: &mut Material,
    language: Language,
    id_salt: &str,
    hint: &str,
) -> bool {
    let before = *material;
    ui.push_id(id_salt, |ui| {
        ui.spacing_mut().item_spacing.y = SECTION_GAP;
        summary_card(ui, material, language, hint);
        section(ui, language.label(Label::BasicProperties), |ui| {
            basic_rows(ui, material, language);
        });
        divider(ui);
        section(ui, language.label(Label::PhysicalProperties), |ui| {
            physical_rows(ui, material, language);
        });
    });
    *material != before
}

fn basic_rows(ui: &mut egui::Ui, material: &mut Material, language: Language) {
    choice_row(
        ui,
        language,
        Label::OriginField,
        &mut material.origin,
        &Origin::ALL,
        Label::Origin,
        |value| Some(Label::OriginHint(value)),
        false,
    );
    choice_row(
        ui,
        language,
        Label::TreatmentField,
        &mut material.treatment,
        &Treatment::ALL,
        Label::Treatment,
        |value| Some(Label::TreatmentHint(value)),
        false,
    );
    thickness_row(ui, material, language);
    color_row(ui, &mut material.color, language);
    choice_row(
        ui,
        language,
        Label::TextureField,
        &mut material.texture,
        &Texture::ALL,
        Label::Texture,
        |_| None,
        false,
    );
}

fn physical_rows(ui: &mut egui::Ui, material: &mut Material, language: Language) {
    choice_row(
        ui,
        language,
        Label::StrengthField,
        &mut material.strength,
        &Strength::ALL,
        Label::Strength,
        |_| None,
        true,
    );
    choice_row(
        ui,
        language,
        Label::FlexibilityField,
        &mut material.flexibility,
        &Flexibility::ALL,
        Label::Flexibility,
        |value| Some(Label::FlexibilityHint(value)),
        true,
    );
    choice_row(
        ui,
        language,
        Label::WaterResistanceField,
        &mut material.water_resistance,
        &WaterResistance::ALL,
        Label::WaterResistance,
        |value| Some(Label::WaterResistanceHint(value)),
        true,
    );
    choice_row(
        ui,
        language,
        Label::HeatResistanceField,
        &mut material.heat_resistance,
        &HeatResistance::ALL,
        Label::HeatResistance,
        |_| None,
        true,
    );
}

/// The swatch of the color, the origin and treatment over `hint`, and the thickness.
fn summary_card(ui: &mut egui::Ui, material: &Material, language: Language, hint: &str) {
    let palette = Palette::of(ui.painter());
    egui::Frame::new()
        .fill(palette.surface.info)
        .stroke(egui::Stroke::new(1.0, palette.border.divider))
        .corner_radius(CARD_RADIUS)
        .inner_margin(CARD_PADDING)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = CARD_GAP;
                let (rect, _) = ui.allocate_exact_size(CARD_SWATCH, egui::Sense::hover());
                swatch(ui.painter(), rect, CARD_SWATCH_RADIUS, material.color);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let thickness = format!(
                        "{:.2} {}",
                        material.thickness_mm,
                        language.unit_symbol(Units::Millimeters)
                    );
                    chip(
                        ui,
                        egui::RichText::new(thickness)
                            .monospace()
                            .color(palette.accent.base),
                        palette.accent.container,
                        palette.accent.border,
                    );
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.spacing_mut().item_spacing.y = CARD_LINE_GAP;
                        let title = format!(
                            "{} · {}",
                            language.label(Label::Origin(material.origin)),
                            language.label(Label::Treatment(material.treatment)),
                        );
                        let title = egui::RichText::new(title)
                            .size(CARD_TITLE_SIZE)
                            .strong()
                            .color(palette.text.primary);
                        ui.add(egui::Label::new(title).truncate());
                        let hint = egui::RichText::new(hint)
                            .size(CARD_HINT_SIZE)
                            .color(palette.text.secondary);
                        ui.add(egui::Label::new(hint).wrap());
                    });
                });
            });
        });
}

/// A titled group of rows.
fn section(ui: &mut egui::Ui, title: &str, add_rows: impl FnOnce(&mut egui::Ui)) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = ROW_GAP;
        let color = Palette::of(ui.painter()).text.section_header;
        let title = egui::RichText::new(title.to_uppercase())
            .size(HEADER_SIZE)
            .strong()
            .extra_letter_spacing(HEADER_LETTER_SPACING)
            .color(color);
        ui.add(egui::Label::new(title).truncate());
        add_rows(ui);
    });
}

/// A 1 px line across the form, between its sections.
fn divider(ui: &mut egui::Ui) {
    let color = Palette::of(ui.painter()).border.divider;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, color);
}

/// A row: `name` in a column at the left, and `add_control` in the rest of the width.
fn row<R>(ui: &mut egui::Ui, name: &str, add_control: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let width = ui.available_width();
    let label_width = (width * LABEL_SHARE).min(MAX_LABEL_WIDTH);
    ui.allocate_ui_with_layout(
        egui::vec2(width, ROW_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = ROW_SPACING;
            ui.allocate_ui_with_layout(
                egui::vec2(label_width, ROW_HEIGHT),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_min_width(label_width);
                    let color = Palette::of(ui.painter()).text.label;
                    let text = egui::RichText::new(name).size(LABEL_SIZE).color(color);
                    ui.add(egui::Label::new(text).truncate());
                },
            );
            add_control(ui)
        },
    )
    .inner
}

/// A row with a dropdown over `options`; `hint` describes a value when the catalog has a
/// description, and `levels` shows the value's place on its scale as bars.
#[allow(clippy::too_many_arguments)]
fn choice_row<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    language: Language,
    field: Label,
    value: &mut T,
    options: &[T],
    name: impl Fn(T) -> Label,
    hint: impl Fn(T) -> Option<Label>,
    levels: bool,
) {
    let field_name = language.label(field);
    row(ui, field_name, |ui| {
        let current = *value;
        let mut dropdown = Dropdown::new(field_name, language.label(name(current)))
            .width(ui.available_width())
            .max_height(POPUP_HEIGHT);
        if levels {
            let lit = level(current, options);
            dropdown = dropdown.leading(LEVEL_SIZE, move |painter, rect| {
                paint_level(painter, rect, lit);
            });
        }
        let response = dropdown
            .show_ui(ui, |ui| {
                for &option in options {
                    let mut item = DropdownItem::new(language.label(name(option)));
                    if levels {
                        let lit = level(option, options);
                        item = item.leading(LEVEL_SIZE, move |painter, rect| {
                            paint_level(painter, rect, lit);
                        });
                    }
                    let item = dropdown_value(ui, value, option, item);
                    if let Some(text) = hint(option) {
                        item.on_hover_text(language.label(text));
                    }
                }
            })
            .response;
        if let Some(text) = hint(current) {
            response.on_hover_text(language.label(text));
        }
    });
}

/// The thickness in mm, whatever the display units are, with its class in a chip after it.
fn thickness_row(ui: &mut egui::Ui, material: &mut Material, language: Language) {
    row(ui, language.label(Label::ThicknessField), |ui| {
        let palette = Palette::of(ui.painter());
        let class =
            egui::RichText::new(language.label(Label::Thickness(material.thickness_category())))
                .size(CHIP_TEXT_SIZE)
                .color(palette.text.label);
        let class_width = egui::WidgetText::from(class.clone())
            .into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                egui::TextStyle::Body,
            )
            .size()
            .x
            + CHIP_PADDING.sum().x;
        let field_width = (ui.available_width() - class_width - THICKNESS_GAP)
            .clamp(MIN_THICKNESS_WIDTH, THICKNESS_WIDTH);
        ui.spacing_mut().item_spacing.x = THICKNESS_GAP;
        ui.add(
            NumberInput::new(&mut material.thickness_mm)
                .id_salt("thickness")
                .range(MIN_THICKNESS_MM..=MAX_THICKNESS_MM)
                .step(THICKNESS_STEP_MM)
                .decimals(2)
                .unit(language.unit_symbol(Units::Millimeters))
                .width(field_width),
        );
        chip(ui, class, palette.surface.chip, palette.border.chip);
    });
}

fn color_row(ui: &mut egui::Ui, value: &mut LeatherColor, language: Language) {
    let field = language.label(Label::ColorField);
    row(ui, field, |ui| {
        let current = *value;
        Dropdown::new(field, language.label(Label::Color(current)))
            .width(ui.available_width())
            .max_height(POPUP_HEIGHT)
            .leading(SWATCH, move |painter, rect| {
                swatch(painter, rect, SWATCH_RADIUS, current);
            })
            .show_ui(ui, |ui| {
                for group in ColorGroup::ALL {
                    group_label(ui, language.label(Label::ColorGroup(group)));
                    for option in LeatherColor::ALL.into_iter().filter(|c| c.group() == group) {
                        let item = DropdownItem::new(language.label(Label::Color(option))).leading(
                            SWATCH,
                            move |painter, rect| {
                                swatch(painter, rect, SWATCH_RADIUS, option);
                            },
                        );
                        dropdown_value(ui, value, option, item);
                    }
                }
            });
    });
}

/// A small framed text, such as the thickness or its class.
fn chip(ui: &mut egui::Ui, text: egui::RichText, fill: egui::Color32, stroke: egui::Color32) {
    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .corner_radius(CHIP_RADIUS)
        .inner_margin(CHIP_PADDING)
        .show(ui, |ui| {
            ui.label(text.size(CHIP_TEXT_SIZE));
        });
}

/// The color is document data, not a theme color, so it is painted as it is; the edge keeps a
/// dark leather visible on a dark field.
fn swatch(painter: &egui::Painter, rect: egui::Rect, radius: f32, color: LeatherColor) {
    let [red, green, blue] = color.rgb();
    let edge = Palette::of(painter).border.chip;
    painter.rect(
        rect,
        radius,
        egui::Color32::from_rgb(red, green, blue),
        egui::Stroke::new(1.0, edge),
        egui::StrokeKind::Inside,
    );
}

/// How many of the `LEVELS` bars `value` lights, from its place in `options` (lowest first).
fn level<T: PartialEq>(value: T, options: &[T]) -> usize {
    let index = options
        .iter()
        .position(|option| *option == value)
        .unwrap_or(0);
    ((index + 1) as f32 * LEVELS as f32 / options.len() as f32).round() as usize
}

/// `lit` bars in the accent, the rest in the field border color.
fn paint_level(painter: &egui::Painter, rect: egui::Rect, lit: usize) {
    let palette = Palette::of(painter);
    for bar in 0..LEVELS {
        let min = rect.min + egui::vec2(bar as f32 * (LEVEL_BAR.x + LEVEL_GAP), 0.0);
        let color = if bar < lit {
            palette.accent.base
        } else {
            palette.border.field
        };
        painter.rect_filled(egui::Rect::from_min_size(min, LEVEL_BAR), 1.0, color);
    }
}
