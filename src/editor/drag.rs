use super::canvas::{Viewport, hole_vertex_at, piece_at, vertex_at};
use super::{DragState, DragTarget, EditorState, send};
use crate::document::ProjectMessage;
use bevy::prelude::*;
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::model::{PatternPiece, Point, Project};

/// What the press grabs on the selected piece, nearest handles first.
fn target_on_selected(
    piece: &PatternPiece,
    origin: egui::Pos2,
    start: Point,
    view: Viewport,
) -> Option<DragTarget> {
    if let Some((hole, vertex)) = hole_vertex_at(piece, origin, view) {
        return Some(DragTarget::HoleVertex { hole, vertex });
    }
    if let Some(vertex) = vertex_at(piece, origin, view) {
        return Some(DragTarget::Vertex(vertex));
    }
    piece.hole_at(start).map(DragTarget::Hole)
}

pub(super) fn begin_drag(
    origin: egui::Pos2,
    view: Viewport,
    project: &Project,
    state: &mut EditorState,
) {
    let selected = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id));
    let start = view.to_world(origin);
    let target = selected.and_then(|piece| target_on_selected(piece, origin, start, view));
    let piece_id = match (target, selected) {
        (Some(_), Some(piece)) => Some(piece.id),
        _ => piece_at(project, start),
    };
    let Some(piece_id) = piece_id else {
        return;
    };
    let Some(piece) = project.pieces.iter().find(|piece| piece.id == piece_id) else {
        return;
    };
    state.selected = Some(piece_id);
    state.selected_hole = match target {
        Some(DragTarget::Hole(hole) | DragTarget::HoleVertex { hole, .. }) => {
            Some((piece_id, hole))
        }
        _ => None,
    };
    if piece.locked {
        return;
    }
    state.drag = Some(DragState {
        piece_id,
        target: target.unwrap_or(DragTarget::Piece),
        start,
        original: piece.points.clone(),
        original_holes: piece.holes.clone(),
        points: piece.points.clone(),
        holes: piece.holes.clone(),
        released: false,
    });
}

fn translate(points: &mut [Point], dx: f64, dy: f64) {
    for point in points {
        point.x += dx;
        point.y += dy;
    }
}

pub(super) fn update_drag(cursor: Point, state: &mut EditorState) {
    let Some(drag) = &state.drag else {
        return;
    };
    let mut points = drag.original.clone();
    let mut holes = drag.original_holes.clone();
    // Snap the offset, not the points, so that what moves keeps its own shape.
    let dx = state.snap(cursor.x - drag.start.x);
    let dy = state.snap(cursor.y - drag.start.y);
    match drag.target {
        DragTarget::Vertex(vertex) => points[vertex] = state.snap_point(cursor),
        DragTarget::HoleVertex { hole, vertex } => {
            holes[hole][vertex] = state.snap_point(cursor);
        }
        DragTarget::Hole(hole) => translate(&mut holes[hole], dx, dy),
        DragTarget::Piece => {
            translate(&mut points, dx, dy);
            for hole in &mut holes {
                translate(hole, dx, dy);
            }
        }
    }
    if let Some(drag) = &mut state.drag {
        drag.points = points;
        drag.holes = holes;
    }
}

pub(super) fn finish_drag(events: &mut MessageWriter<ProjectMessage>, state: &mut EditorState) {
    let Some(drag) = &mut state.drag else {
        return;
    };
    drag.released = true;
    if drag.points != drag.original || drag.holes != drag.original_holes {
        send(
            events,
            ProjectEvent::SetPieceGeometry {
                id: drag.piece_id,
                points: drag.points.clone(),
                holes: drag.holes.clone(),
            },
        );
    }
}
