//! Which palette entry each list row state is painted with.
//!
//! The colors themselves live in `Palette::list`.

use crate::ui_components::palette::{ListColors, ListItemPaint};

/// The interaction state a row is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ItemState {
    Default,
    /// Under the pointer or being pressed.
    Hover,
    /// The current row; hovering it changes nothing.
    Selected,
    /// A row that no longer applies; it is also faded.
    Dimmed,
    /// A dimmed row under the pointer: the hover fill behind the dimmed text.
    DimmedHover,
    Disabled,
}

impl ItemState {
    pub(super) fn new(selected: bool, dimmed: bool, enabled: bool, hovered: bool) -> Self {
        match (enabled, selected, dimmed, hovered) {
            (false, ..) => Self::Disabled,
            (true, true, ..) => Self::Selected,
            (true, false, true, true) => Self::DimmedHover,
            (true, false, true, false) => Self::Dimmed,
            (true, false, false, true) => Self::Hover,
            (true, false, false, false) => Self::Default,
        }
    }

    /// Whether the text and icon are drawn faded.
    pub(super) fn is_dimmed(self) -> bool {
        matches!(self, Self::Dimmed | Self::DimmedHover)
    }
}

/// The paint of a row in `state`, from the list colors of the current theme.
pub(super) fn paint(colors: &ListColors, state: ItemState) -> &ListItemPaint {
    match state {
        ItemState::Default => &colors.default,
        ItemState::Hover => &colors.hover,
        ItemState::Selected => &colors.selected,
        ItemState::Dimmed | ItemState::DimmedHover => &colors.dimmed,
        ItemState::Disabled => &colors.disabled,
    }
}
