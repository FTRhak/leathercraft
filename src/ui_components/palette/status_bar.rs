use bevy_egui::egui::Color32;

use super::hex;

/// The status bar of `ui_components::status_bar`, from the "Status Bar" of the Brilliant design.
/// The design has a dark board only; the light values follow the light title bar and text.
pub(crate) struct StatusBarColors {
    pub(crate) background: Color32,
    /// The line above the bar.
    pub(crate) border: Color32,
    /// The short vertical lines between the items.
    pub(crate) divider: Color32,
    /// The dot before the message, and the message itself, after a success.
    pub(crate) ok_dot: Color32,
    pub(crate) message: Color32,
    /// The dot and the message after an error.
    pub(crate) error_dot: Color32,
    pub(crate) error_message: Color32,
    /// What is selected.
    pub(crate) selection: Color32,
    /// The grid step.
    pub(crate) grid: Color32,
    /// The cursor coordinates.
    pub(crate) cursor: Color32,
    /// The library and graphics backend at the right end.
    pub(crate) renderer: Color32,
}

impl StatusBarColors {
    pub(super) const DARK: Self = Self {
        background: hex(0x111215),
        border: hex(0x25272E),
        divider: hex(0x26282E),
        ok_dot: hex(0x10B981),
        message: hex(0x9CA0AC),
        error_dot: hex(0xE5484D),
        error_message: hex(0xF07B7F),
        selection: hex(0x6E727D),
        grid: hex(0x7E818C),
        cursor: hex(0xB8BBC6),
        renderer: hex(0x585B66),
    };
    pub(super) const LIGHT: Self = Self {
        background: hex(0xEBEDF1),
        border: hex(0xD9DCE2),
        divider: hex(0xCDD0D7),
        ok_dot: hex(0x0E9F6E),
        message: hex(0x4A4E59),
        error_dot: hex(0xD13438),
        error_message: hex(0xB4232A),
        selection: hex(0x6B6F7B),
        grid: hex(0x5E626D),
        cursor: hex(0x2A2C33),
        renderer: hex(0x8A8E99),
    };
}
