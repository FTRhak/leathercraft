use bevy_egui::egui::Color32;

use super::{ViewButtonPaint, hex};

/// The snap controls floating over the canvas, `panels::snap`, from the "Snap Controls" and
/// "Snap Popover" of the Brilliant design. The design has a dark board only, with every target
/// on; the off and hover states and the light values follow the view controls.
pub(crate) struct SnapColors {
    /// The bar with the magnet and the targets button, like the view controls.
    pub(crate) bar_background: Color32,
    pub(crate) bar_border: Color32,
    pub(crate) bar_shadow: Color32,
    /// The magnet with snapping off, and under the pointer.
    pub(crate) toggle_off: ViewButtonPaint,
    pub(crate) toggle_off_hover: ViewButtonPaint,
    /// The magnet with snapping on, and under the pointer.
    pub(crate) toggle_on: ViewButtonPaint,
    pub(crate) toggle_on_hover: ViewButtonPaint,
    /// The targets button closed, under the pointer, and with its popover open.
    pub(crate) targets: ViewButtonPaint,
    pub(crate) targets_hover: ViewButtonPaint,
    pub(crate) targets_open: ViewButtonPaint,
    /// The caret of the targets button.
    pub(crate) caret: Color32,
    /// The ring around a button that has keyboard focus.
    pub(crate) focus: Color32,
    pub(crate) popover: PopoverColors,
}

/// The popover of the snap controls.
pub(crate) struct PopoverColors {
    pub(crate) background: Color32,
    pub(crate) border: Color32,
    pub(crate) shadow: Color32,
    pub(crate) title: Color32,
    /// The hotkey in the header.
    pub(crate) keycap_fill: Color32,
    pub(crate) keycap_border: Color32,
    pub(crate) keycap_text: Color32,
    /// The captions above the target list and the fields.
    pub(crate) caption: Color32,
    /// The well around the target rows.
    pub(crate) list_fill: Color32,
    pub(crate) list_border: Color32,
    /// A target row turned on, and under the pointer.
    pub(crate) row_on: TargetRowPaint,
    pub(crate) row_on_hover: TargetRowPaint,
    /// A target row turned off, and under the pointer.
    pub(crate) row_off: TargetRowPaint,
    pub(crate) row_off_hover: TargetRowPaint,
    /// The line above the hints.
    pub(crate) divider: Color32,
    /// The hints at the bottom, and their icons.
    pub(crate) hint: Color32,
}

/// A target row of the popover in one state; `None` paints no fill.
pub(crate) struct TargetRowPaint {
    pub(crate) fill: Option<Color32>,
    pub(crate) icon: Color32,
    pub(crate) text: Color32,
}

const fn button(fill: Option<u32>, stroke: Option<u32>, content: u32) -> ViewButtonPaint {
    ViewButtonPaint {
        fill: super::hex_opt(fill),
        stroke: super::hex_opt(stroke),
        content: hex(content),
    }
}

const fn row(fill: Option<u32>, icon: u32, text: u32) -> TargetRowPaint {
    TargetRowPaint {
        fill: super::hex_opt(fill),
        icon: hex(icon),
        text: hex(text),
    }
}

impl SnapColors {
    pub(super) const DARK: Self = Self {
        bar_background: hex(0x1A1B20),
        bar_border: hex(0x2B2E37),
        bar_shadow: Color32::from_black_alpha(102),
        toggle_off: button(None, None, 0xA9ADB8),
        toggle_off_hover: button(Some(0x2A2C33), None, 0xECEEF2),
        toggle_on: button(Some(0x1B6B85), Some(0x78C3D7), 0xFFFFFF),
        toggle_on_hover: button(Some(0x227B98), Some(0x9AD3E2), 0xFFFFFF),
        targets: button(None, None, 0xDCDFE6),
        targets_hover: button(Some(0x2A2C33), None, 0xECEEF2),
        targets_open: button(Some(0x23252B), None, 0xDCDFE6),
        caret: hex(0xA9ADB8),
        focus: hex(0x78C3D7),
        popover: PopoverColors {
            background: hex(0x1A1B20),
            border: hex(0x2B2E37),
            shadow: Color32::from_black_alpha(128),
            title: hex(0xDCDFE6),
            keycap_fill: hex(0x141518),
            keycap_border: hex(0x2B2E37),
            keycap_text: hex(0x7E818C),
            caption: hex(0x7E818C),
            list_fill: hex(0x141518),
            list_border: hex(0x2B2E37),
            row_on: row(Some(0x1B6B85), 0xFFFFFF, 0xFFFFFF),
            row_on_hover: row(Some(0x227B98), 0xFFFFFF, 0xFFFFFF),
            row_off: row(None, 0x7E818C, 0xA9ADB8),
            row_off_hover: row(Some(0x23252B), 0xA9ADB8, 0xDCDFE6),
            divider: hex(0x2B2E37),
            hint: hex(0x7E818C),
        },
    };
    pub(super) const LIGHT: Self = Self {
        bar_background: hex(0xFFFFFF),
        bar_border: hex(0xD9DCE2),
        bar_shadow: Color32::from_black_alpha(40),
        toggle_off: button(None, None, 0x4A4E59),
        toggle_off_hover: button(Some(0xE6E8ED), None, 0x17181C),
        toggle_on: button(Some(0x176E8D), Some(0x1E82A5), 0xFFFFFF),
        toggle_on_hover: button(Some(0x1B7C9E), Some(0x1E82A5), 0xFFFFFF),
        targets: button(None, None, 0x2A2C33),
        targets_hover: button(Some(0xE6E8ED), None, 0x17181C),
        targets_open: button(Some(0xEEF0F3), None, 0x2A2C33),
        caret: hex(0x6B6F7B),
        focus: hex(0x1E82A5),
        popover: PopoverColors {
            background: hex(0xFFFFFF),
            border: hex(0xD9DCE2),
            shadow: Color32::from_black_alpha(48),
            title: hex(0x2A2C33),
            keycap_fill: hex(0xF2F3F5),
            keycap_border: hex(0xD9DCE2),
            keycap_text: hex(0x6B6F7B),
            caption: hex(0x6B6F7B),
            list_fill: hex(0xF7F8FA),
            list_border: hex(0xD9DCE2),
            row_on: row(Some(0x176E8D), 0xFFFFFF, 0xFFFFFF),
            row_on_hover: row(Some(0x1B7C9E), 0xFFFFFF, 0xFFFFFF),
            row_off: row(None, 0x6B6F7B, 0x4A4E59),
            row_off_hover: row(Some(0xE6E8ED), 0x4A4E59, 0x17181C),
            divider: hex(0xE3E5EA),
            hint: hex(0x6B6F7B),
        },
    };
}
