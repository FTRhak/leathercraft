//! The pieces that make up a row of the Layout tree: its frame, its painted icons, its square
//! buttons and the "Base" badge. Sizes follow the "Layout Tree Pane" of the Brilliant design;
//! every color comes from `Palette::layout`.

use bevy_egui::egui;

use crate::ui_components::icons::paint_glyph;
use crate::ui_components::palette::Palette;

pub(super) const ROW_HEIGHT: f32 = 28.0;
pub(super) const ROW_RADIUS: u8 = 4;
/// Between the icons, names and buttons of a row.
pub(super) const GAP: f32 = 6.0;
pub(super) const PIECE_BUTTON: egui::Vec2 = egui::vec2(22.0, 22.0);
pub(super) const SHAPE_BUTTON: egui::Vec2 = egui::vec2(20.0, 22.0);
const BUTTON_RADIUS: u8 = 4;
const BADGE_HEIGHT: f32 = 20.0;
const BADGE_PADDING_X: f32 = 6.0;
const BADGE_TEXT_SIZE: f32 = 10.0;
const CARET_SIZE: egui::Vec2 = egui::vec2(11.0, 6.0);
const CARET_STROKE: f32 = 1.5;
/// The box of a glyph; `paint_glyph` draws it at 75%, about 13 px as in the design.
pub(super) const GLYPH_BOX: f32 = 17.0;

/// A full-width row of `height`, clickable as a whole. Buttons added later on top of it win
/// the click.
pub(super) fn row(ui: &mut egui::Ui, height: f32) -> (egui::Rect, egui::Response) {
    ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click(),
    )
}

/// Lays out `add` right to left inside `rect`, and returns where the leftmost widget ends, so
/// that the name before it can be cut to fit.
pub(super) fn trailing<R>(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    spacing: f32,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> (R, f32) {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    child.spacing_mut().item_spacing.x = spacing;
    let inner = add(&mut child);
    let left = if child.min_rect().width() > 0.0 {
        child.min_rect().left()
    } else {
        rect.right()
    };
    (inner, left)
}

/// A clickable caret that opens or closes a branch of the tree, painted pointing down when
/// `open` and right otherwise. Returns whether it was clicked.
pub(super) fn toggle(
    ui: &mut egui::Ui,
    id: egui::Id,
    center: egui::Pos2,
    open: bool,
    color: egui::Color32,
) -> bool {
    let rect = egui::Rect::from_center_size(center, egui::Vec2::splat(CARET_SIZE.x + 4.0));
    let response = ui.interact(rect, id, egui::Sense::click());
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), ""));
    paint_caret(ui.painter(), center, open, color);
    response.clicked()
}

/// A caret centered on `center`, pointing down when `down` and right otherwise.
pub(super) fn paint_caret(
    painter: &egui::Painter,
    center: egui::Pos2,
    down: bool,
    color: egui::Color32,
) {
    let (w, h) = (CARET_SIZE.x / 2.0, CARET_SIZE.y / 2.0);
    let points = if down {
        vec![
            center + egui::vec2(-w, -h),
            center + egui::vec2(0.0, h),
            center + egui::vec2(w, -h),
        ]
    } else {
        vec![
            center + egui::vec2(-h, -w),
            center + egui::vec2(h, 0.0),
            center + egui::vec2(-h, w),
        ]
    };
    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(CARET_STROKE, color),
    ));
}

/// An open folder, the icon of the root row, in a box of 14 × 11 around `center`.
pub(super) fn paint_folder(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let at = |x: f32, y: f32| center + egui::vec2(x, y);
    let stroke = egui::Stroke::new(1.2, color);
    // The back of the folder, with its tab.
    painter.add(egui::Shape::line(
        vec![
            at(-5.0, 1.0),
            at(-5.0, -5.0),
            at(-1.5, -5.0),
            at(0.0, -3.5),
            at(4.0, -3.5),
            at(4.0, -1.5),
        ],
        stroke,
    ));
    // Its open front.
    painter.add(egui::Shape::closed_line(
        vec![at(-5.0, 5.0), at(-3.0, -1.5), at(7.0, -1.5), at(5.0, 5.0)],
        stroke,
    ));
}

/// What a row button shows.
#[derive(Clone, Copy)]
pub(super) enum Icon {
    Glyph(char),
    /// A chevron pointing up or down, for moving a shape earlier or later.
    Chevron {
        up: bool,
    },
}

/// How a row button is drawn when it is not under the pointer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ButtonLook {
    /// A chip with a border, on a selected row.
    Framed,
    /// A faint chip, on other rows.
    Quiet,
}

/// A small square button at the end of a row. `active` marks a toggle that is on, such as the
/// lock of a locked piece. Disable it the egui way, with `ui.add_enabled_ui`.
pub(super) fn button(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    look: ButtonLook,
    active: bool,
    name: &str,
    icon: Icon,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), active, name)
    });
    if ui.is_rect_visible(rect) {
        let colors = &Palette::of(ui.painter()).layout.button;
        let enabled = ui.is_enabled();
        let hovered = enabled && (response.hovered() || response.is_pointer_button_down_on());
        let (fill, stroke) = if hovered {
            (Some(colors.hover.fill), Some(colors.hover.stroke))
        } else {
            match look {
                ButtonLook::Framed => (Some(colors.framed.fill), Some(colors.framed.stroke)),
                ButtonLook::Quiet if enabled => (Some(colors.quiet_fill), None),
                ButtonLook::Quiet => (None, None),
            }
        };
        let icon_color = if !enabled {
            colors.icon_disabled
        } else if hovered {
            colors.icon_hover
        } else if active {
            colors.icon_active
        } else if look == ButtonLook::Framed {
            colors.icon
        } else {
            colors.icon_quiet
        };
        let painter = ui.painter();
        if let Some(fill) = fill {
            painter.rect_filled(rect, BUTTON_RADIUS, fill);
        }
        if let Some(stroke) = stroke {
            painter.rect_stroke(
                rect,
                BUTTON_RADIUS,
                egui::Stroke::new(1.0, stroke),
                egui::StrokeKind::Inside,
            );
        }
        if response.has_focus() {
            painter.rect_stroke(
                rect,
                BUTTON_RADIUS,
                egui::Stroke::new(1.0, Palette::of(painter).accent.base),
                egui::StrokeKind::Inside,
            );
        }
        match icon {
            Icon::Glyph(glyph) => {
                let glyph_rect = egui::Rect::from_center_size(
                    rect.center(),
                    egui::Vec2::splat(size.x.min(size.y) - 4.0),
                );
                paint_glyph(
                    painter,
                    glyph_rect,
                    egui::Stroke::new(1.0, icon_color),
                    glyph,
                );
            }
            Icon::Chevron { up } => {
                let (w, h) = (CARET_SIZE.x * 0.45, CARET_SIZE.y * 0.45);
                let c = rect.center();
                let sign = if up { 1.0 } else { -1.0 };
                painter.add(egui::Shape::line(
                    vec![
                        c + egui::vec2(-w, sign * h),
                        c + egui::vec2(0.0, -sign * h),
                        c + egui::vec2(w, sign * h),
                    ],
                    egui::Stroke::new(CARET_STROKE, icon_color),
                ));
            }
        }
    }
    response.on_hover_text(name)
}

/// The "Base" badge on the row of a piece's base shape.
pub(super) fn badge(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(BADGE_TEXT_SIZE),
        egui::Color32::PLACEHOLDER,
    );
    let size = egui::vec2(galley.size().x + 2.0 * BADGE_PADDING_X, BADGE_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let colors = &Palette::of(ui.painter()).layout;
        ui.painter().rect(
            rect,
            BUTTON_RADIUS,
            colors.badge.fill,
            egui::Stroke::new(1.0, colors.badge.stroke),
            egui::StrokeKind::Inside,
        );
        let pos = rect.center() - galley.size() / 2.0;
        ui.painter().galley(pos, galley, colors.badge_text);
    }
    response
}

/// A galley of `text` at `size`, cut short with an ellipsis to `max_width`.
pub(super) fn text(
    ui: &egui::Ui,
    text: &str,
    font: egui::FontId,
    max_width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat::simple(font, egui::Color32::PLACEHOLDER),
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(max_width.max(0.0));
    ui.painter().layout_job(job)
}
