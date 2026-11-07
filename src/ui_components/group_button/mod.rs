//! The group button of the design system, from the Brilliant `Button Group` and `Segment`
//! components: a row of joined segments, one per option, of which exactly one is selected. Each
//! segment is drawn default, hover, selected, focus or disabled, in the colors of the current
//! theme.
//!
//! ```ignore
//! ui.add(
//!     GroupButton::new(&mut theme)
//!         .option(Theme::Dark, language.label(Label::ThemeDark))
//!         .option(Theme::Light, language.label(Label::ThemeLight)),
//! );
//! ```
//!
//! The response is marked changed when a click picks another option. Disable the whole group
//! the egui way, with `ui.add_enabled(false, GroupButton::new(..))`.

mod style;

use bevy_egui::egui;
use style::{SegmentState, paint};

use crate::ui_components::palette::Palette;

const HEIGHT: f32 = 26.0;
const SEGMENT_PADDING: f32 = 12.0;
const MIN_SEGMENT_WIDTH: f32 = 44.0;
const CORNER_RADIUS: u8 = 5;

/// A row of joined segments choosing one value of `V`, added with `ui.add(...)`.
#[must_use = "add the group button with `ui.add(group)`"]
pub struct GroupButton<'a, V> {
    current: &'a mut V,
    options: Vec<(V, egui::WidgetText)>,
}

impl<'a, V: PartialEq> GroupButton<'a, V> {
    /// A group bound to `current`: the segment whose value equals it is selected, and a click on
    /// another segment sets it.
    pub fn new(current: &'a mut V) -> Self {
        Self {
            current,
            options: Vec::new(),
        }
    }

    /// Adds a segment for `value`, after the ones added before.
    pub fn option(mut self, value: V, text: impl Into<egui::WidgetText>) -> Self {
        self.options.push((value, text.into()));
        self
    }
}

impl<V: PartialEq> egui::Widget for GroupButton<'_, V> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self { current, options } = self;
        let segments: Vec<_> = options
            .into_iter()
            .map(|(value, text)| {
                let galley = text.into_galley(
                    ui,
                    Some(egui::TextWrapMode::Extend),
                    f32::INFINITY,
                    egui::TextStyle::Button,
                );
                let width = (galley.size().x + 2.0 * SEGMENT_PADDING).max(MIN_SEGMENT_WIDTH);
                (value, galley, width)
            })
            .collect();
        let total_width = segments.iter().map(|(_, _, width)| width).sum::<f32>();
        let height = segments
            .iter()
            .map(|(_, galley, _)| galley.size().y + 10.0)
            .fold(HEIGHT, f32::max);
        let (rect, mut response) =
            ui.allocate_exact_size(egui::vec2(total_width, height), egui::Sense::hover());

        let enabled = ui.is_enabled();
        let colors = &Palette::of(ui.painter()).group_button;
        let last = segments.len().saturating_sub(1);
        let mut picked = None;
        let mut painted = Vec::with_capacity(segments.len());
        let mut x = rect.left();
        for (index, (value, galley, width)) in segments.into_iter().enumerate() {
            let segment_rect =
                egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(width, height));
            x += width;
            let segment = ui.interact(segment_rect, response.id.with(index), egui::Sense::click());
            let selected = *current == value;
            segment.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::Button,
                    enabled,
                    selected,
                    galley.text(),
                )
            });
            if segment.clicked() && !selected {
                picked = Some(value);
            }
            let hovered = segment.hovered() || segment.is_pointer_button_down_on();
            let focused = segment.has_focus();
            let state = SegmentState::new(selected, enabled, hovered, focused);
            painted.push((segment_rect, galley, state, focused));
            response |= segment;
        }
        if let Some(value) = picked {
            *current = value;
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let border = if enabled {
                colors.border
            } else {
                colors.disabled_border
            };
            for (index, (segment_rect, galley, state, focused)) in painted.into_iter().enumerate() {
                let style = paint(colors, state);
                let corners = outer_corners(index == 0, index == last);
                painter.rect_filled(segment_rect, corners, style.fill);
                // Each segment but the last ends in a divider; the selected one doesn't, so
                // its fill meets the next segment.
                if index != last && !state.is_selected() {
                    painter.vline(
                        segment_rect.right() - 0.5,
                        segment_rect.y_range(),
                        egui::Stroke::new(1.0, border),
                    );
                }
                if focused && !state.is_disabled() {
                    painter.rect_stroke(
                        segment_rect,
                        corners,
                        egui::Stroke::new(1.0, colors.focus_ring),
                        egui::StrokeKind::Inside,
                    );
                }
                let text_pos = segment_rect.center() - galley.size() / 2.0;
                painter.galley(text_pos, galley, style.text);
            }
            painter.rect_stroke(
                rect,
                CORNER_RADIUS,
                egui::Stroke::new(1.0, border),
                egui::StrokeKind::Inside,
            );
        }
        response
    }
}

/// The rounding of a segment: only the corners on the outside of the group are round.
fn outer_corners(first: bool, last: bool) -> egui::CornerRadius {
    let left = if first { CORNER_RADIUS } else { 0 };
    let right = if last { CORNER_RADIUS } else { 0 };
    egui::CornerRadius {
        nw: left,
        sw: left,
        ne: right,
        se: right,
    }
}
