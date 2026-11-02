use leathercraft::model::Point;

/// What a drag moves. Dragging the piece moves its holes along with it.
#[derive(Clone, Copy)]
pub(super) enum DragTarget {
    Piece,
    Vertex(usize),
    Hole(usize),
    HoleVertex { hole: usize, vertex: usize },
}

pub(super) struct DragState {
    pub(super) piece_id: u64,
    pub(super) target: DragTarget,
    pub(super) start: Point,
    pub(super) original: Vec<Point>,
    pub(super) original_holes: Vec<Vec<Point>>,
    /// The in-progress shape, drawn instead of the project's until the drag is committed.
    pub(super) points: Vec<Point>,
    pub(super) holes: Vec<Vec<Point>>,
    /// Keeps the drag drawn for the frame in which its event is still being applied.
    pub(super) released: bool,
}
