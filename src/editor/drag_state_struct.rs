use leathercraft::model::Point;

pub(super) struct DragState {
    pub(super) piece_id: u64,
    pub(super) vertex: Option<usize>,
    pub(super) start: Point,
    pub(super) original: Vec<Point>,
    /// The in-progress shape, drawn instead of the project's until the drag is committed.
    pub(super) points: Vec<Point>,
    /// Keeps the drag drawn for the frame in which its event is still being applied.
    pub(super) released: bool,
}
