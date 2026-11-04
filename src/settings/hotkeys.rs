//! Configurable keyboard shortcuts, saved as readable strings such as `"Ctrl+Shift+Z"`.

use crate::panels::tools::Tool;
use bevy_egui::egui::{self, Key, KeyboardShortcut, Modifiers};
use leathercraft::i18n::Label;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A command that can be bound to keyboard shortcuts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    New,
    Open,
    Save,
    SaveAs,
    Undo,
    Redo,
    SelectAll,
    Delete,
    ToolSelect,
    ToolEdge,
    ToolDraw,
    ToolRectangle,
    ToolEllipse,
    ToolRegularPolygon,
    ToolCenterline,
    ToolStitchHoles,
    ToggleView,
}

impl Action {
    pub const ALL: [Self; 17] = [
        Self::New,
        Self::Open,
        Self::Save,
        Self::SaveAs,
        Self::Undo,
        Self::Redo,
        Self::SelectAll,
        Self::Delete,
        Self::ToolSelect,
        Self::ToolEdge,
        Self::ToolDraw,
        Self::ToolRectangle,
        Self::ToolEllipse,
        Self::ToolRegularPolygon,
        Self::ToolCenterline,
        Self::ToolStitchHoles,
        Self::ToggleView,
    ];

    pub fn label(self) -> Label {
        match self {
            Self::New => Label::New,
            Self::Open => Label::Open,
            Self::Save => Label::Save,
            Self::SaveAs => Label::SaveAs,
            Self::Undo => Label::Undo,
            Self::Redo => Label::Redo,
            Self::SelectAll => Label::SelectAll,
            Self::Delete => Label::DeleteSelection,
            Self::ToolSelect => Label::Select,
            Self::ToolEdge => Label::Edge,
            Self::ToolDraw => Label::Draw,
            Self::ToolRectangle => Label::Rectangle,
            Self::ToolEllipse => Label::Ellipse,
            Self::ToolRegularPolygon => Label::RegularPolygon,
            Self::ToolCenterline => Label::Centerline,
            Self::ToolStitchHoles => Label::StitchHoles,
            Self::ToggleView => Label::ToggleView,
        }
    }

    /// The tool the action switches to, for the tool actions.
    pub fn tool(self) -> Option<Tool> {
        match self {
            Self::ToolSelect => Some(Tool::Select),
            Self::ToolEdge => Some(Tool::Edge),
            Self::ToolDraw => Some(Tool::Draw),
            Self::ToolRectangle => Some(Tool::Rectangle),
            Self::ToolEllipse => Some(Tool::Ellipse),
            Self::ToolRegularPolygon => Some(Tool::RegularPolygon),
            Self::ToolCenterline => Some(Tool::Centerline),
            Self::ToolStitchHoles => Some(Tool::StitchHoles),
            _ => None,
        }
    }

    pub fn default_shortcuts(self) -> Vec<KeyboardShortcut> {
        let command = |key| KeyboardShortcut::new(Modifiers::COMMAND, key);
        let command_shift =
            |key| KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), key);
        let plain = |key| KeyboardShortcut::new(Modifiers::NONE, key);
        match self {
            Self::New => vec![command(Key::N)],
            Self::Open => vec![command(Key::O)],
            Self::Save => vec![command(Key::S)],
            Self::SaveAs => vec![command_shift(Key::S)],
            Self::Undo => vec![command(Key::Z)],
            Self::Redo => vec![command_shift(Key::Z), command(Key::Y)],
            Self::SelectAll => vec![command(Key::A)],
            Self::Delete => vec![plain(Key::Delete), plain(Key::Backspace)],
            Self::ToolSelect => vec![plain(Key::V)],
            Self::ToolEdge => vec![plain(Key::E)],
            Self::ToolDraw => vec![plain(Key::P)],
            Self::ToolRectangle => vec![plain(Key::R)],
            Self::ToolEllipse => vec![plain(Key::L)],
            Self::ToolRegularPolygon => vec![plain(Key::G)],
            Self::ToolCenterline => vec![plain(Key::C)],
            Self::ToolStitchHoles => vec![plain(Key::H)],
            Self::ToggleView => vec![plain(Key::Tab)],
        }
    }
}

/// The shortcuts of every action. An action with an empty list is unbound.
#[derive(Clone, Debug, PartialEq)]
pub struct Hotkeys {
    bindings: BTreeMap<Action, Vec<KeyboardShortcut>>,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            bindings: Action::ALL
                .into_iter()
                .map(|action| (action, action.default_shortcuts()))
                .collect(),
        }
    }
}

impl Hotkeys {
    /// Reads the saved shortcuts. Actions missing from `value` keep their defaults, and
    /// shortcuts that don't parse are skipped.
    pub fn from_json(value: serde_json::Value) -> Self {
        let mut hotkeys = Self::default();
        let saved: BTreeMap<String, Vec<String>> = match serde_json::from_value(value) {
            Ok(saved) => saved,
            Err(error) => {
                bevy::log::warn!("Ignoring saved hotkeys: {error}");
                return hotkeys;
            }
        };
        for (name, texts) in saved {
            let Ok(action) =
                serde_json::from_value::<Action>(serde_json::Value::String(name.clone()))
            else {
                bevy::log::warn!("Ignoring hotkeys of unknown action {name:?}");
                continue;
            };
            let shortcuts = texts
                .iter()
                .filter_map(|text| {
                    let shortcut = parse_shortcut(text);
                    if shortcut.is_none() {
                        bevy::log::warn!("Ignoring unknown shortcut {text:?} of {name}");
                    }
                    shortcut
                })
                .collect();
            hotkeys.bindings.insert(action, shortcuts);
        }
        hotkeys
    }

    pub fn to_json(&self) -> serde_json::Value {
        let map: BTreeMap<Action, Vec<String>> = self
            .bindings
            .iter()
            .map(|(action, shortcuts)| (*action, shortcuts.iter().map(format_shortcut).collect()))
            .collect();
        serde_json::to_value(map).unwrap_or_default()
    }

    pub fn shortcuts(&self, action: Action) -> &[KeyboardShortcut] {
        self.bindings.get(&action).map_or(&[], Vec::as_slice)
    }

    pub fn shortcuts_mut(&mut self, action: Action) -> &mut Vec<KeyboardShortcut> {
        self.bindings.entry(action).or_default()
    }

    /// The shortcut shown next to the action in menus.
    pub fn first(&self, action: Action) -> Option<&KeyboardShortcut> {
        self.shortcuts(action).first()
    }

    /// Whether `shortcut` is bound to more than one action, or twice to one.
    pub fn is_conflicting(&self, shortcut: &KeyboardShortcut) -> bool {
        self.bindings
            .values()
            .flatten()
            .filter(|bound| *bound == shortcut)
            .count()
            > 1
    }

    pub fn has_conflicts(&self) -> bool {
        self.bindings
            .values()
            .flatten()
            .any(|shortcut| self.is_conflicting(shortcut))
    }

    /// Consumes the first pressed shortcut and returns its action. Shortcuts with more
    /// modifiers are tried first, because matching ignores an extra Shift or Alt: otherwise
    /// Ctrl+Z would also take Ctrl+Shift+Z.
    pub fn triggered(&self, ctx: &egui::Context) -> Option<Action> {
        let mut candidates: Vec<(Action, &KeyboardShortcut)> = self
            .bindings
            .iter()
            .flat_map(|(action, shortcuts)| {
                shortcuts.iter().map(move |shortcut| (*action, shortcut))
            })
            .collect();
        candidates
            .sort_by_key(|(_, shortcut)| std::cmp::Reverse(modifier_count(shortcut.modifiers)));
        ctx.input_mut(|input| {
            candidates
                .into_iter()
                .find(|(_, shortcut)| input.consume_shortcut(shortcut))
                .map(|(action, _)| action)
        })
    }
}

fn modifier_count(modifiers: Modifiers) -> usize {
    usize::from(modifiers.command || modifiers.ctrl)
        + usize::from(modifiers.shift)
        + usize::from(modifiers.alt)
}

/// Formats a shortcut as saved: `Ctrl+Shift+Z`. `Ctrl` stands for Command on macOS.
pub fn format_shortcut(shortcut: &KeyboardShortcut) -> String {
    let modifiers = shortcut.modifiers;
    let mut parts = Vec::new();
    if modifiers.command || modifiers.ctrl {
        parts.push("Ctrl");
    }
    if modifiers.shift {
        parts.push("Shift");
    }
    if modifiers.alt {
        parts.push("Alt");
    }
    parts.push(shortcut.logical_key.name());
    parts.join("+")
}

/// Parses `Ctrl+Shift+Z`; modifier names ignore case, and `Cmd` is accepted for `Ctrl`.
pub fn parse_shortcut(text: &str) -> Option<KeyboardShortcut> {
    let mut parts: Vec<&str> = text.split('+').map(str::trim).collect();
    // A trailing empty part means the key itself is `+`.
    if parts.len() > 1 && parts.last() == Some(&"") {
        parts.pop();
        if let Some(last) = parts.last_mut() {
            *last = "+";
        }
    }
    let key_name = parts.pop()?;
    let mut modifiers = Modifiers::NONE;
    for part in parts {
        let extra = match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" | "cmd" | "command" => Modifiers::COMMAND,
            "shift" => Modifiers::SHIFT,
            "alt" | "option" => Modifiers::ALT,
            _ => return None,
        };
        modifiers = modifiers.plus(extra);
    }
    let key = Key::from_name(key_name).or_else(|| Key::from_name(&capitalize(key_name)))?;
    Some(KeyboardShortcut::new(modifiers, key))
}

fn capitalize(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_round_trip() {
        for action in Action::ALL {
            for shortcut in action.default_shortcuts() {
                let text = format_shortcut(&shortcut);
                assert_eq!(parse_shortcut(&text), Some(shortcut), "{text}");
            }
        }
    }

    #[test]
    fn parsing_ignores_case() {
        assert_eq!(
            parse_shortcut("ctrl+shift+z"),
            Some(KeyboardShortcut::new(
                Modifiers::COMMAND.plus(Modifiers::SHIFT),
                Key::Z
            ))
        );
        assert_eq!(
            parse_shortcut("delete"),
            Some(KeyboardShortcut::new(Modifiers::NONE, Key::Delete))
        );
        assert_eq!(parse_shortcut("Hyper+Z"), None);
        assert_eq!(parse_shortcut("Ctrl+Nonsense"), None);
    }

    #[test]
    fn missing_and_broken_entries_fall_back() {
        let hotkeys = Hotkeys::from_json(serde_json::json!({
            "undo": ["Ctrl+U", "nonsense"],
            "redo": [],
            "unknown_action": ["Ctrl+Q"],
        }));
        assert_eq!(
            hotkeys.shortcuts(Action::Undo),
            [KeyboardShortcut::new(Modifiers::COMMAND, Key::U)]
        );
        assert!(hotkeys.shortcuts(Action::Redo).is_empty());
        assert_eq!(
            hotkeys.shortcuts(Action::Save),
            Action::Save.default_shortcuts()
        );
        assert_eq!(
            Hotkeys::from_json(serde_json::json!(42)),
            Hotkeys::default()
        );
    }

    #[test]
    fn json_round_trip() {
        let hotkeys = Hotkeys::default();
        assert_eq!(Hotkeys::from_json(hotkeys.to_json()), hotkeys);
        assert!(!hotkeys.has_conflicts());
    }
}
