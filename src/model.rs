use crate::i18n::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

pub const PROJECT_VERSION: u32 = 1;
const MAX_PIECES: usize = 10_000;
const MAX_POINTS_PER_PIECE: usize = 2_048;
const MAX_COORDINATE_MM: f64 = 1_000_000.0;
const MIN_POLYGON_AREA_MM2: f64 = 1.0e-6;

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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatternPiece {
    pub id: u64,
    pub name: String,
    pub points: Vec<Point>,
}

impl PatternPiece {
    pub fn new(id: u64, name: impl Into<String>, points: Vec<Point>) -> Self {
        Self {
            id,
            name: name.into(),
            points,
        }
    }

    pub fn bounds(&self) -> Option<Bounds> {
        let first = *self.points.first()?;
        let mut bounds = Bounds {
            min_x: first.x,
            min_y: first.y,
            max_x: first.x,
            max_y: first.y,
        };
        for point in &self.points[1..] {
            bounds.min_x = bounds.min_x.min(point.x);
            bounds.min_y = bounds.min_y.min(point.y);
            bounds.max_x = bounds.max_x.max(point.x);
            bounds.max_y = bounds.max_y.max(point.y);
        }
        Some(bounds)
    }

    pub fn contains(&self, point: Point) -> bool {
        if self.points.len() < 3 {
            return false;
        }

        let mut inside = false;
        let mut previous = self.points.len() - 1;
        for current in 0..self.points.len() {
            let a = self.points[current];
            let b = self.points[previous];
            let crosses = (a.y > point.y) != (b.y > point.y)
                && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x;
            if crosses {
                inside = !inside;
            }
            previous = current;
        }
        inside
    }

    pub fn signed_area(&self) -> f64 {
        let mut area = 0.0;
        for index in 0..self.points.len() {
            let a = self.points[index];
            let b = self.points[(index + 1) % self.points.len()];
            area += a.x * b.y - b.x * a.y;
        }
        area * 0.5
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub units: Units,
    pub pieces: Vec<PatternPiece>,
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
            path: None,
            modified: false,
        }
    }
}

impl Project {
    pub fn bounds(&self) -> Option<Bounds> {
        let mut pieces = self.pieces.iter().filter_map(PatternPiece::bounds);
        let first = pieces.next()?;
        Some(pieces.fold(first, |total, bounds| Bounds {
            min_x: total.min_x.min(bounds.min_x),
            min_y: total.min_y.min(bounds.min_y),
            max_x: total.max_x.max(bounds.max_x),
            max_y: total.max_y.max(bounds.max_y),
        }))
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
            if !(3..=MAX_POINTS_PER_PIECE).contains(&piece.points.len()) {
                return Err(Message::PointCountOutOfRange {
                    name: piece.name.clone(),
                    max: MAX_POINTS_PER_PIECE,
                });
            }

            for (index, point) in piece.points.iter().enumerate() {
                if !point.x.is_finite()
                    || !point.y.is_finite()
                    || point.x.abs() > MAX_COORDINATE_MM
                    || point.y.abs() > MAX_COORDINATE_MM
                {
                    return Err(Message::CoordinateOutOfRange {
                        name: piece.name.clone(),
                    });
                }
                let next = piece.points[(index + 1) % piece.points.len()];
                if (point.x - next.x).hypot(point.y - next.y) < 1.0e-8 {
                    return Err(Message::CoincidentPoints {
                        name: piece.name.clone(),
                    });
                }
            }

            if piece.signed_area().abs() < MIN_POLYGON_AREA_MM2 {
                return Err(Message::ZeroArea {
                    name: piece.name.clone(),
                });
            }
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

    fn rectangle(id: u64) -> PatternPiece {
        PatternPiece::new(
            id,
            "Pocket",
            vec![
                Point::new(0.0, 0.0),
                Point::new(80.0, 0.0),
                Point::new(80.0, 120.0),
                Point::new(0.0, 120.0),
            ],
        )
    }

    #[test]
    fn accepts_a_valid_pattern_and_reports_bounds() {
        let piece = rectangle(1);
        let project = Project {
            pieces: vec![piece.clone()],
            ..Project::default()
        };

        assert_eq!(project.validate(), Ok(()));
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
            vec![
                Point::new(0.0, 0.0),
                Point::new(2.0, 2.0),
                Point::new(4.0, 4.0),
            ],
        )];
        assert_eq!(
            project.validate(),
            Err(Message::ZeroArea {
                name: "Bad".to_owned()
            })
        );
    }

    #[test]
    fn hit_testing_handles_inside_and_outside_points() {
        let piece = rectangle(1);
        assert!(piece.contains(Point::new(40.0, 60.0)));
        assert!(!piece.contains(Point::new(100.0, 60.0)));
    }

    #[test]
    fn converts_display_measurements_without_changing_internal_geometry() {
        assert_eq!(Units::Inches.from_millimeters(25.4), 1.0);
        assert_eq!(Units::Millimeters.from_millimeters(25.4), 25.4);
    }
}
