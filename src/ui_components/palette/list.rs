use bevy_egui::egui::Color32;

use super::{hex, hex_opt};

/// The rows of `ui_components::list`, from the Brilliant `History List` board. The design has
/// no hover, disabled or light values; those are derived from the menu and accent roles.
pub(crate) struct ListColors {
    pub(crate) default: ListItemPaint,
    /// Under the pointer or being pressed.
    pub(crate) hover: ListItemPaint,
    /// The current row.
    pub(crate) selected: ListItemPaint,
    /// A row that no longer applies, such as an undone history entry; the list also fades it.
    pub(crate) dimmed: ListItemPaint,
    pub(crate) disabled: ListItemPaint,
    pub(crate) focus: Color32,
}

/// The paint of a row in one state.
pub(crate) struct ListItemPaint {
    pub(crate) fill: Option<Color32>,
    pub(crate) stroke: Option<Color32>,
    pub(crate) icon: Color32,
    pub(crate) text: Color32,
    /// The text at the end of the row, such as a time or a status.
    pub(crate) trailing: Color32,
}

const fn item(
    fill: Option<u32>,
    stroke: Option<u32>,
    icon: u32,
    text: u32,
    trailing: u32,
) -> ListItemPaint {
    ListItemPaint {
        fill: hex_opt(fill),
        stroke: hex_opt(stroke),
        icon: hex(icon),
        text: hex(text),
        trailing: hex(trailing),
    }
}

impl ListColors {
    pub(super) const DARK: Self = Self {
        default: item(None, None, 0x6E727D, 0x9CA0AC, 0x585B66),
        hover: item(Some(0x23252B), None, 0x8E929F, 0xDCDFE6, 0x6B6F7B),
        selected: item(Some(0x0E5A7A), Some(0x78C3D7), 0xFFFFFF, 0xFFFFFF, 0xB8E8F5),
        dimmed: item(None, None, 0x6E727D, 0x8A8E99, 0x6E727D),
        disabled: item(None, None, 0x3A3D47, 0x5A5E69, 0x3A3D47),
        focus: hex(0x78C3D7),
    };
    pub(super) const LIGHT: Self = Self {
        default: item(None, None, 0x8A8E99, 0x4A4E59, 0x8A8E99),
        hover: item(Some(0xE6E8ED), None, 0x6B6F7B, 0x17181C, 0x6B6F7B),
        selected: item(Some(0x176E8D), Some(0x1E82A5), 0xFFFFFF, 0xFFFFFF, 0xD6EEF5),
        dimmed: item(None, None, 0x8A8E99, 0x6B6F7B, 0x8A8E99),
        disabled: item(None, None, 0xBCC0C9, 0x9A9EA9, 0xBCC0C9),
        focus: hex(0x1E82A5),
    };
}
