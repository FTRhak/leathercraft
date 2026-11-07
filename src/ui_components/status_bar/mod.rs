//! The status bar of the design system, from the "Status Bar" of the Brilliant design: the last
//! message with a dot that tells success from error, and what is selected, on the left; the grid
//! step, the cursor coordinates and the graphics backend on the right.
//!
//! ```ignore
//! status_bar::show(ctx, &StatusBar {
//!     message: &message,
//!     tone: Tone::Ok,
//!     selection: Some(&selection),
//!     grid: None,
//!     cursor: Some(&cursor),
//!     renderer: Some(("wgpu · Vulkan", "NVIDIA GeForce RTX 3060")),
//! });
//! ```
//!
//! Every item but the message is optional and leaves no divider behind when it is `None`. The
//! right items are laid out first, so a long message or selection is cut short, never them.

use bevy_egui::egui;

use crate::ui_components::palette::{Palette, StatusBarColors};

const HEIGHT: f32 = 26.0;
const PADDING_X: i8 = 14;
/// Between the left and the right group, kept free when the left one is cut short.
const GROUP_GAP: f32 = 16.0;
/// Between the items of the left group, and of the right one.
const LEFT_GAP: f32 = 10.0;
const RIGHT_GAP: f32 = 14.0;
const DIVIDER_HEIGHT: f32 = 12.0;

const DOT_SIZE: f32 = 6.0;
const DOT_GAP: f32 = 5.0;
const TEXT_SIZE: f32 = 11.0;
const RENDERER_SIZE: f32 = 10.0;

/// Whether the message reports a success or an error; it picks the color of the dot and text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Ok,
    Error,
}

/// The text of the status bar, already translated.
pub struct StatusBar<'a> {
    pub message: &'a str,
    pub tone: Tone,
    /// What is selected.
    pub selection: Option<&'a str>,
    /// The grid step, while snapping to the grid.
    pub grid: Option<&'a str>,
    /// The coordinates under the pointer, while it is over the canvas.
    pub cursor: Option<&'a str>,
    /// The graphics backend, and a tooltip with the adapter.
    pub renderer: Option<(&'a str, &'a str)>,
}

/// Shows the status bar at the bottom of the window.
pub fn show(ctx: &egui::Context, bar: &StatusBar) {
    let colors = &Palette::of(&ctx.layer_painter(egui::LayerId::background())).status_bar;
    let frame = egui::Frame::new()
        .fill(colors.background)
        .inner_margin(egui::Margin::symmetric(PADDING_X, 0));
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(HEIGHT)
        .resizable(false)
        .show_separator_line(false)
        .frame(frame)
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let row = |ui: &mut egui::Ui, rect: egui::Rect, layout: egui::Layout| {
                ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(layout))
            };

            let mut right = row(ui, rect, egui::Layout::right_to_left(egui::Align::Center));
            right.spacing_mut().item_spacing.x = RIGHT_GAP;
            right_items(&mut right, colors, bar);

            let left_end = if right.min_rect().width() > 0.0 {
                right.min_rect().left() - GROUP_GAP
            } else {
                rect.right()
            };
            let left_rect = egui::Rect::from_x_y_ranges(
                rect.left()..=left_end.max(rect.left()),
                rect.y_range(),
            );
            let mut left = row(
                ui,
                left_rect,
                egui::Layout::left_to_right(egui::Align::Center),
            );
            left.set_clip_rect(left_rect.intersect(ui.clip_rect()));
            left.spacing_mut().item_spacing.x = LEFT_GAP;
            left_items(&mut left, colors, bar);

            ui.painter().hline(
                ui.clip_rect().x_range(),
                rect.top() - 0.5,
                egui::Stroke::new(1.0, colors.border),
            );
        });
}

fn left_items(ui: &mut egui::Ui, colors: &StatusBarColors, bar: &StatusBar) {
    let (dot, text) = match bar.tone {
        Tone::Ok => (colors.ok_dot, colors.message),
        Tone::Error => (colors.error_dot, colors.error_message),
    };
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = DOT_GAP;
        let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(DOT_SIZE), egui::Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), DOT_SIZE / 2.0, dot);
        text_item(ui, bar.message, proportional(), text);
    });
    if let Some(selection) = bar.selection {
        divider(ui, colors);
        text_item(ui, selection, proportional(), colors.selection);
    }
}

/// Laid out right to left, so the renderer comes first.
fn right_items(ui: &mut egui::Ui, colors: &StatusBarColors, bar: &StatusBar) {
    let mut first = true;
    let mut separate = |ui: &mut egui::Ui| {
        if !std::mem::take(&mut first) {
            divider(ui, colors);
        }
    };
    if let Some((renderer, adapter)) = bar.renderer {
        separate(ui);
        let font = egui::FontId::monospace(RENDERER_SIZE);
        let response = text_item(ui, renderer, font, colors.renderer);
        if !adapter.is_empty() {
            response.on_hover_text(adapter);
        }
    }
    if let Some(cursor) = bar.cursor {
        separate(ui);
        text_item(ui, cursor, monospace(), colors.cursor);
    }
    if let Some(grid) = bar.grid {
        separate(ui);
        text_item(ui, grid, monospace(), colors.grid);
    }
}

fn proportional() -> egui::FontId {
    egui::FontId::proportional(TEXT_SIZE)
}

fn monospace() -> egui::FontId {
    egui::FontId::monospace(TEXT_SIZE)
}

/// A line of text, cut short with an ellipsis when the row runs out of room.
fn text_item(
    ui: &mut egui::Ui,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
) -> egui::Response {
    ui.add(
        egui::Label::new(egui::RichText::new(text).font(font).color(color))
            .truncate()
            .selectable(false),
    )
}

fn divider(ui: &mut egui::Ui, colors: &StatusBarColors) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, DIVIDER_HEIGHT), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, colors.divider);
}
