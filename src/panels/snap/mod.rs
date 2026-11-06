//! The Snap panel: which kinds of snapping are on, the grid step and the snap distance.
//! The settings are interface state, saved with the app settings by the caller.

use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::snap::{
    MAX_DISTANCE_PX, MAX_GRID_STEP_MM, MIN_DISTANCE_PX, MIN_GRID_STEP_MM, SnapSettings,
};

pub fn show_panel(ui: &mut egui::Ui, snap: &mut SnapSettings, language: Language) {
    ui.checkbox(&mut snap.enabled, language.label(Label::SnapEnabled));
    ui.add_enabled_ui(snap.enabled, |ui| {
        ui.indent("snap options", |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut snap.grid, language.label(Label::SnapGrid));
                ui.add_enabled(
                    snap.grid,
                    egui::DragValue::new(&mut snap.grid_step_mm)
                        .range(MIN_GRID_STEP_MM..=MAX_GRID_STEP_MM)
                        .speed(0.1)
                        .fixed_decimals(1)
                        .suffix(" mm"),
                )
                .on_hover_text(language.label(Label::SnapGridStep));
            });
            ui.checkbox(&mut snap.vertices, language.label(Label::SnapVertices))
                .on_hover_text(language.label(Label::SnapVerticesHint));
            ui.checkbox(&mut snap.guides, language.label(Label::SnapGuides))
                .on_hover_text(language.label(Label::SnapGuidesHint));
            ui.horizontal(|ui| {
                ui.label(language.label(Label::SnapDistance));
                ui.add(
                    egui::DragValue::new(&mut snap.distance_px)
                        .range(MIN_DISTANCE_PX..=MAX_DISTANCE_PX)
                        .speed(0.2)
                        .fixed_decimals(0),
                );
            });
        });
    });
    ui.small(language.label(Label::SnapVerticesHint));
    ui.small(language.label(Label::SnapAltHint));
}
