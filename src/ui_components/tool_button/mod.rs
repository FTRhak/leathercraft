//! The tool button of the tool bar, from the Brilliant `ToolBtn` component: a 32 px square with
//! an icon, or a chip of the same height with a colored dot and a label, drawn idle, hover,
//! active or disabled in the colors of the current theme.
//!
//! ```ignore
//! let select = ui.add(
//!     ToolButton::glyph(glyph::SELECT, language.label(Label::Select))
//!         .selected(tool == Tool::Select)
//!         .shortcut(Some("V".into())),
//! );
//! let edge = ui.add(ToolButton::chip(palette.edge, language.label(Label::Edge)));
//! ```
//!
//! Disable it the egui way, with `ui.add_enabled(false, ToolButton::..)`.

mod style;

use bevy_egui::egui;
use style::{ToolButtonState, paint};

use crate::ui_components::icons::paint_glyph;
use crate::ui_components::palette::Palette;

const SIZE: f32 = 32.0;
/// The box the icon is painted in; the icon font fills about three quarters of it, 16 px.
const ICON_BOX: f32 = 22.0;
const CORNER_RADIUS: u8 = 6;
const CHIP_PADDING: f32 = 10.0;
const CHIP_GAP: f32 = 6.0;
const DOT_SIZE: f32 = 7.0;

type PaintIcon<'a> = Box<dyn FnOnce(&egui::Painter, egui::Rect, egui::Stroke) + 'a>;

enum Content<'a> {
    Icon(PaintIcon<'a>),
    Chip {
        dot: egui::Color32,
        text: egui::WidgetText,
    },
}

/// A button of the tool bar, added with `ui.add(...)`.
#[must_use = "add the tool button with `ui.add(button)`"]
pub struct ToolButton<'a> {
    content: Content<'a>,
    name: String,
    selected: bool,
    shortcut: Option<String>,
}

impl<'a> ToolButton<'a> {
    /// A square button showing a glyph of the icon font; `name` is its tooltip.
    pub fn glyph(glyph: char, name: impl Into<String>) -> Self {
        Self::icon(
            move |painter, rect, stroke| paint_glyph(painter, rect, stroke, glyph),
            name,
        )
    }

    /// A square button with a painted icon: the closure gets the icon's box and its stroke.
    pub fn icon(
        paint: impl FnOnce(&egui::Painter, egui::Rect, egui::Stroke) + 'a,
        name: impl Into<String>,
    ) -> Self {
        Self {
            content: Content::Icon(Box::new(paint)),
            name: name.into(),
            selected: false,
            shortcut: None,
        }
    }

    /// A chip with a dot of `dot` before `text`, for the tools named by their mark on the canvas.
    pub fn chip(dot: egui::Color32, text: impl Into<String>) -> Self {
        let name = text.into();
        Self {
            content: Content::Chip {
                dot,
                text: name.clone().into(),
            },
            name,
            selected: false,
            shortcut: None,
        }
    }

    /// Draws the button active, as the current tool or mode.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Shown in the tooltip after the name, such as `V`.
    pub fn shortcut(mut self, shortcut: Option<String>) -> Self {
        self.shortcut = shortcut;
        self
    }
}

impl egui::Widget for ToolButton<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self {
            content,
            name,
            selected,
            shortcut,
        } = self;
        let galley = match &content {
            Content::Icon(_) => None,
            Content::Chip { text, .. } => Some(text.clone().into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                egui::TextStyle::Button,
            )),
        };
        let width = galley.as_ref().map_or(SIZE, |galley| {
            2.0 * CHIP_PADDING + DOT_SIZE + CHIP_GAP + galley.size().x
        });
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, SIZE), egui::Sense::click());
        let enabled = ui.is_enabled();
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, selected, &name)
        });

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() || response.is_pointer_button_down_on();
            let state = ToolButtonState::new(selected, enabled, hovered);
            let painter = ui.painter();
            let colors = &Palette::of(painter).toolbar.button;
            let style = paint(colors, state);
            if state == ToolButtonState::Active {
                let shadow = egui::Shadow {
                    offset: [0, 2],
                    blur: 6,
                    spread: 0,
                    color: colors.active_shadow,
                };
                painter.add(shadow.as_shape(rect, CORNER_RADIUS));
            }
            painter.rect(
                rect,
                CORNER_RADIUS,
                style.fill,
                egui::Stroke::new(1.0, style.stroke),
                egui::StrokeKind::Inside,
            );
            if response.has_focus() {
                painter.rect_stroke(
                    rect.expand(2.0),
                    CORNER_RADIUS + 2,
                    egui::Stroke::new(1.0, colors.focus),
                    egui::StrokeKind::Inside,
                );
            }
            let stroke = egui::Stroke::new(1.5, style.content);
            match (content, galley) {
                (Content::Icon(paint_icon), _) => {
                    let icon =
                        egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(ICON_BOX));
                    paint_icon(painter, icon, stroke);
                }
                (Content::Chip { dot, .. }, Some(galley)) => {
                    let dot_center =
                        egui::pos2(rect.left() + CHIP_PADDING + DOT_SIZE / 2.0, rect.center().y);
                    painter.circle_filled(dot_center, DOT_SIZE / 2.0, dot);
                    let text_pos = egui::pos2(
                        rect.left() + CHIP_PADDING + DOT_SIZE + CHIP_GAP,
                        rect.center().y - galley.size().y / 2.0,
                    );
                    painter.galley(text_pos, galley, style.content);
                }
                (Content::Chip { .. }, None) => {}
            }
        }
        let tooltip = match shortcut {
            Some(shortcut) => format!("{name} ({shortcut})"),
            None => name,
        };
        response.on_hover_text(tooltip)
    }
}
