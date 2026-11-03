use crate::editor::format_length;
use crate::preview::{self, PreviewSettings};
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::{PatternPiece, Project, Units};
use leathercraft::shapes::{
    Ellipse, Polygon, REGULAR_POLYGON_SIDES, Rectangle, RegularPolygon, Shape,
};

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

fn selected_piece_details(
    ui: &mut egui::Ui,
    units: Units,
    selection: Selection,
    language: Language,
) -> Option<ProjectEvent> {
    let Some(piece) = selection.piece else {
        *selection.rename_buffer = None;
        return None;
    };
    let mut event = None;

    // The buffer lets the box be empty mid-edit, which the project itself would reject.
    let mut name = selection
        .rename_buffer
        .clone()
        .unwrap_or_else(|| piece.name.clone());
    let response = ui.text_edit_singleline(&mut name);
    if response.changed() && !name.trim().is_empty() {
        event = Some(ProjectEvent::RenamePiece {
            id: piece.id,
            name: name.clone(),
        });
    }
    *selection.rename_buffer = response.has_focus().then_some(name);

    match piece.region() {
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
    let child = selection.child.and_then(|child| piece.child(child));
    let (title, shape) = match child {
        Some(child) => {
            let index = piece.child_index(child.id).unwrap_or_default() + 1;
            let op = language.label(Label::Operation(child.op));
            (format!("{index}. {op}"), &child.shape)
        }
        None => (language.label(Label::Base).to_owned(), &piece.base),
    };
    ui.strong(title);
    let mut edited = shape.clone();
    ui.add_enabled_ui(!piece.locked, |ui| {
        shape_fields(ui, &mut edited, units, language);
        if !matches!(shape, Shape::Polygon(_))
            && ui.button(language.label(Label::ToPolygon)).clicked()
        {
            edited = shape.to_polygon();
        }
    });
    if edited != *shape {
        event = Some(match child {
            Some(child) => ProjectEvent::SetChildShape {
                id,
                child: child.id,
                shape: edited,
            },
            None => ProjectEvent::SetBaseShape { id, shape: edited },
        });
    }
    event
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
        Shape::Polygon(Polygon { points }) => {
            ui.label(language.message(&Message::PointCount {
                count: points.len(),
            }));
        }
        Shape::Rectangle(Rectangle {
            center,
            width,
            height,
            rotation,
        }) => {
            length(ui, Label::CenterX, &mut center.x, f64::MIN);
            length(ui, Label::CenterY, &mut center.y, f64::MIN);
            length(ui, Label::Width, width, MIN_SIZE_MM);
            length(ui, Label::Height, height, MIN_SIZE_MM);
            angle(ui, rotation);
        }
        Shape::Ellipse(Ellipse {
            center,
            radius_x,
            radius_y,
            rotation,
        }) => {
            length(ui, Label::CenterX, &mut center.x, f64::MIN);
            length(ui, Label::CenterY, &mut center.y, f64::MIN);
            length(ui, Label::RadiusX, radius_x, MIN_SIZE_MM);
            length(ui, Label::RadiusY, radius_y, MIN_SIZE_MM);
            angle(ui, rotation);
        }
        Shape::RegularPolygon(RegularPolygon {
            center,
            radius,
            sides,
            rotation,
        }) => {
            length(ui, Label::CenterX, &mut center.x, f64::MIN);
            length(ui, Label::CenterY, &mut center.y, f64::MIN);
            length(ui, Label::Radius, radius, MIN_SIZE_MM);
            ui.horizontal(|ui| {
                ui.label(language.label(Label::Sides));
                ui.add(egui::DragValue::new(sides).range(REGULAR_POLYGON_SIDES));
            });
            angle(ui, rotation);
        }
    }
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
    let details = selected_piece_details(ui, project.units, selection, language);
    units.or(details)
}
