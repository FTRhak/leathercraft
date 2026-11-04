use crate::i18n::Message;
use crate::model::{
    BooleanOp, Bounds, MIN_POLYGON_AREA_MM2, PatternPiece, Point, ring_bounds, ring_contains,
    signed_area,
};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;

/// Removes the float noise that boolean operations leave in coordinates.
const ROUND_PRECISION_MM: f64 = 1.0e-6;

/// The leather a piece is made of: one outline with holes inside it.
#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    pub outline: Vec<Point>,
    pub holes: Vec<Vec<Point>>,
}

impl Region {
    /// The outline followed by each hole.
    pub fn rings(&self) -> impl Iterator<Item = &[Point]> {
        std::iter::once(self.outline.as_slice()).chain(self.holes.iter().map(Vec::as_slice))
    }

    /// Every point of `rings`, in order, as triangulation expects them.
    pub fn vertices(&self) -> impl Iterator<Item = &Point> {
        self.rings().flatten()
    }

    /// Where each hole starts within `vertices`.
    pub fn hole_indices(&self) -> Vec<usize> {
        let mut start = self.outline.len();
        self.holes
            .iter()
            .map(|hole| {
                let index = start;
                start += hole.len();
                index
            })
            .collect()
    }

    pub fn bounds(&self) -> Option<Bounds> {
        ring_bounds(&self.outline)
    }

    /// Whether the point is on the leather: inside the outline and not inside a hole.
    pub fn contains(&self, point: Point) -> bool {
        ring_contains(&self.outline, point)
            && !self.holes.iter().any(|hole| ring_contains(hole, point))
    }
}

fn contour(points: &[Point]) -> Vec<[f64; 2]> {
    points.iter().map(|point| [point.x, point.y]).collect()
}

fn ring(contour: &[[f64; 2]]) -> Vec<Point> {
    let round = |value: f64| (value / ROUND_PRECISION_MM).round() * ROUND_PRECISION_MM;
    contour
        .iter()
        .map(|[x, y]| Point::new(round(*x), round(*y)))
        .collect()
}

impl PatternPiece {
    /// Combines the base with each child in list order; open children, such as centerlines,
    /// are skipped. Fails if nothing is left or the result falls apart into several outlines,
    /// since a piece is cut as one.
    pub fn region(&self) -> Result<Region, Message> {
        let mut shapes = vec![vec![contour(&self.base.ring())]];
        for child in self.children.iter().filter(|child| !child.shape.is_open()) {
            let rule = match child.op {
                BooleanOp::Subtract => OverlayRule::Difference,
                BooleanOp::Union => OverlayRule::Union,
                BooleanOp::Intersect => OverlayRule::Intersect,
            };
            shapes = shapes.overlay(&contour(&child.shape.ring()), rule, FillRule::NonZero);
        }
        // Slivers left by shapes that only touch are not leather.
        shapes.retain(|shape| {
            shape
                .first()
                .is_some_and(|outline| signed_area(&ring(outline)).abs() >= MIN_POLYGON_AREA_MM2)
        });
        let name = self.name.clone();
        match shapes.as_slice() {
            [] => Err(Message::ResultEmpty { name }),
            [shape] => Ok(Region {
                outline: ring(&shape[0]),
                holes: shape[1..].iter().map(|hole| ring(hole)).collect(),
            }),
            _ => Err(Message::ResultSplit { name }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ChildShape;
    use crate::shapes::Shape;

    fn square(x: f64, y: f64, size: f64) -> Shape {
        Shape::rectangle_from_corners(Point::new(x, y), Point::new(x + size, y + size)).unwrap()
    }

    fn piece(children: Vec<(BooleanOp, Shape)>) -> PatternPiece {
        let mut piece = PatternPiece::new(
            1,
            "Pocket",
            Shape::rectangle_from_corners(Point::new(0.0, 0.0), Point::new(80.0, 120.0)).unwrap(),
        );
        piece.children = children
            .into_iter()
            .enumerate()
            .map(|(index, (op, shape))| ChildShape {
                id: index as u64 + 1,
                op,
                shape,
            })
            .collect();
        piece
    }

    fn area(region: &Region) -> f64 {
        signed_area(&region.outline).abs()
            - region
                .holes
                .iter()
                .map(|hole| signed_area(hole).abs())
                .sum::<f64>()
    }

    fn assert_area(region: &Region, expected: f64) {
        assert!(
            (area(region) - expected).abs() < 1.0e-3,
            "area {} != {expected}",
            area(region)
        );
    }

    #[test]
    fn the_base_alone_is_the_region() {
        let region = piece(Vec::new()).region().unwrap();
        assert_eq!(region.outline.len(), 4);
        assert!(region.holes.is_empty());
        assert_area(&region, 9600.0);
    }

    #[test]
    fn subtracting_inside_the_outline_makes_a_hole() {
        let region = piece(vec![(BooleanOp::Subtract, square(10.0, 10.0, 10.0))])
            .region()
            .unwrap();
        assert_eq!(region.holes.len(), 1);
        assert_area(&region, 9500.0);
        assert!(!region.contains(Point::new(15.0, 15.0)));
        assert!(region.contains(Point::new(30.0, 30.0)));
        assert_eq!(region.hole_indices(), vec![region.outline.len()]);
    }

    #[test]
    fn subtracting_across_the_outline_notches_the_edge() {
        let region = piece(vec![(BooleanOp::Subtract, square(70.0, 50.0, 20.0))])
            .region()
            .unwrap();
        assert!(region.holes.is_empty());
        assert_eq!(region.outline.len(), 8);
        assert_area(&region, 9400.0);
    }

    #[test]
    fn union_and_intersection_change_the_outline() {
        let tab = square(70.0, 50.0, 20.0);
        let united = piece(vec![(BooleanOp::Union, tab.clone())])
            .region()
            .unwrap();
        assert_area(&united, 9800.0);
        let crossed = piece(vec![(BooleanOp::Intersect, tab)]).region().unwrap();
        assert_area(&crossed, 200.0);
    }

    #[test]
    fn children_apply_in_list_order() {
        let hole = (BooleanOp::Subtract, square(10.0, 10.0, 20.0));
        // Half of the patch lies over the hole, so it partly fills it when applied after.
        let patch = (BooleanOp::Union, square(5.0, 15.0, 10.0));
        let patched = piece(vec![hole.clone(), patch.clone()]).region().unwrap();
        let cut = piece(vec![patch, hole]).region().unwrap();
        assert_area(&patched, 9600.0 - 400.0 + 50.0);
        assert_area(&cut, 9600.0 - 400.0);
    }

    #[test]
    fn a_subtraction_outside_the_piece_changes_nothing() {
        let region = piece(vec![(BooleanOp::Subtract, square(200.0, 200.0, 10.0))])
            .region()
            .unwrap();
        assert_area(&region, 9600.0);
    }

    #[test]
    fn empty_and_split_results_are_errors() {
        let name = "Pocket".to_owned();
        assert_eq!(
            piece(vec![(BooleanOp::Subtract, square(-10.0, -10.0, 200.0))]).region(),
            Err(Message::ResultEmpty { name: name.clone() })
        );
        let band =
            Shape::rectangle_from_corners(Point::new(-10.0, 50.0), Point::new(90.0, 60.0)).unwrap();
        assert_eq!(
            piece(vec![(BooleanOp::Subtract, band)]).region(),
            Err(Message::ResultSplit { name: name.clone() })
        );
        assert_eq!(
            piece(vec![(BooleanOp::Union, square(200.0, 200.0, 10.0))]).region(),
            Err(Message::ResultSplit { name })
        );
    }

    #[test]
    fn lines_do_not_change_the_region() {
        let line = Shape::line_from_ends(
            Point::new(-20.0, 60.0),
            Point::new(100.0, 60.0),
            false,
            crate::shapes::LineStyle::dash_dot_default(),
        )
        .unwrap();
        let region = piece(vec![(BooleanOp::Intersect, line)]).region().unwrap();
        assert_eq!(region, piece(Vec::new()).region().unwrap());
    }
}
