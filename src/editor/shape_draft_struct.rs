use leathercraft::model::Point;

#[derive(Clone, Copy)]
pub(super) struct ShapeDraft {
    pub(super) start: Point,
    pub(super) end: Point,
    /// Shift was held, which turns an ellipse into a circle.
    pub(super) circle: bool,
}
