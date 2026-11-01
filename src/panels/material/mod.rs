use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language};
use leathercraft::material::{
    ColorGroup, Flexibility, HeatResistance, LeatherColor, MAX_THICKNESS_MM, MIN_THICKNESS_MM,
    Material, Origin, Strength, Texture, Treatment, WaterResistance,
};
use leathercraft::model::{Project, Units};

const COMBO_WIDTH: f32 = 150.0;
const COMBO_POPUP_HEIGHT: f32 = 360.0;
const DIALOG_WIDTH: f32 = 340.0;
const SWATCH_SIZE: f32 = 14.0;

pub enum DialogResult {
    Create,
    Cancel,
}

/// A combo box over `options`; `hint` describes a value when the catalog has a description.
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

fn swatch(ui: &mut egui::Ui, color: LeatherColor) {
    let [red, green, blue] = color.rgb();
    let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(SWATCH_SIZE), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, 2.0, egui::Color32::from_rgb(red, green, blue));
}

fn color_combo(ui: &mut egui::Ui, language: Language, value: &mut LeatherColor) {
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
                    for option in LeatherColor::ALL {
                        if option.group() == group {
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
                }
            });
    });
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

/// The form shared by the new-project dialog and the selected piece.
/// Returns whether the material changed.
pub fn material_fields(
    ui: &mut egui::Ui,
    material: &mut Material,
    language: Language,
    id_salt: &str,
) -> bool {
    let before = *material;
    ui.push_id(id_salt, |ui| {
        ui.strong(language.label(Label::BasicProperties));
        egui::Grid::new("basic").num_columns(2).show(ui, |ui| {
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
            color_combo(ui, language, &mut material.color);
            combo(
                ui,
                language,
                Label::TextureField,
                &mut material.texture,
                &Texture::ALL,
                Label::Texture,
                |_| None,
            );
        });
        ui.add_space(6.0);
        ui.strong(language.label(Label::PhysicalProperties));
        egui::Grid::new("physical").num_columns(2).show(ui, |ui| {
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
        });
    });
    *material != before
}

pub fn show_panel(
    ui: &mut egui::Ui,
    project: &Project,
    selected: Option<u64>,
    language: Language,
) -> Option<ProjectEvent> {
    ui.label(language.label(Label::Material));
    let Some(piece) = selected.and_then(|id| project.pieces.iter().find(|piece| piece.id == id))
    else {
        ui.weak(language.label(Label::NoPieceSelected));
        return None;
    };
    let mut material = piece.material;
    egui::ScrollArea::vertical()
        .show(ui, |ui| {
            material_fields(ui, &mut material, language, "selected_piece")
        })
        .inner
        .then_some(ProjectEvent::SetPieceMaterial {
            id: piece.id,
            material,
        })
}

/// Clicking outside the dialog does nothing, so a stray click cannot lose the filled-in form.
pub fn new_project_dialog(
    ctx: &egui::Context,
    material: &mut Material,
    language: Language,
) -> Option<DialogResult> {
    let modal = egui::Modal::new(egui::Id::new("new_project_dialog")).show(ctx, |ui| {
        ui.set_width(DIALOG_WIDTH);
        ui.heading(language.label(Label::NewProjectTitle));
        ui.add_space(6.0);
        material_fields(ui, material, language, "new_project");
        ui.add_space(6.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(language.label(Label::Create)).clicked() {
                return Some(DialogResult::Create);
            }
            if ui.button(language.label(Label::Cancel)).clicked() {
                return Some(DialogResult::Cancel);
            }
            None
        })
        .inner
    });
    // Esc closes an open combo box first, and only then the dialog.
    let escaped = !modal.any_popup_open
        && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    modal.inner.or(escaped.then_some(DialogResult::Cancel))
}
