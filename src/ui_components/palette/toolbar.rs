use bevy_egui::egui::Color32;

use super::hex;

/// The tool bar under the title bar and its `ui_components::tool_button`s, from the "CAD
/// Toolbar" and the `ToolBtn` component of the Brilliant design. The design has a dark board
/// only, and no hover state; the hover and light values follow the secondary and primary
/// buttons, as the dark ones do.
pub(crate) struct ToolbarColors {
    pub(crate) background: Color32,
    /// The line under the bar.
    pub(crate) border: Color32,
    /// The short vertical lines between the groups of tools.
    pub(crate) divider: Color32,
    pub(crate) button: ToolButtonColors,
}

/// A tool button in each state; `content` paints its icon or label.
pub(crate) struct ToolButtonColors {
    pub(crate) idle: ToolButtonPaint,
    pub(crate) hover: ToolButtonPaint,
    /// The current tool or mode.
    pub(crate) active: ToolButtonPaint,
    pub(crate) disabled: ToolButtonPaint,
    /// The drop shadow under the active button.
    pub(crate) active_shadow: Color32,
    /// The ring around a button that has keyboard focus; the same as `accent.base`.
    pub(crate) focus: Color32,
}

/// The paint of a tool button in one state.
pub(crate) struct ToolButtonPaint {
    pub(crate) fill: Color32,
    pub(crate) stroke: Color32,
    pub(crate) content: Color32,
}

const fn paint(fill: u32, stroke: u32, content: u32) -> ToolButtonPaint {
    ToolButtonPaint {
        fill: hex(fill),
        stroke: hex(stroke),
        content: hex(content),
    }
}

impl ToolbarColors {
    pub(super) const DARK: Self = Self {
        background: hex(0x191A1F),
        border: hex(0x26282E),
        divider: hex(0x2A2C34),
        button: ToolButtonColors {
            idle: paint(0x23252B, 0x32353E, 0xC8CAD2),
            hover: paint(0x2A2C33, 0x4A4E59, 0xECEEF2),
            active: paint(0x1B6B85, 0x78C3D7, 0xFFFFFF),
            disabled: paint(0x1C1D22, 0x26282E, 0x6E717C),
            active_shadow: Color32::from_black_alpha(89),
            focus: hex(0x78C3D7),
        },
    };
    pub(super) const LIGHT: Self = Self {
        background: hex(0xF5F6F8),
        border: hex(0xD9DCE2),
        divider: hex(0xD2D5DC),
        button: ToolButtonColors {
            idle: paint(0xE6E8ED, 0xBCC0C9, 0x4A4E59),
            hover: paint(0xDDE0E6, 0xA9AEB8, 0x17181C),
            active: paint(0x176E8D, 0x1E82A5, 0xFFFFFF),
            disabled: paint(0xEEF0F3, 0xE3E5EA, 0x9A9EA9),
            active_shadow: Color32::from_black_alpha(40),
            focus: hex(0x1E82A5),
        },
    };
}
