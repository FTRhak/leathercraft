//! The tabs of the design system, from the Brilliant `Dialog Tab` component: a strip of
//! underlined tabs, each drawn active, inactive, hovered or disabled, in the colors of the
//! current theme.
//!
//! ```ignore
//! tab_bar(ui, |ui| {
//!     tab_value(ui, &mut tab, Page::General, Tab::new("General"));
//!     tab_value(ui, &mut tab, Page::Keys, Tab::new("Keys").icon(glyph::LOCK));
//! });
//! ```
//!
//! Disable a tab the egui way, with `ui.add_enabled(false, Tab::new(..))`, or the whole bar
//! with `ui.add_enabled_ui(false, ..)` around it.

mod style;

use bevy_egui::egui;
use style::{TabState, paint};

use crate::ui_components::icons::paint_glyph;
use crate::ui_components::palette::Palette;

const BAR_HEIGHT: f32 = 34.0;
/// Between two tabs.
const TAB_GAP: f32 = 2.0;
/// Before the first tab and after the last.
const BAR_PADDING: f32 = 14.0;
const TAB_PADDING: f32 = 12.0;
/// The glyph box; `paint_glyph` draws the glyph at 75% of it, about 13 px as in the design.
const ICON_SIZE: f32 = 17.0;
/// Between the icon and the label.
const ICON_GAP: f32 = 5.0;
const UNDERLINE: f32 = 2.0;
const FOCUS_CORNER_RADIUS: u8 = 3;

/// One tab with a text label and an optional icon, added with `ui.add(...)` inside
/// [`tab_bar`], or with [`tab_value`].
#[must_use = "add the tab with `ui.add(tab)` or `tab_value`"]
pub struct Tab {
    text: egui::WidgetText,
    icon: Option<char>,
    selected: bool,
}

impl Tab {
    pub fn new(text: impl Into<egui::WidgetText>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            selected: false,
        }
    }

    /// Draws a glyph of the icon font (`icons::glyph`) before the label.
    // Unused until the font has the Settings tab glyphs (swatches, keyboard, palette).
    #[allow(dead_code)]
    pub fn icon(mut self, glyph: char) -> Self {
        self.icon = Some(glyph);
        self
    }

    /// Whether this is the open tab.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl egui::Widget for Tab {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let galley = self.text.into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Button,
        );
        let icon_width = if self.icon.is_some() {
            ICON_SIZE + ICON_GAP
        } else {
            0.0
        };
        let size = egui::vec2(
            icon_width + galley.size().x + 2.0 * TAB_PADDING,
            ui.available_height().max(galley.size().y),
        );
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::Button,
                ui.is_enabled(),
                self.selected,
                galley.text(),
            )
        });

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() || response.is_pointer_button_down_on();
            let state = TabState::new(self.selected, ui.is_enabled(), hovered);
            let painter = ui.painter();
            let colors = &Palette::of(painter).tabs;
            let style = paint(colors, state);

            let mut x = rect.left() + TAB_PADDING;
            if let Some(glyph) = self.icon {
                let icon_rect = egui::Rect::from_min_size(
                    egui::pos2(x, rect.center().y - ICON_SIZE / 2.0),
                    egui::Vec2::splat(ICON_SIZE),
                );
                paint_glyph(
                    painter,
                    icon_rect,
                    egui::Stroke::new(1.0, style.icon),
                    glyph,
                );
                x += ICON_SIZE + ICON_GAP;
            }
            let text_pos = egui::pos2(x, rect.center().y - galley.size().y / 2.0);
            painter.galley(text_pos, galley, style.text);

            if let Some(underline) = style.underline {
                let line = egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.bottom() - UNDERLINE),
                    rect.right_bottom(),
                );
                painter.rect_filled(line, 0, underline);
            }
            if response.has_focus() {
                painter.rect_stroke(
                    rect.shrink(2.0),
                    FOCUS_CORNER_RADIUS,
                    egui::Stroke::new(1.0, colors.focus),
                    egui::StrokeKind::Inside,
                );
            }
        }
        response
    }
}

/// The strip the tabs sit on, as wide as the `ui`; `add_tabs` lays them out left to right.
pub fn tab_bar<R>(
    ui: &mut egui::Ui,
    add_tabs: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), BAR_HEIGHT),
        egui::Sense::hover(),
    );
    let colors = &Palette::of(ui.painter()).tabs;
    let painter = ui.painter();
    painter.rect_filled(rect, 0, colors.bar);
    let border = egui::Stroke::new(1.0, colors.bar_border);
    painter.hline(rect.x_range(), rect.top() + 0.5, border);
    painter.hline(rect.x_range(), rect.bottom() - 0.5, border);

    let inner = rect.shrink2(egui::vec2(BAR_PADDING, 0.0));
    let mut tabs_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    tabs_ui.spacing_mut().item_spacing.x = TAB_GAP;
    let response = add_tabs(&mut tabs_ui);
    egui::InnerResponse::new(response, tabs_ui.response())
}

/// A tab that opens `value`: selected while `*current == value`, and sets it when clicked.
/// The tab counterpart of `ui.selectable_value`.
pub fn tab_value<V: PartialEq>(
    ui: &mut egui::Ui,
    current: &mut V,
    value: V,
    tab: Tab,
) -> egui::Response {
    let selected = *current == value;
    let mut response = ui.add(tab.selected(selected));
    if response.clicked() && !selected {
        *current = value;
        response.mark_changed();
    }
    response
}
