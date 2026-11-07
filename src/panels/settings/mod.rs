//! The Settings window: the default material, the hotkeys and the appearance.

mod appearance;
mod hotkeys;
mod material;

use crate::settings::Preferences;
use crate::ui_components::button::Button;
use crate::ui_components::tabs::{Tab, tab_bar, tab_value};
use appearance::appearance_tab;
use bevy_egui::egui::{self, Key, Modifiers};
use hotkeys::{Capture, capture_keys, hotkeys_tab};
use leathercraft::i18n::{Label, Language};
use material::material_tab;

const DIALOG_WIDTH: f32 = 420.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SettingsTab {
    #[default]
    Material,
    Hotkeys,
    Appearance,
}

/// The preferences being edited while the Settings window is open; applied only on OK.
#[derive(Clone, Debug)]
pub struct SettingsDraft {
    pub preferences: Preferences,
    tab: SettingsTab,
    capture: Option<Capture>,
}

impl SettingsDraft {
    pub fn new(preferences: &Preferences) -> Self {
        Self {
            preferences: preferences.clone(),
            tab: SettingsTab::default(),
            capture: None,
        }
    }
}

pub enum SettingsResult {
    Save,
    Cancel,
}

/// Clicking outside the window does nothing, so a stray click cannot lose the changes.
pub fn settings_dialog(
    ctx: &egui::Context,
    draft: &mut SettingsDraft,
    language: Language,
) -> Option<SettingsResult> {
    // Taken before the window is drawn, so that the keys reach neither its widgets nor Esc below.
    capture_keys(ctx, draft);

    let modal = egui::Modal::new(egui::Id::new("settings_dialog")).show(ctx, |ui| {
        ui.set_width(DIALOG_WIDTH);
        ui.heading(language.label(Label::SettingsTitle));
        ui.add_space(6.0);
        tab_bar(ui, |ui| {
            let tabs = [
                (SettingsTab::Material, Label::DefaultMaterial),
                (SettingsTab::Hotkeys, Label::Hotkeys),
                (SettingsTab::Appearance, Label::Appearance),
            ];
            for (tab, label) in tabs {
                tab_value(ui, &mut draft.tab, tab, Tab::new(language.label(label)));
            }
        });
        ui.add_space(8.0);
        match draft.tab {
            SettingsTab::Material => material_tab(ui, draft, language),
            SettingsTab::Hotkeys => hotkeys_tab(ui, draft, language),
            SettingsTab::Appearance => appearance_tab(ui, draft, language),
        }
        ui.add_space(6.0);
        ui.separator();
        // Right to left, so OK ends up at the right edge with Cancel before it.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let can_save = draft.capture.is_none() && !draft.preferences.hotkeys.has_conflicts();
            if ui
                .add_enabled(can_save, Button::primary(language.label(Label::Ok)))
                .clicked()
            {
                return Some(SettingsResult::Save);
            }
            if ui
                .add(Button::secondary(language.label(Label::Cancel)))
                .clicked()
            {
                return Some(SettingsResult::Cancel);
            }
            None
        })
        .inner
    });
    // Esc closes an open dropdown first, and only then the window.
    let escaped = !modal.any_popup_open
        && ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));
    modal.inner.or(escaped.then_some(SettingsResult::Cancel))
}
