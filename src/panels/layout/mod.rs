use crate::icons::{glyph, icon_button, paint_glyph};
use bevy_egui::egui;
use bevy_egui::egui::collapsing_header::{CollapsingState, paint_default_icon};
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::{BooleanOp, ChildShape, PatternPiece, Project};
use leathercraft::shapes::Shape;

const ICON_BUTTON_SIZE: f32 = 20.0;
const ICON_MARGIN: f32 = 4.0;

/// A triangle that expands or collapses a piece's children, or empty space of the same width
/// for a piece without children, so that all names start at the same position.
fn expand_toggle(ui: &mut egui::Ui, state: &mut CollapsingState, has_children: bool) {
    let size = egui::Vec2::splat(ICON_BUTTON_SIZE);
    if !has_children {
        ui.allocate_exact_size(size, egui::Sense::hover());
        return;
    }
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), ""));
    if response.clicked() {
        state.toggle(ui);
    }
    let openness = state.openness(ui.ctx());
    let response = response.with_new_rect(rect.shrink(ICON_MARGIN + 1.0));
    paint_default_icon(ui, openness, &response);
}

fn paint_trash(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    paint_glyph(painter, rect, stroke, glyph::TRASH);
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
        Shape::Polygon(_) => Label::Polygon,
        Shape::Rectangle(_) => Label::Rectangle,
        Shape::Ellipse(_) => Label::Ellipse,
        Shape::RegularPolygon(_) => Label::RegularPolygon,
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
    let row = egui::Button::selectable(selected, (text, egui::Atom::grow()))
        .truncate()
        .min_size(egui::vec2(ui.available_width(), 0.0));
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
        |painter, rect, stroke| {
            let icon = if piece.locked {
                glyph::LOCK
            } else {
                glyph::UNLOCK
            };
            paint_glyph(painter, rect, stroke, icon);
        },
    )
    .clicked()
    {
        event = Some(ProjectEvent::SetPieceLocked {
            id: piece.id,
            locked: !piece.locked,
        });
    }
    let selected = *selection.piece == Some(piece.id) && selection.child.is_none();
    let row = egui::Button::selectable(selected, (piece.name.as_str(), egui::Atom::grow()))
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
                    let id = ui.make_persistent_id(("layout_piece", piece.id));
                    let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, true);
                    let has_children = !piece.children.is_empty();
                    let row = ui.push_id(piece.id, |ui| {
                        ui.horizontal(|ui| {
                            expand_toggle(ui, &mut state, has_children);
                            // Right to left, so that a long name gives way to the buttons.
                            ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), ICON_BUTTON_SIZE),
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| piece_row(ui, piece, &mut selection, language),
                            )
                            .inner
                        })
                        .inner
                    });
                    state.store(ui.ctx());
                    event = row.inner.or(event.take());
                    if !(has_children && state.is_open()) {
                        continue;
                    }
                    // Children are listed in the order their operations apply, indented so that
                    // their names start under the name of their piece.
                    for (index, child) in piece.children.iter().enumerate() {
                        let row = ui.push_id((piece.id, child.id), |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(ICON_BUTTON_SIZE);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(ui.available_width(), ICON_BUTTON_SIZE),
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        child_row(ui, piece, index, child, &mut selection, language)
                                    },
                                )
                                .inner
                            })
                            .inner
                        });
                        event = row.inner.or(event.take());
                    }
                }
            });
    });
    event
}
