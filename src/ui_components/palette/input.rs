use bevy_egui::egui::Color32;

use super::hex;

/// The text, number and search fields of `ui_components::input`, from the Brilliant
/// `Number Input` and `Search Field` components. The design has no hover state and no light
/// boards for them yet; those follow the dropdown, which shares their dark values.
pub(crate) struct InputColors {
    pub(crate) default: InputPaint,
    /// Under the pointer.
    pub(crate) hover: InputPaint,
    /// With keyboard focus, while the text is being edited.
    pub(crate) focus: InputPaint,
    /// A number that matches a canvas layer, such as the edge offset; the same hue as `edge`.
    pub(crate) accent: InputPaint,
    pub(crate) disabled: InputPaint,
}

/// The paint of a field in one state.
pub(crate) struct InputPaint {
    pub(crate) fill: Color32,
    pub(crate) stroke: Color32,
    pub(crate) text: Color32,
    /// The hint shown while the field is empty.
    pub(crate) placeholder: Color32,
    /// The unit after a number.
    pub(crate) unit: Color32,
    /// The search icon and the number stepper.
    pub(crate) icon: Color32,
}

const fn input(
    fill: u32,
    stroke: u32,
    text: u32,
    placeholder: u32,
    unit: u32,
    icon: u32,
) -> InputPaint {
    InputPaint {
        fill: hex(fill),
        stroke: hex(stroke),
        text: hex(text),
        placeholder: hex(placeholder),
        unit: hex(unit),
        icon: hex(icon),
    }
}

impl InputColors {
    pub(super) const DARK: Self = Self {
        default: input(0x131418, 0x2E313B, 0xECEEF2, 0x6B6F7B, 0x8E929F, 0x8E929F),
        hover: input(0x16171B, 0x3A3D47, 0xECEEF2, 0x6B6F7B, 0x8E929F, 0xB8BBC6),
        focus: input(0x131418, 0x78C3D7, 0xECEEF2, 0x6B6F7B, 0x8E929F, 0x78C3D7),
        accent: input(0x131418, 0x5E5023, 0xFAD75A, 0x6B6F7B, 0xB39A45, 0x8E929F),
        disabled: input(0x16171B, 0x25272F, 0x5A5E69, 0x454852, 0x454852, 0x3A3D47),
    };
    pub(super) const LIGHT: Self = Self {
        default: input(0xFFFFFF, 0xC9CCD4, 0x17181C, 0x8A8E99, 0x6B6F7B, 0x6B6F7B),
        hover: input(0xF7F8FA, 0xA9AEB8, 0x17181C, 0x8A8E99, 0x6B6F7B, 0x4A4E59),
        focus: input(0xFFFFFF, 0x1E82A5, 0x17181C, 0x8A8E99, 0x6B6F7B, 0x1E82A5),
        accent: input(0xFFFFFF, 0xE0C66E, 0x8A6400, 0x8A8E99, 0x9C7F2E, 0x6B6F7B),
        disabled: input(0xF2F3F6, 0xE3E5EA, 0x9A9EA9, 0xBCC0C9, 0xBCC0C9, 0xBCC0C9),
    };
}
