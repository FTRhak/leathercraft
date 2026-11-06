use super::canvas::{Viewport, child_at, handle_at, piece_at};
use super::drag_state_struct::ShapeRef;
use super::{DragState, DragTarget, EditorState, send};
use crate::document::ProjectMessage;
use bevy::prelude::*;
use bevy_egui::egui;
use leathercraft::events::ProjectEvent;
use leathercraft::model::{PatternPiece, Point, Project};
use leathercraft::snap::{self, Moving, SnapScope, SnapTargets};

/// What the press grabs on the selected piece: handles first, then a child, then the piece.
fn target_on_selected(
    piece: &PatternPiece,
    selected_child: Option<u64>,
    origin: egui::Pos2,
    start: Point,
    view: Viewport,
) -> Option<DragTarget> {
    if let Some((shape, handle)) = handle_at(piece, selected_child, origin, view) {
        return Some(DragTarget::Handle(shape, handle));
    }
    child_at(piece, start, view.hit_tolerance()).map(DragTarget::Child)
}

/// Grabbing a whole piece or a child that is selected moves the whole selection of pieces or
/// children; with Shift, an unselected one is added to it first.
pub(super) fn begin_drag(
    origin: egui::Pos2,
    shift: bool,
    view: Viewport,
    project: &Project,
    state: &mut EditorState,
) {
    let selected = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id));
    let start = view.to_world(origin);
    let target = selected.and_then(|piece| {
        target_on_selected(piece, state.selected_child_id(), origin, start, view)
    });
    let piece_id = match (target, selected) {
        (Some(_), Some(piece)) => Some(piece.id),
        _ => piece_at(project, start, view.hit_tolerance()),
    };
    let Some(piece_id) = piece_id else {
        return;
    };
    let Some(piece) = project.pieces.iter().find(|piece| piece.id == piece_id) else {
        return;
    };
    let child = match target {
        Some(DragTarget::Child(child) | DragTarget::Handle(ShapeRef::Child(child), _)) => {
            Some(child)
        }
        _ => None,
    };
    let child_selected = |state: &EditorState, child: u64| {
        state.selected == Some(piece_id) && state.selected_children().contains(&child)
    };
    match (target, child) {
        (Some(_), Some(child)) if child_selected(state, child) => state.focus_child(child),
        (Some(DragTarget::Child(_)), Some(child)) if shift => state.toggle_child(piece_id, child),
        (Some(_), _) => state.select(Some(piece_id), child),
        (None, _) if state.is_selected(piece_id) => state.focus(piece_id),
        (None, _) if shift => state.toggle_piece(piece_id),
        (None, _) => state.select(Some(piece_id), None),
    }
    if piece.locked {
        return;
    }
    let group = match target {
        Some(_) => Vec::new(),
        None => project
            .pieces
            .iter()
            .filter(|other| other.id != piece_id && !other.locked && state.is_selected(other.id))
            .map(|other| other.id)
            .collect(),
    };
    let child_group = match target {
        Some(DragTarget::Child(child)) => state
            .selected_children()
            .iter()
            .copied()
            .filter(|other| *other != child)
            .collect(),
        _ => Vec::new(),
    };
    let target = target.unwrap_or(DragTarget::Piece);
    let (targets, moving) = snap_setup(project, piece, target, &group, &child_group, start);
    state.drag = Some(DragState {
        piece_id,
        target,
        group,
        child_group,
        start,
        offset: (0.0, 0.0),
        original: piece.clone(),
        current: piece.clone(),
        targets,
        moving,
        released: false,
    });
}

/// What a drag snaps to, and what it moves. A piece snaps to the other pieces; a child, or a
/// handle of one, only to the rest of its own piece; a handle of the base to everything but
/// itself and the box of its own piece.
fn snap_setup(
    project: &Project,
    piece: &PatternPiece,
    target: DragTarget,
    group: &[u64],
    child_group: &[u64],
    start: Point,
) -> (SnapTargets, Option<Moving>) {
    match target {
        DragTarget::Piece => {
            let exclude: Vec<u64> = [piece.id]
                .into_iter()
                .chain(group.iter().copied())
                .collect();
            let shapes = project
                .pieces
                .iter()
                .filter(|other| exclude.contains(&other.id))
                .flat_map(|other| {
                    std::iter::once(&other.base)
                        .chain(other.children.iter().map(|child| &child.shape))
                });
            let moving = Moving::shapes(shapes, start);
            let targets = SnapTargets::collect(project, &SnapScope::Project { exclude });
            (targets, moving)
        }
        DragTarget::Child(child) => {
            let exclude: Vec<u64> = [child]
                .into_iter()
                .chain(child_group.iter().copied())
                .collect();
            let shapes = piece
                .children
                .iter()
                .filter(|other| exclude.contains(&other.id))
                .map(|other| &other.shape);
            let moving = Moving::shapes(shapes, start);
            let scope = SnapScope::Piece {
                id: piece.id,
                exclude,
            };
            (SnapTargets::collect(project, &scope), moving)
        }
        DragTarget::Handle(shape, handle) => {
            let Some(dragged) = shape.get(piece) else {
                return (SnapTargets::default(), None);
            };
            let (scope, own_box) = match shape {
                ShapeRef::Base => (
                    SnapScope::Project {
                        exclude: Vec::new(),
                    },
                    piece.bounds(),
                ),
                ShapeRef::Child(_) => (
                    SnapScope::Piece {
                        id: piece.id,
                        exclude: Vec::new(),
                    },
                    dragged.bounds(),
                ),
            };
            let mut targets = SnapTargets::collect(project, &scope).without_box(own_box);
            if let Some(point) = dragged.handles().get(handle) {
                targets = targets.without_point(*point);
            }
            (targets, None)
        }
    }
}

/// Starts dragging a handle of the selected piece without changing the selection, so that the
/// Edge tool keeps its selected edge.
pub(super) fn begin_handle_drag(
    origin: egui::Pos2,
    view: Viewport,
    project: &Project,
    state: &mut EditorState,
) {
    let Some(piece) = state
        .selected
        .and_then(|id| project.pieces.iter().find(|piece| piece.id == id))
    else {
        return;
    };
    if piece.locked {
        return;
    }
    let Some((shape, handle)) = handle_at(piece, state.selected_child_id(), origin, view) else {
        return;
    };
    let target = DragTarget::Handle(shape, handle);
    let start = view.to_world(origin);
    let (targets, moving) = snap_setup(project, piece, target, &[], &[], start);
    state.drag = Some(DragState {
        piece_id: piece.id,
        target,
        group: Vec::new(),
        child_group: Vec::new(),
        start,
        offset: (0.0, 0.0),
        original: piece.clone(),
        current: piece.clone(),
        targets,
        moving,
        released: false,
    });
}

/// `tolerance` is how far, in mm, a vertex or guide reaches.
pub(super) fn update_drag(cursor: Point, tolerance: f64, state: &mut EditorState) {
    let settings = state.active_snap();
    let Some(drag) = &state.drag else {
        return;
    };
    let mut current = drag.original.clone();
    let raw = (cursor.x - drag.start.x, cursor.y - drag.start.y);
    // Snap the offset, not the points, so that what moves keeps its own shape.
    let ((dx, dy), mut feedback) = match &drag.moving {
        Some(moving) => snap::snap_offset(raw, moving, &drag.targets, &settings, tolerance),
        None => (raw, Default::default()),
    };
    match drag.target {
        DragTarget::Piece => current.translate(dx, dy),
        DragTarget::Child(child) => {
            for child in [child].iter().chain(&drag.child_group) {
                if let Some(shape) = ShapeRef::Child(*child).get_mut(&mut current) {
                    shape.translate(dx, dy);
                }
            }
        }
        DragTarget::Handle(shape, handle) => {
            let (point, handle_feedback) =
                snap::snap_point(cursor, &drag.targets, &settings, tolerance);
            feedback = handle_feedback;
            if let Some(shape) = shape.get_mut(&mut current) {
                *shape = shape.with_handle_at(handle, point);
            }
        }
    }
    state.snap_feedback = feedback;
    if let Some(drag) = &mut state.drag {
        drag.current = current;
        drag.offset = (dx, dy);
    }
}

pub(super) fn finish_drag(events: &mut MessageWriter<ProjectMessage>, state: &mut EditorState) {
    let Some(drag) = &mut state.drag else {
        return;
    };
    drag.released = true;
    if drag.current == drag.original {
        return;
    }
    let id = drag.piece_id;
    let shape = match drag.target {
        DragTarget::Piece => {
            let (dx, dy) = drag.offset;
            let event = if drag.group.is_empty() {
                ProjectEvent::MovePiece { id, dx, dy }
            } else {
                let ids = [id].into_iter().chain(drag.group.iter().copied()).collect();
                ProjectEvent::MovePieces { ids, dx, dy }
            };
            send(events, event);
            return;
        }
        DragTarget::Child(child) if !drag.child_group.is_empty() => {
            let (dx, dy) = drag.offset;
            let children = [child]
                .into_iter()
                .chain(drag.child_group.iter().copied())
                .collect();
            send(
                events,
                ProjectEvent::MoveChildren {
                    id,
                    children,
                    dx,
                    dy,
                },
            );
            return;
        }
        DragTarget::Child(child) => ShapeRef::Child(child),
        DragTarget::Handle(shape, _) => shape,
    };
    let Some(changed) = shape.get(&drag.current).cloned() else {
        return;
    };
    let event = match shape {
        ShapeRef::Base => ProjectEvent::SetBaseShape { id, shape: changed },
        ShapeRef::Child(child) => ProjectEvent::SetChildShape {
            id,
            child,
            shape: changed,
        },
    };
    send(events, event);
}
