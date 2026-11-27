use bevy_egui::egui::Color32;

use super::hex;

/// The rulers along the top and left edges of the canvas, from the "Ruler Top", "Ruler Left" and
/// "Ruler Corner" of the Brilliant canvas design. The light values are derived; the design has a
/// dark canvas only.
pub(crate) struct RulerColors {
    /// The strips and the corner square where they meet.
    pub(crate) background: Color32,
    /// The line between a strip and the canvas.
    pub(crate) border: Color32,
    /// The tick marks.
    pub(crate) tick: Color32,
    /// The numbers at the long ticks.
    pub(crate) label: Color32,
    /// The mark of the cursor position, the accent.
    pub(crate) cursor: Color32,
}

impl RulerColors {
    pub(super) const DARK: Self = Self {
        background: hex(0x16171B),
        border: hex(0x282A32),
        tick: hex(0x3A3D47),
        label: hex(0x6B6F7B),
        cursor: hex(0x78C3D7),
    };
    pub(super) const LIGHT: Self = Self {
        background: hex(0xECEEF2),
        border: hex(0xD9DCE2),
        tick: hex(0xB8BECB),
        label: hex(0x8A8E99),
        cursor: hex(0x1E82A5),
    };
}
