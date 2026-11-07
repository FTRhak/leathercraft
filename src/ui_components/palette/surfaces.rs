use bevy_egui::egui::Color32;

use super::hex;

/// Backgrounds, from the deepest to the highest.
#[allow(dead_code)]
pub(crate) struct Surfaces {
    /// The canvas and the window behind it.
    pub(crate) canvas: Color32,
    /// Text fields, selects and the tab strip.
    pub(crate) input: Color32,
    /// Info blocks, such as the material card.
    pub(crate) info: Color32,
    /// The canvas rulers and dialog footers.
    pub(crate) ruler: Color32,
    /// The docked panels and dialogs.
    pub(crate) panel: Color32,
    /// Popups, such as the cursor coordinates.
    pub(crate) popup: Color32,
    /// Chips, keycaps and secondary buttons.
    pub(crate) chip: Color32,
}

impl Surfaces {
    pub(super) const DARK: Self = Self {
        canvas: hex(0x121316),
        input: hex(0x131418),
        info: hex(0x141519),
        ruler: hex(0x16171B),
        panel: hex(0x1A1B20),
        popup: hex(0x1F2128),
        chip: hex(0x23252B),
    };
    pub(super) const LIGHT: Self = Self {
        canvas: hex(0xFAFAFB),
        input: hex(0xFFFFFF),
        info: hex(0xF2F3F6),
        ruler: hex(0xECEEF2),
        panel: hex(0xF0F1F4),
        popup: hex(0xFFFFFF),
        chip: hex(0xE6E8ED),
    };
}
