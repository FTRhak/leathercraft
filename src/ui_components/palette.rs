//! The colors of the dark and the light theme.
//!
//! The values come from the "Palette — Leathercraft Pattern Studio" (dark) and "Palette — Light"
//! boards of the Brilliant design. Every color the app paints is read from here; see
//! `docs/design.md`.

use bevy_egui::egui::{self, Color32};

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
    pub(crate) tabs: TabColors,
    pub(crate) dropdown: DropdownColors,
    pub(crate) input: InputColors,
}

/// Backgrounds, from the deepest to the highest.
#[allow(dead_code)]
pub(crate) struct Surfaces {
    /// The canvas and the window behind it.
    pub(crate) canvas: Color32,
    /// Text fields, selects and the tab strip.
    pub(crate) input: Color32,
    /// Info blocks, such as the material card.
    pub(crate) info: Color32,
    /// The canvas rulers and dialog footers.
    pub(crate) ruler: Color32,
    /// The docked panels and dialogs.
    pub(crate) panel: Color32,
    /// Popups, such as the cursor coordinates.
    pub(crate) popup: Color32,
    /// Chips, keycaps and secondary buttons.
    pub(crate) chip: Color32,
}

/// Thin 1 px lines; the brighter ones are for interactive controls.
#[allow(dead_code)]
pub(crate) struct Borders {
    /// Between sections inside a panel.
    pub(crate) divider: Color32,
    /// The edges of the docked panels.
    pub(crate) panel: Color32,
    /// Buttons and floating panels.
    pub(crate) control: Color32,
    /// Text fields and selects.
    pub(crate) field: Color32,
    /// Chips, keycaps and "+" buttons.
    pub(crate) chip: Color32,
}

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
    /// Section headers, in capitals at 10 px.
    pub(crate) section_header: Color32,
    /// Placeholders and ruler marks.
    pub(crate) muted: Color32,
}

/// The single accent of the interface: active mode, selection, focus.
#[allow(dead_code)]
pub(crate) struct AccentColors {
    /// Active icons and the selection border; the same as `centerline`.
    pub(crate) base: Color32,
    /// The primary button and the active mode.
    pub(crate) fill: Color32,
    /// The border of accent chips.
    pub(crate) border: Color32,
    /// The background of accent chips.
    pub(crate) container: Color32,
    /// The background of the selected row, or of a hotkey being recorded.
    pub(crate) selected_row: Color32,
}

/// Warnings, recording and the axes.
#[allow(dead_code)]
pub(crate) struct StatusColors {
    /// Hotkey conflicts.
    pub(crate) warning: Color32,
    /// The background of a conflict chip.
    pub(crate) warning_container: Color32,
    /// The X axis of the gizmo, and the recording indicator.
    pub(crate) axis_x: Color32,
    /// The Y axis of the gizmo.
    pub(crate) axis_y: Color32,
}

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

/// The tab strip of `ui_components::tabs`, from the Brilliant `Dialog Tab` component.
pub(crate) struct TabColors {
    /// The background of the strip; the same as `surface.input`.
    pub(crate) bar: Color32,
    /// The lines above and below the strip.
    pub(crate) bar_border: Color32,
    /// The open tab.
    pub(crate) active: TabPaint,
    pub(crate) inactive: TabPaint,
    /// An inactive tab under the pointer.
    pub(crate) hover: TabPaint,
    /// An inactive tab that can't be opened.
    pub(crate) disabled: TabPaint,
    /// The open tab of a disabled strip.
    pub(crate) disabled_active: TabPaint,
    /// The ring inside a tab that has keyboard focus; the same as `accent.base`.
    pub(crate) focus: Color32,
}

/// The paint of a tab in one state. A `None` underline draws no line under the tab.
pub(crate) struct TabPaint {
    pub(crate) text: Color32,
    pub(crate) icon: Color32,
    pub(crate) underline: Option<Color32>,
}

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

const fn field(fill: u32, stroke: u32, text: u32, caret: u32) -> FieldPaint {
    FieldPaint {
        fill: hex(fill),
        stroke: hex(stroke),
        text: hex(text),
        caret: hex(caret),
    }
}

const fn tab(text: u32, icon: u32, underline: Option<u32>) -> TabPaint {
    TabPaint {
        text: hex(text),
        icon: hex(icon),
        underline: match underline {
            Some(rgb) => Some(hex(rgb)),
            None => None,
        },
    }
}

const fn paint(fill: Option<u32>, stroke: u32, text: u32) -> ButtonPaint {
    ButtonPaint {
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
        surface: Surfaces {
            canvas: hex(0x121316),
            input: hex(0x131418),
            info: hex(0x141519),
            ruler: hex(0x16171B),
            panel: hex(0x1A1B20),
            popup: hex(0x1F2128),
            chip: hex(0x23252B),
        },
        border: Borders {
            divider: hex(0x25272F),
            panel: hex(0x282A32),
            control: hex(0x2B2E37),
            field: hex(0x2E313B),
            chip: hex(0x383B46),
        },
        text: TextColors {
            primary: hex(0xECEEF2),
            strong: hex(0xDCDFE6),
            label: hex(0xB8BBC6),
            secondary: hex(0x8E929F),
            section_header: hex(0x7E818C),
            muted: hex(0x6B6F7B),
        },
        accent: AccentColors {
            base: hex(0x78C3D7),
            fill: hex(0x1B6B85),
            border: hex(0x365966),
            container: hex(0x16323B),
            selected_row: hex(0x16232A),
        },
        status: StatusColors {
            warning: hex(0xF0A83A),
            warning_container: hex(0x2A1F14),
            axis_x: hex(0xE06C6C),
            axis_y: hex(0x6CC07A),
        },
        button: ButtonColors {
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
        },
        tabs: TabColors {
            bar: hex(0x131418),
            bar_border: hex(0x26282E),
            active: tab(0xECEEF2, 0x78C3D7, Some(0x78C3D7)),
            inactive: tab(0x8E929F, 0x7E818C, None),
            hover: tab(0xDCDFE6, 0xB8BBC6, Some(0x383B46)),
            disabled: tab(0x5A5E69, 0x4A4E59, None),
            disabled_active: tab(0x6B6F7B, 0x365966, Some(0x365966)),
            focus: hex(0x78C3D7),
        },
        dropdown: DropdownColors {
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
        },
        input: InputColors {
            default: input(0x131418, 0x2E313B, 0xECEEF2, 0x6B6F7B, 0x8E929F, 0x8E929F),
            hover: input(0x16171B, 0x3A3D47, 0xECEEF2, 0x6B6F7B, 0x8E929F, 0xB8BBC6),
            focus: input(0x131418, 0x78C3D7, 0xECEEF2, 0x6B6F7B, 0x8E929F, 0x78C3D7),
            accent: input(0x131418, 0x5E5023, 0xFAD75A, 0x6B6F7B, 0xB39A45, 0x8E929F),
            disabled: input(0x16171B, 0x25272F, 0x5A5E69, 0x454852, 0x454852, 0x3A3D47),
        },
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
        surface: Surfaces {
            canvas: hex(0xFAFAFB),
            input: hex(0xFFFFFF),
            info: hex(0xF2F3F6),
            ruler: hex(0xECEEF2),
            panel: hex(0xF0F1F4),
            popup: hex(0xFFFFFF),
            chip: hex(0xE6E8ED),
        },
        border: Borders {
            divider: hex(0xE3E5EA),
            panel: hex(0xD9DCE2),
            control: hex(0xD2D5DC),
            field: hex(0xC9CCD4),
            chip: hex(0xBCC0C9),
        },
        text: TextColors {
            primary: hex(0x17181C),
            strong: hex(0x2A2C33),
            label: hex(0x4A4E59),
            secondary: hex(0x6B6F7B),
            section_header: hex(0x7E818C),
            muted: hex(0x8A8E99),
        },
        accent: AccentColors {
            base: hex(0x1E82A5),
            fill: hex(0x176E8D),
            border: hex(0x8CC4D6),
            container: hex(0xE1F1F6),
            selected_row: hex(0xEAF4F8),
        },
        status: StatusColors {
            warning: hex(0xC77A0A),
            warning_container: hex(0xFDF1DE),
            axis_x: hex(0xD14B4B),
            axis_y: hex(0x2F9E48),
        },
        button: ButtonColors {
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
        },
        tabs: TabColors {
            bar: hex(0xFFFFFF),
            bar_border: hex(0xE3E5EA),
            active: tab(0x17181C, 0x1E82A5, Some(0x1E82A5)),
            inactive: tab(0x6B6F7B, 0x7E818C, None),
            hover: tab(0x2A2C33, 0x4A4E59, Some(0xC9CCD4)),
            disabled: tab(0x9A9EA9, 0xB0B4BD, None),
            disabled_active: tab(0x8A8E99, 0x8CC4D6, Some(0x8CC4D6)),
            focus: hex(0x1E82A5),
        },
        dropdown: DropdownColors {
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
        },
        input: InputColors {
            default: input(0xFFFFFF, 0xC9CCD4, 0x17181C, 0x8A8E99, 0x6B6F7B, 0x6B6F7B),
            hover: input(0xF7F8FA, 0xA9AEB8, 0x17181C, 0x8A8E99, 0x6B6F7B, 0x4A4E59),
            focus: input(0xFFFFFF, 0x1E82A5, 0x17181C, 0x8A8E99, 0x6B6F7B, 0x1E82A5),
            accent: input(0xFFFFFF, 0xE0C66E, 0x8A6400, 0x8A8E99, 0x9C7F2E, 0x6B6F7B),
            disabled: input(0xF2F3F6, 0xE3E5EA, 0x9A9EA9, 0xBCC0C9, 0xBCC0C9, 0xBCC0C9),
        },
    };

    /// The palette of the current theme, from `visuals.dark_mode`.
    pub(crate) fn of(painter: &egui::Painter) -> &'static Self {
        if painter.ctx().style().visuals.dark_mode {
            &Self::DARK
        } else {
            &Self::LIGHT
        }
    }
}
