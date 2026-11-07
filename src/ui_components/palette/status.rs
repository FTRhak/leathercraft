use bevy_egui::egui::Color32;

use super::hex;

/// Warnings, recording and the axes.
#[allow(dead_code)]
pub(crate) struct StatusColors {
    /// Hotkey conflicts.
    pub(crate) warning: Color32,
    /// The background of a conflict chip.
    pub(crate) warning_container: Color32,
    /// The X axis of the gizmo, and the recording indicator.
    pub(crate) axis_x: Color32,
    /// The Y axis of the gizmo.
    pub(crate) axis_y: Color32,
}

impl StatusColors {
    pub(super) const DARK: Self = Self {
        warning: hex(0xF0A83A),
        warning_container: hex(0x2A1F14),
        axis_x: hex(0xE06C6C),
        axis_y: hex(0x6CC07A),
    };
    pub(super) const LIGHT: Self = Self {
        warning: hex(0xC77A0A),
        warning_container: hex(0xFDF1DE),
        axis_x: hex(0xD14B4B),
        axis_y: hex(0x2F9E48),
    };
}
