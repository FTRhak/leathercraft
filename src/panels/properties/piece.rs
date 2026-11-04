use super::Selection;
use super::shape::shape_fields;
use super::stitch::stitch_details;
use crate::editor::format_length;
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Units;
use leathercraft::shapes::Shape;

/// The name, size or result error of the selected piece, then the fields of its selected base
/// or child, with a To Polygon button for a primitive.
pub(super) fn selected_piece_details(
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
            let kind = match child.shape {
                Shape::Line(_) => Label::Centerline,
                Shape::StitchLine(_) => Label::StitchHoles,
                _ => Label::Operation(child.op),
            };
            (format!("{index}. {}", language.label(kind)), &child.shape)
        }
        None => (language.label(Label::Base).to_owned(), &piece.base),
    };
    ui.strong(title);
    let mut edited = shape.clone();
    ui.add_enabled_ui(!piece.locked, |ui| {
        shape_fields(ui, &mut edited, selection.part, units, language);
        if let Shape::StitchLine(line) = shape {
            stitch_details(ui, line, piece.material.thickness_mm, units, language);
        }
        if !matches!(shape, Shape::Polygon(_))
            && !shape.is_open()
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
