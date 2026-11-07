//! Which palette entry each tab state is painted with.
//!
//! The colors themselves live in `Palette::tabs`.

use crate::ui_components::palette::{TabColors, TabPaint};

/// The interaction state a tab is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TabState {
    /// The open tab; hovering it changes nothing.
    Active,
    Inactive,
    /// An inactive tab under the pointer or being pressed.
    Hover,
    /// An inactive tab that can't be opened.
    Disabled,
    /// The open tab, while the whole tab bar is disabled.
    DisabledActive,
}

impl TabState {
    pub(super) fn new(selected: bool, enabled: bool, hovered: bool) -> Self {
        match (selected, enabled) {
            (true, true) => Self::Active,
            (true, false) => Self::DisabledActive,
            (false, false) => Self::Disabled,
            (false, true) if hovered => Self::Hover,
            (false, true) => Self::Inactive,
        }
    }
}

/// The paint of a tab in `state`, from the tab colors of the current theme.
pub(super) fn paint(colors: &TabColors, state: TabState) -> &TabPaint {
    match state {
        TabState::Active => &colors.active,
        TabState::Inactive => &colors.inactive,
        TabState::Hover => &colors.hover,
        TabState::Disabled => &colors.disabled,
        TabState::DisabledActive => &colors.disabled_active,
    }
}
