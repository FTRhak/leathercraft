use bevy_egui::egui::Color32;

use super::hex;

/// The dropdown of `ui_components::dropdown`, from the Brilliant `Select` component; its
/// open list has no board in the design yet and follows the popup and accent roles.
pub(crate) struct DropdownColors {
    pub(crate) default: FieldPaint,
    /// Under the pointer.
    pub(crate) hover: FieldPaint,
    /// With keyboard focus, or while its list is open.
    pub(crate) focus: FieldPaint,
    pub(crate) disabled: FieldPaint,
    pub(crate) menu: MenuColors,
}

/// The paint of the closed dropdown in one state.
pub(crate) struct FieldPaint {
    pub(crate) fill: Color32,
    pub(crate) stroke: Color32,
    pub(crate) text: Color32,
    pub(crate) caret: Color32,
}

/// The open list of a dropdown.
pub(crate) struct MenuColors {
    /// The background of the list; the same as `surface.popup`.
    pub(crate) background: Color32,
    pub(crate) border: Color32,
    pub(crate) item_text: Color32,
    /// The background of the item under the pointer.
    pub(crate) item_hover: Color32,
    /// The background of the chosen item; the same as `accent.container`.
    pub(crate) selected: Color32,
    pub(crate) selected_text: Color32,
    pub(crate) disabled_text: Color32,
    /// The heading of a group of items; the same as `text.section_header`.
    pub(crate) group: Color32,
}

const fn field(fill: u32, stroke: u32, text: u32, caret: u32) -> FieldPaint {
    FieldPaint {
        fill: hex(fill),
        stroke: hex(stroke),
        text: hex(text),
        caret: hex(caret),
    }
}

impl DropdownColors {
    pub(super) const DARK: Self = Self {
        default: field(0x131418, 0x2E313B, 0xECEEF2, 0x8E929F),
        hover: field(0x16171B, 0x3A3D47, 0xECEEF2, 0xB8BBC6),
        focus: field(0x131418, 0x78C3D7, 0xECEEF2, 0x78C3D7),
        disabled: field(0x16171B, 0x25272F, 0x5A5E69, 0x3A3D47),
        menu: MenuColors {
            background: hex(0x1F2128),
            border: hex(0x383B46),
            item_text: hex(0xDCDFE6),
            item_hover: hex(0x2A2C33),
            selected: hex(0x16323B),
            selected_text: hex(0xECEEF2),
            disabled_text: hex(0x5A5E69),
            group: hex(0x7E818C),
        },
    };
    pub(super) const LIGHT: Self = Self {
        default: field(0xFFFFFF, 0xC9CCD4, 0x17181C, 0x6B6F7B),
        hover: field(0xF7F8FA, 0xA9AEB8, 0x17181C, 0x4A4E59),
        focus: field(0xFFFFFF, 0x1E82A5, 0x17181C, 0x1E82A5),
        disabled: field(0xF2F3F6, 0xE3E5EA, 0x9A9EA9, 0xBCC0C9),
        menu: MenuColors {
            background: hex(0xFFFFFF),
            border: hex(0xD2D5DC),
            item_text: hex(0x2A2C33),
            item_hover: hex(0xF0F1F4),
            selected: hex(0xE1F1F6),
            selected_text: hex(0x17181C),
            disabled_text: hex(0x9A9EA9),
            group: hex(0x7E818C),
        },
    };
}
