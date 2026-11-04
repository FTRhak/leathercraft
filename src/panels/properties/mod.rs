//! The Properties panel. Each shape type has its own file of fields; `shape` picks one,
//! `piece` shows the selected piece around it, and `fields` holds the shared inputs.

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
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::{PatternPiece, Project, Units};

/// The canvas options of the 2D editor.
pub struct EditorView<'a> {
    /// What the current tool does.
    pub hint: Label,
    pub snap_enabled: &'a mut bool,
    pub snap_mm: &'a mut f64,
    pub fit_requested: &'a mut bool,
}

/// The controls at the top of the panel, which depend on the view mode.
pub enum ViewControls<'a> {
    Editor(EditorView<'a>),
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

fn view_controls(ui: &mut egui::Ui, view: EditorView, language: Language) {
    ui.small(language.label(view.hint));
    ui.small(language.label(Label::NavigationHint));
    ui.horizontal(|ui| {
        ui.checkbox(view.snap_enabled, language.label(Label::SnapMillimeters));
        ui.add_enabled(
            *view.snap_enabled,
            egui::DragValue::new(view.snap_mm)
                .range(0.1..=100.0)
                .speed(0.1),
        );
    });
    if ui.button(language.label(Label::ZoomToFit)).clicked() {
        *view.fit_requested = true;
    }
}

fn units_control(ui: &mut egui::Ui, project: &Project, language: Language) -> Option<ProjectEvent> {
    let mut units = project.units;
    egui::ComboBox::from_label(language.label(Label::DisplayUnits))
        .selected_text(language.unit_symbol(units))
        .show_ui(ui, |ui| {
            for option in [Units::Millimeters, Units::Inches] {
                ui.selectable_value(&mut units, option, language.unit_symbol(option));
            }
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
    match view {
        ViewControls::Editor(view) => view_controls(ui, view, language),
        ViewControls::Preview(settings) => preview::show_controls(ui, settings, language),
    }
    ui.separator();
    let units = units_control(ui, project, language);
    ui.separator();
    let details = piece::selected_piece_details(ui, project.units, selection, language);
    units.or(details)
}
