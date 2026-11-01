use super::{DragState, EditorState, Viewport, piece_at, send, vertex_at};
use crate::document::ProjectMessage;
use bevy::prelude::*;
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::model::{Point, Project};

pub(super) fn begin_drag(
    origin: egui::Pos2,
    view: Viewport,
    project: &Project,
    state: &mut EditorState,
) {
    let selected = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id));
    let vertex = selected.and_then(|piece| vertex_at(piece, origin, view));
    let start = view.to_world(origin);
    let piece_id = match (vertex, selected) {
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
    if piece.locked {
        return;
    }
    state.drag = Some(DragState {
        piece_id,
        vertex,
        start,
        original: piece.points.clone(),
        points: piece.points.clone(),
        released: false,
    });
}

pub(super) fn update_drag(cursor: Point, state: &mut EditorState) {
    let Some(drag) = &state.drag else {
        return;
    };
    let mut points = drag.original.clone();
    match drag.vertex {
        Some(vertex) => points[vertex] = state.snap_point(cursor),
        None => {
            // Snap the offset, not the points, so the piece keeps its own shape.
            let dx = state.snap(cursor.x - drag.start.x);
            let dy = state.snap(cursor.y - drag.start.y);
            for point in &mut points {
                point.x += dx;
                point.y += dy;
            }
        }
    }
    if let Some(drag) = &mut state.drag {
        drag.points = points;
    }
}

pub(super) fn finish_drag(events: &mut MessageWriter<ProjectMessage>, state: &mut EditorState) {
    let Some(drag) = &mut state.drag else {
        return;
    };
    drag.released = true;
    if drag.points != drag.original {
        send(
            events,
            ProjectEvent::SetPiecePoints {
                id: drag.piece_id,
                points: drag.points.clone(),
            },
        );
    }
}
