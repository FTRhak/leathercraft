use bevy_egui::egui::Color32;

use super::hex;

/// The text of the Properties panel, from the "Properties Body" of the Brilliant design. Its
/// surfaces, borders and the accent of the piece name come from the shared groups.
pub(crate) struct PropertiesColors {
    /// The title of the hint box, "Canvas controls".
    pub(crate) hint_title: Color32,
    /// The hint of the current tool.
    pub(crate) hint_text: Color32,
    /// The names of the parameter rows.
    pub(crate) param_label: Color32,
    /// The W and H before the piece's size.
    pub(crate) size_prefix: Color32,
}

impl PropertiesColors {
    pub(super) const DARK: Self = Self {
        hint_title: hex(0xC8CAD2),
        hint_text: hex(0x8A8E99),
        param_label: hex(0xC8CAD2),
        size_prefix: hex(0x6E727D),
    };
    pub(super) const LIGHT: Self = Self {
        hint_title: hex(0x2A2C33),
        hint_text: hex(0x5E626D),
        param_label: hex(0x33363E),
        size_prefix: hex(0x8A8E99),
    };
}
