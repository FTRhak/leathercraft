//! Which palette entry each dropdown state is painted with.
//!
//! The colors themselves live in `Palette::dropdown`.

use crate::ui_components::palette::{DropdownColors, FieldPaint};

/// The interaction state the closed dropdown is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DropdownState {
    Default,
    Hover,
    /// With keyboard focus, or while its list is open; wins over `Hover`.
    Focus,
    Disabled,
}

impl DropdownState {
    pub(super) fn new(enabled: bool, focused: bool, hovered: bool) -> Self {
        if !enabled {
            Self::Disabled
        } else if focused {
            Self::Focus
        } else if hovered {
            Self::Hover
        } else {
            Self::Default
        }
    }
}

/// The paint of the dropdown in `state`, from the dropdown colors of the current theme.
pub(super) fn paint(colors: &DropdownColors, state: DropdownState) -> &FieldPaint {
    match state {
        DropdownState::Default => &colors.default,
        DropdownState::Hover => &colors.hover,
        DropdownState::Focus => &colors.focus,
        DropdownState::Disabled => &colors.disabled,
    }
}
