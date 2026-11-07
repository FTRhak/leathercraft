//! The Appearance tab of the Settings window: the interface theme.

use super::SettingsDraft;
use crate::settings::Theme;
use crate::ui_components::group_button::GroupButton;
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};

pub(super) fn appearance_tab(ui: &mut egui::Ui, draft: &mut SettingsDraft, language: Language) {
    egui::Grid::new("settings_appearance")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label(language.label(Label::ThemeField));
            let group = Theme::ALL.into_iter().fold(
                GroupButton::new(&mut draft.preferences.theme),
                |group, theme| group.option(theme, language.label(theme.label())),
            );
            ui.add(group);
            ui.end_row();
        });
}
