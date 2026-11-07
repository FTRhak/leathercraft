use bevy_egui::egui::Color32;

use super::{hex, hex_opt};

/// The text button of `ui_components::button`, from the Brilliant `Button` component.
pub(crate) struct ButtonColors {
    /// The main action of a dialog, such as OK.
    pub(crate) primary: ButtonStates,
    /// The action beside the main one, such as Cancel.
    pub(crate) secondary: ButtonStates,
    /// Helper actions, such as Reset to defaults.
    pub(crate) outline: ButtonStates,
    /// The ring around a button that has keyboard focus; the same as `accent.base`.
    pub(crate) focus: Color32,
}

/// The paint of one button kind in each interaction state.
pub(crate) struct ButtonStates {
    pub(crate) default: ButtonPaint,
    pub(crate) hover: ButtonPaint,
    pub(crate) disabled: ButtonPaint,
}

/// The paint of a button in one state. A `None` fill leaves the background showing through.
pub(crate) struct ButtonPaint {
    pub(crate) fill: Option<Color32>,
    pub(crate) stroke: Color32,
    pub(crate) text: Color32,
}

const fn paint(fill: Option<u32>, stroke: u32, text: u32) -> ButtonPaint {
    ButtonPaint {
        fill: hex_opt(fill),
        stroke: hex(stroke),
        text: hex(text),
    }
}

impl ButtonColors {
    pub(super) const DARK: Self = Self {
        primary: ButtonStates {
            default: paint(Some(0x1B6B85), 0x78C3D7, 0xFFFFFF),
            hover: paint(Some(0x21809E), 0x9AD4E3, 0xFFFFFF),
            disabled: paint(Some(0x1B3B47), 0x2E4F5A, 0x6B8C96),
        },
        secondary: ButtonStates {
            default: paint(Some(0x23252B), 0x383B46, 0xDCDFE6),
            hover: paint(Some(0x2A2C33), 0x4A4E59, 0xECEEF2),
            disabled: paint(Some(0x1C1D22), 0x2B2E37, 0x5A5E69),
        },
        outline: ButtonStates {
            default: paint(None, 0x2E313B, 0xB8BBC6),
            hover: paint(Some(0x1F2128), 0x3A3D47, 0xDCDFE6),
            disabled: paint(None, 0x25272F, 0x5A5E69),
        },
        focus: hex(0x78C3D7),
    };
    pub(super) const LIGHT: Self = Self {
        primary: ButtonStates {
            default: paint(Some(0x176E8D), 0x1E82A5, 0xFFFFFF),
            hover: paint(Some(0x1E82A5), 0x4FA3C2, 0xFFFFFF),
            disabled: paint(Some(0xD5E6EC), 0xB7D3DD, 0x8AA9B4),
        },
        secondary: ButtonStates {
            default: paint(Some(0xE6E8ED), 0xBCC0C9, 0x2A2C33),
            hover: paint(Some(0xDDE0E6), 0xA9AEB8, 0x17181C),
            disabled: paint(Some(0xEEF0F3), 0xE3E5EA, 0x9A9EA9),
        },
        outline: ButtonStates {
            default: paint(None, 0xC9CCD4, 0x4A4E59),
            hover: paint(Some(0xF0F1F4), 0xBCC0C9, 0x2A2C33),
            disabled: paint(None, 0xE3E5EA, 0x9A9EA9),
        },
        focus: hex(0x1E82A5),
    };
}
