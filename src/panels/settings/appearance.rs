//! The Appearance tab of the Settings window: the interface theme.

use super::SettingsDraft;
use crate::settings::Theme;
use crate::ui_components::radio::radio_value;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};

pub(super) fn appearance_tab(ui: &mut egui::Ui, draft: &mut SettingsDraft, language: Language) {
    egui::Grid::new("settings_appearance")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label(language.label(Label::ThemeField));
            ui.horizontal(|ui| {
                for theme in Theme::ALL {
                    radio_value(
                        ui,
                        &mut draft.preferences.theme,
                        theme,
                        language.label(theme.label()),
                    );
                }
            });
            ui.end_row();
        });
}
