//! The Hotkeys tab of the Settings window: the shortcuts of each action, and capturing new ones.

use super::SettingsDraft;
use crate::settings::{Action, Hotkeys};
use crate::ui_components::button::Button;
use bevy_egui::egui::{self, Key, KeyboardShortcut, Modifiers};
use leathercraft::i18n::{Label, Language};

const HOTKEYS_HEIGHT: f32 = 360.0;

/// Which shortcut is waiting for keys: an existing one by index, or a new one.
#[derive(Clone, Copy, Debug)]
pub(super) struct Capture {
    action: Action,
    index: Option<usize>,
}

enum HotkeyChange {
    Capture(Capture),
    Remove(Action, usize),
    Reset,
}

pub(super) fn hotkeys_tab(ui: &mut egui::Ui, draft: &mut SettingsDraft, language: Language) {
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
        if ui
            .add(Button::outline(language.label(Label::ResetToDefaults)))
            .clicked()
        {
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
pub(super) fn capture_keys(ctx: &egui::Context, draft: &mut SettingsDraft) {
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
