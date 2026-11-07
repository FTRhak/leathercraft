//! The switch between the 2D editor and the 3D preview, in the middle of the title bar: a
//! track holding one option per mode, each an icon and a label, of which the current one is
//! filled with the accent.

use bevy_egui::egui;

use super::icons;
use crate::ui_components::palette::{ModePaint, ModeSwitchColors, Palette};

const TRACK_PADDING: f32 = 3.0;
const TRACK_RADIUS: u8 = 7;
const GAP: f32 = 2.0;
const OPTION_HEIGHT: f32 = 23.0;
const OPTION_PADDING: f32 = 12.0;
const OPTION_RADIUS: u8 = 5;
const ICON_SIZE: f32 = 13.0;
const ICON_STROKE: f32 = 1.2;
const ICON_GAP: f32 = 6.0;
const FONT_SIZE: f32 = 12.0;

/// The icon of an option.
#[derive(Clone, Copy, Debug)]
pub enum ModeIcon {
    /// A bounding box, for a flat editor.
    Flat,
    /// A cube, for a 3D view.
    Solid,
}

impl ModeIcon {
    fn paint(self, painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
        match self {
            Self::Flat => icons::bounding_box(painter, rect, stroke),
            Self::Solid => icons::cube(painter, rect, stroke),
        }
    }
}

/// The mode switch bound to a `&mut V`, added with `ui.add(...)`. The response is marked changed
/// when a click picks another mode.
#[must_use = "add the mode switch with `ui.add(switch)`"]
pub struct ModeSwitch<'a, V> {
    current: &'a mut V,
    options: Vec<(V, ModeIcon, egui::WidgetText)>,
}

impl<'a, V: PartialEq> ModeSwitch<'a, V> {
    pub fn new(current: &'a mut V) -> Self {
        Self {
            current,
            options: Vec::new(),
        }
    }

    /// Adds an option for `value`, after the ones added before.
    pub fn option(mut self, value: V, icon: ModeIcon, text: impl Into<egui::WidgetText>) -> Self {
        self.options.push((value, icon, text.into()));
        self
    }
}

impl<V: PartialEq> egui::Widget for ModeSwitch<'_, V> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self { current, options } = self;
        let font = egui::FontId::proportional(FONT_SIZE);
        let options: Vec<_> = options
            .into_iter()
            .map(|(value, icon, text)| {
                let galley = text.into_galley(
                    ui,
                    Some(egui::TextWrapMode::Extend),
                    f32::INFINITY,
                    font.clone(),
                );
                let width = 2.0 * OPTION_PADDING + ICON_SIZE + ICON_GAP + galley.size().x;
                (value, icon, galley, width)
            })
            .collect();
        let width = options.iter().map(|option| option.3).sum::<f32>()
            + GAP * options.len().saturating_sub(1) as f32
            + 2.0 * TRACK_PADDING;
        let size = egui::vec2(width, OPTION_HEIGHT + 2.0 * TRACK_PADDING);
        let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::hover());

        let colors = &Palette::of(ui.painter()).title_bar.mode;
        let enabled = ui.is_enabled();
        let mut picked = None;
        let mut painted = Vec::with_capacity(options.len());
        let mut x = rect.left() + TRACK_PADDING;
        for (index, (value, icon, galley, width)) in options.into_iter().enumerate() {
            let option_rect = egui::Rect::from_min_size(
                egui::pos2(x, rect.top() + TRACK_PADDING),
                egui::vec2(width, OPTION_HEIGHT),
            );
            x += width + GAP;
            let option = ui.interact(option_rect, response.id.with(index), egui::Sense::click());
            let selected = *current == value;
            option.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::Button,
                    enabled,
                    selected,
                    galley.text(),
                )
            });
            if option.clicked() && !selected {
                picked = Some(value);
            }
            let hovered = enabled && (option.hovered() || option.is_pointer_button_down_on());
            let style = option_paint(colors, selected, hovered);
            painted.push((option_rect, icon, galley, style, option.has_focus()));
            response |= option;
        }
        if let Some(value) = picked {
            *current = value;
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect(
                rect,
                TRACK_RADIUS,
                colors.track,
                egui::Stroke::new(1.0, colors.track_border),
                egui::StrokeKind::Inside,
            );
            for (option_rect, icon, galley, style, focused) in painted {
                if let Some(fill) = style.fill {
                    painter.rect_filled(option_rect, OPTION_RADIUS, fill);
                }
                let border = if focused {
                    Some(colors.focus)
                } else {
                    style.border
                };
                if let Some(border) = border {
                    painter.rect_stroke(
                        option_rect,
                        OPTION_RADIUS,
                        egui::Stroke::new(1.0, border),
                        egui::StrokeKind::Inside,
                    );
                }
                let text = if enabled {
                    style.text
                } else {
                    style.text.gamma_multiply(0.5)
                };
                let icon_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        option_rect.left() + OPTION_PADDING,
                        option_rect.center().y - ICON_SIZE / 2.0,
                    ),
                    egui::Vec2::splat(ICON_SIZE),
                );
                icon.paint(painter, icon_rect, egui::Stroke::new(ICON_STROKE, text));
                let text_pos = egui::pos2(
                    icon_rect.right() + ICON_GAP,
                    option_rect.center().y - galley.size().y / 2.0,
                );
                painter.galley(text_pos, galley, text);
            }
        }
        response
    }
}

fn option_paint(colors: &ModeSwitchColors, selected: bool, hovered: bool) -> &ModePaint {
    if selected {
        &colors.active
    } else if hovered {
        &colors.hover
    } else {
        &colors.inactive
    }
}
