//! The list of the design system, from the Brilliant `History List` board: full-width rows with
//! an optional icon, a label truncated to fit, and an optional trailing text such as a time or
//! a status. A row is drawn default, hover, selected (the accent fill and border), dimmed
//! (faded, for rows that no longer apply) or disabled, in the colors of the current theme.
//!
//! ```ignore
//! list(ui, |ui| {
//!     for (index, entry) in entries.iter().enumerate() {
//!         let item = ListItem::new(&entry.name)
//!             .icon(glyph::NEW_PIECE)
//!             .trailing(&entry.time)
//!             .selected(index == current);
//!         if ui.add(item).clicked() { .. }
//!     }
//! });
//! ```
//!
//! Disable a row the egui way, with `ui.add_enabled(false, ListItem::new(..))`.

mod style;

use bevy_egui::egui;
use style::{ItemState, paint};

use crate::ui_components::icons::paint_glyph;
use crate::ui_components::palette::Palette;

const ROW_HEIGHT: f32 = 28.0;
/// Between two rows.
const ROW_GAP: f32 = 3.0;
const PADDING_X: f32 = 8.0;
/// The glyph box; `paint_glyph` draws the glyph at 75% of it, about 13 px as in the design.
const ICON_SIZE: f32 = 17.0;
/// Between the icon and the label, and between the label and the trailing text.
const GAP: f32 = 8.0;
const LABEL_SIZE: f32 = 12.0;
const TRAILING_SIZE: f32 = 10.0;
const CORNER_RADIUS: u8 = 4;
/// The opacity of a dimmed row's icon and text.
const DIMMED_OPACITY: f32 = 0.45;
const FOCUS_STROKE: f32 = 1.0;

/// One row, added with `ui.add(...)` inside [`list`]. It is as wide as the `ui`.
#[must_use = "add the row with `ui.add(item)`"]
pub struct ListItem {
    text: egui::WidgetText,
    icon: Option<char>,
    trailing: Option<egui::WidgetText>,
    monospace_trailing: bool,
    selected: bool,
    dimmed: bool,
    strikethrough: bool,
}

impl ListItem {
    pub fn new(text: impl Into<egui::WidgetText>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            trailing: None,
            monospace_trailing: false,
            selected: false,
            dimmed: false,
            strikethrough: false,
        }
    }

    /// Draws a glyph of the icon font (`icons::glyph`) before the label.
    // The History panel picks its icons per entry with `icon_opt`.
    #[allow(dead_code)]
    pub fn icon(mut self, glyph: char) -> Self {
        self.icon = Some(glyph);
        self
    }

    /// Like [`ListItem::icon`], drawing nothing for `None`; the label still lines up with rows
    /// that have one.
    pub fn icon_opt(mut self, glyph: Option<char>) -> Self {
        self.icon = glyph;
        self
    }

    /// A short text at the end of the row, such as a status.
    pub fn trailing(mut self, text: impl Into<egui::WidgetText>) -> Self {
        self.trailing = Some(text.into());
        self.monospace_trailing = false;
        self
    }

    /// A trailing text in the monospaced font, such as a time.
    #[allow(dead_code)]
    pub fn trailing_monospace(mut self, text: impl Into<egui::WidgetText>) -> Self {
        self.trailing = Some(text.into());
        self.monospace_trailing = true;
        self
    }

    /// Whether this is the current row.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Fades a row that no longer applies, such as an undone history entry. A selected row is
    /// never dimmed.
    pub fn dimmed(mut self, dimmed: bool) -> Self {
        self.dimmed = dimmed;
        self
    }

    /// Strikes the label through.
    pub fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.strikethrough = strikethrough;
        self
    }
}

impl egui::Widget for ListItem {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let width = ui.available_width();
        let trailing = self.trailing.map(|text| {
            let family = if self.monospace_trailing {
                egui::FontFamily::Monospace
            } else {
                egui::FontFamily::Proportional
            };
            text.into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                egui::FontSelection::FontId(egui::FontId::new(TRAILING_SIZE, family)),
            )
        });
        let icon_width = if self.icon.is_some() {
            ICON_SIZE + GAP
        } else {
            0.0
        };
        let trailing_width = trailing
            .as_ref()
            .map_or(0.0, |galley| galley.size().x + GAP);
        let label_width = (width - 2.0 * PADDING_X - icon_width - trailing_width).max(0.0);
        let galley = self.text.into_galley(
            ui,
            Some(egui::TextWrapMode::Truncate),
            label_width,
            egui::FontSelection::FontId(egui::FontId::proportional(LABEL_SIZE)),
        );

        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, ROW_HEIGHT), egui::Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                ui.is_enabled(),
                self.selected,
                galley.text(),
            )
        });

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() || response.is_pointer_button_down_on();
            let state = ItemState::new(self.selected, self.dimmed, ui.is_enabled(), hovered);
            let painter = ui.painter();
            let colors = &Palette::of(painter).list;
            let style = paint(colors, state);
            let fill = match state {
                ItemState::DimmedHover => colors.hover.fill,
                _ => style.fill,
            };
            let fade = |color: egui::Color32| {
                if state.is_dimmed() {
                    color.gamma_multiply(DIMMED_OPACITY)
                } else {
                    color
                }
            };

            if let Some(fill) = fill {
                painter.rect_filled(rect, CORNER_RADIUS, fill);
            }
            if let Some(stroke) = style.stroke {
                painter.rect_stroke(
                    rect,
                    CORNER_RADIUS,
                    egui::Stroke::new(1.0, stroke),
                    egui::StrokeKind::Inside,
                );
            }
            if response.has_focus() && state != ItemState::Selected {
                painter.rect_stroke(
                    rect,
                    CORNER_RADIUS,
                    egui::Stroke::new(FOCUS_STROKE, colors.focus),
                    egui::StrokeKind::Inside,
                );
            }

            let mut x = rect.left() + PADDING_X;
            if let Some(glyph) = self.icon {
                let icon_rect = egui::Rect::from_min_size(
                    egui::pos2(x, rect.center().y - ICON_SIZE / 2.0),
                    egui::Vec2::splat(ICON_SIZE),
                );
                paint_glyph(
                    painter,
                    icon_rect,
                    egui::Stroke::new(1.0, fade(style.icon)),
                    glyph,
                );
                x += icon_width;
            }

            let text_color = fade(style.text);
            let text_rect = egui::Rect::from_min_size(
                egui::pos2(x, rect.center().y - galley.size().y / 2.0),
                galley.size(),
            );
            if self.strikethrough {
                painter.hline(
                    text_rect.x_range(),
                    text_rect.center().y,
                    egui::Stroke::new(1.0, text_color),
                );
            }
            painter.galley(text_rect.min, galley, text_color);

            if let Some(trailing) = trailing {
                let pos = egui::pos2(
                    rect.right() - PADDING_X - trailing.size().x,
                    rect.center().y - trailing.size().y / 2.0,
                );
                painter.galley(pos, trailing, fade(style.trailing));
            }
        }
        response
    }
}

/// A vertical list of [`ListItem`]s, spaced as in the design; `add_items` adds the rows.
pub fn list<R>(
    ui: &mut egui::Ui,
    add_items: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = ROW_GAP;
        add_items(ui)
    })
}
