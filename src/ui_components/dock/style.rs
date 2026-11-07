//! The look of the docked panels, from the Brilliant `DockTab` component and the dock tab
//! strips of the main window.
//!
//! `egui_dock` draws the strips, tabs, close crosses and separators from [`dock_style`]. What
//! its style can't express is added by [`TabDecor`]: the icon color of each tab state, the
//! accent line along the top of the open tab, and the divider right of an inactive one.

use bevy_egui::egui::{self, Color32, CornerRadius, Stroke};
use egui_dock::{DockState, TabInteractionStyle};

use super::PanelTab;
use crate::ui_components::icons::icon_font_id;
use crate::ui_components::palette::{DockTabPaint, Palette};

const BAR_HEIGHT: f32 = 32.0;
/// The glyph size, 13 px as in the design.
const ICON_SIZE: f32 = 13.0;
/// Between the icon and the label.
const ICON_GAP: f32 = 6.0;
const ACCENT_WIDTH: f32 = 2.0;

/// The `egui_dock` style of the current theme; build it every frame, since the theme can
/// change in the Settings window.
pub fn dock_style(style: &egui::Style) -> egui_dock::Style {
    let colors = &Palette::of_visuals(&style.visuals).dock;
    let mut dock = egui_dock::Style::from_egui(style);
    dock.main_surface_border_stroke = Stroke::NONE;

    dock.tab_bar.bg_fill = colors.bar;
    dock.tab_bar.height = BAR_HEIGHT;
    dock.tab_bar.corner_radius = CornerRadius::ZERO;
    dock.tab_bar.inner_margin = egui::Margin::ZERO;
    dock.tab_bar.hline_color = colors.bar_border;

    let active = tab_style(&colors.active);
    let inactive = tab_style(&colors.inactive);
    let hover = tab_style(&colors.hover);
    dock.tab.active = active.clone();
    dock.tab.focused = active.clone();
    dock.tab.active_with_kb_focus = active.clone();
    dock.tab.focused_with_kb_focus = active;
    dock.tab.inactive = inactive;
    dock.tab.hovered = hover.clone();
    dock.tab.inactive_with_kb_focus = hover;
    dock.tab.spacing = 0.0;
    dock.tab.hline_below_active_tab_name = false;
    dock.tab.minimum_width = None;

    // The separators between groups are their only border.
    dock.tab.tab_body.bg_fill = colors.body;
    dock.tab.tab_body.stroke = Stroke::NONE;
    dock.tab.tab_body.corner_radius = CornerRadius::ZERO;

    dock.buttons.close_tab_color = colors.close;
    dock.buttons.close_tab_active_color = colors.close_hover;
    dock.buttons.close_tab_bg_fill = Color32::TRANSPARENT;

    dock.separator.width = 1.0;
    dock.separator.color_idle = colors.separator;
    dock.separator.color_hovered = colors.separator_hover;
    dock.separator.color_dragged = colors.separator_drag;
    dock
}

/// `egui_dock` strokes all four sides of a tab; the outline takes the fill so that it vanishes,
/// and [`TabDecor`] draws the edges the design has.
fn tab_style(paint: &DockTabPaint) -> TabInteractionStyle {
    TabInteractionStyle {
        outline_color: paint.fill,
        corner_radius: CornerRadius::ZERO,
        bg_fill: paint.fill,
        text_color: paint.text,
    }
}

/// The tab states `egui_dock` doesn't pass to the `TabViewer`, kept across frames.
///
/// Both are a frame old when a title is built, which is never seen: the app redraws every frame.
#[derive(Default)]
pub struct TabDecor {
    /// The open tab of each group.
    active: Vec<PanelTab>,
    hovered: Option<PanelTab>,
    next_hovered: Option<PanelTab>,
}

impl TabDecor {
    /// Call before the dock is shown.
    pub fn begin_frame(&mut self, state: &DockState<PanelTab>) {
        self.active.clear();
        self.active.extend(
            state
                .iter_leaves()
                .filter_map(|(_, leaf)| leaf.tabs.get(leaf.active.0).copied()),
        );
        self.hovered = self.next_hovered.take();
    }

    fn paint<'p>(&self, palette: &'p Palette, tab: PanelTab) -> &'p DockTabPaint {
        let colors = &palette.dock;
        if self.active.contains(&tab) {
            &colors.active
        } else if self.hovered == Some(tab) {
            &colors.hover
        } else {
            &colors.inactive
        }
    }

    /// A tab title: the glyph in the icon color of the tab's state, then the label, which
    /// `egui_dock` paints in the text color of that state.
    pub fn title(
        &self,
        style: &egui::Style,
        tab: PanelTab,
        glyph: Option<char>,
        text: &str,
    ) -> egui::WidgetText {
        let mut job = egui::text::LayoutJob::default();
        let mut gap = 0.0;
        if let Some(glyph) = glyph {
            let icon = self.paint(Palette::of_visuals(&style.visuals), tab).icon;
            job.append(
                glyph.encode_utf8(&mut [0; 4]),
                0.0,
                egui::TextFormat {
                    font_id: icon_font_id(ICON_SIZE),
                    color: icon,
                    valign: egui::Align::Center,
                    ..Default::default()
                },
            );
            gap = ICON_GAP;
        }
        job.append(
            text,
            gap,
            egui::TextFormat {
                font_id: egui::TextStyle::Button.resolve(style),
                color: Color32::PLACEHOLDER,
                valign: egui::Align::Center,
                ..Default::default()
            },
        );
        job.into()
    }

    /// Paints the edges of a tab button that `egui_dock` has just drawn, from
    /// `TabViewer::on_tab_button`.
    pub fn decorate(&mut self, tab: PanelTab, response: &egui::Response) {
        if response.hovered() {
            self.next_hovered = Some(tab);
        }
        let ctx = &response.ctx;
        let rect = response.rect;
        // A dragged tab is drawn on its own layer under the pointer, away from this rect.
        let dragged = ctx.dragged_id().is_some()
            && ctx.input(|input| {
                input.pointer.is_decidedly_dragging()
                    && input
                        .pointer
                        .press_origin()
                        .is_some_and(|origin| rect.contains(origin))
            });
        if dragged {
            return;
        }
        let painter = ctx
            .layer_painter(response.layer_id)
            .with_clip_rect(response.interact_rect);
        let colors = &Palette::of(&painter).dock;
        if self.active.contains(&tab) {
            let line = egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.right(), rect.top() + ACCENT_WIDTH),
            );
            painter.rect_filled(line, 0, colors.accent);
        } else {
            let px = ctx.pixels_per_point().recip();
            painter.vline(
                rect.right() - px * 0.5,
                rect.y_range(),
                Stroke::new(px, colors.bar_border),
            );
        }
    }
}
