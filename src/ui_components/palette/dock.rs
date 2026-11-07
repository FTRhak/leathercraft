use bevy_egui::egui::Color32;

use super::hex;

/// The docked panels, from the Brilliant `DockTab` component and the dock tab strips.
pub(crate) struct DockColors {
    /// The strip behind the tabs.
    pub(crate) bar: Color32,
    /// The line under the strip, and the divider right of an inactive tab.
    pub(crate) bar_border: Color32,
    /// The open tab of a panel group; it joins the panel body below it.
    pub(crate) active: DockTabPaint,
    pub(crate) inactive: DockTabPaint,
    /// An inactive tab under the pointer.
    pub(crate) hover: DockTabPaint,
    /// The line along the top edge of the open tab.
    pub(crate) accent: Color32,
    /// The close cross of a tab.
    pub(crate) close: Color32,
    /// The close cross under the pointer.
    pub(crate) close_hover: Color32,
    /// The panel body.
    pub(crate) body: Color32,
    /// The lines between panel groups, idle, under the pointer and dragged.
    pub(crate) separator: Color32,
    pub(crate) separator_hover: Color32,
    pub(crate) separator_drag: Color32,
}

/// The paint of a dock tab in one state.
pub(crate) struct DockTabPaint {
    pub(crate) fill: Color32,
    pub(crate) text: Color32,
    pub(crate) icon: Color32,
}

const fn tab(fill: u32, text: u32, icon: u32) -> DockTabPaint {
    DockTabPaint {
        fill: hex(fill),
        text: hex(text),
        icon: hex(icon),
    }
}

impl DockColors {
    pub(super) const DARK: Self = Self {
        bar: hex(0x131418),
        bar_border: hex(0x26282E),
        active: tab(0x1D1E23, 0xECEEF2, 0x78C3D7),
        inactive: tab(0x15161A, 0x8A8E99, 0x7E818C),
        hover: tab(0x15161A, 0xDCDFE6, 0xB8BBC6),
        accent: hex(0x78C3D7),
        close: hex(0x6E717C),
        close_hover: hex(0xECEEF2),
        body: hex(0x1A1B20),
        separator: hex(0x282A32),
        separator_hover: hex(0x365966),
        separator_drag: hex(0x78C3D7),
    };
    pub(super) const LIGHT: Self = Self {
        bar: hex(0xE6E8ED),
        bar_border: hex(0xD2D5DC),
        active: tab(0xF7F8FA, 0x17181C, 0x1E82A5),
        inactive: tab(0xEBEDF1, 0x6B6F7B, 0x7E818C),
        hover: tab(0xEBEDF1, 0x2A2C33, 0x4A4E59),
        accent: hex(0x1E82A5),
        close: hex(0x8A8E99),
        close_hover: hex(0x17181C),
        body: hex(0xF0F1F4),
        separator: hex(0xD9DCE2),
        separator_hover: hex(0x8CC4D6),
        separator_drag: hex(0x1E82A5),
    };
}
