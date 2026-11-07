//! The colors of the dark and the light theme.
//!
//! The values come from the "Palette — Leathercraft Pattern Studio" (dark) and "Palette — Light"
//! boards of the Brilliant design. Every color the app paints is read from here; see
//! `docs/design.md`. Each color group lives in its own submodule, with its dark and light values.

mod accent;
mod align_button;
mod borders;
mod button;
mod checkbox;
mod dock;
mod dropdown;
mod group_button;
mod input;
mod layout;
mod list;
mod properties;
mod status;
mod status_bar;
mod surfaces;
mod tabs;
mod text;
mod title_bar;
mod toolbar;

use bevy_egui::egui::{self, Color32};

use accent::AccentColors;
use borders::Borders;
use status::StatusColors;
use surfaces::Surfaces;
use text::TextColors;

pub(crate) use align_button::{AlignButtonColors, AlignButtonPaint};
pub(crate) use button::{ButtonColors, ButtonPaint};
pub(crate) use checkbox::{CheckboxColors, CheckboxPaint};
pub(crate) use dock::{DockColors, DockTabPaint};
pub(crate) use dropdown::{DropdownColors, FieldPaint, FieldTone};
pub(crate) use group_button::{GroupButtonColors, SegmentPaint};
pub(crate) use input::{InputColors, InputPaint};
pub(crate) use layout::LayoutColors;
pub(crate) use list::{ListColors, ListItemPaint};
pub(crate) use properties::PropertiesColors;
pub(crate) use status_bar::StatusBarColors;
pub(crate) use tabs::{TabColors, TabPaint};
pub(crate) use title_bar::{ModePaint, ModeSwitchColors, TitleBarColors};
pub(crate) use toolbar::{ToolButtonColors, ToolButtonPaint, ToolbarColors};

/// The colors of one theme, picked with [`Palette::of`].
///
/// The canvas colors sit at the top level; the interface roles are grouped by area.
// Not every role is painted yet: the interface still uses egui's own visuals in places.
#[allow(dead_code)]
pub(crate) struct Palette {
    pub(crate) grid_line: Color32,
    pub(crate) grid_axis: Color32,
    pub(crate) snap_dot: Color32,
    /// The stroke of a piece that isn't selected.
    pub(crate) outline: Color32,
    /// The name of a piece on the canvas.
    pub(crate) label: Color32,
    /// Stitch holes.
    pub(crate) hole: Color32,
    pub(crate) centerline: Color32,
    /// The edge picked with the Edge tool.
    pub(crate) edge: Color32,

    pub(crate) surface: Surfaces,
    pub(crate) border: Borders,
    pub(crate) text: TextColors,
    pub(crate) accent: AccentColors,
    pub(crate) status: StatusColors,
    pub(crate) button: ButtonColors,
    pub(crate) align_button: AlignButtonColors,
    pub(crate) tabs: TabColors,
    pub(crate) dock: DockColors,
    pub(crate) dropdown: DropdownColors,
    pub(crate) input: InputColors,
    pub(crate) checkbox: CheckboxColors,
    pub(crate) group_button: GroupButtonColors,
    pub(crate) list: ListColors,
    pub(crate) layout: LayoutColors,
    pub(crate) properties: PropertiesColors,
    pub(crate) title_bar: TitleBarColors,
    pub(crate) status_bar: StatusBarColors,
    pub(crate) toolbar: ToolbarColors,
}

/// A color written as `0xRRGGBB`.
const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// An optional color written as `0xRRGGBB`; `None` paints nothing.
const fn hex_opt(rgb: Option<u32>) -> Option<Color32> {
    match rgb {
        Some(rgb) => Some(hex(rgb)),
        None => None,
    }
}

impl Palette {
    const DARK: Self = Self {
        grid_line: hex(0x191B20),
        grid_axis: hex(0x323844),
        snap_dot: hex(0x5F5F5F),
        outline: hex(0x5C606C),
        label: hex(0xE6E8EE),
        hole: hex(0xD6D6D6),
        centerline: hex(0x78C3D7),
        edge: hex(0xFAD75A),
        surface: Surfaces::DARK,
        border: Borders::DARK,
        text: TextColors::DARK,
        accent: AccentColors::DARK,
        status: StatusColors::DARK,
        button: ButtonColors::DARK,
        align_button: AlignButtonColors::DARK,
        tabs: TabColors::DARK,
        dock: DockColors::DARK,
        dropdown: DropdownColors::DARK,
        input: InputColors::DARK,
        checkbox: CheckboxColors::DARK,
        group_button: GroupButtonColors::DARK,
        list: ListColors::DARK,
        layout: LayoutColors::DARK,
        properties: PropertiesColors::DARK,
        title_bar: TitleBarColors::DARK,
        status_bar: StatusBarColors::DARK,
        toolbar: ToolbarColors::DARK,
    };
    const LIGHT: Self = Self {
        grid_line: hex(0xE6E8EC),
        grid_axis: hex(0xB8BECB),
        snap_dot: hex(0xAAAAAA),
        outline: hex(0x8A8F9B),
        label: hex(0x2A2C33),
        hole: hex(0x4A4E59),
        centerline: hex(0x1E82A5),
        edge: hex(0xC89100),
        surface: Surfaces::LIGHT,
        border: Borders::LIGHT,
        text: TextColors::LIGHT,
        accent: AccentColors::LIGHT,
        status: StatusColors::LIGHT,
        button: ButtonColors::LIGHT,
        align_button: AlignButtonColors::LIGHT,
        tabs: TabColors::LIGHT,
        dock: DockColors::LIGHT,
        dropdown: DropdownColors::LIGHT,
        input: InputColors::LIGHT,
        checkbox: CheckboxColors::LIGHT,
        group_button: GroupButtonColors::LIGHT,
        list: ListColors::LIGHT,
        layout: LayoutColors::LIGHT,
        properties: PropertiesColors::LIGHT,
        title_bar: TitleBarColors::LIGHT,
        status_bar: StatusBarColors::LIGHT,
        toolbar: ToolbarColors::LIGHT,
    };

    /// The palette of the current theme, from `visuals.dark_mode`.
    pub(crate) fn of(painter: &egui::Painter) -> &'static Self {
        Self::of_visuals(&painter.ctx().style().visuals)
    }

    /// The palette of the theme `visuals` belong to, where no painter is at hand.
    pub(crate) fn of_visuals(visuals: &egui::Visuals) -> &'static Self {
        if visuals.dark_mode {
            &Self::DARK
        } else {
            &Self::LIGHT
        }
    }
}
