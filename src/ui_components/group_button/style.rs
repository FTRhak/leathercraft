//! Which palette entry each segment state is painted with.
//!
//! The colors themselves live in `Palette::group_button`.

use crate::ui_components::palette::{GroupButtonColors, SegmentPaint};

/// The interaction state one segment is drawn in. Focus also adds a ring inside the segment, in
/// any state but the disabled ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SegmentState {
    Default,
    /// An unselected segment under the pointer or being pressed.
    Hover,
    /// The chosen option; hovering it changes nothing.
    Selected,
    /// An unselected segment with keyboard focus.
    Focus,
    Disabled,
    /// The chosen option of a disabled group.
    DisabledSelected,
}

impl SegmentState {
    pub(super) fn new(selected: bool, enabled: bool, hovered: bool, focused: bool) -> Self {
        match (enabled, selected) {
            (false, true) => Self::DisabledSelected,
            (false, false) => Self::Disabled,
            (true, true) => Self::Selected,
            (true, false) if hovered => Self::Hover,
            (true, false) if focused => Self::Focus,
            (true, false) => Self::Default,
        }
    }

    pub(super) fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled | Self::DisabledSelected)
    }

    pub(super) fn is_selected(self) -> bool {
        matches!(self, Self::Selected | Self::DisabledSelected)
    }
}

/// The paint of a segment in `state`, from the group button colors of the current theme.
pub(super) fn paint(colors: &GroupButtonColors, state: SegmentState) -> &SegmentPaint {
    match state {
        SegmentState::Default => &colors.default,
        SegmentState::Hover => &colors.hover,
        SegmentState::Selected => &colors.selected,
        SegmentState::Focus => &colors.focus,
        SegmentState::Disabled => &colors.disabled,
        SegmentState::DisabledSelected => &colors.disabled_selected,
    }
}
