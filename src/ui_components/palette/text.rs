use bevy_egui::egui::Color32;

use super::hex;

/// Six steps of text brightness: hierarchy without a change of hue.
#[allow(dead_code)]
pub(crate) struct TextColors {
    /// Values and headings.
    pub(crate) primary: Color32,
    /// Monospaced values and coordinates.
    pub(crate) strong: Color32,
    /// Field names.
    pub(crate) label: Color32,
    /// Inactive tabs and icons.
    pub(crate) secondary: Color32,
    /// Group titles inside a panel, such as "Align objects".
    pub(crate) group_title: Color32,
    /// Section headers, in capitals at 10 px.
    pub(crate) section_header: Color32,
    /// Placeholders and ruler marks.
    pub(crate) muted: Color32,
}

impl TextColors {
    pub(super) const DARK: Self = Self {
        primary: hex(0xECEEF2),
        strong: hex(0xDCDFE6),
        label: hex(0xB8BBC6),
        secondary: hex(0x8E929F),
        group_title: hex(0x9CA0AC),
        section_header: hex(0x7E818C),
        muted: hex(0x6B6F7B),
    };
    pub(super) const LIGHT: Self = Self {
        primary: hex(0x17181C),
        strong: hex(0x2A2C33),
        label: hex(0x4A4E59),
        secondary: hex(0x6B6F7B),
        group_title: hex(0x5E626D),
        section_header: hex(0x7E818C),
        muted: hex(0x8A8E99),
    };
}
