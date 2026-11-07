//! Which palette entry each radio value and state is painted with.
//!
//! The radio button shares its colors with the checkbox, in `Palette::checkbox`: a selected
//! button is painted like a checked box, with the mark color on its dot.

use crate::ui_components::palette::{CheckboxColors, CheckboxPaint};

/// The interaction state the radio button is drawn in. Focus is not a state of its own: it adds
/// a ring around the circle in any enabled state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RadioState {
    Default,
    /// Under the pointer or being pressed.
    Hover,
    Disabled,
}

impl RadioState {
    pub(super) fn new(enabled: bool, hovered: bool) -> Self {
        match (enabled, hovered) {
            (false, _) => Self::Disabled,
            (true, true) => Self::Hover,
            (true, false) => Self::Default,
        }
    }
}

/// The paint of a radio button, selected or not, in `state`.
pub(super) fn paint(colors: &CheckboxColors, selected: bool, state: RadioState) -> &CheckboxPaint {
    let states = if selected { &colors.on } else { &colors.off };
    match state {
        RadioState::Default => &states.default,
        RadioState::Hover => &states.hover,
        RadioState::Disabled => &states.disabled,
    }
}
