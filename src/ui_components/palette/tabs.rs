use bevy_egui::egui::Color32;

use super::{hex, hex_opt};

/// The tab strip of `ui_components::tabs`, from the Brilliant `Dialog Tab` component.
pub(crate) struct TabColors {
    /// The background of the strip; the same as `surface.input`.
    pub(crate) bar: Color32,
    /// The lines above and below the strip.
    pub(crate) bar_border: Color32,
    /// The open tab.
    pub(crate) active: TabPaint,
    pub(crate) inactive: TabPaint,
    /// An inactive tab under the pointer.
    pub(crate) hover: TabPaint,
    /// An inactive tab that can't be opened.
    pub(crate) disabled: TabPaint,
    /// The open tab of a disabled strip.
    pub(crate) disabled_active: TabPaint,
    /// The ring inside a tab that has keyboard focus; the same as `accent.base`.
    pub(crate) focus: Color32,
}

/// The paint of a tab in one state. A `None` underline draws no line under the tab.
pub(crate) struct TabPaint {
    pub(crate) text: Color32,
    pub(crate) icon: Color32,
    pub(crate) underline: Option<Color32>,
}

const fn tab(text: u32, icon: u32, underline: Option<u32>) -> TabPaint {
    TabPaint {
        text: hex(text),
        icon: hex(icon),
        underline: hex_opt(underline),
    }
}

impl TabColors {
    pub(super) const DARK: Self = Self {
        bar: hex(0x131418),
        bar_border: hex(0x26282E),
        active: tab(0xECEEF2, 0x78C3D7, Some(0x78C3D7)),
        inactive: tab(0x8E929F, 0x7E818C, None),
        hover: tab(0xDCDFE6, 0xB8BBC6, Some(0x383B46)),
        disabled: tab(0x5A5E69, 0x4A4E59, None),
        disabled_active: tab(0x6B6F7B, 0x365966, Some(0x365966)),
        focus: hex(0x78C3D7),
    };
    pub(super) const LIGHT: Self = Self {
        bar: hex(0xFFFFFF),
        bar_border: hex(0xE3E5EA),
        active: tab(0x17181C, 0x1E82A5, Some(0x1E82A5)),
        inactive: tab(0x6B6F7B, 0x7E818C, None),
        hover: tab(0x2A2C33, 0x4A4E59, Some(0xC9CCD4)),
        disabled: tab(0x9A9EA9, 0xB0B4BD, None),
        disabled_active: tab(0x8A8E99, 0x8CC4D6, Some(0x8CC4D6)),
        focus: hex(0x1E82A5),
    };
}
