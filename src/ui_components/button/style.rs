//! Which palette entry each button kind and state is painted with.
//!
//! The colors themselves live in `Palette::button`.

use crate::ui_components::palette::{ButtonColors, ButtonPaint};

/// What the button is for: one `Primary` per dialog (OK), `Secondary` beside it (Cancel),
/// `Outline` for helper actions (Reset to defaults).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    #[default]
    Secondary,
    Outline,
}

/// The interaction state the button is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ButtonState {
    Default,
    Hover,
    Disabled,
}

/// The paint of `kind` in `state`, from the button colors of the current theme.
pub(super) fn paint(colors: &ButtonColors, kind: ButtonKind, state: ButtonState) -> &ButtonPaint {
    let states = match kind {
        ButtonKind::Primary => &colors.primary,
        ButtonKind::Secondary => &colors.secondary,
        ButtonKind::Outline => &colors.outline,
    };
    match state {
        ButtonState::Default => &states.default,
        ButtonState::Hover => &states.hover,
        ButtonState::Disabled => &states.disabled,
    }
}
