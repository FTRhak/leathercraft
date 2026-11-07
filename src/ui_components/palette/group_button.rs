use bevy_egui::egui::Color32;

use super::hex;

/// The group button of `ui_components::group_button`, from the Brilliant `Button Group` and
/// `Segment` components. The design has dark boards only; the light values follow the light
/// secondary button (unselected) and primary button (selected), as the dark ones do.
pub(crate) struct GroupButtonColors {
    pub(crate) default: SegmentPaint,
    /// An unselected segment under the pointer.
    pub(crate) hover: SegmentPaint,
    /// The chosen option; the same as the primary button.
    pub(crate) selected: SegmentPaint,
    /// An unselected segment with keyboard focus.
    pub(crate) focus: SegmentPaint,
    pub(crate) disabled: SegmentPaint,
    /// The chosen option of a disabled group; the design has no board for it, so it follows the
    /// disabled primary button.
    pub(crate) disabled_selected: SegmentPaint,
    /// The edge of the group and the dividers between segments.
    pub(crate) border: Color32,
    pub(crate) disabled_border: Color32,
    /// The ring inside a focused segment; the same as `accent.base`.
    pub(crate) focus_ring: Color32,
}

/// The paint of one segment in one state.
pub(crate) struct SegmentPaint {
    pub(crate) fill: Color32,
    pub(crate) text: Color32,
}

const fn segment(fill: u32, text: u32) -> SegmentPaint {
    SegmentPaint {
        fill: hex(fill),
        text: hex(text),
    }
}

impl GroupButtonColors {
    pub(super) const DARK: Self = Self {
        default: segment(0x23252B, 0xB8BBC6),
        hover: segment(0x2A2C33, 0xECEEF2),
        selected: segment(0x1B6B85, 0xFFFFFF),
        focus: segment(0x23252B, 0xDCDFE6),
        disabled: segment(0x1C1D22, 0x5A5E69),
        disabled_selected: segment(0x1B3B47, 0x6B8C96),
        border: hex(0x383B46),
        disabled_border: hex(0x2B2E37),
        focus_ring: hex(0x78C3D7),
    };
    pub(super) const LIGHT: Self = Self {
        default: segment(0xE6E8ED, 0x4A4E59),
        hover: segment(0xDDE0E6, 0x17181C),
        selected: segment(0x176E8D, 0xFFFFFF),
        focus: segment(0xE6E8ED, 0x2A2C33),
        disabled: segment(0xEEF0F3, 0x9A9EA9),
        disabled_selected: segment(0xD5E6EC, 0x8AA9B4),
        border: hex(0xBCC0C9),
        disabled_border: hex(0xE3E5EA),
        focus_ring: hex(0x1E82A5),
    };
}
