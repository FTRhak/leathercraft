/// What the preview shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PreviewMode {
    /// Each piece flat on the ground, with its holes cut through.
    Flat,
    /// The pieces sewn together along their seam links.
    #[default]
    Sewn,
}
