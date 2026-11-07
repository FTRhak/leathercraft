//! The dropdown of the design system, from the Brilliant `Select` component: a field showing
//! the chosen value with a caret, drawn in its default, hover, focus and disabled state, that
//! opens a list of items below it.
//!
//! ```ignore
//! Dropdown::new("units", language.label(units.label()))
//!     .width(200.0)
//!     .show_ui(ui, |ui| {
//!         for option in Units::ALL {
//!             dropdown_value(ui, &mut units, option, DropdownItem::new(language.label(option.label())));
//!         }
//!     });
//! ```
//!
//! Both the field and its items take an optional leading slot painted before the text, such as
//! a color swatch. Disable the dropdown the egui way, with `ui.add_enabled_ui(false, ..)`
//! around it; an item is disabled with `ui.add_enabled(false, DropdownItem::new(..))`.

mod style;

use bevy_egui::egui;
use style::{DropdownState, paint};

use crate::ui_components::palette::Palette;

const HEIGHT: f32 = 28.0;
const PADDING: egui::Vec2 = egui::vec2(10.0, 5.0);
const CORNER_RADIUS: u8 = 5;
/// Between the leading slot, the text and the caret.
const GAP: f32 = 8.0;
const CARET_SIZE: egui::Vec2 = egui::vec2(11.0, 6.0);
const CARET_STROKE: f32 = 1.5;
const FOCUS_STROKE: f32 = 1.0;
/// Between the field and its open list.
const MENU_GAP: f32 = 2.0;
const MENU_PADDING: f32 = 4.0;
const MENU_MAX_HEIGHT: f32 = 360.0;
const ITEM_HEIGHT: f32 = 24.0;
const ITEM_PADDING: egui::Vec2 = egui::vec2(8.0, 4.0);
const ITEM_CORNER_RADIUS: u8 = 3;
/// Above a group label, except the first.
const GROUP_SPACING: f32 = 4.0;

/// What a leading slot paints into its rect, such as a color swatch.
type LeadingPaint<'a> = Box<dyn FnOnce(&egui::Painter, egui::Rect) + 'a>;

/// Content painted before the text of a dropdown or an item, in a box of `size`.
struct Leading<'a> {
    size: egui::Vec2,
    paint: LeadingPaint<'a>,
}

impl Leading<'_> {
    fn width(&self) -> f32 {
        self.size.x + GAP
    }

    /// Paints into a box of its size at `x`, centered on `center_y`; returns where the text starts.
    fn paint_at(self, painter: &egui::Painter, x: f32, center_y: f32) -> f32 {
        let rect =
            egui::Rect::from_min_size(egui::pos2(x, center_y - self.size.y / 2.0), self.size);
        (self.paint)(painter, rect);
        x + self.size.x + GAP
    }
}

/// A field that shows the chosen value and opens a list of items below it. Show it with
/// [`Dropdown::show_ui`], which takes the items, usually added with [`dropdown_value`].
#[must_use = "show the dropdown with `show_ui`"]
pub struct Dropdown<'a> {
    id_salt: egui::Id,
    text: egui::WidgetText,
    width: Option<f32>,
    max_height: f32,
    leading: Option<Leading<'a>>,
}

impl<'a> Dropdown<'a> {
    /// A dropdown showing `selected_text`; `id_salt` keeps its list open across frames, so it
    /// must be unique within the parent `Ui`.
    pub fn new(id_salt: impl std::hash::Hash, selected_text: impl Into<egui::WidgetText>) -> Self {
        Self {
            id_salt: egui::Id::new(id_salt),
            text: selected_text.into(),
            width: None,
            max_height: MENU_MAX_HEIGHT,
            leading: None,
        }
    }

    /// The width of the field and of its list; egui's `combo_width` by default. A text that
    /// doesn't fit is cut short with an ellipsis.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// How tall the list grows before it scrolls.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    /// Paints something of `size` before the text, such as the swatch of the chosen color.
    pub fn leading(
        mut self,
        size: egui::Vec2,
        paint: impl FnOnce(&egui::Painter, egui::Rect) + 'a,
    ) -> Self {
        self.leading = Some(Leading {
            size,
            paint: Box::new(paint),
        });
        self
    }

    /// Draws the field and, while it is open, its list with `add_items`.
    ///
    /// `inner` is `None` while the list is closed. A click on an item closes the list.
    pub fn show_ui<R>(
        self,
        ui: &mut egui::Ui,
        add_items: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        let Self {
            id_salt,
            text,
            width,
            max_height,
            leading,
        } = self;
        let id = ui.make_persistent_id(id_salt);
        let popup_id = id.with("popup");
        let width = width.unwrap_or_else(|| ui.spacing().combo_width);

        let leading_width = leading.as_ref().map_or(0.0, Leading::width);
        let text_width = width - 2.0 * PADDING.x - leading_width - GAP - CARET_SIZE.x;
        let galley = text.into_galley(
            ui,
            Some(egui::TextWrapMode::Truncate),
            text_width.max(0.0),
            egui::TextStyle::Button,
        );
        let height = HEIGHT.max(galley.size().y + 2.0 * PADDING.y);
        let (_, rect) = ui.allocate_space(egui::vec2(width, height));
        let response = ui.interact(rect, id, egui::Sense::click());
        response.widget_info(|| {
            let mut info = egui::WidgetInfo::new(egui::WidgetType::ComboBox);
            info.enabled = ui.is_enabled();
            info.current_text_value = Some(galley.text().to_owned());
            info
        });

        let palette = Palette::of(ui.painter());
        let colors = &palette.dropdown;
        if ui.is_rect_visible(rect) {
            let open = egui::Popup::is_id_open(ui.ctx(), popup_id);
            let hovered = response.hovered() || response.is_pointer_button_down_on();
            let state = DropdownState::new(ui.is_enabled(), open || response.has_focus(), hovered);
            let style = paint(colors, state);
            let painter = ui.painter();
            painter.rect(
                rect,
                CORNER_RADIUS,
                style.fill,
                egui::Stroke::new(FOCUS_STROKE, style.stroke),
                egui::StrokeKind::Inside,
            );

            let mut x = rect.left() + PADDING.x;
            if let Some(leading) = leading {
                x = leading.paint_at(painter, x, rect.center().y);
            }
            let text_pos = egui::pos2(x, rect.center().y - galley.size().y / 2.0);
            painter.galley(text_pos, galley, style.text);

            let caret = egui::Rect::from_min_size(
                egui::pos2(
                    rect.right() - PADDING.x - CARET_SIZE.x,
                    rect.center().y - CARET_SIZE.y / 2.0,
                ),
                CARET_SIZE,
            );
            paint_caret(painter, caret, egui::Stroke::new(CARET_STROKE, style.caret));
        }

        let menu = &colors.menu;
        let frame = egui::Frame::new()
            .fill(menu.background)
            .stroke(egui::Stroke::new(1.0, menu.border))
            .corner_radius(CORNER_RADIUS)
            .inner_margin(MENU_PADDING)
            .shadow(ui.visuals().popup_shadow);
        let inner = egui::Popup::menu(&response)
            .id(popup_id)
            .width(rect.width())
            .gap(MENU_GAP)
            .frame(frame)
            .show(|ui| {
                ui.set_min_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 1.0;
                egui::ScrollArea::vertical()
                    .max_height(max_height)
                    .show(ui, add_items)
                    .inner
            })
            .map(|inner| inner.inner);

        egui::InnerResponse::new(inner, response)
    }
}

/// A caret pointing down, as wide and tall as `rect`.
fn paint_caret(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let rect = rect.shrink(stroke.width / 2.0);
    painter.add(egui::Shape::line(
        vec![rect.left_top(), rect.center_bottom(), rect.right_top()],
        stroke,
    ));
}

/// One item of an open dropdown, added with `ui.add(...)` inside [`Dropdown::show_ui`], or with
/// [`dropdown_value`].
#[must_use = "add the item with `ui.add(item)` or `dropdown_value`"]
pub struct DropdownItem<'a> {
    text: egui::WidgetText,
    selected: bool,
    leading: Option<Leading<'a>>,
}

impl<'a> DropdownItem<'a> {
    pub fn new(text: impl Into<egui::WidgetText>) -> Self {
        Self {
            text: text.into(),
            selected: false,
            leading: None,
        }
    }

    /// Whether this is the chosen value.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Paints something of `size` before the text, such as a color swatch.
    pub fn leading(
        mut self,
        size: egui::Vec2,
        paint: impl FnOnce(&egui::Painter, egui::Rect) + 'a,
    ) -> Self {
        self.leading = Some(Leading {
            size,
            paint: Box::new(paint),
        });
        self
    }
}

impl egui::Widget for DropdownItem<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let galley = self.text.into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Button,
        );
        let leading_width = self.leading.as_ref().map_or(0.0, Leading::width);
        let size = egui::vec2(
            ui.available_width()
                .max(leading_width + galley.size().x + 2.0 * ITEM_PADDING.x),
            ITEM_HEIGHT.max(galley.size().y + 2.0 * ITEM_PADDING.y),
        );
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                ui.is_enabled(),
                self.selected,
                galley.text(),
            )
        });

        if ui.is_rect_visible(rect) {
            let palette = Palette::of(ui.painter());
            let menu = &palette.dropdown.menu;
            let enabled = ui.is_enabled();
            let hovered = enabled && (response.hovered() || response.is_pointer_button_down_on());
            let painter = ui.painter();
            let fill = if self.selected {
                Some(menu.selected)
            } else if hovered {
                Some(menu.item_hover)
            } else {
                None
            };
            if let Some(fill) = fill {
                painter.rect_filled(rect, ITEM_CORNER_RADIUS, fill);
            }
            if response.has_focus() {
                painter.rect_stroke(
                    rect,
                    ITEM_CORNER_RADIUS,
                    egui::Stroke::new(FOCUS_STROKE, palette.dropdown.focus.stroke),
                    egui::StrokeKind::Inside,
                );
            }

            let mut x = rect.left() + ITEM_PADDING.x;
            if let Some(leading) = self.leading {
                x = leading.paint_at(painter, x, rect.center().y);
            }
            let text = if !enabled {
                menu.disabled_text
            } else if self.selected {
                menu.selected_text
            } else {
                menu.item_text
            };
            let text_pos = egui::pos2(x, rect.center().y - galley.size().y / 2.0);
            painter.galley(text_pos, galley, text);
        }
        response
    }
}

/// An item that chooses `value`: selected while `*current == value`, and sets it when clicked.
/// The dropdown counterpart of `ui.selectable_value`.
pub fn dropdown_value<V: PartialEq>(
    ui: &mut egui::Ui,
    current: &mut V,
    value: V,
    item: DropdownItem<'_>,
) -> egui::Response {
    let selected = *current == value;
    let mut response = ui.add(item.selected(selected));
    if response.clicked() && !selected {
        *current = value;
        response.mark_changed();
    }
    response
}

/// The heading of a group of items in an open dropdown, such as a family of colors.
pub fn group_label(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Response {
    let galley = text.into().into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::TextStyle::Small,
    );
    if ui.min_rect().height() > 0.0 {
        ui.add_space(GROUP_SPACING);
    }
    let size = egui::vec2(
        ui.available_width()
            .max(galley.size().x + 2.0 * ITEM_PADDING.x),
        galley.size().y + ITEM_PADDING.y,
    );
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let color = Palette::of(ui.painter()).dropdown.menu.group;
        let text_pos = egui::pos2(
            rect.left() + ITEM_PADDING.x,
            rect.bottom() - galley.size().y,
        );
        ui.painter().galley(text_pos, galley, color);
    }
    response
}
