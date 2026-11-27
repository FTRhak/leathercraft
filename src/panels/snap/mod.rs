//! The snap controls floating in the top right corner of the canvas, from the "Snap Controls"
//! and "Snap Popover" of the Brilliant design: the magnet turns snapping on and off, and the
//! targets button opens a popover with the targets (vertices, smart guides, grid), the grid
//! step, the snap distance and two hints.
//!
//! The settings are interface state, saved with the app settings by the caller. The bar is
//! added on top of the canvas after the canvas has allocated its own response, so its buttons,
//! and the bar behind them, take the clicks over the canvas; the popover is a layer of its own.

use crate::ui_components::input::NumberInput;
use crate::ui_components::palette::{
    Palette, PopoverColors, SnapColors, TargetRowPaint, ViewButtonPaint,
};
use bevy_egui::egui;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Units;
use leathercraft::snap::{
    MAX_DISTANCE_PX, MAX_GRID_STEP_MM, MIN_DISTANCE_PX, MIN_GRID_STEP_MM, SnapSettings,
};

/// The distance from the canvas's top and right edges.
const MARGIN: f32 = 16.0;
const BAR_PADDING: f32 = 3.0;
const BAR_GAP: f32 = 2.0;
const BAR_RADIUS: u8 = 7;
const BUTTON_RADIUS: u8 = 5;
const BUTTON_HEIGHT: f32 = 26.0;
const TOGGLE_WIDTH: f32 = 28.0;
const TARGETS_PADDING_X: f32 = 6.0;
const TARGETS_GAP: f32 = 3.0;
const ICON_SIZE: f32 = 14.0;
const CARET_SIZE: egui::Vec2 = egui::vec2(9.0, 5.0);

const POPOVER_WIDTH: f32 = 260.0;
const POPOVER_PADDING: i8 = 10;
const POPOVER_GAP: f32 = 6.0;
/// Between the sections of the popover.
const SECTION_GAP: f32 = 10.0;
/// Between a caption and what it names.
const CAPTION_GAP: f32 = 6.0;
const TITLE_SIZE: f32 = 12.0;
const TEXT_SIZE: f32 = 11.0;
const HINT_SIZE: f32 = 10.0;
const KEYCAP_PADDING: egui::Vec2 = egui::vec2(6.0, 2.0);
const KEYCAP_RADIUS: u8 = 4;
const ROW_HEIGHT: f32 = 26.0;
const ROW_PADDING_X: f32 = 8.0;
const ROW_GAP: f32 = 8.0;
const ROW_RADIUS: u8 = 4;
const LIST_PADDING: f32 = 1.0;
const CHECK_SIZE: egui::Vec2 = egui::vec2(12.0, 9.0);
const HINT_ICON_SIZE: f32 = 12.0;
const HINT_GAP: f32 = 6.0;

/// The icon of a button or a row, painted in a 14 px box.
#[derive(Clone, Copy)]
enum Icon {
    Magnet,
    Vertices,
    Guides,
    Grid,
    Info,
    Keyboard,
}

/// Shows the snap controls in the top right corner of `canvas`. `hotkey` is the shortcut that
/// turns snapping on and off, as shown to the user.
pub fn show(
    ui: &mut egui::Ui,
    canvas: egui::Rect,
    snap: &mut SnapSettings,
    hotkey: Option<&str>,
    language: Language,
) {
    let colors = &Palette::of(ui.painter()).snap;
    let targets_width = 2.0 * TARGETS_PADDING_X + ICON_SIZE + TARGETS_GAP + CARET_SIZE.x;
    let size = egui::vec2(
        TOGGLE_WIDTH + BAR_GAP + targets_width + 2.0 * BAR_PADDING,
        BUTTON_HEIGHT + 2.0 * BAR_PADDING,
    );
    let bar = egui::Rect::from_min_size(
        canvas.right_top() + egui::vec2(-MARGIN - size.x, MARGIN),
        size,
    );
    // A canvas too small to hold the bar goes without it.
    if bar.left() < canvas.left() || bar.bottom() > canvas.bottom() {
        return;
    }

    let id = ui.id().with("snap_controls");
    // Takes the clicks and the hover between the buttons, so they don't reach the canvas.
    ui.interact(bar, id, egui::Sense::click());
    let shadow = egui::Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: colors.bar_shadow,
    };
    ui.painter().add(shadow.as_shape(bar, BAR_RADIUS));
    ui.painter().rect(
        bar,
        BAR_RADIUS,
        colors.bar_background,
        egui::Stroke::new(1.0, colors.bar_border),
        egui::StrokeKind::Inside,
    );

    let top = bar.top() + BAR_PADDING;
    let toggle = egui::Rect::from_min_size(
        egui::pos2(bar.left() + BAR_PADDING, top),
        egui::vec2(TOGGLE_WIDTH, BUTTON_HEIGHT),
    );
    let targets = egui::Rect::from_min_size(
        egui::pos2(toggle.right() + BAR_GAP, top),
        egui::vec2(targets_width, BUTTON_HEIGHT),
    );

    let toggle_name = match hotkey {
        Some(hotkey) => format!("{} ({hotkey})", language.label(Label::ToggleSnap)),
        None => language.label(Label::ToggleSnap).to_owned(),
    };
    let response = button(
        ui,
        toggle,
        id.with("toggle"),
        &toggle_name,
        Some(snap.enabled),
    );
    if response.clicked() {
        snap.enabled = !snap.enabled;
    }
    let style = match (snap.enabled, hovered(&response)) {
        (true, false) => &colors.toggle_on,
        (true, true) => &colors.toggle_on_hover,
        (false, false) => &colors.toggle_off,
        (false, true) => &colors.toggle_off_hover,
    };
    paint_button(ui.painter(), toggle, style, &response, colors);
    let icon = egui::Rect::from_center_size(toggle.center(), egui::Vec2::splat(ICON_SIZE));
    paint_icon(ui.painter(), icon, Icon::Magnet, style.content);

    let targets_name = language.label(Label::SnapTargets);
    let response = button(ui, targets, id.with("targets"), targets_name, None);
    let popup_id = id.with("popover");
    let open = egui::Popup::is_id_open(ui.ctx(), popup_id);
    let style = if open {
        &colors.targets_open
    } else if hovered(&response) {
        &colors.targets_hover
    } else {
        &colors.targets
    };
    paint_button(ui.painter(), targets, style, &response, colors);
    let icon = egui::Rect::from_min_size(
        egui::pos2(
            targets.left() + TARGETS_PADDING_X,
            targets.center().y - ICON_SIZE / 2.0,
        ),
        egui::Vec2::splat(ICON_SIZE),
    );
    paint_icon(ui.painter(), icon, Icon::Grid, style.content);
    let caret = egui::Rect::from_min_size(
        egui::pos2(
            icon.right() + TARGETS_GAP,
            targets.center().y - CARET_SIZE.y / 2.0,
        ),
        CARET_SIZE,
    );
    paint_caret(ui.painter(), caret, colors.caret);

    let popover = &colors.popover;
    let frame = egui::Frame::new()
        .fill(popover.background)
        .stroke(egui::Stroke::new(1.0, popover.border))
        .corner_radius(BAR_RADIUS)
        .inner_margin(POPOVER_PADDING)
        .shadow(egui::Shadow {
            offset: [0, 8],
            blur: 20,
            spread: 0,
            color: popover.shadow,
        });
    egui::Popup::from_toggle_button_response(&response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .align(egui::RectAlign::BOTTOM_END)
        .gap(POPOVER_GAP + BAR_PADDING)
        .width(POPOVER_WIDTH)
        .frame(frame)
        .show(|ui| show_popover(ui, snap, hotkey, language, popover));
}

/// The content of the popover: the header, the targets, the two numbers and the hints.
fn show_popover(
    ui: &mut egui::Ui,
    snap: &mut SnapSettings,
    hotkey: Option<&str>,
    language: Language,
    colors: &PopoverColors,
) {
    let width = POPOVER_WIDTH - 2.0 * f32::from(POPOVER_PADDING);
    ui.set_width(width);
    ui.spacing_mut().item_spacing = egui::vec2(0.0, SECTION_GAP);

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(language.label(Label::Snap))
                .size(TITLE_SIZE)
                .strong()
                .color(colors.title),
        );
        if let Some(hotkey) = hotkey {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                keycap(ui, hotkey, colors);
            });
        }
    });

    section(ui, language.label(Label::SnapTargets), colors, |ui| {
        target_list(ui, snap, language, colors);
    });

    section(ui, language.label(Label::SnapGridStep), colors, |ui| {
        let field = NumberInput::new(&mut snap.grid_step_mm)
            .range(MIN_GRID_STEP_MM..=MAX_GRID_STEP_MM)
            .step(0.1)
            .decimals(1)
            .unit(language.unit_symbol(Units::Millimeters))
            .width(width)
            .id_salt("snap_grid_step");
        ui.add_enabled(snap.grid, field);
    });

    section(ui, language.label(Label::SnapDistance), colors, |ui| {
        let field = NumberInput::new(&mut snap.distance_px)
            .range(MIN_DISTANCE_PX..=MAX_DISTANCE_PX)
            .step(1.0)
            .decimals(0)
            .width(width)
            .id_salt("snap_distance");
        // The distance applies to vertices and guides; the grid always snaps.
        ui.add_enabled(snap.vertices || snap.guides, field);
    });

    let (_, divider) = ui.allocate_space(egui::vec2(width, 1.0));
    ui.painter().rect_filled(divider, 0.0, colors.divider);

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = CAPTION_GAP;
        hint(
            ui,
            Icon::Info,
            language.label(Label::SnapVerticesHint),
            colors,
        );
        hint(
            ui,
            Icon::Keyboard,
            language.label(Label::SnapAltHint),
            colors,
        );
    });
}

/// A caption over its content.
fn section(
    ui: &mut egui::Ui,
    caption: &str,
    colors: &PopoverColors,
    content: impl FnOnce(&mut egui::Ui),
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = CAPTION_GAP;
        ui.label(
            egui::RichText::new(caption)
                .size(TEXT_SIZE)
                .color(colors.caption),
        );
        content(ui);
    });
}

/// The hotkey in the header, in a small key frame.
fn keycap(ui: &mut egui::Ui, text: &str, colors: &PopoverColors) {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::monospace(HINT_SIZE),
        colors.keycap_text,
    );
    let size = galley.size() + 2.0 * KEYCAP_PADDING;
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect(
        rect,
        KEYCAP_RADIUS,
        colors.keycap_fill,
        egui::Stroke::new(1.0, colors.keycap_border),
        egui::StrokeKind::Inside,
    );
    ui.painter()
        .galley(rect.min + KEYCAP_PADDING, galley, colors.keycap_text);
}

/// The three targets, each a row that toggles on a click.
fn target_list(
    ui: &mut egui::Ui,
    snap: &mut SnapSettings,
    language: Language,
    colors: &PopoverColors,
) {
    let rows = [
        (
            Icon::Vertices,
            Label::SnapVertices,
            Some(Label::SnapVerticesHint),
            &mut snap.vertices,
        ),
        (
            Icon::Guides,
            Label::SnapGuides,
            Some(Label::SnapGuidesHint),
            &mut snap.guides,
        ),
        (Icon::Grid, Label::SnapGrid, None, &mut snap.grid),
    ];
    let width = ui.available_width();
    let height = rows.len() as f32 * (ROW_HEIGHT + LIST_PADDING) + LIST_PADDING;
    let (list, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().rect(
        list,
        BUTTON_RADIUS,
        colors.list_fill,
        egui::Stroke::new(1.0, colors.list_border),
        egui::StrokeKind::Inside,
    );
    let mut top = list.top() + LIST_PADDING;
    for (icon, label, tooltip, on) in rows {
        let rect = egui::Rect::from_min_size(
            egui::pos2(list.left() + LIST_PADDING, top),
            egui::vec2(width - 2.0 * LIST_PADDING, ROW_HEIGHT),
        );
        target_row(ui, rect, icon, language.label(label), on, colors)
            .on_hover_text_at_pointer(tooltip.map_or("", |tooltip| language.label(tooltip)));
        top += ROW_HEIGHT + LIST_PADDING;
    }
}

/// One target: its icon, its name and a check mark while it is on.
fn target_row(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    icon: Icon,
    text: &str,
    on: &mut bool,
    colors: &PopoverColors,
) -> egui::Response {
    let id = ui.id().with(("snap_target", text));
    let mut response = ui.interact(rect, id, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let checked = *on;
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, checked, text)
    });
    let style: &TargetRowPaint = match (checked, hovered(&response)) {
        (true, false) => &colors.row_on,
        (true, true) => &colors.row_on_hover,
        (false, false) => &colors.row_off,
        (false, true) => &colors.row_off_hover,
    };
    let painter = ui.painter();
    if let Some(fill) = style.fill {
        painter.rect_filled(rect, ROW_RADIUS, fill);
    }
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(
            rect.left() + ROW_PADDING_X,
            rect.center().y - ICON_SIZE / 2.0,
        ),
        egui::Vec2::splat(ICON_SIZE),
    );
    paint_icon(painter, icon_rect, icon, style.icon);
    let galley = painter.layout(
        text.to_owned(),
        egui::FontId::proportional(TEXT_SIZE),
        style.text,
        rect.width() - 2.0 * (ROW_PADDING_X + ICON_SIZE + ROW_GAP),
    );
    painter.galley(
        egui::pos2(
            icon_rect.right() + ROW_GAP,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        style.text,
    );
    if checked {
        let check = egui::Rect::from_min_size(
            egui::pos2(
                rect.right() - ROW_PADDING_X - CHECK_SIZE.x,
                rect.center().y - CHECK_SIZE.y / 2.0,
            ),
            CHECK_SIZE,
        );
        let at = |x: f32, y: f32| check.min + egui::vec2(x, y);
        painter.line(
            vec![at(0.5, 4.5), at(4.2, 8.2), at(11.5, 0.8)],
            egui::Stroke::new(1.3, style.icon),
        );
    }
    if response.has_focus() {
        painter.rect_stroke(
            rect,
            ROW_RADIUS,
            egui::Stroke::new(1.0, style.text),
            egui::StrokeKind::Inside,
        );
    }
    response
}

/// A hint at the bottom of the popover: an icon and text that wraps.
fn hint(ui: &mut egui::Ui, icon: Icon, text: &str, colors: &PopoverColors) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = HINT_GAP;
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(HINT_ICON_SIZE, HINT_SIZE * 1.2),
            egui::Sense::hover(),
        );
        let icon_rect =
            egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(HINT_ICON_SIZE));
        paint_icon(ui.painter(), icon_rect, icon, colors.hint);
        ui.add(
            egui::Label::new(egui::RichText::new(text).size(HINT_SIZE).color(colors.hint)).wrap(),
        );
    });
}

/// The response of a bar button, with its name for screen readers and as its tooltip.
/// `selected` marks a toggle.
fn button(
    ui: &egui::Ui,
    rect: egui::Rect,
    id: egui::Id,
    name: &str,
    selected: Option<bool>,
) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    response.widget_info(|| match selected {
        Some(selected) => {
            egui::WidgetInfo::selected(egui::WidgetType::Button, true, selected, name)
        }
        None => egui::WidgetInfo::labeled(egui::WidgetType::Button, true, name),
    });
    response.on_hover_text(name)
}

fn hovered(response: &egui::Response) -> bool {
    response.hovered() || response.is_pointer_button_down_on()
}

fn paint_button(
    painter: &egui::Painter,
    rect: egui::Rect,
    style: &ViewButtonPaint,
    response: &egui::Response,
    colors: &SnapColors,
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

/// A caret pointing down, as wide and tall as `rect`.
fn paint_caret(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.2, color);
    let rect = rect.shrink(stroke.width / 2.0);
    painter.line(
        vec![rect.left_top(), rect.center_bottom(), rect.right_top()],
        stroke,
    );
}

/// Paints an icon in a square box, after the icons of the design, drawn on a 14 px grid.
fn paint_icon(painter: &egui::Painter, rect: egui::Rect, icon: Icon, color: egui::Color32) {
    let scale = rect.width() / ICON_SIZE;
    let at = |x: f32, y: f32| rect.min + egui::vec2(x, y) * scale;
    let stroke = egui::Stroke::new(1.2, color);
    match icon {
        Icon::Magnet => {
            // A horseshoe opening to the top right: two arcs joined by the ends of its arms,
            // with a band across each pole.
            let center = egui::vec2(7.0, 7.0);
            let (angle_sin, angle_cos) = std::f32::consts::FRAC_PI_4.sin_cos();
            let turn = |x: f32, y: f32| {
                let v = egui::vec2(x, y) - center;
                let turned = egui::vec2(
                    v.x * angle_cos - v.y * angle_sin,
                    v.x * angle_sin + v.y * angle_cos,
                );
                at(center.x + turned.x, center.y + turned.y)
            };
            // The arc bottom is at (7, 8.5), and the arms reach up to y = 1.5.
            let (outer, inner, arc_y, tip_y, pole_y) = (5.2, 1.8, 7.8, 1.2, 4.0);
            let arc = |radius: f32| -> Vec<(f32, f32)> {
                (0..=12)
                    .map(|i| {
                        let a = std::f32::consts::PI * i as f32 / 12.0;
                        (7.0 - radius * a.cos(), arc_y + radius * a.sin())
                    })
                    .collect()
            };
            let mut outline = vec![(7.0 - inner, tip_y), (7.0 - outer, tip_y)];
            outline.extend(arc(outer));
            outline.push((7.0 + outer, tip_y));
            outline.push((7.0 + inner, tip_y));
            outline.extend(arc(inner).into_iter().rev());
            let points: Vec<_> = outline.into_iter().map(|(x, y)| turn(x, y)).collect();
            painter.add(egui::Shape::closed_line(points, stroke));
            for side in [-1.0, 1.0] {
                painter.line_segment(
                    [
                        turn(7.0 + side * inner, pole_y),
                        turn(7.0 + side * outer, pole_y),
                    ],
                    stroke,
                );
            }
        }
        Icon::Vertices => {
            let corners = [
                (2.5, 6.0),
                (6.5, 1.8),
                (11.8, 4.0),
                (10.5, 11.0),
                (4.0, 12.0),
            ];
            let points: Vec<_> = corners.iter().map(|&(x, y)| at(x, y)).collect();
            painter.add(egui::Shape::closed_line(points, stroke));
            for (x, y) in [corners[0], corners[2], corners[3]] {
                painter.circle(at(x, y), 1.6 * scale, color, egui::Stroke::NONE);
            }
        }
        Icon::Guides => {
            painter.line_segment([at(7.0, 0.5), at(7.0, 13.5)], stroke);
            for (left, right) in [(1.5, 4.8), (9.2, 12.5)] {
                let rect = egui::Rect::from_min_max(at(left, 3.0), at(right, 11.0));
                painter.rect_stroke(rect, 1.0, stroke, egui::StrokeKind::Middle);
            }
        }
        Icon::Grid => {
            let frame = egui::Rect::from_min_max(at(1.0, 1.0), at(13.0, 13.0));
            painter.rect_stroke(frame, 1.5, stroke, egui::StrokeKind::Middle);
            painter.line_segment([at(7.0, 1.0), at(7.0, 13.0)], stroke);
            painter.line_segment([at(1.0, 7.0), at(13.0, 7.0)], stroke);
        }
        Icon::Info => {
            painter.circle_stroke(at(7.0, 7.0), 6.0 * scale, stroke);
            painter.circle_filled(at(7.0, 4.2), 0.9 * scale, color);
            painter.line_segment([at(7.0, 6.3), at(7.0, 10.3)], stroke);
        }
        Icon::Keyboard => {
            let frame = egui::Rect::from_min_max(at(0.8, 3.0), at(13.2, 11.0));
            painter.rect_stroke(frame, 1.5, stroke, egui::StrokeKind::Middle);
            for y in [5.4, 7.4] {
                for x in [3.4, 5.6, 7.8, 10.0] {
                    painter.circle_filled(at(x + 0.3, y), 0.6 * scale, color);
                }
            }
            painter.line_segment([at(4.5, 9.3), at(9.5, 9.3)], stroke);
        }
    }
}
