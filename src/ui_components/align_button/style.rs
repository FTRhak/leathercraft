//! Which palette entry each align button state is painted with.
//!
//! The colors themselves live in `Palette::align_button`.

use crate::ui_components::palette::{AlignButtonColors, AlignButtonPaint};

/// The interaction state the button is drawn in. Focus adds a ring around it in any state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AlignButtonState {
    Default,
    /// Under the pointer or being pressed.
    Hover,
    Disabled,
}

impl AlignButtonState {
    pub(super) fn new(enabled: bool, hovered: bool) -> Self {
        match (enabled, hovered) {
            (false, _) => Self::Disabled,
            (true, true) => Self::Hover,
            (true, false) => Self::Default,
        }
    }
}

/// The paint of an align button in `state`, from the colors of the current theme.
pub(super) fn paint(colors: &AlignButtonColors, state: AlignButtonState) -> &AlignButtonPaint {
    match state {
        AlignButtonState::Default => &colors.default,
        AlignButtonState::Hover => &colors.hover,
        AlignButtonState::Disabled => &colors.disabled,
    }
}
