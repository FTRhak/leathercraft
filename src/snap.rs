//! Snapping of points and moved geometry: to the vertices of other shapes, to smart guides
//! (lining up the edges and centers of bounding boxes), and to a grid around the origin.
//! The editor decides what can be snapped to (`SnapTargets`); this module only measures.

use crate::model::{Bounds, PatternPiece, Point, Project};
use crate::shapes::Shape;
use serde::{Deserialize, Serialize};

pub const MIN_GRID_STEP_MM: f64 = 1.0;
pub const MAX_GRID_STEP_MM: f64 = 10.0;
pub const MIN_DISTANCE_PX: f32 = 2.0;
pub const MAX_DISTANCE_PX: f32 = 30.0;

/// What snaps, and how close it must come. Interface state, saved with the app settings.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SnapSettings {
    /// Turns all snapping on or off.
    pub enabled: bool,
    pub grid: bool,
    /// The grid step, counted from the origin.
    pub grid_step_mm: f64,
    pub vertices: bool,
    /// Lining up the edges and centers of bounding boxes.
    pub guides: bool,
    /// How close, on screen, a vertex or guide has to come before it snaps.
    pub distance_px: f32,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            grid: true,
            grid_step_mm: 1.0,
            vertices: true,
            guides: true,
            distance_px: 8.0,
        }
    }
}

impl SnapSettings {
    /// The settings with their numbers brought into range, for values read from a file.
    pub fn clamped(self) -> Self {
        let step = if self.grid_step_mm.is_finite() {
            self.grid_step_mm
        } else {
            1.0
        };
        let distance = if self.distance_px.is_finite() {
            self.distance_px
        } else {
            8.0
        };
        Self {
            grid_step_mm: step.clamp(MIN_GRID_STEP_MM, MAX_GRID_STEP_MM),
            distance_px: distance.clamp(MIN_DISTANCE_PX, MAX_DISTANCE_PX),
            ..self
        }
    }
}

/// Which shapes can be snapped to.
#[derive(Clone, Debug, PartialEq)]
pub enum SnapScope {
    /// Every piece except these.
    Project { exclude: Vec<u64> },
    /// Only the base and children of one piece, except these children. A child snaps only
    /// within its own piece.
    Piece { id: u64, exclude: Vec<u64> },
}

/// The vertices and boxes that snapping is drawn to.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SnapTargets {
    pub points: Vec<Point>,
    pub boxes: Vec<Bounds>,
}

impl SnapTargets {
    /// The handles of every shape in scope, and the boxes of every piece in project scope, or
    /// of each shape in piece scope. Handles, not ring points, so that an arc doesn't attract.
    pub fn collect(project: &Project, scope: &SnapScope) -> Self {
        let mut targets = Self::default();
        match scope {
            SnapScope::Project { exclude } => {
                for piece in project.pieces.iter().filter(|p| !exclude.contains(&p.id)) {
                    targets.add_points(piece, &[]);
                    targets.boxes.extend(piece.bounds());
                }
            }
            SnapScope::Piece { id, exclude } => {
                if let Some(piece) = project.pieces.iter().find(|piece| piece.id == *id) {
                    targets.add_points(piece, exclude);
                    targets.boxes.extend(piece.base.bounds());
                    targets.boxes.extend(
                        piece
                            .children
                            .iter()
                            .filter(|child| !exclude.contains(&child.id))
                            .filter_map(|child| child.shape.bounds()),
                    );
                }
            }
        }
        targets
    }

    fn add_points(&mut self, piece: &PatternPiece, exclude: &[u64]) {
        self.points.extend(piece.base.handles());
        for child in piece.children.iter().filter(|c| !exclude.contains(&c.id)) {
            self.points.extend(child.shape.handles());
        }
    }

    pub fn with_points(mut self, points: impl IntoIterator<Item = Point>) -> Self {
        self.points.extend(points);
        self
    }

    /// Drops a point, such as the original place of a handle being dragged.
    pub fn without_point(mut self, point: Point) -> Self {
        self.points.retain(|other| *other != point);
        self
    }

    /// Drops a box, such as that of the shape being reshaped.
    pub fn without_box(mut self, bounds: Option<Bounds>) -> Self {
        if let Some(bounds) = bounds {
            self.boxes.retain(|other| *other != bounds);
        }
        self
    }
}

/// The geometry being moved by a drag.
#[derive(Clone, Debug, PartialEq)]
pub struct Moving {
    pub handles: Vec<Point>,
    pub bounds: Bounds,
    /// The point that goes on the grid: the handle nearest where the drag started.
    pub anchor: Point,
}

impl Moving {
    pub fn point(point: Point) -> Self {
        Self {
            handles: vec![point],
            bounds: point_box(point),
            anchor: point,
        }
    }

    /// The moved shapes, grabbed at `grab`; `None` without any shape.
    pub fn shapes<'a>(shapes: impl IntoIterator<Item = &'a Shape>, grab: Point) -> Option<Self> {
        let mut handles = Vec::new();
        let mut bounds: Option<Bounds> = None;
        for shape in shapes {
            handles.extend(shape.handles());
            if let Some(shape_bounds) = shape.bounds() {
                bounds = Some(bounds.map_or(shape_bounds, |b| b.union(shape_bounds)));
            }
        }
        let anchor = handles
            .iter()
            .copied()
            .min_by(|a, b| distance(*a, grab).total_cmp(&distance(*b, grab)))
            .unwrap_or(grab);
        Some(Self {
            handles,
            bounds: bounds?,
            anchor,
        })
    }
}

/// A line along which two boxes are lined up: vertical at `x = at` for `GuideAxis::X`,
/// horizontal at `y = at` for `GuideAxis::Y`, running from `from` to `to` along the other axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    pub axis: GuideAxis,
    pub at: f64,
    pub from: f64,
    pub to: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuideAxis {
    X,
    Y,
}

/// What a snap lined up with, for drawing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SnapFeedback {
    /// The target vertex that was hit.
    pub vertex: Option<Point>,
    pub guides: Vec<Guide>,
}

impl SnapFeedback {
    pub fn is_empty(&self) -> bool {
        self.vertex.is_none() && self.guides.is_empty()
    }
}

/// The point snapped to a vertex, else per axis to a guide, else per axis to the grid.
/// `tolerance` is in mm.
pub fn snap_point(
    point: Point,
    targets: &SnapTargets,
    settings: &SnapSettings,
    tolerance: f64,
) -> (Point, SnapFeedback) {
    let ((dx, dy), feedback) = snap_offset(
        (0.0, 0.0),
        &Moving::point(point),
        targets,
        settings,
        tolerance,
    );
    (Point::new(point.x + dx, point.y + dy), feedback)
}

/// The offset of a move snapped so that a moved handle lands on a target vertex, else per axis
/// so that an edge or center of the moved box lines up with one of a target box, else per axis
/// so that the anchor lands on the grid. `tolerance` is in mm.
pub fn snap_offset(
    raw: (f64, f64),
    moving: &Moving,
    targets: &SnapTargets,
    settings: &SnapSettings,
    tolerance: f64,
) -> ((f64, f64), SnapFeedback) {
    if !settings.enabled {
        return (raw, SnapFeedback::default());
    }
    if settings.vertices
        && let Some((offset, vertex)) = vertex_snap(raw, moving, targets, tolerance)
    {
        let feedback = SnapFeedback {
            vertex: Some(vertex),
            guides: Vec::new(),
        };
        return (offset, feedback);
    }
    let guide_x = settings
        .guides
        .then(|| line_snap(raw.0, xs(moving.bounds), targets, xs, tolerance))
        .flatten();
    let guide_y = settings
        .guides
        .then(|| line_snap(raw.1, ys(moving.bounds), targets, ys, tolerance))
        .flatten();
    let grid = |value: f64, anchor: f64| {
        if settings.grid && settings.grid_step_mm > 0.0 {
            let step = settings.grid_step_mm;
            ((anchor + value) / step).round() * step - anchor
        } else {
            value
        }
    };
    let dx = guide_x.map_or_else(|| grid(raw.0, moving.anchor.x), |(dx, ..)| dx);
    let dy = guide_y.map_or_else(|| grid(raw.1, moving.anchor.y), |(dy, ..)| dy);

    let moved = Bounds {
        min_x: moving.bounds.min_x + dx,
        min_y: moving.bounds.min_y + dy,
        max_x: moving.bounds.max_x + dx,
        max_y: moving.bounds.max_y + dy,
    };
    let mut guides = Vec::new();
    if let Some((_, at, target)) = guide_x {
        guides.push(Guide {
            axis: GuideAxis::X,
            at,
            from: moved.min_y.min(target.min_y),
            to: moved.max_y.max(target.max_y),
        });
    }
    if let Some((_, at, target)) = guide_y {
        guides.push(Guide {
            axis: GuideAxis::Y,
            at,
            from: moved.min_x.min(target.min_x),
            to: moved.max_x.max(target.max_x),
        });
    }
    let feedback = SnapFeedback {
        vertex: None,
        guides,
    };
    ((dx, dy), feedback)
}

/// The offset that puts the nearest moved handle on a target vertex, and that vertex.
fn vertex_snap(
    raw: (f64, f64),
    moving: &Moving,
    targets: &SnapTargets,
    tolerance: f64,
) -> Option<((f64, f64), Point)> {
    let mut best: Option<(f64, (f64, f64), Point)> = None;
    for handle in &moving.handles {
        let moved = Point::new(handle.x + raw.0, handle.y + raw.1);
        for target in &targets.points {
            let gap = distance(moved, *target);
            if gap <= tolerance && best.is_none_or(|(nearest, ..)| gap < nearest) {
                let offset = (target.x - handle.x, target.y - handle.y);
                best = Some((gap, offset, *target));
            }
        }
    }
    best.map(|(_, offset, vertex)| (offset, vertex))
}

/// Along one axis: the offset that lines one of the moved lines up with the nearest target
/// line within reach, the line's position, and the target box it belongs to.
fn line_snap(
    raw: f64,
    moving: [f64; 3],
    targets: &SnapTargets,
    lines: fn(Bounds) -> [f64; 3],
    tolerance: f64,
) -> Option<(f64, f64, Bounds)> {
    let mut best: Option<(f64, f64, f64, Bounds)> = None;
    for target in &targets.boxes {
        for at in lines(*target) {
            for line in moving {
                let gap = (line + raw - at).abs();
                if gap <= tolerance && best.is_none_or(|(nearest, ..)| gap < nearest) {
                    best = Some((gap, at - line, at, *target));
                }
            }
        }
    }
    best.map(|(_, offset, at, target)| (offset, at, target))
}

fn xs(bounds: Bounds) -> [f64; 3] {
    [
        bounds.min_x,
        (bounds.min_x + bounds.max_x) * 0.5,
        bounds.max_x,
    ]
}

fn ys(bounds: Bounds) -> [f64; 3] {
    [
        bounds.min_y,
        (bounds.min_y + bounds.max_y) * 0.5,
        bounds.max_y,
    ]
}

fn point_box(point: Point) -> Bounds {
    Bounds {
        min_x: point.x,
        min_y: point.y,
        max_x: point.x,
        max_y: point.y,
    }
}

fn distance(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BooleanOp, ChildShape};

    fn square(id: u64, x: f64, y: f64, size: f64) -> PatternPiece {
        let base = Shape::rectangle_from_corners(Point::new(x, y), Point::new(x + size, y + size))
            .expect("a square");
        PatternPiece::new(id, format!("Piece {id}"), base)
    }

    fn project(pieces: Vec<PatternPiece>) -> Project {
        let mut project = Project::default();
        project.pieces = pieces;
        project
    }

    fn settings(grid: bool, vertices: bool, guides: bool) -> SnapSettings {
        SnapSettings {
            enabled: true,
            grid,
            grid_step_mm: 5.0,
            vertices,
            guides,
            distance_px: 8.0,
        }
    }

    fn assert_point(actual: Point, expected: Point) {
        assert!(
            (actual.x - expected.x).abs() < 1e-9 && (actual.y - expected.y).abs() < 1e-9,
            "{actual:?} != {expected:?}"
        );
    }

    #[test]
    fn grid_rounds_around_the_origin() {
        let targets = SnapTargets::default();
        let (point, feedback) = snap_point(
            Point::new(7.4, -7.6),
            &targets,
            &settings(true, true, true),
            1.0,
        );
        assert_point(point, Point::new(5.0, -10.0));
        assert!(feedback.is_empty());
    }

    #[test]
    fn disabled_snapping_leaves_the_point() {
        let targets = SnapTargets::default().with_points([Point::new(1.0, 1.0)]);
        let mut off = settings(true, true, true);
        off.enabled = false;
        let (point, _) = snap_point(Point::new(1.2, 1.3), &targets, &off, 1.0);
        assert_point(point, Point::new(1.2, 1.3));
    }

    #[test]
    fn a_vertex_beats_a_guide_and_the_grid() {
        let project = project(vec![square(1, 0.0, 0.0, 10.0)]);
        let targets = SnapTargets::collect(&project, &SnapScope::Project { exclude: vec![] });
        let (point, feedback) = snap_point(
            Point::new(10.4, 9.7),
            &targets,
            &settings(true, true, true),
            1.0,
        );
        assert_point(point, Point::new(10.0, 10.0));
        assert_eq!(feedback.vertex, Some(Point::new(10.0, 10.0)));
    }

    #[test]
    fn a_guide_beats_the_grid_and_axes_resolve_on_their_own() {
        let project = project(vec![square(1, 0.0, 0.0, 10.0)]);
        let targets = SnapTargets::collect(&project, &SnapScope::Project { exclude: vec![] });
        // Near the square's center line x = 5 but far from its vertices; y goes to the grid.
        let (point, feedback) = snap_point(
            Point::new(5.6, 33.0),
            &targets,
            &settings(true, true, true),
            1.0,
        );
        assert_point(point, Point::new(5.0, 35.0));
        assert_eq!(feedback.guides.len(), 1);
        assert_eq!(feedback.guides[0].axis, GuideAxis::X);
        assert!((feedback.guides[0].at - 5.0).abs() < 1e-9);
    }

    #[test]
    fn nothing_beyond_the_tolerance_snaps() {
        let project = project(vec![square(1, 0.0, 0.0, 10.0)]);
        let targets = SnapTargets::collect(&project, &SnapScope::Project { exclude: vec![] });
        let (point, feedback) = snap_point(
            Point::new(12.3, 13.4),
            &targets,
            &settings(false, true, true),
            1.0,
        );
        assert_point(point, Point::new(12.3, 13.4));
        assert!(feedback.is_empty());
    }

    #[test]
    fn excluded_pieces_do_not_attract() {
        let project = project(vec![square(1, 0.0, 0.0, 10.0), square(2, 50.0, 0.0, 10.0)]);
        let targets = SnapTargets::collect(&project, &SnapScope::Project { exclude: vec![1] });
        assert!(!targets.points.contains(&Point::new(10.0, 10.0)));
        assert!(targets.points.contains(&Point::new(60.0, 10.0)));
        assert_eq!(targets.boxes.len(), 1);
    }

    #[test]
    fn piece_scope_keeps_only_that_piece() {
        let mut piece = square(1, 0.0, 0.0, 10.0);
        let hole = Shape::rectangle_from_corners(Point::new(2.0, 2.0), Point::new(4.0, 4.0))
            .expect("a hole");
        let other = Shape::rectangle_from_corners(Point::new(6.0, 6.0), Point::new(8.0, 8.0))
            .expect("another hole");
        piece.children.push(ChildShape {
            id: 1,
            op: BooleanOp::Subtract,
            shape: hole,
        });
        piece.children.push(ChildShape {
            id: 2,
            op: BooleanOp::Subtract,
            shape: other,
        });
        let project = project(vec![piece, square(2, 50.0, 0.0, 10.0)]);
        let targets = SnapTargets::collect(
            &project,
            &SnapScope::Piece {
                id: 1,
                exclude: vec![2],
            },
        );
        assert!(targets.points.contains(&Point::new(2.0, 2.0)));
        assert!(targets.points.contains(&Point::new(10.0, 10.0)));
        assert!(!targets.points.contains(&Point::new(6.0, 6.0)));
        assert!(!targets.points.contains(&Point::new(50.0, 0.0)));
        // The base and the one remaining child.
        assert_eq!(targets.boxes.len(), 2);
    }

    #[test]
    fn a_moved_vertex_lands_exactly_on_a_target() {
        let project = project(vec![square(1, 0.0, 0.0, 10.0), square(2, 30.0, 0.0, 10.0)]);
        let targets = SnapTargets::collect(&project, &SnapScope::Project { exclude: vec![2] });
        let moving = Moving::shapes([&project.pieces[1].base], Point::new(31.0, 1.0))
            .expect("a moving square");
        // Dragged left by 19.6 mm: its left corners come within 0.4 mm of the right corners.
        let ((dx, dy), feedback) = snap_offset(
            (-19.6, 0.3),
            &moving,
            &targets,
            &settings(true, true, true),
            1.0,
        );
        assert!((dx + 20.0).abs() < 1e-9 && dy.abs() < 1e-9);
        assert!(feedback.vertex.is_some());
    }

    #[test]
    fn the_grid_puts_the_anchor_on_the_grid() {
        let piece = square(1, 0.3, 0.3, 10.0);
        let moving = Moving::shapes([&piece.base], Point::new(0.0, 0.0)).expect("a square");
        assert_point(moving.anchor, Point::new(0.3, 0.3));
        let ((dx, dy), _) = snap_offset(
            (6.0, 6.0),
            &moving,
            &SnapTargets::default(),
            &settings(true, false, false),
            1.0,
        );
        assert_point(Point::new(0.3 + dx, 0.3 + dy), Point::new(5.0, 5.0));
    }

    #[test]
    fn settings_from_a_file_are_clamped() {
        let wild = SnapSettings {
            grid_step_mm: 50.0,
            distance_px: f32::NAN,
            ..SnapSettings::default()
        };
        let clamped = wild.clamped();
        assert_eq!(clamped.grid_step_mm, MAX_GRID_STEP_MM);
        assert_eq!(clamped.distance_px, 8.0);
    }
}
