use bevy_egui::egui::Color32;

use super::{hex, hex_opt};

/// The checkbox and radio button of `ui_components::checkbox`, from the Brilliant `Checkbox`
/// and `Radio` components, which share their colors (`mark` is the radio's dot). The
/// design has dark boards only; the light values follow the light primary button (checked) and
/// input field (unchecked), as the dark ones do.
pub(crate) struct CheckboxColors {
    /// Checked or mixed.
    pub(crate) on: CheckboxStates,
    pub(crate) off: CheckboxStates,
    /// The edge of a focused box and the ring around it, drawn at 45% opacity; the same as
    /// `accent.base`.
    pub(crate) focus: Color32,
}

/// The paint of a checkbox value in each interaction state.
pub(crate) struct CheckboxStates {
    pub(crate) default: CheckboxPaint,
    /// Under the pointer.
    pub(crate) hover: CheckboxPaint,
    pub(crate) disabled: CheckboxPaint,
}

/// The paint of a checkbox in one state. An unchecked box has no `mark`.
pub(crate) struct CheckboxPaint {
    pub(crate) fill: Color32,
    pub(crate) stroke: Color32,
    /// The check or the dash.
    pub(crate) mark: Option<Color32>,
    /// The label after the box.
    pub(crate) text: Color32,
}

const fn check(fill: u32, stroke: u32, mark: Option<u32>, text: u32) -> CheckboxPaint {
    CheckboxPaint {
        fill: hex(fill),
        stroke: hex(stroke),
        mark: hex_opt(mark),
        text: hex(text),
    }
}

impl CheckboxColors {
    pub(super) const DARK: Self = Self {
        on: CheckboxStates {
            default: check(0x1B6B85, 0x78C3D7, Some(0xFFFFFF), 0xDCDFE6),
            hover: check(0x21809E, 0x9AD4E3, Some(0xFFFFFF), 0xECEEF2),
            disabled: check(0x1B3B47, 0x2E4F5A, Some(0x6B8C96), 0x5A5E69),
        },
        off: CheckboxStates {
            default: check(0x131418, 0x3A3D47, None, 0xB8BBC6),
            hover: check(0x1A1B20, 0x4A4E59, None, 0xDCDFE6),
            disabled: check(0x1C1D22, 0x2B2E37, None, 0x5A5E69),
        },
        focus: hex(0x78C3D7),
    };
    pub(super) const LIGHT: Self = Self {
        on: CheckboxStates {
            default: check(0x176E8D, 0x1E82A5, Some(0xFFFFFF), 0x2A2C33),
            hover: check(0x1E82A5, 0x4FA3C2, Some(0xFFFFFF), 0x17181C),
            disabled: check(0xD5E6EC, 0xB7D3DD, Some(0x8AA9B4), 0x9A9EA9),
        },
        off: CheckboxStates {
            default: check(0xFFFFFF, 0xA9AEB8, None, 0x4A4E59),
            hover: check(0xF7F8FA, 0x8A8F9B, None, 0x2A2C33),
            disabled: check(0xF2F3F6, 0xE3E5EA, None, 0x9A9EA9),
        },
        focus: hex(0x1E82A5),
    };
}
