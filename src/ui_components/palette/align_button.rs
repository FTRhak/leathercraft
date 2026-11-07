use bevy_egui::egui::Color32;

use super::hex;

/// The icon button of `ui_components::align_button`, from the Brilliant `AlignBtn` component.
/// The design has a dark idle state only; the hover, disabled and light values follow the tool
/// buttons, as the dark idle one does.
pub(crate) struct AlignButtonColors {
    pub(crate) default: AlignButtonPaint,
    pub(crate) hover: AlignButtonPaint,
    pub(crate) disabled: AlignButtonPaint,
    /// The ring around a button that has keyboard focus; the same as `accent.base`.
    pub(crate) focus: Color32,
}

/// The paint of an align button in one state; `icon` is the glyph's color.
pub(crate) struct AlignButtonPaint {
    pub(crate) fill: Color32,
    pub(crate) stroke: Color32,
    pub(crate) icon: Color32,
}

const fn paint(fill: u32, stroke: u32, icon: u32) -> AlignButtonPaint {
    AlignButtonPaint {
        fill: hex(fill),
        stroke: hex(stroke),
        icon: hex(icon),
    }
}

impl AlignButtonColors {
    pub(super) const DARK: Self = Self {
        default: paint(0x25272E, 0x32353E, 0xB8BBC6),
        hover: paint(0x2A2C33, 0x4A4E59, 0xECEEF2),
        disabled: paint(0x1C1D22, 0x26282E, 0x5A5E69),
        focus: hex(0x78C3D7),
    };
    pub(super) const LIGHT: Self = Self {
        default: paint(0xE6E8ED, 0xBCC0C9, 0x4A4E59),
        hover: paint(0xDDE0E6, 0xA9AEB8, 0x17181C),
        disabled: paint(0xEEF0F3, 0xE3E5EA, 0x9A9EA9),
        focus: hex(0x1E82A5),
    };
}
