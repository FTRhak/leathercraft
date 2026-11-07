//! Which palette entry each input state is painted with.
//!
//! The colors themselves live in `Palette::input`.

use crate::ui_components::palette::{InputColors, InputPaint};

/// The interaction state a field is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InputState {
    Default,
    Hover,
    /// With keyboard focus, while its text is edited; wins over `Accent` and `Hover`.
    Focus,
    /// A number that matches a canvas layer; wins over `Hover`, so the meaning stays visible.
    Accent,
    Disabled,
}

impl InputState {
    pub(super) fn new(enabled: bool, focused: bool, accent: bool, hovered: bool) -> Self {
        if !enabled {
            Self::Disabled
        } else if focused {
            Self::Focus
        } else if accent {
            Self::Accent
        } else if hovered {
            Self::Hover
        } else {
            Self::Default
        }
    }
}

/// The paint of a field in `state`, from the input colors of the current theme.
pub(super) fn paint(colors: &InputColors, state: InputState) -> &InputPaint {
    match state {
        InputState::Default => &colors.default,
        InputState::Hover => &colors.hover,
        InputState::Focus => &colors.focus,
        InputState::Accent => &colors.accent,
        InputState::Disabled => &colors.disabled,
    }
}
