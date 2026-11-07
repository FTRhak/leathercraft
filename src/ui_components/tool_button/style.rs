//! Which palette entry each tool button state is painted with.
//!
//! The colors themselves live in `Palette::toolbar.button`.

use crate::ui_components::palette::{ToolButtonColors, ToolButtonPaint};

/// The interaction state the button is drawn in. Focus adds a ring around it in any state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ToolButtonState {
    Idle,
    /// Under the pointer or being pressed, and not active.
    Hover,
    /// The current tool or mode; hovering it changes nothing.
    Active,
    Disabled,
}

impl ToolButtonState {
    pub(super) fn new(selected: bool, enabled: bool, hovered: bool) -> Self {
        match (enabled, selected) {
            (false, _) => Self::Disabled,
            (true, true) => Self::Active,
            (true, false) if hovered => Self::Hover,
            (true, false) => Self::Idle,
        }
    }
}

/// The paint of a tool button in `state`, from the colors of the current theme.
pub(super) fn paint(colors: &ToolButtonColors, state: ToolButtonState) -> &ToolButtonPaint {
    match state {
        ToolButtonState::Idle => &colors.idle,
        ToolButtonState::Hover => &colors.hover,
        ToolButtonState::Active => &colors.active,
        ToolButtonState::Disabled => &colors.disabled,
    }
}
