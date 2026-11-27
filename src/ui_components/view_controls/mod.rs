//! The zoom panel floating in the bottom right corner of the canvas, from the "View Controls"
//! of the Brilliant design: Zoom out, the zoom percentage (a click sets the actual size), Zoom
//! in, a divider, Zoom to fit, and the accent Zoom to selection button with its label.
//!
//! ```ignore
//! let command = view_controls::show(ui, canvas_rect, &ViewControls {
//!     zoom_percent: 125.0,
//!     can_fit_selection: true,
//!     zoom_out: "Zoom out",
//!     zoom_in: "Zoom in",
//!     actual_size: "Actual size (100%)",
//!     fit_all: "Zoom to fit",
//!     fit_selection: "To selection",
//!     fit_selection_tooltip: "Zoom to selection",
//! });
//! ```
//!
//! It is added on top of the canvas, after the canvas has allocated its own response, so its
//! buttons, and the panel behind them, take the clicks over the canvas.

use bevy_egui::egui;

use crate::ui_components::palette::{Palette, ViewButtonPaint, ViewControlsColors};

/// The distance from the canvas's right and bottom edges.
const MARGIN: f32 = 16.0;
const PADDING: f32 = 3.0;
const GAP: f32 = 2.0;
const PANEL_RADIUS: u8 = 7;
const BUTTON_RADIUS: u8 = 5;
const BUTTON_HEIGHT: f32 = 26.0;
const ICON_BUTTON_WIDTH: f32 = 28.0;
const ICON_SIZE: f32 = 14.0;
const VALUE_PADDING_X: f32 = 6.0;
const PRIMARY_PADDING_X: f32 = 8.0;
const PRIMARY_GAP: f32 = 5.0;
const DIVIDER_HEIGHT: f32 = 16.0;
const TEXT_SIZE: f32 = 11.0;

/// What a click on the view controls asks the canvas to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewCommand {
    ZoomOut,
    ZoomIn,
    /// Zoom to 100%, the actual size.
    ActualSize,
    FitAll,
    FitSelection,
}

/// The state and the text of the view controls, already translated.
pub struct ViewControls<'a> {
    /// The zoom, where 100% is the actual size.
    pub zoom_percent: f32,
    /// Whether anything is selected; Zoom to selection is disabled otherwise.
    pub can_fit_selection: bool,
    pub zoom_out: &'a str,
    pub zoom_in: &'a str,
    /// The tooltip of the percentage.
    pub actual_size: &'a str,
    pub fit_all: &'a str,
    /// The label of the Zoom to selection button, and its tooltip.
    pub fit_selection: &'a str,
    pub fit_selection_tooltip: &'a str,
}

/// The icon of a button, painted in a 14 px box.
#[derive(Clone, Copy)]
enum Icon {
    ZoomOut,
    ZoomIn,
    FitAll,
    FitSelection,
}

/// Shows the view controls in the bottom right corner of `canvas` and returns the clicked command.
pub fn show(ui: &mut egui::Ui, canvas: egui::Rect, controls: &ViewControls) -> Option<ViewCommand> {
    let colors = &Palette::of(ui.painter()).view_controls;
    let value = ui.painter().layout_no_wrap(
        format!("{:.0}%", controls.zoom_percent),
        egui::FontId::monospace(TEXT_SIZE),
        colors.value,
    );
    let label = ui.painter().layout_no_wrap(
        controls.fit_selection.to_owned(),
        egui::FontId::proportional(TEXT_SIZE),
        egui::Color32::PLACEHOLDER,
    );
    let value_width = value.size().x + 2.0 * VALUE_PADDING_X;
    let primary_width = 2.0 * PRIMARY_PADDING_X + ICON_SIZE + PRIMARY_GAP + label.size().x;
    let widths = [
        ICON_BUTTON_WIDTH,
        value_width,
        ICON_BUTTON_WIDTH,
        1.0,
        ICON_BUTTON_WIDTH,
        primary_width,
    ];
    let content_width = widths.iter().sum::<f32>() + GAP * (widths.len() - 1) as f32;
    let size = egui::vec2(
        content_width + 2.0 * PADDING,
        BUTTON_HEIGHT + 2.0 * PADDING,
    );
    let panel = egui::Rect::from_min_size(canvas.right_bottom() - size - egui::Vec2::splat(MARGIN), size);
    // A canvas too small to hold the panel goes without it.
    if panel.left() < canvas.left() || panel.top() < canvas.top() {
        return None;
    }

    let id = ui.id().with("view_controls");
    // Takes the clicks and the hover between the buttons, so they don't reach the canvas.
    ui.interact(panel, id, egui::Sense::click());

    let painter = ui.painter();
    let shadow = egui::Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: colors.shadow,
    };
    painter.add(shadow.as_shape(panel, PANEL_RADIUS));
    painter.rect(
        panel,
        PANEL_RADIUS,
        colors.background,
        egui::Stroke::new(1.0, colors.border),
        egui::StrokeKind::Inside,
    );

    let mut rects = Vec::with_capacity(widths.len());
    let mut x = panel.left() + PADDING;
    for width in widths {
        rects.push(egui::Rect::from_min_size(
            egui::pos2(x, panel.top() + PADDING),
            egui::vec2(width, BUTTON_HEIGHT),
        ));
        x += width + GAP;
    }

    let mut command = None;
    let mut click = |clicked: bool, which: ViewCommand| {
        if clicked {
            command = Some(which);
        }
    };
    let zoom_out = icon_button(ui, rects[0], id.with(0), Icon::ZoomOut, controls.zoom_out, colors);
    click(zoom_out, ViewCommand::ZoomOut);

    let response = button(ui, rects[1], id.with(1), true, controls.actual_size);
    paint_quiet(ui, rects[1], &response, colors);
    ui.painter().galley(
        rects[1].center() - value.size() / 2.0,
        value,
        colors.value,
    );
    click(response.clicked(), ViewCommand::ActualSize);

    let zoom_in = icon_button(ui, rects[2], id.with(2), Icon::ZoomIn, controls.zoom_in, colors);
    click(zoom_in, ViewCommand::ZoomIn);

    let divider = egui::Rect::from_center_size(rects[3].center(), egui::vec2(1.0, DIVIDER_HEIGHT));
    ui.painter().rect_filled(divider, 0.0, colors.divider);

    let fit_all = icon_button(ui, rects[4], id.with(4), Icon::FitAll, controls.fit_all, colors);
    click(fit_all, ViewCommand::FitAll);

    let enabled = controls.can_fit_selection;
    let response = button(ui, rects[5], id.with(5), enabled, controls.fit_selection_tooltip);
    let style = if !enabled {
        &colors.disabled
    } else if hovered(&response) {
        &colors.primary_hover
    } else {
        &colors.primary
    };
    paint_frame(ui.painter(), rects[5], style, &response, colors);
    let icon = egui::Rect::from_min_size(
        egui::pos2(
            rects[5].left() + PRIMARY_PADDING_X,
            rects[5].center().y - ICON_SIZE / 2.0,
        ),
        egui::Vec2::splat(ICON_SIZE),
    );
    paint_icon(ui.painter(), icon, Icon::FitSelection, style.content);
    ui.painter().galley(
        egui::pos2(
            icon.right() + PRIMARY_GAP,
            rects[5].center().y - label.size().y / 2.0,
        ),
        label,
        style.content,
    );
    click(response.clicked(), ViewCommand::FitSelection);

    command
}

/// The response of one button, with its name for screen readers and as its tooltip.
fn button(
    ui: &egui::Ui,
    rect: egui::Rect,
    id: egui::Id,
    enabled: bool,
    name: &str,
) -> egui::Response {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let response = ui.interact(rect, id, sense);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, name));
    response.on_hover_text(name)
}

fn hovered(response: &egui::Response) -> bool {
    response.hovered() || response.is_pointer_button_down_on()
}

/// A quiet square button with an icon; returns whether it was clicked.
fn icon_button(
    ui: &egui::Ui,
    rect: egui::Rect,
    id: egui::Id,
    icon: Icon,
    name: &str,
    colors: &ViewControlsColors,
) -> bool {
    let response = button(ui, rect, id, true, name);
    let style = paint_quiet(ui, rect, &response, colors);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(ICON_SIZE));
    paint_icon(ui.painter(), icon_rect, icon, style.content);
    response.clicked()
}

/// Paints a quiet button idle or hovered, and returns its paint.
fn paint_quiet<'c>(
    ui: &egui::Ui,
    rect: egui::Rect,
    response: &egui::Response,
    colors: &'c ViewControlsColors,
) -> &'c ViewButtonPaint {
    let style = if hovered(response) {
        &colors.hover
    } else {
        &colors.idle
    };
    paint_frame(ui.painter(), rect, style, response, colors);
    style
}

fn paint_frame(
    painter: &egui::Painter,
    rect: egui::Rect,
    style: &ViewButtonPaint,
    response: &egui::Response,
    colors: &ViewControlsColors,
) {
    if let Some(fill) = style.fill {
        painter.rect_filled(rect, BUTTON_RADIUS, fill);
    }
    if let Some(stroke) = style.stroke {
        painter.rect_stroke(
            rect,
            BUTTON_RADIUS,
            egui::Stroke::new(1.0, stroke),
            egui::StrokeKind::Inside,
        );
    }
    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
            BUTTON_RADIUS + 2,
            egui::Stroke::new(1.0, colors.focus),
            egui::StrokeKind::Inside,
        );
    }
}

/// Paints an icon in a 14 px box, after the icons of the design.
fn paint_icon(painter: &egui::Painter, rect: egui::Rect, icon: Icon, color: egui::Color32) {
    let at = |x: f32, y: f32| rect.min + egui::vec2(x, y) * (rect.width() / ICON_SIZE);
    let stroke = egui::Stroke::new(1.2, color);
    match icon {
        Icon::ZoomOut | Icon::ZoomIn => {
            painter.circle_stroke(at(5.9, 5.9), 4.5, stroke);
            painter.line_segment([at(9.4, 9.4), at(13.3, 13.3)], egui::Stroke::new(1.5, color));
            painter.line_segment([at(3.8, 5.9), at(8.0, 5.9)], stroke);
            if matches!(icon, Icon::ZoomIn) {
                painter.line_segment([at(5.9, 3.8), at(5.9, 8.0)], stroke);
            }
        }
        Icon::FitAll => {
            let frame = egui::Rect::from_min_max(at(0.6, 1.6), at(13.4, 12.4));
            painter.rect_stroke(frame, 1.5, stroke, egui::StrokeKind::Middle);
            painter.line(vec![at(8.8, 4.0), at(11.0, 4.0), at(11.0, 6.2)], stroke);
            painter.line(vec![at(3.0, 7.8), at(3.0, 10.0), at(5.2, 10.0)], stroke);
        }
        Icon::FitSelection => {
            // A dashed square: an L at each corner and a dash in the middle of each side.
            let (lo, hi, arm) = (0.6, 13.4, 3.0);
            for (x, y, dx, dy) in [
                (lo, lo, arm, arm),
                (hi, lo, -arm, arm),
                (lo, hi, arm, -arm),
                (hi, hi, -arm, -arm),
            ] {
                painter.line(vec![at(x + dx, y), at(x, y), at(x, y + dy)], stroke);
            }
            let (a, b) = (5.4, 8.6);
            for [from, to] in [
                [at(a, lo), at(b, lo)],
                [at(a, hi), at(b, hi)],
                [at(lo, a), at(lo, b)],
                [at(hi, a), at(hi, b)],
            ] {
                painter.line_segment([from, to], stroke);
            }
        }
    }
}
