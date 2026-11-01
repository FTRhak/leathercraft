use super::icon_button;
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::i18n::{Label, Language};
use leathercraft::model::{PatternPiece, Project};

const ICON_BUTTON_SIZE: f32 = 20.0;
const ICON_MARGIN: f32 = 4.0;

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

fn piece_row(
    ui: &mut egui::Ui,
    piece: &PatternPiece,
    selected: &mut Option<u64>,
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
    let row = egui::Button::selectable(*selected == Some(piece.id), piece.name.as_str())
        .truncate()
        .min_size(egui::vec2(ui.available_width(), 0.0));
    if ui.add(row).clicked() {
        *selected = Some(piece.id);
    }
    event
}

pub fn show_panel(
    ui: &mut egui::Ui,
    project: &Project,
    selected: &mut Option<u64>,
    language: Language,
) -> Option<ProjectEvent> {
    let mut event = None;
    ui.label(language.label(Label::Layout));
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
                            |ui| piece_row(ui, piece, selected, language),
                        )
                        .inner
                    });
                    event = row.inner.or(event.take());
                }
            });
    });
    event
}
