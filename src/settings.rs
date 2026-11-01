use leathercraft::i18n::Language;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct Settings {
    language: String,
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
    let contents = fs::read_to_string(settings_path()?).ok()?;
    let settings: Settings = serde_json::from_str(&contents).ok()?;
    Language::from_tag(&settings.language)
}

/// The preference is not worth interrupting the user over, so a failure is only logged.
pub fn save_language(language: Language) {
    if let Err(error) = write_language(language) {
        bevy::log::error!("Could not save settings: {error}");
    }
}

fn write_language(language: Language) -> io::Result<()> {
    let path = settings_path().ok_or_else(|| io::Error::other("no config directory"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let settings = Settings {
        language: language.tag().to_owned(),
    };
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&settings)?),
    )
}
