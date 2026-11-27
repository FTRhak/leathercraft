use bevy_egui::egui::Color32;

use super::hex;

/// The dimension lines that measure the selection on the canvas, from the "Dim W" and "Dim H"
/// overlays of the Brilliant canvas design. The light values are derived; the design has a dark
/// canvas only.
pub(crate) struct DimensionColors {
    /// The dimension line and its end ticks.
    pub(crate) line: Color32,
    /// The plate behind the measurement, which cuts the line under it.
    pub(crate) label_fill: Color32,
    /// The measurement.
    pub(crate) label_text: Color32,
}

impl DimensionColors {
    pub(super) const DARK: Self = Self {
        line: hex(0x6B6F7B),
        label_fill: hex(0x121316),
        label_text: hex(0xDCDFE6),
    };
    pub(super) const LIGHT: Self = Self {
        line: hex(0x9298A4),
        label_fill: hex(0xFAFAFB),
        label_text: hex(0x2A2C33),
    };
}
