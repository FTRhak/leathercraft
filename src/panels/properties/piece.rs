use super::Selection;
use super::fields::{ROW_GAP, section_header};
use super::shape::shape_fields;
use super::stitch::stitch_details;
use crate::editor::format_length;
use crate::ui_components::button::Button;
use crate::ui_components::input::TextInput;
use crate::ui_components::palette::Palette;
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::Units;
use leathercraft::shapes::Shape;

const SIZE_BOX_HEIGHT: f32 = 28.0;
const SIZE_BOX_PADDING: f32 = 8.0;
const SIZE_BOX_RADIUS: u8 = 5;
const SIZE_PREFIX_SIZE: f32 = 11.0;

/// A read-only box with a one-letter prefix at the left and a length at the right, as the
/// "Width Input" of the design; the size is the result of the shapes, so it can't be typed.
fn size_box(ui: &mut egui::Ui, width: f32, prefix: &str, value: &str, name: &str) {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, SIZE_BOX_HEIGHT), egui::Sense::hover());
    response.on_hover_text(name);
    if !ui.is_rect_visible(rect) {
        return;
    }
    let palette = Palette::of(ui.painter());
    let painter = ui.painter();
    painter.rect(
        rect,
        SIZE_BOX_RADIUS,
        palette.surface.input,
        egui::Stroke::new(1.0, palette.border.control),
        egui::StrokeKind::Inside,
    );
    let content = rect.shrink2(egui::vec2(SIZE_BOX_PADDING, 0.0));
    painter.text(
        content.left_center(),
        egui::Align2::LEFT_CENTER,
        prefix,
        egui::FontId::proportional(SIZE_PREFIX_SIZE),
        palette.properties.size_prefix,
    );
    painter.text(
        content.right_center(),
        egui::Align2::RIGHT_CENTER,
        value,
        egui::TextStyle::Monospace.resolve(ui.style()),
        palette.text.primary,
    );
}

/// The width and height of the piece's outline, side by side.
fn size_row(ui: &mut egui::Ui, width_mm: f64, height_mm: f64, units: Units, language: Language) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = ROW_GAP;
        let width = ((ui.available_width() - ROW_GAP) / 2.0).max(0.0);
        for (prefix, name, millimeters) in [
            (Label::WidthShort, Label::Width, width_mm),
            (Label::HeightShort, Label::Height, height_mm),
        ] {
            let value = format_length(language, units, millimeters);
            let (prefix, name) = (language.label(prefix), language.label(name));
            size_box(ui, width, prefix, &value, name);
        }
    });
}

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

    section_header(
        ui,
        language.label(Label::PieceParameters),
        Some(&piece.name),
    );

    // The buffer lets the box be empty mid-edit, which the project itself would reject.
    let mut name = selection
        .rename_buffer
        .clone()
        .unwrap_or_else(|| piece.name.clone());
    let field = TextInput::new(&mut name)
        .id_salt("piece_name")
        .hint(language.label(Label::PieceName))
        .width(ui.available_width());
    let response = ui.add(field);
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
                size_row(ui, bounds.width(), bounds.height(), units, language);
            }
        }
        Err(error) => {
            ui.colored_label(ui.visuals().error_fg_color, language.message(&error));
        }
    }

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
    ui.add_space(ROW_GAP / 2.0);
    section_header(ui, &title, None);
    let mut edited = shape.clone();
    ui.add_enabled_ui(!piece.locked, |ui| {
        ui.spacing_mut().item_spacing.y = ROW_GAP;
        shape_fields(ui, &mut edited, selection.part, units, language);
        if let Shape::StitchLine(line) = shape {
            stitch_details(ui, line, piece.material.thickness_mm, units, language);
        }
        if !matches!(shape, Shape::Polygon(_))
            && !shape.is_open()
            && ui
                .add(Button::outline(language.label(Label::ToPolygon)))
                .clicked()
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
