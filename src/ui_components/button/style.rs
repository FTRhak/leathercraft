//! The colors of each button kind and state, for the dark and the light theme.
//!
//! The values come from the `Button` component of the Brilliant design ("UX Components — Forms")
//! and its light palette.

use bevy_egui::egui::Color32;

/// What the button is for: one `Primary` per dialog (OK), `Secondary` beside it (Cancel),
/// `Outline` for helper actions (Reset to defaults).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    #[default]
    Secondary,
    Outline,
}

/// The interaction state the button is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ButtonState {
    Default,
    Hover,
    Disabled,
}

/// The paint of one kind in one state. A `None` fill leaves the background showing through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ButtonStyle {
    pub fill: Option<Color32>,
    pub stroke: Color32,
    pub text: Color32,
}

const fn style(fill: Option<u32>, stroke: u32, text: u32) -> ButtonStyle {
    ButtonStyle {
        fill: match fill {
            Some(rgb) => Some(hex(rgb)),
            None => None,
        },
        stroke: hex(stroke),
        text: hex(text),
    }
}

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

impl ButtonStyle {
    pub(super) fn of(kind: ButtonKind, state: ButtonState, dark_mode: bool) -> Self {
        use ButtonKind::*;
        use ButtonState::*;
        if dark_mode {
            match (kind, state) {
                (Primary, Default) => style(Some(0x1B6B85), 0x78C3D7, 0xFFFFFF),
                (Primary, Hover) => style(Some(0x21809E), 0x9AD4E3, 0xFFFFFF),
                (Primary, Disabled) => style(Some(0x1B3B47), 0x2E4F5A, 0x6B8C96),
                (Secondary, Default) => style(Some(0x23252B), 0x383B46, 0xDCDFE6),
                (Secondary, Hover) => style(Some(0x2A2C33), 0x4A4E59, 0xECEEF2),
                (Secondary, Disabled) => style(Some(0x1C1D22), 0x2B2E37, 0x5A5E69),
                (Outline, Default) => style(None, 0x2E313B, 0xB8BBC6),
                (Outline, Hover) => style(Some(0x1F2128), 0x3A3D47, 0xDCDFE6),
                (Outline, Disabled) => style(None, 0x25272F, 0x5A5E69),
            }
        } else {
            match (kind, state) {
                (Primary, Default) => style(Some(0x176E8D), 0x1E82A5, 0xFFFFFF),
                (Primary, Hover) => style(Some(0x1E82A5), 0x4FA3C2, 0xFFFFFF),
                (Primary, Disabled) => style(Some(0xD5E6EC), 0xB7D3DD, 0x8AA9B4),
                (Secondary, Default) => style(Some(0xE6E8ED), 0xBCC0C9, 0x2A2C33),
                (Secondary, Hover) => style(Some(0xDDE0E6), 0xA9AEB8, 0x17181C),
                (Secondary, Disabled) => style(Some(0xEEF0F3), 0xE3E5EA, 0x9A9EA9),
                (Outline, Default) => style(None, 0xC9CCD4, 0x4A4E59),
                (Outline, Hover) => style(Some(0xF0F1F4), 0xBCC0C9, 0x2A2C33),
                (Outline, Disabled) => style(None, 0xE3E5EA, 0x9A9EA9),
            }
        }
    }
}

/// The ring drawn around a button that has keyboard focus: the accent of the theme.
pub(super) fn focus_color(dark_mode: bool) -> Color32 {
    if dark_mode {
        hex(0x78C3D7)
    } else {
        hex(0x1E82A5)
    }
}
