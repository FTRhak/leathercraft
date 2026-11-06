/// The axis pieces move along: `Horizontal` is X, `Vertical` is Y, which grows downward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Which part of a box is lined up: left or top, center, right or bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Min,
    Center,
    Max,
}

/// The operations of Illustrator's Align panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrange {
    /// Moves each piece's edge or center onto the reference's.
    Align(Axis, Edge),
    /// Spaces the pieces' edges or centers evenly.
    Distribute(Axis, Edge),
    /// Makes the gaps between the pieces equal.
    Space(Axis),
}

impl Arrange {
    /// In the order of Illustrator's panel.
    pub const ALIGN: [Self; 6] = [
        Self::Align(Axis::Horizontal, Edge::Min),
        Self::Align(Axis::Horizontal, Edge::Center),
        Self::Align(Axis::Horizontal, Edge::Max),
        Self::Align(Axis::Vertical, Edge::Min),
        Self::Align(Axis::Vertical, Edge::Center),
        Self::Align(Axis::Vertical, Edge::Max),
    ];
    pub const DISTRIBUTE: [Self; 6] = [
        Self::Distribute(Axis::Vertical, Edge::Min),
        Self::Distribute(Axis::Vertical, Edge::Center),
        Self::Distribute(Axis::Vertical, Edge::Max),
        Self::Distribute(Axis::Horizontal, Edge::Min),
        Self::Distribute(Axis::Horizontal, Edge::Center),
        Self::Distribute(Axis::Horizontal, Edge::Max),
    ];
    pub const SPACE: [Self; 2] = [Self::Space(Axis::Vertical), Self::Space(Axis::Horizontal)];

    pub fn axis(self) -> Axis {
        match self {
            Self::Align(axis, _) | Self::Distribute(axis, _) | Self::Space(axis) => axis,
        }
    }
}

/// What the pieces are aligned to. Project bounds stand in for Illustrator's artboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignTo {
    Selection,
    /// A piece or child that stays where it is; the others line up with it.
    KeyObject(u64),
    Project,
    /// The base shape of the children's piece. Only children are aligned to it; `offsets`
    /// returns nothing for it.
    Parent,
}

impl AlignTo {
    /// A box the items are spread within, rather than the span of the items themselves.
    pub(super) fn is_outer(self) -> bool {
        matches!(self, Self::Project | Self::Parent)
    }

    pub(super) fn key(self) -> Option<u64> {
        match self {
            Self::KeyObject(key) => Some(key),
            _ => None,
        }
    }
}

/// The gap of `Arrange::Space`. A fixed gap needs a key object to measure from, as in
/// Illustrator; otherwise the gap is automatic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Spacing {
    Auto,
    Fixed(f64),
}

pub(super) struct Item {
    pub(super) id: u64,
    pub(super) min: f64,
    pub(super) max: f64,
    /// Locked pieces and the key object stay, but still count toward the reference.
    pub(super) fixed: bool,
}

impl Item {
    pub(super) fn at(&self, edge: Edge) -> f64 {
        match edge {
            Edge::Min => self.min,
            Edge::Center => (self.min + self.max) * 0.5,
            Edge::Max => self.max,
        }
    }

    pub(super) fn size(&self) -> f64 {
        self.max - self.min
    }
}
