use bevy_egui::egui::Color32;

use super::hex;

/// Background images: their entries in the panel, the opacity slider, and the selection frame
/// and calibration marks on the canvas. From the "Background Images Panel" and "Background
/// Image — Canvas" boards of the Brilliant design.
pub(crate) struct BackgroundColors {
    /// The card of an image in the panel.
    pub(crate) entry_fill: Color32,
    pub(crate) entry_border: Color32,
    /// The line under an open entry's header.
    pub(crate) entry_divider: Color32,
    /// The part of the slider up to its value.
    pub(crate) slider_fill: Color32,
    /// The edge of that part.
    pub(crate) slider_edge: Color32,
    /// The dashed frame and handles of the selected image, and the calibration marks.
    pub(crate) frame: Color32,
    pub(crate) handle_fill: Color32,
    /// An image whose file can't be read: its card border; the text uses `status.warning`.
    pub(crate) missing_border: Color32,
    /// The dashed border of the empty panel's hint.
    pub(crate) empty_border: Color32,
}

impl BackgroundColors {
    pub(super) const DARK: Self = Self {
        entry_fill: hex(0x16171B),
        entry_border: hex(0x2B2E37),
        entry_divider: hex(0x25272F),
        slider_fill: hex(0x16323B),
        slider_edge: hex(0x365966),
        frame: hex(0x78C3D7),
        handle_fill: hex(0x121316),
        missing_border: hex(0x6B4A1E),
        empty_border: hex(0x2E313B),
    };
    pub(super) const LIGHT: Self = Self {
        entry_fill: hex(0xF7F8FA),
        entry_border: hex(0xD2D5DC),
        entry_divider: hex(0xE3E5EA),
        slider_fill: hex(0xE1F1F6),
        slider_edge: hex(0x8CC4D6),
        frame: hex(0x1E82A5),
        handle_fill: hex(0xFAFAFB),
        missing_border: hex(0xE8C48A),
        empty_border: hex(0xC9CCD4),
    };
}
