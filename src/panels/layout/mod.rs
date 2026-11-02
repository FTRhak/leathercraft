use super::icon_button;
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::{BooleanOp, ChildShape, PatternPiece, Project};
use leathercraft::shapes::Shape;

const ICON_BUTTON_SIZE: f32 = 20.0;
const ICON_MARGIN: f32 = 4.0;
const CHILD_INDENT: f32 = 16.0;

fn paint_lock(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke, locked: bool) {
    let rect = rect.shrink(ICON_MARGIN);
    let at = |x: f32, y: f32| rect.lerp_inside(egui::vec2(x, y));
    painter.rect_stroke(
        egui::Rect::from_min_max(at(0.1, 0.5), at(0.9, 1.0)),
        0.0,
        stroke,
        egui::StrokeKind::Middle,
    );
    // An open shackle stops short of the body on the right.
    let right_end = if locked { 0.5 } else { 0.25 };
    painter.add(egui::Shape::line(
        vec![
            at(0.28, 0.5),
            at(0.28, 0.15),
            at(0.4, 0.0),
            at(0.6, 0.0),
            at(0.72, 0.15),
            at(0.72, right_end),
        ],
        stroke,
    ));
}

fn paint_trash(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let rect = rect.shrink(ICON_MARGIN);
    let at = |x: f32, y: f32| rect.lerp_inside(egui::vec2(x, y));
    painter.line_segment([at(0.0, 0.2), at(1.0, 0.2)], stroke);
    painter.add(egui::Shape::line(
        vec![at(0.35, 0.2), at(0.35, 0.0), at(0.65, 0.0), at(0.65, 0.2)],
        stroke,
    ));
    painter.add(egui::Shape::line(
        vec![at(0.15, 0.2), at(0.22, 1.0), at(0.78, 1.0), at(0.85, 0.2)],
        stroke,
    ));
}

/// A triangle pointing up or down, for moving a child earlier or later in the list.
fn paint_arrow(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke, up: bool) {
    let rect = rect.shrink(ICON_MARGIN + 1.0);
    let at = |x: f32, y: f32| rect.lerp_inside(egui::vec2(x, if up { 1.0 - y } else { y }));
    painter.add(egui::Shape::closed_line(
        vec![at(0.0, 0.25), at(1.0, 0.25), at(0.5, 0.8)],
        stroke,
    ));
}

fn shape_label(shape: &Shape) -> Label {
    match shape {
        Shape::Polygon { .. } => Label::Polygon,
        Shape::Rectangle { .. } => Label::Rectangle,
        Shape::Ellipse { .. } => Label::Ellipse,
    }
}

/// Selection of a piece and, optionally, one of its children.
pub struct Selection<'a> {
    pub piece: &'a mut Option<u64>,
    pub child: &'a mut Option<(u64, u64)>,
}

/// Buttons right to left, then the op picker and the name, which takes the remaining width.
fn child_row(
    ui: &mut egui::Ui,
    piece: &PatternPiece,
    index: usize,
    child: &ChildShape,
    selection: &mut Selection,
    language: Language,
) -> Option<ProjectEvent> {
    let mut event = None;
    let (id, child_id) = (piece.id, child.id);
    ui.add_enabled_ui(!piece.locked, |ui| {
        let delete = language.label(Label::DeleteChild);
        if icon_button(ui, ICON_BUTTON_SIZE, false, delete, paint_trash).clicked() {
            event = Some(ProjectEvent::RemoveChild {
                id,
                child: child_id,
            });
        }
        let last = index + 1 == piece.children.len();
        ui.add_enabled_ui(!last, |ui| {
            let name = language.label(Label::MoveDown);
            let paint =
                |painter: &egui::Painter, rect, stroke| paint_arrow(painter, rect, stroke, false);
            if icon_button(ui, ICON_BUTTON_SIZE, false, name, paint).clicked() {
                event = Some(ProjectEvent::ReorderChild {
                    id,
                    child: child_id,
                    index: index + 1,
                });
            }
        });
        ui.add_enabled_ui(index > 0, |ui| {
            let name = language.label(Label::MoveUp);
            let paint =
                |painter: &egui::Painter, rect, stroke| paint_arrow(painter, rect, stroke, true);
            if icon_button(ui, ICON_BUTTON_SIZE, false, name, paint).clicked() {
                event = Some(ProjectEvent::ReorderChild {
                    id,
                    child: child_id,
                    index: index - 1,
                });
            }
        });
        let mut op = child.op;
        egui::ComboBox::from_id_salt(("child_op", id, child_id))
            .selected_text(language.label(Label::Operation(op)))
            .show_ui(ui, |ui| {
                for option in BooleanOp::ALL {
                    ui.selectable_value(&mut op, option, language.label(Label::Operation(option)));
                }
            });
        if op != child.op {
            event = Some(ProjectEvent::SetChildOp {
                id,
                child: child_id,
                op,
            });
        }
    });
    let selected = *selection.piece == Some(id) && *selection.child == Some((id, child_id));
    let text = format!(
        "{}. {}",
        index + 1,
        language.label(shape_label(&child.shape))
    );
    let row = egui::Button::selectable(selected, text)
        .truncate()
        .min_size(egui::vec2(ui.available_width() - CHILD_INDENT, 0.0));
    if ui.add(row).clicked() {
        *selection.piece = Some(id);
        *selection.child = Some((id, child_id));
    }
    event
}

fn piece_row(
    ui: &mut egui::Ui,
    piece: &PatternPiece,
    selection: &mut Selection,
    language: Language,
) -> Option<ProjectEvent> {
    let mut event = None;
    ui.add_enabled_ui(!piece.locked, |ui| {
        let name = language.label(Label::DeletePiece);
        if icon_button(
            ui,
            ICON_BUTTON_SIZE,
            false,
            name,
            |painter, rect, stroke| {
                paint_trash(painter, rect, stroke);
            },
        )
        .clicked()
        {
            event = Some(ProjectEvent::RemovePiece { id: piece.id });
        }
    });
    let name = language.label(if piece.locked {
        Label::UnlockPiece
    } else {
        Label::LockPiece
    });
    if icon_button(
        ui,
        ICON_BUTTON_SIZE,
        piece.locked,
        name,
        |painter, rect, stroke| paint_lock(painter, rect, stroke, piece.locked),
    )
    .clicked()
    {
        event = Some(ProjectEvent::SetPieceLocked {
            id: piece.id,
            locked: !piece.locked,
        });
    }
    let selected = *selection.piece == Some(piece.id) && selection.child.is_none();
    let row = egui::Button::selectable(selected, piece.name.as_str())
        .truncate()
        .min_size(egui::vec2(ui.available_width(), 0.0));
    if ui.add(row).clicked() {
        *selection.piece = Some(piece.id);
        *selection.child = None;
    }
    event
}

pub fn show_panel(
    ui: &mut egui::Ui,
    project: &Project,
    mut selection: Selection,
    language: Language,
) -> Option<ProjectEvent> {
    let mut event = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::CollapsingHeader::new(language.label(Label::Pieces))
            .id_salt("layout_pieces")
            .default_open(true)
            .show(ui, |ui| {
                for piece in &project.pieces {
                    let row = ui.push_id(piece.id, |ui| {
                        // Right to left, so that a long name gives way to the buttons.
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), ICON_BUTTON_SIZE),
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| piece_row(ui, piece, &mut selection, language),
                        )
                        .inner
                    });
                    event = row.inner.or(event.take());
                    // Children are listed in the order their operations apply.
                    for (index, child) in piece.children.iter().enumerate() {
                        let row = ui.push_id((piece.id, child.id), |ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), ICON_BUTTON_SIZE),
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| child_row(ui, piece, index, child, &mut selection, language),
                            )
                            .inner
                        });
                        event = row.inner.or(event.take());
                    }
                }
            });
    });
    event
}
