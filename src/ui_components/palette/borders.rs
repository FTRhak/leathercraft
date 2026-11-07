use bevy_egui::egui::Color32;

use super::hex;

/// Thin 1 px lines; the brighter ones are for interactive controls.
#[allow(dead_code)]
pub(crate) struct Borders {
    /// Between sections inside a panel.
    pub(crate) divider: Color32,
    /// The edges of the docked panels.
    pub(crate) panel: Color32,
    /// Buttons and floating panels.
    pub(crate) control: Color32,
    /// Text fields and selects.
    pub(crate) field: Color32,
    /// Chips, keycaps and "+" buttons.
    pub(crate) chip: Color32,
}

impl Borders {
    pub(super) const DARK: Self = Self {
        divider: hex(0x25272F),
        panel: hex(0x282A32),
        control: hex(0x2B2E37),
        field: hex(0x2E313B),
        chip: hex(0x383B46),
    };
    pub(super) const LIGHT: Self = Self {
        divider: hex(0xE3E5EA),
        panel: hex(0xD9DCE2),
        control: hex(0xD2D5DC),
        field: hex(0xC9CCD4),
        chip: hex(0xBCC0C9),
    };
}
