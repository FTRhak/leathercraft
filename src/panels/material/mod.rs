//! The Material panel and the new-project dialog. Both show the material form of `form`: the
//! panel on the selected piece's material, the dialog on the default material of the new
//! project.

mod form;

use crate::ui_components::button::Button;
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use form::material_form;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::material::Material;
use leathercraft::model::Project;

/// Between the panel's edges and its content, as in the design.
const PANEL_MARGIN: i8 = 10;
const EMPTY_TEXT_SIZE: f32 = 12.0;
const DIALOG_WIDTH: f32 = 400.0;
const DIALOG_GAP: f32 = 12.0;
const DIALOG_TITLE_SIZE: f32 = 14.0;

pub enum DialogResult {
    Create,
    Cancel,
}

/// The material of the selected piece; returns the event that changes it.
pub fn show_panel(
    ui: &mut egui::Ui,
    project: &Project,
    selected: Option<u64>,
    language: Language,
) -> Option<ProjectEvent> {
    let piece = selected.and_then(|id| project.pieces.iter().find(|piece| piece.id == id));
    egui::ScrollArea::vertical()
        .show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(PANEL_MARGIN)
                .show(ui, |ui| {
                    let Some(piece) = piece else {
                        let color = Palette::of(ui.painter()).text.muted;
                        ui.label(
                            egui::RichText::new(language.label(Label::NoPieceSelected))
                                .size(EMPTY_TEXT_SIZE)
                                .color(color),
                        );
                        return None;
                    };
                    let mut material = piece.material;
                    let hint = language.message(&Message::PieceMaterial {
                        name: piece.name.clone(),
                    });
                    material_form(ui, &mut material, language, "selected_piece", &hint).then_some(
                        ProjectEvent::SetPieceMaterial {
                            id: piece.id,
                            material,
                        },
                    )
                })
                .inner
        })
        .inner
}

/// Clicking outside the dialog does nothing, so a stray click cannot lose the filled-in form.
pub fn new_project_dialog(
    ctx: &egui::Context,
    material: &mut Material,
    language: Language,
) -> Option<DialogResult> {
    let modal = egui::Modal::new(egui::Id::new("new_project_dialog")).show(ctx, |ui| {
        ui.set_width(DIALOG_WIDTH);
        ui.spacing_mut().item_spacing.y = DIALOG_GAP;
        let color = Palette::of(ui.painter()).text.primary;
        ui.label(
            egui::RichText::new(language.label(Label::NewProjectTitle))
                .size(DIALOG_TITLE_SIZE)
                .strong()
                .color(color),
        );
        material_form(
            ui,
            material,
            language,
            "new_project",
            language.label(Label::DefaultMaterialHint),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(Button::primary(language.label(Label::Create)))
                .clicked()
            {
                return Some(DialogResult::Create);
            }
            if ui
                .add(Button::secondary(language.label(Label::Cancel)))
                .clicked()
            {
                return Some(DialogResult::Cancel);
            }
            None
        })
        .inner
    });
    // Esc closes an open dropdown first, and only then the dialog.
    let escaped = !modal.any_popup_open
        && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    modal.inner.or(escaped.then_some(DialogResult::Cancel))
}
