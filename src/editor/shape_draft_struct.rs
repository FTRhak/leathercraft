use leathercraft::model::Point;

#[derive(Clone, Copy)]
pub(super) struct ShapeDraft {
    pub(super) start: Point,
    pub(super) end: Point,
    /// Shift was held, which turns an ellipse into a circle and turns a regular polygon in steps.
    pub(super) constrained: bool,
}
