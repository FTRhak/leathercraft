//! Which palette entry each checkbox value and state is painted with.
//!
//! The colors themselves live in `Palette::checkbox`.

use crate::ui_components::palette::{CheckboxColors, CheckboxPaint};

/// What the box shows: a check, nothing, or a dash for a value that differs across a selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CheckboxValue {
    On,
    Off,
    Mixed,
}

impl CheckboxValue {
    pub(super) fn new(checked: bool, mixed: bool) -> Self {
        match (mixed, checked) {
            (true, _) => Self::Mixed,
            (false, true) => Self::On,
            (false, false) => Self::Off,
        }
    }
}

/// The interaction state the checkbox is drawn in. Focus is not a state of its own: it adds a
/// ring around the box in any enabled state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CheckboxState {
    Default,
    /// Under the pointer or being pressed.
    Hover,
    Disabled,
}

impl CheckboxState {
    pub(super) fn new(enabled: bool, hovered: bool) -> Self {
        match (enabled, hovered) {
            (false, _) => Self::Disabled,
            (true, true) => Self::Hover,
            (true, false) => Self::Default,
        }
    }
}

/// The paint of `value` in `state`, from the checkbox colors of the current theme. A mixed box
/// is filled like a checked one.
pub(super) fn paint(
    colors: &CheckboxColors,
    value: CheckboxValue,
    state: CheckboxState,
) -> &CheckboxPaint {
    let states = match value {
        CheckboxValue::On | CheckboxValue::Mixed => &colors.on,
        CheckboxValue::Off => &colors.off,
    };
    match state {
        CheckboxState::Default => &states.default,
        CheckboxState::Hover => &states.hover,
        CheckboxState::Disabled => &states.disabled,
    }
}
