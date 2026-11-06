//! The interface color theme, a preference saved with the others.

use super::Preferences;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use leathercraft::i18n::Label;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub const ALL: [Self; 2] = [Self::Dark, Self::Light];

    pub fn label(self) -> Label {
        match self {
            Self::Dark => Label::ThemeDark,
            Self::Light => Label::ThemeLight,
        }
    }

    fn egui(self) -> egui::Theme {
        match self {
            Self::Dark => egui::Theme::Dark,
            Self::Light => egui::Theme::Light,
        }
    }
}

/// Applies the theme to egui on startup and whenever the preferences change. The 3D preview
/// follows it on its own, in `preview`.
pub fn apply_theme(
    mut contexts: EguiContexts,
    preferences: Res<Preferences>,
    mut applied: Local<Option<Theme>>,
) -> Result {
    let theme = preferences.theme;
    if *applied == Some(theme) {
        return Ok(());
    }
    contexts.ctx_mut()?.set_theme(theme.egui());
    *applied = Some(theme);
    Ok(())
}
