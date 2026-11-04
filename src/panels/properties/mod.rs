use crate::editor::{ShapePart, format_length};
use crate::preview::{self, PreviewSettings};
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language, Message};
use leathercraft::model::{PatternPiece, Project, Units};
use leathercraft::shapes::{
    Ellipse, LineStyle, MIN_DASH_MM, REGULAR_POLYGON_SIDES, Rectangle, RegularPolygon, Shape,
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
            // A line's operation is ignored, so it is named for what it is.
            let kind = if child.shape.is_open() {
                Label::Centerline
            } else {
                Label::Operation(child.op)
            };
            (format!("{index}. {}", language.label(kind)), &child.shape)
        }
        None => (language.label(Label::Base).to_owned(), &piece.base),
    };
    ui.strong(title);
    let mut edited = shape.clone();
    ui.add_enabled_ui(!piece.locked, |ui| {
        shape_fields(ui, &mut edited, selection.part, units, language);
        if !matches!(shape, Shape::Polygon(_) | Shape::Line(_))
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

/// A length in display units, at least `min` mm.
fn length_field(
    ui: &mut egui::Ui,
    label: Label,
    millimeters: &mut f64,
    min: f64,
    units: Units,
    language: Language,
) {
    let scale = units.from_millimeters(1.0);
    let mut value = *millimeters * scale;
    ui.horizontal(|ui| {
        ui.label(language.label(label));
        let field = egui::DragValue::new(&mut value)
            .speed(0.1 * scale)
            .max_decimals(3)
            .range(min * scale..=f64::MAX)
            .suffix(format!(" {}", language.unit_symbol(units)));
        if ui.add(field).changed() {
            *millimeters = value / scale;
        }
    });
}

/// The type of a line and, for a dashed one, its dash and gap in display units. Choosing
/// another type starts from that type's default lengths.
pub(crate) fn line_style_fields(
    ui: &mut egui::Ui,
    style: &mut LineStyle,
    units: Units,
    language: Language,
) {
    let kinds = [
        (LineStyle::Solid, Label::LineSolid),
        (LineStyle::dashed_default(), Label::LineDashed),
        (LineStyle::dash_dot_default(), Label::LineDashDot),
    ];
    let same_kind =
        |a: &LineStyle, b: &LineStyle| std::mem::discriminant(a) == std::mem::discriminant(b);
    let current = kinds
        .iter()
        .find(|(kind, _)| same_kind(kind, style))
        .map_or(Label::LineSolid, |(_, label)| *label);
    ui.horizontal(|ui| {
        ui.label(language.label(Label::LineType));
        egui::ComboBox::from_id_salt("line_style")
            .selected_text(language.label(current))
            .show_ui(ui, |ui| {
                for (kind, label) in kinds {
                    let chosen = same_kind(&kind, style);
                    if ui.selectable_label(chosen, language.label(label)).clicked() && !chosen {
                        *style = kind;
                    }
                }
            });
    });
    if let LineStyle::Dashed { dash, gap } | LineStyle::DashDot { dash, gap } = style {
        length_field(ui, Label::DashLength, dash, MIN_DASH_MM, units, language);
        length_field(ui, Label::GapLength, gap, MIN_DASH_MM, units, language);
    }
}

/// Fields for the parameters of a primitive, in display units; a polygon shows its point count
/// and the length of the selected edge. A selected vertex shows the radius of its corner.
fn shape_fields(
    ui: &mut egui::Ui,
    shape: &mut Shape,
    part: Option<ShapePart>,
    units: Units,
    language: Language,
) {
    let length = |ui: &mut egui::Ui, label: Label, millimeters: &mut f64, min: f64| {
        length_field(ui, label, millimeters, min, units, language);
    };
    let angle_field = |ui: &mut egui::Ui, label: Label, radians: &mut f64| {
        let mut degrees = radians.to_degrees();
        ui.horizontal(|ui| {
            ui.label(language.label(label));
            let field = egui::DragValue::new(&mut degrees)
                .speed(1.0)
                .max_decimals(2)
                .range(-360.0..=360.0);
            if ui.add(field).changed() {
                *radians = degrees.to_radians();
            }
        });
    };
    let angle = |ui: &mut egui::Ui, radians: &mut f64| angle_field(ui, Label::Rotation, radians);
    // Sizes stay above zero, which the project would reject.
    const MIN_SIZE_MM: f64 = 0.01;
    match shape {
        Shape::Polygon(polygon) => {
            ui.label(language.message(&Message::PointCount {
                count: polygon.points.len(),
            }));
            if let Some(ShapePart::Edge(edge)) = part
                && let Some(current) = polygon.edge_length(edge)
            {
                ui.label(language.message(&Message::SelectedEdge { number: edge + 1 }));
                let mut value = current;
                length(ui, Label::EdgeLength, &mut value, MIN_SIZE_MM);
                if value != current
                    && let Some(resized) = polygon.with_edge_length(edge, value)
                {
                    *polygon = resized;
                }
            }
        }
        Shape::Rectangle(Rectangle {
            center,
            width,
            height,
            rotation,
            ..
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
            ..
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
        Shape::Line(line) => {
            length(ui, Label::StartX, &mut line.start.x, f64::MIN);
            length(ui, Label::StartY, &mut line.start.y, f64::MIN);
            length(ui, Label::EndX, &mut line.end.x, f64::MIN);
            length(ui, Label::EndY, &mut line.end.y, f64::MIN);
            // Length and angle turn and stretch the line around its start.
            let current = line.length();
            let mut value = current;
            length(ui, Label::EdgeLength, &mut value, MIN_SIZE_MM);
            if value != current {
                *line = line.with_length(value);
            }
            let current = line.angle();
            let mut value = current;
            angle_field(ui, Label::Angle, &mut value);
            if value != current {
                *line = line.with_angle(value);
            }
            ui.separator();
            line_style_fields(ui, &mut line.style, units, language);
        }
    }
    // The radius is kept as typed; the ring reduces it where the corner's edges are too short.
    if let Some(ShapePart::Vertex(vertex)) = part
        && let Some(current) = shape.corner_radius(vertex)
    {
        ui.separator();
        ui.label(language.message(&Message::SelectedVertex { number: vertex + 1 }));
        let mut value = current;
        length(ui, Label::CornerRadius, &mut value, 0.0);
        if value != current
            && let Some(rounded) = shape.with_corner_radius(vertex, value)
        {
            *shape = rounded;
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
