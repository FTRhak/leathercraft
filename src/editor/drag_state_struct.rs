use leathercraft::model::{PatternPiece, Point};
use leathercraft::shapes::Shape;

/// One of the shapes a piece is built from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShapeRef {
    Base,
    Child(u64),
}

impl ShapeRef {
    pub(super) fn get(self, piece: &PatternPiece) -> Option<&Shape> {
        match self {
            Self::Base => Some(&piece.base),
            Self::Child(id) => piece.child(id).map(|child| &child.shape),
        }
    }

    pub(super) fn get_mut(self, piece: &mut PatternPiece) -> Option<&mut Shape> {
        match self {
            Self::Base => Some(&mut piece.base),
            Self::Child(id) => piece
                .children
                .iter_mut()
                .find(|child| child.id == id)
                .map(|child| &mut child.shape),
        }
    }
}

/// What a drag moves. Dragging the piece moves its children along with it.
#[derive(Clone, Copy)]
pub(super) enum DragTarget {
    Piece,
    Child(u64),
    /// A handle from `Shape::handles`.
    Handle(ShapeRef, usize),
}

pub(super) struct DragState {
    pub(super) piece_id: u64,
    pub(super) target: DragTarget,
    pub(super) start: Point,
    /// The snapped offset of a whole-piece drag.
    pub(super) offset: (f64, f64),
    pub(super) original: PatternPiece,
    /// The in-progress piece, drawn instead of the project's until the drag is committed.
    pub(super) current: PatternPiece,
    /// Keeps the drag drawn for the frame in which its event is still being applied.
    pub(super) released: bool,
}
