use bevy_egui::egui::Color32;

use super::hex;

/// The zoom panel floating over the canvas, `ui_components::view_controls`, from the "View
/// Controls" of the Brilliant design. The design has a dark board only and no hover or disabled
/// state; those and the light values follow the tool bar buttons.
pub(crate) struct ViewControlsColors {
    pub(crate) background: Color32,
    pub(crate) border: Color32,
    pub(crate) shadow: Color32,
    /// The short vertical line before the fit buttons.
    pub(crate) divider: Color32,
    /// The zoom percentage.
    pub(crate) value: Color32,
    /// The quiet icon buttons.
    pub(crate) idle: ViewButtonPaint,
    pub(crate) hover: ViewButtonPaint,
    pub(crate) disabled: ViewButtonPaint,
    /// Zoom to selection, the accent button.
    pub(crate) primary: ViewButtonPaint,
    pub(crate) primary_hover: ViewButtonPaint,
    /// The ring around a button that has keyboard focus.
    pub(crate) focus: Color32,
}

/// A button of the view controls in one state; `None` paints no fill or stroke.
pub(crate) struct ViewButtonPaint {
    pub(crate) fill: Option<Color32>,
    pub(crate) stroke: Option<Color32>,
    pub(crate) content: Color32,
}

const fn paint(fill: Option<u32>, stroke: Option<u32>, content: u32) -> ViewButtonPaint {
    ViewButtonPaint {
        fill: super::hex_opt(fill),
        stroke: super::hex_opt(stroke),
        content: hex(content),
    }
}

impl ViewControlsColors {
    pub(super) const DARK: Self = Self {
        background: hex(0x1A1B20),
        border: hex(0x2B2E37),
        shadow: Color32::from_black_alpha(102),
        divider: hex(0x2B2E37),
        value: hex(0xDCDFE6),
        idle: paint(None, None, 0xA9ADB8),
        hover: paint(Some(0x2A2C33), None, 0xECEEF2),
        disabled: paint(None, None, 0x55585F),
        primary: paint(Some(0x1B6B85), Some(0x78C3D7), 0xFFFFFF),
        primary_hover: paint(Some(0x227B98), Some(0x9AD3E2), 0xFFFFFF),
        focus: hex(0x78C3D7),
    };
    pub(super) const LIGHT: Self = Self {
        background: hex(0xFFFFFF),
        border: hex(0xD9DCE2),
        shadow: Color32::from_black_alpha(40),
        divider: hex(0xD9DCE2),
        value: hex(0x2A2C33),
        idle: paint(None, None, 0x4A4E59),
        hover: paint(Some(0xE6E8ED), None, 0x17181C),
        disabled: paint(None, None, 0xB4B8C1),
        primary: paint(Some(0x176E8D), Some(0x1E82A5), 0xFFFFFF),
        primary_hover: paint(Some(0x1B7C9E), Some(0x1E82A5), 0xFFFFFF),
        focus: hex(0x1E82A5),
    };
}
