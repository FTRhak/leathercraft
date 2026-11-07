//! The Properties panel, from the "Properties Body" of the Brilliant design: a hint box with the
//! canvas controls, the display units, then the parameters of the selected piece. Each shape
//! type has its own file of fields; `shape` picks one, `piece` shows the selected piece around
//! it, and `fields` holds the rows, headers and inputs they share.

mod ellipse;
mod fields;
mod line;
mod piece;
mod polygon;
mod rectangle;
mod regular_polygon;
mod shape;
mod stitch;

pub(crate) use line::line_style_fields;
pub(crate) use stitch::stitch_fields;

use crate::editor::ShapePart;
use crate::preview::{self, PreviewSettings};
use crate::ui_components::dropdown::{Dropdown, DropdownItem, dropdown_value};
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::{PatternPiece, Project, Units};

/// The canvas options of the 2D editor.
pub struct EditorView {
    /// What the current tool does.
    pub hint: Label,
}

/// The controls at the top of the panel, which depend on the view mode.
pub enum ViewControls<'a> {
    Editor(EditorView),
    Preview(&'a mut PreviewSettings),
}

pub struct Selection<'a> {
    /// The selected piece as currently shown, so with a drag in progress applied.
    pub piece: Option<&'a PatternPiece>,
    pub child: Option<u64>,
    /// The edge or vertex of the shown shape picked with the Edge tool.
    pub part: Option<ShapePart>,
    /// The name being typed, kept while the name box has focus.
    pub rename_buffer: &'a mut Option<String>,
}

/// Between the panel's edges and its content, as in the design.
const PANEL_MARGIN: i8 = 12;
/// Between the hint box, the units row, the divider and the piece's sections.
const SECTION_GAP: f32 = 12.0;
const HINT_PADDING: egui::Margin = egui::Margin::symmetric(10, 9);
const HINT_RADIUS: u8 = 6;
const HINT_GAP: f32 = 6.0;
const HINT_TEXT_SIZE: f32 = 11.0;
const INFO_ICON_SIZE: f32 = 13.0;
const UNITS_WIDTH: f32 = 112.0;

/// A circled "i", as wide and tall as `rect`.
fn paint_info_icon(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.2, color);
    let center = rect.center();
    let radius = rect.width() / 2.0 - stroke.width / 2.0;
    painter.circle_stroke(center, radius, stroke);
    painter.circle_filled(center - egui::vec2(0.0, radius * 0.45), 0.9, color);
    painter.line_segment(
        [
            center - egui::vec2(0.0, radius * 0.1),
            center + egui::vec2(0.0, radius * 0.5),
        ],
        stroke,
    );
}

/// The hint box: what the current tool does and how to move around the canvas.
fn view_controls(ui: &mut egui::Ui, view: EditorView, language: Language) {
    let palette = Palette::of(ui.painter());
    egui::Frame::new()
        .fill(palette.surface.info)
        .stroke(egui::Stroke::new(1.0, palette.border.divider))
        .corner_radius(HINT_RADIUS)
        .inner_margin(HINT_PADDING)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = HINT_GAP;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = HINT_GAP;
                let (rect, _) =
                    ui.allocate_exact_size(egui::Vec2::splat(INFO_ICON_SIZE), egui::Sense::hover());
                paint_info_icon(ui.painter(), rect, palette.accent.base);
                ui.label(
                    egui::RichText::new(language.label(Label::CanvasControls))
                        .size(HINT_TEXT_SIZE)
                        .strong()
                        .color(palette.properties.hint_title),
                );
            });
            for (label, color) in [
                (view.hint, palette.properties.hint_text),
                (Label::NavigationHint, palette.text.muted),
            ] {
                ui.label(
                    egui::RichText::new(language.label(label))
                        .size(HINT_TEXT_SIZE)
                        .color(color),
                );
            }
        });
}

fn units_control(ui: &mut egui::Ui, project: &Project, language: Language) -> Option<ProjectEvent> {
    let mut units = project.units;
    fields::param_row(ui, language.label(Label::DisplayUnits), None, |ui| {
        Dropdown::new("display_units", language.unit_symbol(units))
            .width(UNITS_WIDTH)
            .show_ui(ui, |ui| {
                for option in [Units::Millimeters, Units::Inches] {
                    let item = DropdownItem::new(language.unit_symbol(option));
                    dropdown_value(ui, &mut units, option, item);
                }
            });
    });
    (units != project.units).then_some(ProjectEvent::SetUnits(units))
}

/// The view controls, the display units, then the name, size and shape fields of the selected
/// piece; returns the `ProjectEvent` to send.
pub fn show_panel(
    ui: &mut egui::Ui,
    project: &Project,
    view: ViewControls,
    selection: Selection,
    language: Language,
) -> Option<ProjectEvent> {
    egui::ScrollArea::vertical()
        .show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(PANEL_MARGIN)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = SECTION_GAP;
                    match view {
                        ViewControls::Editor(view) => view_controls(ui, view, language),
                        ViewControls::Preview(settings) => {
                            preview::show_controls(ui, settings, language);
                        }
                    }
                    let units = units_control(ui, project, language);
                    fields::divider(ui);
                    ui.spacing_mut().item_spacing.y = fields::ROW_GAP;
                    let details =
                        piece::selected_piece_details(ui, project.units, selection, language);
                    units.or(details)
                })
                .inner
        })
        .inner
}
