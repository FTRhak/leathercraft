//! The title bar of the design system, from the "Title & Menu Bar" of the Brilliant design. It
//! replaces the system title bar, which the window is created without: the app brand and the
//! menus on the left, a switch in the middle, app controls and the window buttons on the right.
//!
//! ```ignore
//! let command = title_bar::show(ctx, maximized, &labels, |ui, slot| match slot {
//!     Slot::Left => {
//!         title_bar::menu(ui, "file", language.label(Label::File), MenuConfig::new(), |ui| {
//!             ..
//!         });
//!     }
//!     Slot::Center => {
//!         ui.add(ModeSwitch::new(&mut mode).option(..));
//!     }
//!     Slot::Right => {
//!         title_bar::caption(ui, language.label(Label::Language));
//!     }
//! });
//! ```
//!
//! The bar doesn't touch the window itself: [`show`] and [`resize_handles`] return a
//! [`WindowCommand`] for the caller to apply, when the user drags the bar or an edge of the window,
//! double-clicks the bar, or clicks a window button.

mod icons;
mod mode_switch;

pub use mode_switch::{ModeIcon, ModeSwitch};

use bevy::math::CompassOctant;
use bevy_egui::egui;

use crate::ui_components::palette::{Palette, TitleBarColors};

const HEIGHT: f32 = 38.0;
const PADDING_X: i8 = 14;
/// Between the groups of the bar: brand, divider, menus.
const GROUP_GAP: f32 = 18.0;
/// Between the controls of the right group.
const RIGHT_GAP: f32 = 12.0;
/// Kept free around the switch in the middle.
const CENTER_GAP: f32 = 16.0;
const DIVIDER_HEIGHT: f32 = 16.0;

const LOGO_SIZE: f32 = 22.0;
const LOGO_RADIUS: u8 = 5;
const LOGO_ICON: egui::Vec2 = egui::vec2(13.0, 10.5);
const BRAND_GAP: f32 = 8.0;
const BRAND_FONT_SIZE: f32 = 13.0;
/// The brand is a name, so it is the same in every language.
const APP_NAME: &str = "Leathercraft";
const APP_SUBTITLE: &str = "Pattern Studio";

const MENU_FONT_SIZE: f32 = 12.0;
const MENU_PADDING: egui::Vec2 = egui::vec2(8.0, 4.0);
const MENU_RADIUS: u8 = 4;
const MENU_GAP: f32 = 2.0;

const CAPTION_FONT_SIZE: f32 = 11.0;

const WINDOW_BUTTON: egui::Vec2 = egui::vec2(26.0, 26.0);
const WINDOW_BUTTON_RADIUS: u8 = 4;
const WINDOW_ICON_SIZE: f32 = 11.0;
const WINDOW_ICON_STROKE: f32 = 1.2;

/// How far from the edge of the window the pointer resizes it.
const RESIZE_MARGIN: f32 = 5.0;
/// The size of the corner regions, which resize in both directions.
const RESIZE_CORNER: f32 = 12.0;

/// What the title bar asks of the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCommand {
    Minimize,
    /// Maximize the window, or restore its size when it is maximized.
    ToggleMaximize,
    Close,
    /// Move the window with the pointer, from a press on the bar.
    StartMove,
    /// Resize the window with the pointer, from a press on its edge.
    StartResize(CompassOctant),
}

/// The tooltips of the window buttons, in the current language.
pub struct WindowLabels<'a> {
    pub minimize: &'a str,
    pub maximize: &'a str,
    pub restore: &'a str,
    pub close: &'a str,
}

/// Which part of the bar [`show`] asks its closure to fill.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// After the brand: the menus.
    Left,
    /// Before the window buttons, laid out right to left.
    Right,
    /// Centered in the bar, or in the space the other slots leave, if that is narrower.
    Center,
}

/// Shows the title bar at the top of the window. `add_contents` is called once per [`Slot`],
/// left, right, then center.
pub fn show(
    ctx: &egui::Context,
    maximized: bool,
    labels: &WindowLabels,
    mut add_contents: impl FnMut(&mut egui::Ui, Slot),
) -> Option<WindowCommand> {
    let colors = &Palette::of(&ctx.layer_painter(egui::LayerId::background())).title_bar;
    let frame = egui::Frame::new()
        .fill(colors.background)
        .inner_margin(egui::Margin::symmetric(PADDING_X, 0));
    egui::TopBottomPanel::top("title_bar")
        .exact_height(HEIGHT)
        .resizable(false)
        .show_separator_line(false)
        .frame(frame)
        .show(ctx, |ui| {
            let bar = ui.max_rect();
            // Registered before the controls, so that they take the pointer over it.
            let background = ui.interact(
                bar,
                ui.id().with("title_bar_drag"),
                egui::Sense::click_and_drag(),
            );
            let mut command = if background.double_clicked() {
                Some(WindowCommand::ToggleMaximize)
            } else if background.drag_started_by(egui::PointerButton::Primary) {
                Some(WindowCommand::StartMove)
            } else {
                None
            };

            let row = |ui: &mut egui::Ui, layout: egui::Layout| {
                ui.new_child(egui::UiBuilder::new().max_rect(bar).layout(layout))
            };
            let mut left = row(ui, egui::Layout::left_to_right(egui::Align::Center));
            left.spacing_mut().item_spacing.x = GROUP_GAP;
            brand(&mut left, colors);
            divider(&mut left, colors);
            left.scope(|ui| {
                ui.spacing_mut().item_spacing.x = MENU_GAP;
                add_contents(ui, Slot::Left);
            });

            let mut right = row(ui, egui::Layout::right_to_left(egui::Align::Center));
            right.spacing_mut().item_spacing.x = RIGHT_GAP;
            right.scope(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                if let Some(clicked) = window_buttons(ui, colors, maximized, labels) {
                    command = Some(clicked);
                }
            });
            divider(&mut right, colors);
            add_contents(&mut right, Slot::Right);

            let left_end = left.min_rect().right() + CENTER_GAP;
            let right_start = right.min_rect().left() - CENTER_GAP;
            let half = (bar.center().x - left_end).min(right_start - bar.center().x);
            let (from, to) = if half > 0.0 {
                (bar.center().x - half, bar.center().x + half)
            } else {
                (left_end, right_start.max(left_end))
            };
            let center = egui::Rect::from_x_y_ranges(from..=to, bar.y_range());
            let mut middle = ui.new_child(
                egui::UiBuilder::new().max_rect(center).layout(
                    egui::Layout::left_to_right(egui::Align::Center)
                        .with_main_align(egui::Align::Center),
                ),
            );
            middle.set_clip_rect(center.intersect(ui.clip_rect()));
            add_contents(&mut middle, Slot::Center);

            ui.painter().hline(
                ui.clip_rect().x_range(),
                bar.bottom() + 0.5,
                egui::Stroke::new(1.0, colors.border),
            );
            command
        })
        .inner
}

/// A menu title that opens `add_contents` below it when clicked.
pub fn menu<R>(
    ui: &mut egui::Ui,
    id_salt: impl std::hash::Hash,
    title: impl Into<egui::WidgetText>,
    config: egui::containers::menu::MenuConfig,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    let id = ui.make_persistent_id(id_salt);
    let galley = title.into().into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::FontId::proportional(MENU_FONT_SIZE),
    );
    let (rect, _) =
        ui.allocate_exact_size(galley.size() + 2.0 * MENU_PADDING, egui::Sense::hover());
    let response = ui.interact(rect, id, egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), galley.text())
    });

    if ui.is_rect_visible(rect) {
        let colors = &Palette::of(ui.painter()).title_bar.menu;
        let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&response));
        let style = if open {
            &colors.open
        } else if response.hovered() {
            &colors.hover
        } else {
            &colors.default
        };
        if let Some(fill) = style.fill {
            ui.painter().rect_filled(rect, MENU_RADIUS, fill);
        }
        let text_pos = rect.center() - galley.size() / 2.0;
        ui.painter().galley(text_pos, galley, style.text);
    }

    egui::Popup::menu(&response)
        .close_behavior(config.close_behavior)
        .style(config.style)
        .show(add_contents)
        .map(|inner| inner.inner)
}

/// A small text before a control, such as "Language" before its dropdown.
pub fn caption(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    let color = Palette::of(ui.painter()).title_bar.caption;
    ui.label(
        egui::RichText::new(text)
            .size(CAPTION_FONT_SIZE)
            .color(color),
    )
}

/// The thin edges around the window, which resize it, since the window has no system frame.
/// Show them only while the window isn't maximized.
pub fn resize_handles(ctx: &egui::Context) -> Option<WindowCommand> {
    use CompassOctant::*;
    let screen = ctx.content_rect();
    let (margin, corner) = (RESIZE_MARGIN, RESIZE_CORNER);
    let (left, right, top, bottom) = (screen.left(), screen.right(), screen.top(), screen.bottom());
    let span = |from: f32, to: f32| from..=to;
    let regions = [
        (
            NorthWest,
            span(left, left + corner),
            span(top, top + corner),
        ),
        (
            NorthEast,
            span(right - corner, right),
            span(top, top + corner),
        ),
        (
            SouthWest,
            span(left, left + corner),
            span(bottom - corner, bottom),
        ),
        (
            SouthEast,
            span(right - corner, right),
            span(bottom - corner, bottom),
        ),
        (
            North,
            span(left + corner, right - corner),
            span(top, top + margin),
        ),
        (
            South,
            span(left + corner, right - corner),
            span(bottom - margin, bottom),
        ),
        (
            West,
            span(left, left + margin),
            span(top + corner, bottom - corner),
        ),
        (
            East,
            span(right - margin, right),
            span(top + corner, bottom - corner),
        ),
    ];
    let mut command = None;
    for (direction, x, y) in regions {
        let rect = egui::Rect::from_x_y_ranges(x, y);
        egui::Area::new(egui::Id::new(("window_resize", direction as u8)))
            .order(egui::Order::Foreground)
            .fixed_pos(rect.min)
            .constrain(false)
            .show(ctx, |ui| {
                // A drag without a click starts on the press, while the button is still down.
                let response = ui.allocate_rect(rect, egui::Sense::drag());
                if response.drag_started_by(egui::PointerButton::Primary) {
                    command = Some(WindowCommand::StartResize(direction));
                }
                response.on_hover_cursor(resize_cursor(direction));
            });
    }
    command
}

fn resize_cursor(direction: CompassOctant) -> egui::CursorIcon {
    use CompassOctant::*;
    match direction {
        North => egui::CursorIcon::ResizeNorth,
        NorthEast => egui::CursorIcon::ResizeNorthEast,
        East => egui::CursorIcon::ResizeEast,
        SouthEast => egui::CursorIcon::ResizeSouthEast,
        South => egui::CursorIcon::ResizeSouth,
        SouthWest => egui::CursorIcon::ResizeSouthWest,
        West => egui::CursorIcon::ResizeWest,
        NorthWest => egui::CursorIcon::ResizeNorthWest,
    }
}

/// The logo badge and the app name.
fn brand(ui: &mut egui::Ui, colors: &TitleBarColors) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = BRAND_GAP;
        let (logo, _) = ui.allocate_exact_size(egui::Vec2::splat(LOGO_SIZE), egui::Sense::hover());
        ui.painter().rect(
            logo,
            LOGO_RADIUS,
            colors.logo_fill,
            egui::Stroke::new(1.0, colors.logo_border),
            egui::StrokeKind::Inside,
        );
        let icon = egui::Rect::from_center_size(logo.center(), LOGO_ICON);
        icons::scissors(ui.painter(), icon, egui::Stroke::new(1.2, colors.logo_icon));
        let font = egui::FontId::proportional(BRAND_FONT_SIZE);
        ui.add(
            egui::Label::new(
                egui::RichText::new(APP_NAME)
                    .font(font.clone())
                    .strong()
                    .color(colors.app_name),
            )
            .selectable(false),
        );
        ui.add(
            egui::Label::new(
                egui::RichText::new(APP_SUBTITLE)
                    .font(font)
                    .color(colors.app_subtitle),
            )
            .selectable(false),
        );
    });
}

fn divider(ui: &mut egui::Ui, colors: &TitleBarColors) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, DIVIDER_HEIGHT), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, colors.divider);
}

/// Minimize, maximize or restore, and close, laid out right to left.
fn window_buttons(
    ui: &mut egui::Ui,
    colors: &TitleBarColors,
    maximized: bool,
    labels: &WindowLabels,
) -> Option<WindowCommand> {
    let (maximize_label, maximize_icon): (_, fn(&egui::Painter, egui::Rect, egui::Stroke)) =
        if maximized {
            (labels.restore, icons::restore)
        } else {
            (labels.maximize, icons::maximize)
        };
    let buttons = [
        (
            WindowCommand::Close,
            labels.close,
            icons::close as fn(&_, _, _),
        ),
        (WindowCommand::ToggleMaximize, maximize_label, maximize_icon),
        (WindowCommand::Minimize, labels.minimize, icons::minimize),
    ];
    let mut command = None;
    for (action, label, paint) in buttons {
        let (rect, response) = ui.allocate_exact_size(WINDOW_BUTTON, egui::Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
        });
        if ui.is_rect_visible(rect) {
            let paint_colors = &colors.window_button;
            let hovered = response.hovered() || response.is_pointer_button_down_on();
            let icon_color = match (hovered, action) {
                (false, _) => paint_colors.icon,
                (true, WindowCommand::Close) => {
                    ui.painter()
                        .rect_filled(rect, WINDOW_BUTTON_RADIUS, paint_colors.close_fill);
                    paint_colors.close_icon
                }
                (true, _) => {
                    ui.painter()
                        .rect_filled(rect, WINDOW_BUTTON_RADIUS, paint_colors.hover_fill);
                    paint_colors.hover_icon
                }
            };
            let icon =
                egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(WINDOW_ICON_SIZE));
            paint(
                ui.painter(),
                icon,
                egui::Stroke::new(WINDOW_ICON_STROKE, icon_color),
            );
        }
        if response.on_hover_text(label).clicked() {
            command = Some(action);
        }
    }
    command
}
