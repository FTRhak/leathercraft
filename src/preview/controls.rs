//! The preview's controls in the Properties panel: the View switch, Sew again and the camera.

use super::constants::{MAX_DISTANCE, MAX_ELEVATION, MIN_DISTANCE, MIN_ELEVATION};
use super::{PreviewMode, PreviewSettings};
use crate::ui_components::button::Button;
use crate::ui_components::group_button::GroupButton;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};

pub fn show_controls(ui: &mut egui::Ui, settings: &mut PreviewSettings, language: Language) {
    ui.label(language.label(Label::PreviewShows));
    ui.horizontal(|ui| {
        ui.add(
            GroupButton::new(&mut settings.mode)
                .option(PreviewMode::Sewn, language.label(Label::PreviewSewn))
                .option(PreviewMode::Flat, language.label(Label::PreviewFlat)),
        );
        if settings.mode == PreviewMode::Sewn
            && ui
                .add(Button::outline(language.label(Label::SewAgain)))
                .on_hover_text(language.label(Label::SewAgainHint))
                .clicked()
        {
            settings.resew = true;
        }
    });
    ui.label(language.label(Label::Camera));
    ui.add(
        egui::Slider::new(&mut settings.azimuth, 0.0..=std::f32::consts::TAU)
            .text(language.label(Label::Orbit)),
    );
    ui.add(
        egui::Slider::new(&mut settings.elevation, MIN_ELEVATION..=MAX_ELEVATION)
            .text(language.label(Label::Tilt)),
    );
    ui.add(
        egui::Slider::new(&mut settings.distance, MIN_DISTANCE..=MAX_DISTANCE)
            .text(language.label(Label::Zoom)),
    );
    ui.weak(language.label(Label::PreviewNavigationHint));
}
