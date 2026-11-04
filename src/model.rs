use crate::i18n::Message;
use crate::material::Material;
use crate::shapes::{MAX_DASHES_PER_LINE, MAX_HOLES_PER_LINE, Shape, StitchLine};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

pub const PROJECT_VERSION: u32 = 2;
const MAX_PIECES: usize = 10_000;
const MAX_POINTS_PER_PIECE: usize = 2_048;
const MAX_CHILDREN_PER_PIECE: usize = 512;
const MAX_COORDINATE_MM: f64 = 1_000_000.0;
pub(crate) const MIN_POLYGON_AREA_MM2: f64 = 1.0e-6;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Units {
    #[default]
    Millimeters,
    Inches,
}

impl Units {
    pub fn from_millimeters(self, value: f64) -> f64 {
        match self {
            Self::Millimeters => value,
            Self::Inches => value / 25.4,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Bounds {
    pub fn width(self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }
}

pub(crate) fn ring_bounds(ring: &[Point]) -> Option<Bounds> {
    let first = *ring.first()?;
    let mut bounds = Bounds {
        min_x: first.x,
        min_y: first.y,
        max_x: first.x,
        max_y: first.y,
    };
    for point in &ring[1..] {
        bounds.min_x = bounds.min_x.min(point.x);
        bounds.min_y = bounds.min_y.min(point.y);
        bounds.max_x = bounds.max_x.max(point.x);
        bounds.max_y = bounds.max_y.max(point.y);
    }
    Some(bounds)
}

pub(crate) fn ring_contains(ring: &[Point], point: Point) -> bool {
    if ring.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut previous = ring.len() - 1;
    for current in 0..ring.len() {
        let a = ring[current];
        let b = ring[previous];
        let crosses = (a.y > point.y) != (b.y > point.y)
            && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x;
        if crosses {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

pub fn signed_area(ring: &[Point]) -> f64 {
    let mut area = 0.0;
    for index in 0..ring.len() {
        let a = ring[index];
        let b = ring[(index + 1) % ring.len()];
        area += a.x * b.y - b.x * a.y;
    }
    area * 0.5
}

/// How a child shape changes the piece built so far.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BooleanOp {
    #[default]
    Subtract,
    Union,
    Intersect,
}

impl BooleanOp {
    pub const ALL: [Self; 3] = [Self::Subtract, Self::Union, Self::Intersect];
}

/// A shape combined with the piece. `id` is unique within its piece and survives reordering.
/// An open shape, such as a centerline, only marks the piece: its `op` is ignored.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChildShape {
    pub id: u64,
    pub op: BooleanOp,
    pub shape: Shape,
}

/// A base shape and the child shapes combined with it in list order. The resulting outline
/// is computed by `region`, and may be invalid while the piece is being edited.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatternPiece {
    pub id: u64,
    pub name: String,
    pub base: Shape,
    #[serde(default)]
    pub children: Vec<ChildShape>,
    /// A locked piece cannot be moved, reshaped or deleted.
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub material: Material,
}

impl PatternPiece {
    pub fn new(id: u64, name: impl Into<String>, base: Shape) -> Self {
        Self {
            id,
            name: name.into(),
            base,
            children: Vec::new(),
            locked: false,
            material: Material::default(),
        }
    }

    /// The extent of the base and every child, which is what the editor shows and hit tests.
    pub fn bounds(&self) -> Option<Bounds> {
        std::iter::once(&self.base)
            .chain(self.children.iter().map(|child| &child.shape))
            .filter_map(Shape::bounds)
            .reduce(Bounds::union)
    }

    pub fn child(&self, id: u64) -> Option<&ChildShape> {
        self.children.iter().find(|child| child.id == id)
    }

    pub fn child_index(&self, id: u64) -> Option<usize> {
        self.children.iter().position(|child| child.id == id)
    }

    pub fn next_child_id(&self) -> u64 {
        self.children
            .iter()
            .map(|child| child.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.base.translate(dx, dy);
        for child in &mut self.children {
            child.shape.translate(dx, dy);
        }
    }

    fn validate_shape(&self, shape: &Shape) -> Result<(), Message> {
        if !shape.has_valid_size() {
            return Err(Message::InvalidShapeSize {
                name: self.name.clone(),
            });
        }
        match shape {
            Shape::Line(line) => {
                self.validate_coordinates(&[line.start, line.end])?;
                if line.dash_count() > MAX_DASHES_PER_LINE {
                    return Err(Message::TooManyDashes {
                        name: self.name.clone(),
                        max: MAX_DASHES_PER_LINE,
                    });
                }
                Ok(())
            }
            Shape::StitchLine(line) => self.validate_stitch_line(line),
            _ => self.validate_ring(&shape.ring()),
        }
    }

    fn validate_stitch_line(&self, line: &StitchLine) -> Result<(), Message> {
        let name = || self.name.clone();
        self.validate_coordinates(&line.points)?;
        let coincident = line
            .points
            .windows(2)
            .any(|pair| (pair[0].x - pair[1].x).hypot(pair[0].y - pair[1].y) < 1.0e-6);
        if coincident {
            return Err(Message::CoincidentPoints { name: name() });
        }
        if line.path().len() > MAX_POINTS_PER_PIECE {
            return Err(Message::PointCountOutOfRange {
                name: name(),
                max: MAX_POINTS_PER_PIECE,
            });
        }
        if !line.has_valid_parameters() {
            return Err(Message::InvalidHoleParameters { name: name() });
        }
        if line.hole_count() > MAX_HOLES_PER_LINE {
            return Err(Message::TooManyHoles {
                name: name(),
                max: MAX_HOLES_PER_LINE,
            });
        }
        if line.holes_overlap() {
            return Err(Message::HolesOverlap { name: name() });
        }
        Ok(())
    }

    fn validate_coordinates(&self, points: &[Point]) -> Result<(), Message> {
        let out_of_range = |point: &Point| {
            !point.x.is_finite()
                || !point.y.is_finite()
                || point.x.abs() > MAX_COORDINATE_MM
                || point.y.abs() > MAX_COORDINATE_MM
        };
        if points.iter().any(out_of_range) {
            return Err(Message::CoordinateOutOfRange {
                name: self.name.clone(),
            });
        }
        Ok(())
    }

    fn validate_ring(&self, ring: &[Point]) -> Result<(), Message> {
        if !(3..=MAX_POINTS_PER_PIECE).contains(&ring.len()) {
            return Err(Message::PointCountOutOfRange {
                name: self.name.clone(),
                max: MAX_POINTS_PER_PIECE,
            });
        }
        self.validate_coordinates(ring)?;
        for (index, point) in ring.iter().enumerate() {
            let next = ring[(index + 1) % ring.len()];
            if (point.x - next.x).hypot(point.y - next.y) < 1.0e-8 {
                return Err(Message::CoincidentPoints {
                    name: self.name.clone(),
                });
            }
        }
        if signed_area(ring).abs() < MIN_POLYGON_AREA_MM2 {
            return Err(Message::ZeroArea {
                name: self.name.clone(),
            });
        }
        Ok(())
    }

    /// Checks the parameters of every shape. Whether they combine into one outline is
    /// `region`'s concern, since an edit in progress may pass through invalid results.
    fn validate_shapes(&self) -> Result<(), Message> {
        if self.children.len() > MAX_CHILDREN_PER_PIECE {
            return Err(Message::TooManyChildren {
                name: self.name.clone(),
                max: MAX_CHILDREN_PER_PIECE,
            });
        }
        if self.base.is_open() {
            return Err(Message::BaseMustBeClosed {
                name: self.name.clone(),
            });
        }
        self.validate_shape(&self.base)?;
        let mut ids = HashSet::with_capacity(self.children.len());
        for child in &self.children {
            if child.id == 0 || !ids.insert(child.id) {
                return Err(Message::InvalidChildId {
                    name: self.name.clone(),
                });
            }
            self.validate_shape(&child.shape)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub units: Units,
    pub pieces: Vec<PatternPiece>,
    /// The material given to each new piece. Older files lack the field.
    #[serde(default)]
    pub default_material: Material,
    /// Where the document lives on disk; not part of the file format.
    #[serde(skip)]
    pub path: Option<PathBuf>,
    /// Whether there are edits since the last save or load; not part of the file format.
    #[serde(skip)]
    pub modified: bool,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: PROJECT_VERSION,
            units: Units::Millimeters,
            pieces: Vec::new(),
            default_material: Material::default(),
            path: None,
            modified: false,
        }
    }
}

impl Project {
    pub fn bounds(&self) -> Option<Bounds> {
        self.pieces
            .iter()
            .filter_map(PatternPiece::bounds)
            .reduce(Bounds::union)
    }

    pub fn validate(&self) -> Result<(), Message> {
        if self.version != PROJECT_VERSION {
            return Err(Message::UnsupportedVersion {
                found: self.version,
                expected: PROJECT_VERSION,
            });
        }
        if self.pieces.len() > MAX_PIECES {
            return Err(Message::TooManyPieces { max: MAX_PIECES });
        }
        if !self.default_material.has_valid_thickness() {
            return Err(Message::DefaultThicknessOutOfRange);
        }

        let mut ids = HashSet::with_capacity(self.pieces.len());
        for piece in &self.pieces {
            if piece.id == 0 || !ids.insert(piece.id) {
                return Err(Message::InvalidPieceId {
                    name: piece.name.clone(),
                });
            }
            if piece.name.trim().is_empty() {
                return Err(Message::PieceNameRequired { id: piece.id });
            }
            if !piece.material.has_valid_thickness() {
                return Err(Message::ThicknessOutOfRange {
                    name: piece.name.clone(),
                });
            }
            piece.validate_shapes()?;
        }
        Ok(())
    }

    pub fn next_piece_id(&self) -> u64 {
        self.pieces
            .iter()
            .map(|piece| piece.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
    }

    pub fn add_piece(&mut self, piece: PatternPiece) {
        self.pieces.push(piece);
    }

    pub fn remove_piece(&mut self, id: u64) -> bool {
        let original_len = self.pieces.len();
        self.pieces.retain(|piece| piece.id != id);
        self.pieces.len() != original_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::{Ellipse, Polygon};

    fn rectangle(id: u64) -> PatternPiece {
        PatternPiece::new(
            id,
            "Pocket",
            Shape::rectangle_from_corners(Point::new(0.0, 0.0), Point::new(80.0, 120.0)).unwrap(),
        )
    }

    fn project(piece: PatternPiece) -> Project {
        Project {
            pieces: vec![piece],
            ..Project::default()
        }
    }

    #[test]
    fn accepts_a_valid_pattern_and_reports_bounds() {
        let piece = rectangle(1);
        assert_eq!(project(piece.clone()).validate(), Ok(()));
        assert_eq!(
            piece.bounds(),
            Some(Bounds {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 80.0,
                max_y: 120.0,
            })
        );
    }

    #[test]
    fn bounds_include_children_outside_the_base() {
        let mut piece = rectangle(1);
        piece.children.push(ChildShape {
            id: 1,
            op: BooleanOp::Union,
            shape: Shape::rectangle_from_corners(Point::new(70.0, 10.0), Point::new(100.0, 20.0))
                .unwrap(),
        });
        assert_eq!(piece.bounds().map(|bounds| bounds.max_x), Some(100.0));
    }

    #[test]
    fn rejects_invalid_polygons_and_duplicate_ids() {
        let mut project = Project {
            pieces: vec![rectangle(1), rectangle(1)],
            ..Project::default()
        };
        assert_eq!(
            project.validate(),
            Err(Message::InvalidPieceId {
                name: "Pocket".to_owned()
            })
        );

        project.pieces = vec![PatternPiece::new(
            2,
            "Bad",
            Shape::Polygon(Polygon {
                points: vec![
                    Point::new(0.0, 0.0),
                    Point::new(2.0, 2.0),
                    Point::new(4.0, 4.0),
                ],
                ..Default::default()
            }),
        )];
        assert_eq!(
            project.validate(),
            Err(Message::ZeroArea {
                name: "Bad".to_owned()
            })
        );
    }

    #[test]
    fn rejects_primitives_without_a_size() {
        let piece = PatternPiece::new(
            1,
            "Pocket",
            Shape::Ellipse(Ellipse {
                center: Point::new(0.0, 0.0),
                radius_x: 0.0,
                radius_y: 5.0,
                rotation: 0.0,
            }),
        );
        assert_eq!(
            project(piece).validate(),
            Err(Message::InvalidShapeSize {
                name: "Pocket".to_owned()
            })
        );
    }

    #[test]
    fn rejects_duplicate_child_ids() {
        let mut piece = rectangle(1);
        let child = ChildShape {
            id: 1,
            op: BooleanOp::Subtract,
            shape: Shape::rectangle_from_corners(Point::new(10.0, 10.0), Point::new(20.0, 20.0))
                .unwrap(),
        };
        piece.children = vec![child.clone(), child];
        assert_eq!(
            project(piece).validate(),
            Err(Message::InvalidChildId {
                name: "Pocket".to_owned()
            })
        );
    }

    #[test]
    fn converts_display_measurements_without_changing_internal_geometry() {
        assert_eq!(Units::Inches.from_millimeters(25.4), 1.0);
        assert_eq!(Units::Millimeters.from_millimeters(25.4), 25.4);
    }

    fn line(x: f64, style: crate::shapes::LineStyle) -> Shape {
        Shape::line_from_ends(Point::new(0.0, 60.0), Point::new(x, 60.0), false, style).unwrap()
    }

    #[test]
    fn a_line_is_a_valid_child_but_not_a_base() {
        use crate::shapes::LineStyle;
        let mut piece = rectangle(1);
        piece.children.push(ChildShape {
            id: 1,
            op: BooleanOp::Subtract,
            shape: line(200.0, LineStyle::dash_dot_default()),
        });
        assert_eq!(project(piece).validate(), Ok(()));

        let base = PatternPiece::new(1, "Axis", line(10.0, LineStyle::Solid));
        assert_eq!(
            project(base).validate(),
            Err(Message::BaseMustBeClosed {
                name: "Axis".to_owned()
            })
        );
    }

    #[test]
    fn line_styles_are_checked() {
        use crate::shapes::LineStyle;
        let with_line = |shape| {
            let mut piece = rectangle(1);
            piece.children.push(ChildShape {
                id: 1,
                op: BooleanOp::Subtract,
                shape,
            });
            project(piece).validate()
        };
        let no_gap = LineStyle::Dashed {
            dash: 5.0,
            gap: 0.0,
        };
        assert_eq!(
            with_line(line(100.0, no_gap)),
            Err(Message::InvalidShapeSize {
                name: "Pocket".to_owned()
            })
        );
        let fine = LineStyle::Dashed {
            dash: 0.1,
            gap: 0.1,
        };
        assert_eq!(
            with_line(line(10_000.0, fine)),
            Err(Message::TooManyDashes {
                name: "Pocket".to_owned(),
                max: crate::shapes::MAX_DASHES_PER_LINE
            })
        );
    }
}
