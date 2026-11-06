use crate::model::{Bounds, PatternPiece, Project};

mod align_edges;
mod distribute;
mod model;
mod space;

use model::Item;
pub use model::{AlignTo, Arrange, Axis, Edge, Spacing};

/// Below this, an offset counts as no move.
const MIN_OFFSET_MM: f64 = 1.0e-9;

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
        Arrange::Align(_, at) => align_edges::align(&items, at, (low, high)),
        Arrange::Distribute(_, at) => {
            let order = sorted(&items, |item| item.at(at));
            distribute::distribute(&items, &order, at, to, (low, high))
        }
        Arrange::Space(_) => {
            let order = sorted(&items, |item| item.min);
            match (spacing, to.key()) {
                (Spacing::Fixed(gap), Some(key)) => space::pack(&items, &order, key, gap),
                _ => space::space(&items, &order, to, (low, high)),
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

    #[test]
    fn lines_align_like_other_children() {
        let mut project = holes();
        project.pieces[0].children.push(ChildShape {
            id: 3,
            op: BooleanOp::Subtract,
            shape: Shape::line_from_ends(
                Point::new(70.0, 80.0),
                Point::new(130.0, 80.0),
                false,
                crate::shapes::LineStyle::Solid,
            )
            .unwrap(),
        });
        let centered = moved_children(
            &project,
            &[3],
            Arrange::Align(Axis::Horizontal, Edge::Center),
            AlignTo::Parent,
        );
        let bounds = child_boxes(&centered)[2];
        assert_near(bounds.min_x, 20.0);
        assert_near(bounds.max_x, 80.0);

        let left = moved_children(
            &project,
            &[2, 3],
            Arrange::Align(Axis::Horizontal, Edge::Min),
            AlignTo::KeyObject(2),
        );
        assert_near(child_boxes(&left)[2].min_x, 50.0);
    }
}
