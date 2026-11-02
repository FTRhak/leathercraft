mod canvas;
mod confirm_discard;
mod drag;
mod drag_state_struct;
mod editor_state_struct;
mod grid;
mod panel_viewer;
mod shape_draft_struct;
mod ui;

pub use editor_state_struct::EditorState;
pub use ui::editor_ui;

use crate::document::ProjectMessage;
use bevy::prelude::*;
use bevy_egui::egui;
use canvas::canvas;
use drag_state_struct::{DragState, DragTarget};
use leathercraft::events::ProjectEvent;
use leathercraft::exports::ExportError;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::{Project, Units};
use leathercraft::persistence;
use leathercraft::shapes::Shape;
use shape_draft_struct::ShapeDraft;

const PROJECT_EXTENSION: &str = "leathercraft";

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Editor2d,
    Preview3d,
}

fn send(events: &mut MessageWriter<ProjectMessage>, event: ProjectEvent) {
    events.write(ProjectMessage(event));
}

fn export(
    project: &Project,
    state: &mut EditorState,
    label: &str,
    extension: &str,
    encode: fn(&Project) -> std::result::Result<String, ExportError>,
) {
    let contents = match encode(project) {
        Ok(contents) => contents,
        Err(error) => {
            state.report_error(error.into());
            return;
        }
    };
    let Some(mut path) = rfd::FileDialog::new()
        .add_filter(label, &[extension])
        .save_file()
    else {
        return;
    };
    if path.extension().is_none() {
        path.set_extension(extension);
    }
    match persistence::write_export(&path, &contents) {
        Ok(()) => state.report(Message::Exported {
            path: path.display().to_string(),
        }),
        Err(error) => state.report_error(Message::CouldNotWrite {
            path: path.display().to_string(),
            detail: error.to_string(),
        }),
    }
}

fn view_controls(ui: &mut egui::Ui, state: &mut EditorState) {
    let language = state.language;
    ui.small(language.label(state.tool.hint()));
    ui.small(language.label(Label::NavigationHint));
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut state.snap_enabled,
            language.label(Label::SnapMillimeters),
        );
        ui.add_enabled(
            state.snap_enabled,
            egui::DragValue::new(&mut state.snap_mm)
                .range(0.1..=100.0)
                .speed(0.1),
        );
    });
    if ui.button(language.label(Label::ZoomToFit)).clicked() {
        state.fit_requested = true;
    }
}

fn units_control(
    ui: &mut egui::Ui,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    language: Language,
) {
    let mut units = project.units;
    egui::ComboBox::from_label(language.label(Label::DisplayUnits))
        .selected_text(language.unit_symbol(units))
        .show_ui(ui, |ui| {
            for option in [Units::Millimeters, Units::Inches] {
                ui.selectable_value(&mut units, option, language.unit_symbol(option));
            }
        });
    if units != project.units {
        send(events, ProjectEvent::SetUnits(units));
    }
}

fn selected_piece_details(
    ui: &mut egui::Ui,
    project: &Project,
    events: &mut MessageWriter<ProjectMessage>,
    state: &mut EditorState,
) {
    let language = state.language;
    let units = project.units;
    let Some(piece) = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id))
    else {
        state.rename_buffer = None;
        return;
    };

    // The buffer lets the box be empty mid-edit, which the project itself would reject.
    let mut name = state
        .rename_buffer
        .clone()
        .unwrap_or_else(|| piece.name.clone());
    let response = ui.text_edit_singleline(&mut name);
    if response.changed() && !name.trim().is_empty() {
        send(
            events,
            ProjectEvent::RenamePiece {
                id: piece.id,
                name: name.clone(),
            },
        );
    }
    state.rename_buffer = response.has_focus().then_some(name);

    let shown = state.displayed(piece);
    match shown.region() {
        Ok(region) => {
            if let Some(bounds) = region.bounds() {
                ui.label(format!(
                    "{} × {}",
                    format_length(language, units, bounds.width()),
                    format_length(language, units, bounds.height())
                ));
            }
        }
        Err(error) => {
            ui.colored_label(ui.visuals().error_fg_color, language.message(&error));
        }
    }

    ui.separator();
    let id = piece.id;
    let child = state
        .selected_child_id()
        .and_then(|child| shown.child(child));
    let (title, shape) = match child {
        Some(child) => {
            let index = shown.child_index(child.id).unwrap_or_default() + 1;
            let op = language.label(Label::Operation(child.op));
            (format!("{index}. {op}"), &child.shape)
        }
        None => (language.label(Label::Base).to_owned(), &shown.base),
    };
    ui.strong(title);
    let mut edited = shape.clone();
    ui.add_enabled_ui(!piece.locked, |ui| {
        shape_fields(ui, &mut edited, units, language);
    });
    if edited != *shape {
        let event = match child {
            Some(child) => ProjectEvent::SetChildShape {
                id,
                child: child.id,
                shape: edited,
            },
            None => ProjectEvent::SetBaseShape { id, shape: edited },
        };
        send(events, event);
    }
}

/// Fields for the parameters of a primitive, in display units; a polygon shows its point count.
fn shape_fields(ui: &mut egui::Ui, shape: &mut Shape, units: Units, language: Language) {
    let scale = units.from_millimeters(1.0);
    let symbol = language.unit_symbol(units);
    let length = |ui: &mut egui::Ui, label: Label, millimeters: &mut f64, min: f64| {
        let mut value = *millimeters * scale;
        ui.horizontal(|ui| {
            ui.label(language.label(label));
            let field = egui::DragValue::new(&mut value)
                .speed(0.1 * scale)
                .max_decimals(3)
                .range(min * scale..=f64::MAX)
                .suffix(format!(" {symbol}"));
            if ui.add(field).changed() {
                *millimeters = value / scale;
            }
        });
    };
    let angle = |ui: &mut egui::Ui, radians: &mut f64| {
        let mut degrees = radians.to_degrees();
        ui.horizontal(|ui| {
            ui.label(language.label(Label::Rotation));
            let field = egui::DragValue::new(&mut degrees)
                .speed(1.0)
                .max_decimals(2)
                .range(-360.0..=360.0);
            if ui.add(field).changed() {
                *radians = degrees.to_radians();
            }
        });
    };
    // Sizes stay above zero, which the project would reject.
    const MIN_SIZE_MM: f64 = 0.01;
    match shape {
        Shape::Polygon { points } => {
            ui.label(language.message(&Message::PointCount {
                count: points.len(),
            }));
        }
        Shape::Rectangle {
            center,
            width,
            height,
            rotation,
        } => {
            length(ui, Label::CenterX, &mut center.x, f64::MIN);
            length(ui, Label::CenterY, &mut center.y, f64::MIN);
            length(ui, Label::Width, width, MIN_SIZE_MM);
            length(ui, Label::Height, height, MIN_SIZE_MM);
            angle(ui, rotation);
        }
        Shape::Ellipse {
            center,
            radius_x,
            radius_y,
            rotation,
        } => {
            length(ui, Label::CenterX, &mut center.x, f64::MIN);
            length(ui, Label::CenterY, &mut center.y, f64::MIN);
            length(ui, Label::RadiusX, radius_x, MIN_SIZE_MM);
            length(ui, Label::RadiusY, radius_y, MIN_SIZE_MM);
            angle(ui, rotation);
        }
    }
}

fn format_length(language: Language, units: Units, millimeters: f64) -> String {
    format!(
        "{:.2} {}",
        units.from_millimeters(millimeters),
        language.unit_symbol(units)
    )
}
