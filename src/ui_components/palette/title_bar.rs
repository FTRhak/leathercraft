use bevy_egui::egui::Color32;

use super::{hex, hex_opt};

/// The title bar of `ui_components::title_bar`, from the "Title & Menu Bar" of the Brilliant
/// design. The design has a dark board only; the light values follow the light surfaces and the
/// group button, as the dark ones do.
pub(crate) struct TitleBarColors {
    pub(crate) background: Color32,
    /// The line under the bar.
    pub(crate) border: Color32,
    /// The short vertical lines between the groups of the bar.
    pub(crate) divider: Color32,
    /// The background and edge of the logo badge, and its scissors.
    pub(crate) logo_fill: Color32,
    pub(crate) logo_border: Color32,
    pub(crate) logo_icon: Color32,
    /// "Leathercraft", the first word of the app name.
    pub(crate) app_name: Color32,
    /// "Pattern Studio", the rest of it, in the accent.
    pub(crate) app_subtitle: Color32,
    /// A menu title, at rest, under the pointer and while its menu is open.
    pub(crate) menu: MenuTitleStates,
    /// The caption before a control, such as "Language".
    pub(crate) caption: Color32,
    pub(crate) mode: ModeSwitchColors,
    pub(crate) window_button: WindowButtonColors,
}

/// The paint of a menu title in each state.
pub(crate) struct MenuTitleStates {
    pub(crate) default: MenuTitlePaint,
    pub(crate) hover: MenuTitlePaint,
    pub(crate) open: MenuTitlePaint,
}

/// The paint of a menu title in one state. A `None` fill leaves the bar showing through.
pub(crate) struct MenuTitlePaint {
    pub(crate) fill: Option<Color32>,
    pub(crate) text: Color32,
}

/// The 2D editor and 3D preview switch in the middle of the title bar.
pub(crate) struct ModeSwitchColors {
    /// The background and edge of the track holding the options.
    pub(crate) track: Color32,
    pub(crate) track_border: Color32,
    /// The current mode; the same as the primary button.
    pub(crate) active: ModePaint,
    pub(crate) inactive: ModePaint,
    /// Another mode under the pointer.
    pub(crate) hover: ModePaint,
    /// The ring inside an option that has keyboard focus; the same as `accent.base`.
    pub(crate) focus: Color32,
}

/// The paint of one option of the mode switch. A `None` fill or border draws none.
pub(crate) struct ModePaint {
    pub(crate) fill: Option<Color32>,
    pub(crate) border: Option<Color32>,
    /// The text and the icon.
    pub(crate) text: Color32,
}

/// Minimize, maximize and close at the right end of the title bar.
pub(crate) struct WindowButtonColors {
    pub(crate) icon: Color32,
    pub(crate) hover_fill: Color32,
    pub(crate) hover_icon: Color32,
    /// The close button under the pointer, red as on the system title bar.
    pub(crate) close_fill: Color32,
    pub(crate) close_icon: Color32,
}

const fn menu_title(fill: Option<u32>, text: u32) -> MenuTitlePaint {
    MenuTitlePaint {
        fill: hex_opt(fill),
        text: hex(text),
    }
}

const fn mode(fill: Option<u32>, border: Option<u32>, text: u32) -> ModePaint {
    ModePaint {
        fill: hex_opt(fill),
        border: hex_opt(border),
        text: hex(text),
    }
}

impl TitleBarColors {
    pub(super) const DARK: Self = Self {
        background: hex(0x111215),
        border: hex(0x25272E),
        divider: hex(0x282A32),
        logo_fill: hex(0x1B6B85),
        logo_border: hex(0x78C3D7),
        logo_icon: hex(0xFAFAF9),
        app_name: hex(0xF3F4F6),
        app_subtitle: hex(0x78C3D7),
        menu: MenuTitleStates {
            default: menu_title(None, 0xC8CAD2),
            hover: menu_title(Some(0x1F2128), 0xECEEF2),
            open: menu_title(Some(0x23252B), 0xECEEF2),
        },
        caption: hex(0x7E818C),
        mode: ModeSwitchColors {
            track: hex(0x1B1C21),
            track_border: hex(0x2B2E37),
            active: mode(Some(0x1B6B85), Some(0x78C3D7), 0xFFFFFF),
            inactive: mode(None, None, 0x8E929F),
            hover: mode(Some(0x23252B), None, 0xDCDFE6),
            focus: hex(0x78C3D7),
        },
        window_button: WindowButtonColors {
            icon: hex(0x8E929F),
            hover_fill: hex(0x23252B),
            hover_icon: hex(0xECEEF2),
            close_fill: hex(0xC42B1C),
            close_icon: hex(0xFFFFFF),
        },
    };
    pub(super) const LIGHT: Self = Self {
        background: hex(0xEBEDF1),
        border: hex(0xD9DCE2),
        divider: hex(0xCDD0D7),
        logo_fill: hex(0x176E8D),
        logo_border: hex(0x1E82A5),
        logo_icon: hex(0xFFFFFF),
        app_name: hex(0x17181C),
        app_subtitle: hex(0x1E82A5),
        menu: MenuTitleStates {
            default: menu_title(None, 0x3A3D47),
            hover: menu_title(Some(0xDDE0E6), 0x17181C),
            open: menu_title(Some(0xD5D8DF), 0x17181C),
        },
        caption: hex(0x7E818C),
        mode: ModeSwitchColors {
            track: hex(0xE2E4E9),
            track_border: hex(0xD2D5DC),
            active: mode(Some(0x176E8D), Some(0x1E82A5), 0xFFFFFF),
            inactive: mode(None, None, 0x6B6F7B),
            hover: mode(Some(0xD5D8DF), None, 0x2A2C33),
            focus: hex(0x1E82A5),
        },
        window_button: WindowButtonColors {
            icon: hex(0x6B6F7B),
            hover_fill: hex(0xDDE0E6),
            hover_icon: hex(0x17181C),
            close_fill: hex(0xC42B1C),
            close_icon: hex(0xFFFFFF),
        },
    };
}
