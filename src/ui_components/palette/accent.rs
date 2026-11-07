use bevy_egui::egui::Color32;

use super::hex;

/// The single accent of the interface: active mode, selection, focus.
#[allow(dead_code)]
pub(crate) struct AccentColors {
    /// Active icons and the selection border; the same as `centerline`.
    pub(crate) base: Color32,
    /// The primary button and the active mode.
    pub(crate) fill: Color32,
    /// The border of accent chips.
    pub(crate) border: Color32,
    /// The background of accent chips.
    pub(crate) container: Color32,
    /// The background of the selected row, or of a hotkey being recorded.
    pub(crate) selected_row: Color32,
}

impl AccentColors {
    pub(super) const DARK: Self = Self {
        base: hex(0x78C3D7),
        fill: hex(0x1B6B85),
        border: hex(0x365966),
        container: hex(0x16323B),
        selected_row: hex(0x16232A),
    };
    pub(super) const LIGHT: Self = Self {
        base: hex(0x1E82A5),
        fill: hex(0x176E8D),
        border: hex(0x8CC4D6),
        container: hex(0xE1F1F6),
        selected_row: hex(0xEAF4F8),
    };
}
