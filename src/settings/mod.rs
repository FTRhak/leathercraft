mod hotkeys;

pub use hotkeys::{Action, Hotkeys};

use bevy::prelude::*;
use leathercraft::i18n::Language;
use leathercraft::material::Material;
use leathercraft::snap::SnapSettings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
struct Settings {
    /// Absent until the user chooses a language, so that the OS locale keeps applying.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    /// Kept as raw JSON, so that a layout that no longer parses doesn't lose the other settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dock: Option<serde_json::Value>,
    /// The sections below are also kept as raw JSON, each parsed on its own with a fallback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_material: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hotkeys: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    snap: Option<serde_json::Value>,
}

/// The application preferences edited in the Settings window. Interface state, never part
/// of a project.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct Preferences {
    /// The material the new-project dialog and the startup project start from.
    pub default_material: Material,
    pub hotkeys: Hotkeys,
}

/// The saved preferences; a missing or unusable section keeps its default.
pub fn load_preferences() -> Preferences {
    let Some(settings) = read_settings() else {
        return Preferences::default();
    };
    let default_material = settings
        .default_material
        .and_then(|value| match serde_json::from_value::<Material>(value) {
            Ok(material) => Some(material),
            Err(error) => {
                bevy::log::warn!("Ignoring saved default material: {error}");
                None
            }
        })
        .filter(Material::has_valid_thickness)
        .unwrap_or_default();
    let hotkeys = settings.hotkeys.map(Hotkeys::from_json).unwrap_or_default();
    Preferences {
        default_material,
        hotkeys,
    }
}

pub fn save_preferences(preferences: &Preferences) {
    let material = serde_json::to_value(preferences.default_material).ok();
    let hotkeys = preferences.hotkeys.to_json();
    update_settings(|settings| {
        settings.default_material = material;
        settings.hotkeys = Some(hotkeys);
    });
}

fn settings_path() -> Option<PathBuf> {
    Some(
        dirs::config_dir()?
            .join("leathercraft")
            .join("settings.json"),
    )
}

/// The saved language, else the OS locale's, else the default.
pub fn load_language() -> Language {
    saved_language()
        .or_else(|| sys_locale::get_locale().and_then(|locale| Language::from_tag(&locale)))
        .unwrap_or_default()
}

fn saved_language() -> Option<Language> {
    Language::from_tag(&read_settings()?.language?)
}

/// The saved snapping settings, brought into range; the default if missing or unusable.
pub fn load_snap() -> SnapSettings {
    read_settings()
        .and_then(|settings| settings.snap)
        .and_then(
            |value| match serde_json::from_value::<SnapSettings>(value) {
                Ok(snap) => Some(snap.clamped()),
                Err(error) => {
                    bevy::log::warn!("Ignoring saved snap settings: {error}");
                    None
                }
            },
        )
        .unwrap_or_default()
}

pub fn save_snap(snap: &SnapSettings) {
    let value = serde_json::to_value(snap).ok();
    update_settings(|settings| settings.snap = value);
}

pub fn load_dock() -> Option<serde_json::Value> {
    read_settings()?.dock
}

fn read_settings() -> Option<Settings> {
    let contents = fs::read_to_string(settings_path()?).ok()?;
    serde_json::from_str(&contents).ok()
}

pub fn save_language(language: Language) {
    update_settings(|settings| settings.language = Some(language.tag().to_owned()));
}

pub fn save_dock(dock: &serde_json::Value) {
    update_settings(|settings| settings.dock = Some(dock.clone()));
}

/// The preferences are not worth interrupting the user over, so a failure is only logged.
fn update_settings(change: impl FnOnce(&mut Settings)) {
    let mut settings = read_settings().unwrap_or_default();
    change(&mut settings);
    if let Err(error) = write_settings(&settings) {
        bevy::log::error!("Could not save settings: {error}");
    }
}

fn write_settings(settings: &Settings) -> io::Result<()> {
    let path = settings_path().ok_or_else(|| io::Error::other("no config directory"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(settings)?),
    )
}
