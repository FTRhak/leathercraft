//! The Settings window: the default material and the hotkeys.

use crate::panels::material::material_fields;
use crate::settings::{Action, Hotkeys, Preferences};
use bevy_egui::egui::{self, Key, KeyboardShortcut, Modifiers};
use leathercraft::i18n::{Label, Language};

const DIALOG_WIDTH: f32 = 420.0;
const HOTKEYS_HEIGHT: f32 = 360.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SettingsTab {
    #[default]
    Material,
    Hotkeys,
}

/// Which shortcut is waiting for keys: an existing one by index, or a new one.
#[derive(Clone, Copy, Debug)]
struct Capture {
    action: Action,
    index: Option<usize>,
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

enum HotkeyChange {
    Capture(Capture),
    Remove(Action, usize),
    Reset,
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
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut draft.tab,
                SettingsTab::Material,
                language.label(Label::DefaultMaterial),
            );
            ui.selectable_value(
                &mut draft.tab,
                SettingsTab::Hotkeys,
                language.label(Label::Hotkeys),
            );
        });
        ui.separator();
        match draft.tab {
            SettingsTab::Material => {
                material_fields(
                    ui,
                    &mut draft.preferences.default_material,
                    language,
                    "settings",
                );
            }
            SettingsTab::Hotkeys => hotkeys_tab(ui, draft, language),
        }
        ui.add_space(6.0);
        ui.separator();
        ui.horizontal(|ui| {
            let can_save = draft.capture.is_none() && !draft.preferences.hotkeys.has_conflicts();
            if ui
                .add_enabled(can_save, egui::Button::new(language.label(Label::Ok)))
                .clicked()
            {
                return Some(SettingsResult::Save);
            }
            if ui.button(language.label(Label::Cancel)).clicked() {
                return Some(SettingsResult::Cancel);
            }
            None
        })
        .inner
    });
    // Esc closes an open combo box first, and only then the window.
    let escaped = !modal.any_popup_open
        && ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));
    modal.inner.or(escaped.then_some(SettingsResult::Cancel))
}

fn hotkeys_tab(ui: &mut egui::Ui, draft: &mut SettingsDraft, language: Language) {
    let hotkeys = &draft.preferences.hotkeys;
    let mut change = None;
    egui::ScrollArea::vertical()
        .max_height(HOTKEYS_HEIGHT)
        .show(ui, |ui| {
            egui::Grid::new("hotkeys_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for action in Action::ALL {
                        ui.label(language.label(action.label()));
                        ui.horizontal_wrapped(|ui| {
                            for (index, shortcut) in hotkeys.shortcuts(action).iter().enumerate() {
                                let capture = Capture {
                                    action,
                                    index: Some(index),
                                };
                                if let Some(next) = shortcut_button(
                                    ui,
                                    draft.capture,
                                    capture,
                                    Some(shortcut),
                                    hotkeys,
                                    language,
                                ) {
                                    change = Some(next);
                                }
                                if ui
                                    .small_button("×")
                                    .on_hover_text(language.label(Label::RemoveShortcut))
                                    .clicked()
                                {
                                    change = Some(HotkeyChange::Remove(action, index));
                                }
                            }
                            let adding = Capture {
                                action,
                                index: None,
                            };
                            if let Some(next) =
                                shortcut_button(ui, draft.capture, adding, None, hotkeys, language)
                            {
                                change = Some(next);
                            }
                        });
                        ui.end_row();
                    }
                });
        });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button(language.label(Label::ResetToDefaults)).clicked() {
            change = Some(HotkeyChange::Reset);
        }
        if hotkeys.has_conflicts() {
            ui.colored_label(
                ui.visuals().error_fg_color,
                language.label(Label::ShortcutConflict),
            );
        }
    });

    match change {
        Some(HotkeyChange::Capture(capture)) => draft.capture = Some(capture),
        Some(HotkeyChange::Remove(action, index)) => {
            draft
                .preferences
                .hotkeys
                .shortcuts_mut(action)
                .remove(index);
            draft.capture = None;
        }
        Some(HotkeyChange::Reset) => {
            draft.preferences.hotkeys = Hotkeys::default();
            draft.capture = None;
        }
        None => {}
    }
}

/// The button of an existing shortcut, or the `+` button when `shortcut` is `None`.
/// While it waits for keys it says so; clicking it starts waiting.
fn shortcut_button(
    ui: &mut egui::Ui,
    current: Option<Capture>,
    this: Capture,
    shortcut: Option<&KeyboardShortcut>,
    hotkeys: &Hotkeys,
    language: Language,
) -> Option<HotkeyChange> {
    let waiting =
        current.is_some_and(|capture| capture.action == this.action && capture.index == this.index);
    let button = if waiting {
        egui::Button::new(language.label(Label::PressKeys)).selected(true)
    } else {
        match shortcut {
            Some(shortcut) => {
                let text = egui::RichText::new(ui.ctx().format_shortcut(shortcut)).monospace();
                let text = if hotkeys.is_conflicting(shortcut) {
                    text.color(ui.visuals().error_fg_color)
                } else {
                    text
                };
                egui::Button::new(text)
            }
            None => egui::Button::new("+"),
        }
    };
    let mut response = ui.add(button);
    if shortcut.is_none() && !waiting {
        response = response.on_hover_text(language.label(Label::AddShortcut));
    }
    response.clicked().then_some(HotkeyChange::Capture(this))
}

/// While a shortcut waits for keys, the next key press becomes it; Esc alone cancels.
fn capture_keys(ctx: &egui::Context, draft: &mut SettingsDraft) {
    let Some(capture) = draft.capture else {
        return;
    };
    let pressed = ctx.input_mut(|input| {
        let position = input.events.iter().position(|event| {
            matches!(
                event,
                egui::Event::Key {
                    pressed: true,
                    repeat: false,
                    ..
                }
            )
        })?;
        match input.events.remove(position) {
            egui::Event::Key { key, modifiers, .. } => Some((key, modifiers)),
            _ => None,
        }
    });
    let Some((key, modifiers)) = pressed else {
        return;
    };
    draft.capture = None;
    let modifiers = normalize(modifiers);
    if key == Key::Escape && modifiers.is_none() {
        return;
    }
    let shortcut = KeyboardShortcut::new(modifiers, key);
    let shortcuts = draft.preferences.hotkeys.shortcuts_mut(capture.action);
    match capture.index {
        Some(index) if index < shortcuts.len() => shortcuts[index] = shortcut,
        _ if !shortcuts.contains(&shortcut) => shortcuts.push(shortcut),
        _ => {}
    }
}

/// Ctrl is saved as the platform's command key, the way the default shortcuts are written.
fn normalize(modifiers: Modifiers) -> Modifiers {
    let mut normalized = Modifiers::NONE;
    if modifiers.ctrl || modifiers.command || modifiers.mac_cmd {
        normalized = normalized.plus(Modifiers::COMMAND);
    }
    if modifiers.shift {
        normalized = normalized.plus(Modifiers::SHIFT);
    }
    if modifiers.alt {
        normalized = normalized.plus(Modifiers::ALT);
    }
    normalized
}
