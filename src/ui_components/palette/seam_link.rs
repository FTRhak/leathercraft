use bevy_egui::egui::Color32;

use super::hex;

/// The Link seams dialog and the link lines on the canvas, from the "Stitch Link Dialog" of the
/// design; the light values are picked to match, as the design has a dark board only.
pub(crate) struct SeamLinkPaint {
    /// The line and holes of an automatic pair.
    pub(crate) link_auto: Color32,
    /// The line and holes of a manual pair, and the warning icon.
    pub(crate) link_manual: Color32,
    /// Holes without a pair and their numbers.
    pub(crate) link_free: Color32,
    /// The inside of a hole on the schematic.
    pub(crate) hole_fill: Color32,
    /// The inside of a hole without a pair.
    pub(crate) free_fill: Color32,
    /// The dashed guide each seam's holes sit on.
    pub(crate) guide: Color32,
    /// The arrow at the end of a guide, which shows the direction.
    pub(crate) arrow: Color32,
    /// The square behind the A and B letters.
    pub(crate) side_badge: Color32,
    pub(crate) warning_bg: Color32,
    pub(crate) warning_border: Color32,
    pub(crate) warning_text: Color32,
    pub(crate) warning_detail: Color32,
    /// The table rows of manual pairs and of unpaired holes, and their badges.
    pub(crate) row_manual: Color32,
    pub(crate) row_free: Color32,
    pub(crate) badge_manual: Color32,
    pub(crate) badge_free: Color32,
    /// Between the rows of the pair table.
    pub(crate) row_divider: Color32,
    /// The background of the chosen stitch or assembly tile; its border is `link_auto`.
    pub(crate) tile_selected: Color32,
    /// The thread on the tiles that aren't chosen.
    pub(crate) thread_idle: Color32,
    /// The two pieces in the assembly drawings: A, and B.
    pub(crate) leather_a: Color32,
    pub(crate) leather_b: Color32,
}

impl SeamLinkPaint {
    pub(super) const DARK: Self = Self {
        link_auto: hex(0x78C3D7),
        link_manual: hex(0xF0A04B),
        link_free: hex(0xE5604F),
        hole_fill: hex(0x1A1B20),
        free_fill: hex(0x3A1F1F),
        guide: hex(0x3A3E4A),
        arrow: hex(0x5A5F6C),
        side_badge: hex(0x1B3A45),
        warning_bg: hex(0x2B2216),
        warning_border: hex(0x5C4320),
        warning_text: hex(0xF3C58A),
        warning_detail: hex(0xB89A72),
        row_manual: hex(0x2E2414),
        row_free: hex(0x2E1A1A),
        badge_manual: hex(0x3D2E16),
        badge_free: hex(0x3D1C1C),
        row_divider: hex(0x22242A),
        tile_selected: hex(0x16262C),
        thread_idle: hex(0xA9ADB8),
        leather_a: hex(0x8A6A4A),
        leather_b: hex(0x5E5348),
    };
    pub(super) const LIGHT: Self = Self {
        link_auto: hex(0x1E82A5),
        link_manual: hex(0xC0711A),
        link_free: hex(0xC8412F),
        hole_fill: hex(0xFFFFFF),
        free_fill: hex(0xFBE3DF),
        guide: hex(0xC9CCD4),
        arrow: hex(0x9A9EA8),
        side_badge: hex(0xDCEEF4),
        warning_bg: hex(0xFFF4E5),
        warning_border: hex(0xE8C48E),
        warning_text: hex(0x8A5200),
        warning_detail: hex(0x9C7A4E),
        row_manual: hex(0xFFF1DE),
        row_free: hex(0xFDE9E6),
        badge_manual: hex(0xFBE2BF),
        badge_free: hex(0xF9D5CF),
        row_divider: hex(0xE6E8EC),
        tile_selected: hex(0xE8F4F8),
        thread_idle: hex(0x7A7E88),
        leather_a: hex(0xB08A62),
        leather_b: hex(0x8C7F70),
    };
}
