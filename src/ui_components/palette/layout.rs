use bevy_egui::egui::Color32;

use super::{FieldTone, hex};

/// The tree of the Layout panel, from the "Layout Tree Pane" of the Brilliant design. The design
/// has no hover, light or intersect values; those are derived from the list and accent roles.
pub(crate) struct LayoutColors {
    /// The caret of the root row, "Pattern pieces".
    pub(crate) root_caret: Color32,
    /// The folder before the root title.
    pub(crate) root_icon: Color32,
    pub(crate) root_title: Color32,
    /// The number of pieces at the end of the root row.
    pub(crate) root_count: Color32,
    /// The card of a selected piece, around its row and its shapes.
    pub(crate) card_selected: CardPaint,
    pub(crate) card: CardPaint,
    pub(crate) piece_selected: PieceRowPaint,
    pub(crate) piece: PieceRowPaint,
    /// The background of a piece row under the pointer.
    pub(crate) piece_hover: Color32,
    /// The background and border of a selected shape row.
    pub(crate) shape_selected: CardPaint,
    /// The background of a shape row under the pointer.
    pub(crate) shape_hover: Color32,
    pub(crate) shape_text: Color32,
    pub(crate) shape_text_selected: Color32,
    /// The icon of the base and of the lines, which have no operation.
    pub(crate) shape_icon: Color32,
    pub(crate) button: RowButtonColors,
    /// The "Base" badge on the base shape.
    pub(crate) badge: CardPaint,
    pub(crate) badge_text: Color32,
    pub(crate) union: FieldTone,
    pub(crate) subtract: FieldTone,
    pub(crate) intersect: FieldTone,
}

/// A background with a 1 px border.
pub(crate) struct CardPaint {
    pub(crate) fill: Color32,
    pub(crate) stroke: Color32,
}

/// The row of a piece: its caret, name and material.
pub(crate) struct PieceRowPaint {
    /// `None` paints no background.
    pub(crate) fill: Option<Color32>,
    pub(crate) caret: Color32,
    pub(crate) name: Color32,
    /// The material after the name.
    pub(crate) material: Color32,
}

/// The small square buttons at the end of the rows: visibility, lock, reorder and delete.
pub(crate) struct RowButtonColors {
    /// On a selected row: a chip with a border.
    pub(crate) framed: CardPaint,
    /// On other rows: a faint chip without a border.
    pub(crate) quiet_fill: Color32,
    pub(crate) hover: CardPaint,
    pub(crate) icon: Color32,
    pub(crate) icon_quiet: Color32,
    pub(crate) icon_hover: Color32,
    /// The icon of an active toggle, such as the lock of a locked piece.
    pub(crate) icon_active: Color32,
    pub(crate) icon_disabled: Color32,
}

const fn card(fill: u32, stroke: u32) -> CardPaint {
    CardPaint {
        fill: hex(fill),
        stroke: hex(stroke),
    }
}

const fn piece(fill: Option<u32>, caret: u32, name: u32, material: u32) -> PieceRowPaint {
    PieceRowPaint {
        fill: match fill {
            Some(fill) => Some(hex(fill)),
            None => None,
        },
        caret: hex(caret),
        name: hex(name),
        material: hex(material),
    }
}

const fn tone(text: u32, stroke: u32, hover_stroke: u32) -> FieldTone {
    FieldTone {
        text: hex(text),
        stroke: hex(stroke),
        hover_stroke: hex(hover_stroke),
    }
}

impl LayoutColors {
    pub(super) const DARK: Self = Self {
        root_caret: hex(0x8E929F),
        root_icon: hex(0x78C3D7),
        root_title: hex(0xECEEF2),
        root_count: hex(0x6E727D),
        card_selected: card(0x141519, 0x28363D),
        card: card(0x16171C, 0x24262E),
        piece_selected: piece(Some(0x1A2D36), 0x78C3D7, 0xFFFFFF, 0x78C3D7),
        piece: piece(None, 0x6E727D, 0x9CA0AC, 0x6E727D),
        piece_hover: hex(0x1E2026),
        shape_selected: card(0x21242D, 0x3A3F4D),
        shape_hover: hex(0x1C1E24),
        shape_text: hex(0xC8CAD2),
        shape_text_selected: hex(0xECEEF2),
        shape_icon: hex(0x9CA0AC),
        button: RowButtonColors {
            framed: card(0x23252B, 0x32353E),
            quiet_fill: hex(0x1E2026),
            hover: card(0x2A2C33, 0x3A3D47),
            icon: hex(0x8E929F),
            icon_quiet: hex(0x6E727D),
            icon_hover: hex(0xDCDFE6),
            icon_active: hex(0x78C3D7),
            icon_disabled: hex(0x3A3D47),
        },
        badge: card(0x1E2026, 0x2E313B),
        badge_text: hex(0x7E818C),
        union: tone(0x78C3D7, 0x365966, 0x4E8496),
        subtract: tone(0xFAD75A, 0x5E5023, 0x8A7632),
        intersect: tone(0xB79CF0, 0x4A3F66, 0x6B5C94),
    };
    pub(super) const LIGHT: Self = Self {
        root_caret: hex(0x6B6F7B),
        root_icon: hex(0x1E82A5),
        root_title: hex(0x17181C),
        root_count: hex(0x8A8E99),
        card_selected: card(0xFFFFFF, 0xB9DCE8),
        card: card(0xF7F8FA, 0xDDE0E6),
        piece_selected: piece(Some(0xE1F1F6), 0x1E82A5, 0x17181C, 0x1E82A5),
        piece: piece(None, 0x8A8E99, 0x4A4E59, 0x8A8E99),
        piece_hover: hex(0xE9EBEF),
        shape_selected: card(0xEEF0F4, 0xC9CCD4),
        shape_hover: hex(0xF2F3F6),
        shape_text: hex(0x33363E),
        shape_text_selected: hex(0x17181C),
        shape_icon: hex(0x6B6F7B),
        button: RowButtonColors {
            framed: card(0xF0F1F4, 0xD2D5DC),
            quiet_fill: hex(0xF2F3F6),
            hover: card(0xE6E8ED, 0xBCC0C9),
            icon: hex(0x6B6F7B),
            icon_quiet: hex(0x8A8E99),
            icon_hover: hex(0x17181C),
            icon_active: hex(0x1E82A5),
            icon_disabled: hex(0xC9CCD4),
        },
        badge: card(0xF2F3F6, 0xD2D5DC),
        badge_text: hex(0x7E818C),
        union: tone(0x1E82A5, 0x8CC4D6, 0x5FA9C2),
        subtract: tone(0xA67C00, 0xE3CC7A, 0xC9A93E),
        intersect: tone(0x6B4FC8, 0xC7B8EE, 0x9C86DC),
    };
}
