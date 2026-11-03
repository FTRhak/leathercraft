use crate::model::{Bounds, PatternPiece, Project};

/// Below this, an offset counts as no move.
const MIN_OFFSET_MM: f64 = 1.0e-9;

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
    fn is_outer(self) -> bool {
        matches!(self, Self::Project | Self::Parent)
    }

    fn key(self) -> Option<u64> {
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

/// The box a piece is aligned by: its outline, or every shape while the result is invalid.
pub fn piece_box(piece: &PatternPiece) -> Option<Bounds> {
    piece
        .region()
        .ok()
        .and_then(|region| region.bounds())
        .or_else(|| piece.bounds())
}

fn span(bounds: Bounds, axis: Axis) -> (f64, f64) {
    match axis {
        Axis::Horizontal => (bounds.min_x, bounds.max_x),
        Axis::Vertical => (bounds.min_y, bounds.max_y),
    }
}

struct Item {
    id: u64,
    min: f64,
    max: f64,
    /// Locked pieces and the key object stay, but still count toward the reference.
    fixed: bool,
}

impl Item {
    fn at(&self, edge: Edge) -> f64 {
        match edge {
            Edge::Min => self.min,
            Edge::Center => (self.min + self.max) * 0.5,
            Edge::Max => self.max,
        }
    }

    fn size(&self) -> f64 {
        self.max - self.min
    }
}

/// The offset of each piece that `arrange` moves. Unknown and repeated ids are skipped.
pub fn offsets(
    project: &Project,
    ids: &[u64],
    arrange: Arrange,
    to: AlignTo,
    spacing: Spacing,
) -> Vec<(u64, f64, f64)> {
    let axis = arrange.axis();
    let key = to.key();
    let items = project
        .pieces
        .iter()
        .filter(|piece| ids.contains(&piece.id))
        .filter_map(|piece| {
            let (min, max) = span(piece_box(piece)?, axis);
            Some(Item {
                id: piece.id,
                min,
                max,
                fixed: piece.locked || key == Some(piece.id),
            })
        })
        .collect();
    let outer = match to {
        AlignTo::Selection | AlignTo::Parent => None,
        AlignTo::KeyObject(key) => project
            .pieces
            .iter()
            .find(|piece| piece.id == key)
            .and_then(piece_box),
        AlignTo::Project => project_box(project),
    };
    arrange_items(items, outer, arrange, to, spacing)
}

/// The offset of each child of the piece `id` that `arrange` moves, measured by its own shape.
/// Against `AlignTo::Parent`, children line up with the piece's base shape. Unknown and
/// repeated ids are skipped.
pub fn child_offsets(
    project: &Project,
    id: u64,
    children: &[u64],
    arrange: Arrange,
    to: AlignTo,
    spacing: Spacing,
) -> Vec<(u64, f64, f64)> {
    let Some(piece) = project.pieces.iter().find(|piece| piece.id == id) else {
        return Vec::new();
    };
    let axis = arrange.axis();
    let key = to.key();
    let items = piece
        .children
        .iter()
        .filter(|child| children.contains(&child.id))
        .filter_map(|child| {
            let (min, max) = span(child.shape.bounds()?, axis);
            Some(Item {
                id: child.id,
                min,
                max,
                fixed: key == Some(child.id),
            })
        })
        .collect();
    let outer = match to {
        AlignTo::Selection => None,
        AlignTo::KeyObject(key) => piece.child(key).and_then(|child| child.shape.bounds()),
        AlignTo::Project => project_box(project),
        AlignTo::Parent => piece.base.bounds(),
    };
    arrange_items(items, outer, arrange, to, spacing)
}

fn project_box(project: &Project) -> Option<Bounds> {
    project
        .pieces
        .iter()
        .filter_map(piece_box)
        .reduce(Bounds::union)
}

/// The offsets of the items that move. `outer` is the reference box, except against the
/// selection, which is measured from the items themselves.
fn arrange_items(
    items: Vec<Item>,
    outer: Option<Bounds>,
    arrange: Arrange,
    to: AlignTo,
    spacing: Spacing,
) -> Vec<(u64, f64, f64)> {
    let axis = arrange.axis();
    let reference = match to {
        AlignTo::Selection => items
            .iter()
            .map(|item| (item.min, item.max))
            .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1))),
        _ => outer.map(|bounds| span(bounds, axis)),
    };
    let Some((low, high)) = reference else {
        return Vec::new();
    };

    // The target of each item's min edge.
    let targets: Vec<(usize, f64)> = match arrange {
        Arrange::Align(_, at) => {
            let target = match at {
                Edge::Min => low,
                Edge::Center => (low + high) * 0.5,
                Edge::Max => high,
            };
            (0..items.len())
                .map(|index| (index, items[index].min + target - items[index].at(at)))
                .collect()
        }
        Arrange::Distribute(_, at) => {
            let order = sorted(&items, |item| item.at(at));
            distribute(&items, &order, at, to, (low, high))
        }
        Arrange::Space(_) => {
            let order = sorted(&items, |item| item.min);
            match (spacing, to.key()) {
                (Spacing::Fixed(gap), Some(key)) => pack(&items, &order, key, gap),
                _ => space(&items, &order, to, (low, high)),
            }
        }
    };

    targets
        .into_iter()
        .filter(|(index, _)| !items[*index].fixed)
        .map(|(index, min)| (items[index].id, min - items[index].min))
        .filter(|(_, offset)| offset.abs() >= MIN_OFFSET_MM)
        .map(|(id, offset)| match axis {
            Axis::Horizontal => (id, offset, 0.0),
            Axis::Vertical => (id, 0.0, offset),
        })
        .collect()
}

/// Item indices, ordered by `key`; ties keep the project order.
fn sorted(items: &[Item], key: impl Fn(&Item) -> f64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by(|a, b| key(&items[*a]).total_cmp(&key(&items[*b])));
    order
}

/// The outer items stay and the rest are spread between them; against the project or the
/// parent, the outer items move to its edges first.
fn distribute(
    items: &[Item],
    order: &[usize],
    at: Edge,
    to: AlignTo,
    (low, high): (f64, f64),
) -> Vec<(usize, f64)> {
    let (Some(&first), Some(&last)) = (order.first(), order.last()) else {
        return Vec::new();
    };
    if order.len() < 2 {
        return Vec::new();
    }
    let (start, end) = if to.is_outer() {
        (
            low + items[first].at(at) - items[first].min,
            high - (items[last].max - items[last].at(at)),
        )
    } else {
        (items[first].at(at), items[last].at(at))
    };
    let steps = (order.len() - 1) as f64;
    order
        .iter()
        .enumerate()
        .map(|(position, &index)| {
            let target = start + (end - start) * position as f64 / steps;
            (index, items[index].min + target - items[index].at(at))
        })
        .collect()
}

/// Equal gaps between the outer items, or between the edges of the project or the parent.
fn space(
    items: &[Item],
    order: &[usize],
    to: AlignTo,
    (low, high): (f64, f64),
) -> Vec<(usize, f64)> {
    if order.len() < 2 {
        return Vec::new();
    }
    let (start, end) = if to.is_outer() {
        (low, high)
    } else {
        (
            items[order[0]].min,
            order
                .iter()
                .map(|&index| items[index].max)
                .fold(f64::MIN, f64::max),
        )
    };
    let total: f64 = order.iter().map(|&index| items[index].size()).sum();
    let gap = (end - start - total) / (order.len() - 1) as f64;
    let mut position = start;
    order
        .iter()
        .map(|&index| {
            let target = position;
            position += items[index].size() + gap;
            (index, target)
        })
        .collect()
}

/// The key piece stays, and the others are packed away from it with `gap` between them.
fn pack(items: &[Item], order: &[usize], key: u64, gap: f64) -> Vec<(usize, f64)> {
    let Some(center) = order.iter().position(|&index| items[index].id == key) else {
        return Vec::new();
    };
    let mut targets = Vec::with_capacity(order.len());
    let mut position = items[order[center]].max + gap;
    for &index in &order[center + 1..] {
        targets.push((index, position));
        position += items[index].size() + gap;
    }
    let mut position = items[order[center]].min - gap;
    for &index in order[..center].iter().rev() {
        let min = position - items[index].size();
        targets.push((index, min));
        position = min - gap;
    }
    targets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BooleanOp, ChildShape, Point};
    use crate::shapes::Shape;

    fn square(id: u64, x: f64, y: f64, size: f64) -> PatternPiece {
        let base = Shape::rectangle_from_corners(Point::new(x, y), Point::new(x + size, y + size))
            .unwrap();
        PatternPiece::new(id, format!("Piece {id}"), base)
    }

    /// Squares of sizes 10, 20 and 10 at x = 0, 30 and 100, at different heights.
    fn project() -> Project {
        Project {
            pieces: vec![
                square(1, 0.0, 0.0, 10.0),
                square(2, 30.0, 50.0, 20.0),
                square(3, 100.0, 20.0, 10.0),
            ],
            ..Project::default()
        }
    }

    fn moved(project: &Project, ids: &[u64], arrange: Arrange, to: AlignTo) -> Project {
        moved_with(project, ids, arrange, to, Spacing::Auto)
    }

    fn moved_with(
        project: &Project,
        ids: &[u64],
        arrange: Arrange,
        to: AlignTo,
        spacing: Spacing,
    ) -> Project {
        let mut result = project.clone();
        for (id, dx, dy) in offsets(project, ids, arrange, to, spacing) {
            let piece = result
                .pieces
                .iter_mut()
                .find(|piece| piece.id == id)
                .unwrap();
            piece.translate(dx, dy);
        }
        result
    }

    fn boxes(project: &Project) -> Vec<Bounds> {
        project
            .pieces
            .iter()
            .map(|piece| piece_box(piece).unwrap())
            .collect()
    }

    fn assert_near(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1.0e-6, "{actual} != {expected}");
    }

    #[test]
    fn aligns_left_edges_to_the_selection() {
        let result = moved(
            &project(),
            &[1, 2, 3],
            Arrange::Align(Axis::Horizontal, Edge::Min),
            AlignTo::Selection,
        );
        for bounds in boxes(&result) {
            assert_near(bounds.min_x, 0.0);
        }
        assert_near(boxes(&result)[1].min_y, 50.0);
    }

    #[test]
    fn the_key_object_stays_put() {
        let result = moved(
            &project(),
            &[1, 2, 3],
            Arrange::Align(Axis::Vertical, Edge::Max),
            AlignTo::KeyObject(2),
        );
        for bounds in boxes(&result) {
            assert_near(bounds.max_y, 70.0);
        }
        assert_eq!(result.pieces[1], project().pieces[1]);
    }

    #[test]
    fn a_single_piece_aligns_to_the_project() {
        let result = moved(
            &project(),
            &[1],
            Arrange::Align(Axis::Horizontal, Edge::Center),
            AlignTo::Project,
        );
        assert_near(boxes(&result)[0].min_x, 50.0);
    }

    #[test]
    fn distributes_centers_evenly() {
        let result = moved(
            &project(),
            &[1, 2, 3],
            Arrange::Distribute(Axis::Horizontal, Edge::Center),
            AlignTo::Selection,
        );
        let centers: Vec<f64> = boxes(&result)
            .iter()
            .map(|bounds| (bounds.min_x + bounds.max_x) * 0.5)
            .collect();
        assert_near(centers[0], 5.0);
        assert_near(centers[1], 55.0);
        assert_near(centers[2], 105.0);
    }

    #[test]
    fn automatic_spacing_makes_equal_gaps() {
        let result = moved(
            &project(),
            &[1, 2, 3],
            Arrange::Space(Axis::Horizontal),
            AlignTo::Selection,
        );
        let boxes = boxes(&result);
        assert_near(boxes[1].min_x - boxes[0].max_x, 35.0);
        assert_near(boxes[2].min_x - boxes[1].max_x, 35.0);
        assert_near(boxes[2].min_x, 100.0);
    }

    #[test]
    fn fixed_spacing_packs_away_from_the_key() {
        let result = moved_with(
            &project(),
            &[1, 2, 3],
            Arrange::Space(Axis::Horizontal),
            AlignTo::KeyObject(2),
            Spacing::Fixed(5.0),
        );
        let boxes = boxes(&result);
        assert_near(boxes[0].max_x, 25.0);
        assert_near(boxes[1].min_x, 30.0);
        assert_near(boxes[2].min_x, 55.0);
    }

    #[test]
    fn locked_pieces_stay_but_widen_the_selection() {
        let mut project = project();
        project.pieces[0].locked = true;
        let result = moved(
            &project,
            &[1, 2, 3],
            Arrange::Align(Axis::Horizontal, Edge::Min),
            AlignTo::Selection,
        );
        assert_eq!(result.pieces[0], project.pieces[0]);
        assert_near(boxes(&result)[2].min_x, 0.0);
    }

    /// A 100 mm square piece with square holes of sizes 10 and 20 at (10, 10) and (50, 30).
    fn holes() -> Project {
        let mut piece = square(1, 0.0, 0.0, 100.0);
        for (id, x, y, size) in [(1, 10.0, 10.0, 10.0), (2, 50.0, 30.0, 20.0)] {
            piece.children.push(ChildShape {
                id,
                op: BooleanOp::Subtract,
                shape: square(0, x, y, size).base,
            });
        }
        Project {
            pieces: vec![piece],
            ..Project::default()
        }
    }

    fn moved_children(project: &Project, ids: &[u64], arrange: Arrange, to: AlignTo) -> Project {
        let mut result = project.clone();
        for (id, dx, dy) in child_offsets(project, 1, ids, arrange, to, Spacing::Auto) {
            let piece = &mut result.pieces[0];
            let index = piece.child_index(id).unwrap();
            piece.children[index].shape.translate(dx, dy);
        }
        result
    }

    fn child_boxes(project: &Project) -> Vec<Bounds> {
        project.pieces[0]
            .children
            .iter()
            .map(|child| child.shape.bounds().unwrap())
            .collect()
    }

    #[test]
    fn centers_a_child_in_its_parent() {
        let center =
            |project: &Project, arrange| moved_children(project, &[1], arrange, AlignTo::Parent);
        let result = center(&holes(), Arrange::Align(Axis::Horizontal, Edge::Center));
        let result = center(&result, Arrange::Align(Axis::Vertical, Edge::Center));
        let bounds = child_boxes(&result)[0];
        assert_near(bounds.min_x, 45.0);
        assert_near(bounds.min_y, 45.0);
        assert_eq!(result.pieces[0].children[1], holes().pieces[0].children[1]);
    }

    #[test]
    fn aligns_children_to_their_selection() {
        let result = moved_children(
            &holes(),
            &[1, 2],
            Arrange::Align(Axis::Vertical, Edge::Max),
            AlignTo::Selection,
        );
        for bounds in child_boxes(&result) {
            assert_near(bounds.max_y, 50.0);
        }
    }

    #[test]
    fn the_key_child_stays_put() {
        let result = moved_children(
            &holes(),
            &[1, 2],
            Arrange::Align(Axis::Horizontal, Edge::Min),
            AlignTo::KeyObject(2),
        );
        assert_eq!(result.pieces[0].children[1], holes().pieces[0].children[1]);
        assert_near(child_boxes(&result)[0].min_x, 50.0);
    }

    #[test]
    fn distributes_children_across_their_parent() {
        let result = moved_children(
            &holes(),
            &[1, 2],
            Arrange::Distribute(Axis::Horizontal, Edge::Min),
            AlignTo::Parent,
        );
        let boxes = child_boxes(&result);
        assert_near(boxes[0].min_x, 0.0);
        assert_near(boxes[1].max_x, 100.0);
    }

    #[test]
    fn pieces_have_no_parent() {
        let offsets = offsets(
            &project(),
            &[1, 2],
            Arrange::Align(Axis::Horizontal, Edge::Min),
            AlignTo::Parent,
            Spacing::Auto,
        );
        assert!(offsets.is_empty());
    }

    #[test]
    fn an_empty_result_falls_back_to_its_shapes() {
        let mut project = project();
        let cover = square(0, -5.0, -5.0, 40.0).base;
        project.pieces[0].children.push(ChildShape {
            id: 1,
            op: BooleanOp::Subtract,
            shape: cover,
        });
        assert!(project.pieces[0].region().is_err());
        assert_near(piece_box(&project.pieces[0]).unwrap().min_x, -5.0);
    }
}
