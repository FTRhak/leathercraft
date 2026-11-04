use crate::model::Point;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

/// The shortest dash or gap, in mm, so a pattern stays visible and its dashes countable.
pub const MIN_DASH_MM: f64 = 0.1;
/// The most visible segments one line may split into; validation rejects longer patterns.
pub const MAX_DASHES_PER_LINE: usize = 4_096;
/// The length of the dot in a dash-dot pattern, in mm.
pub const DOT_MM: f64 = 1.0;
const ANGLE_SNAP: f64 = TAU / 24.0;

/// How a line is stroked. Dash and gap lengths are in mm, so they scale with the drawing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed {
        dash: f64,
        gap: f64,
    },
    /// Dash, gap, dot, gap: the usual stroke of an axis.
    DashDot {
        dash: f64,
        gap: f64,
    },
}

impl LineStyle {
    pub const fn dashed_default() -> Self {
        Self::Dashed {
            dash: 5.0,
            gap: 2.0,
        }
    }

    pub const fn dash_dot_default() -> Self {
        Self::DashDot {
            dash: 10.0,
            gap: 2.0,
        }
    }

    /// Alternating lengths of visible and blank parts, starting with a visible one, or `None`
    /// for a solid line.
    pub fn pattern(self) -> Option<Vec<f64>> {
        match self {
            Self::Solid => None,
            Self::Dashed { dash, gap } => Some(vec![dash, gap]),
            Self::DashDot { dash, gap } => Some(vec![dash, gap, DOT_MM, gap]),
        }
    }

    pub(crate) fn is_valid(self) -> bool {
        match self {
            Self::Solid => true,
            Self::Dashed { dash, gap } | Self::DashDot { dash, gap } => [dash, gap]
                .iter()
                .all(|value| value.is_finite() && *value >= MIN_DASH_MM),
        }
    }
}

/// An open straight line, such as a centerline. It marks the piece and takes no part in its
/// outline; its ends are its handles.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub start: Point,
    pub end: Point,
    #[serde(default)]
    pub style: LineStyle,
}

impl Line {
    /// The line dragged from `start` to `end`, or `None` if it has no length. `snap` turns it
    /// in 15° steps around `start`, keeping its length.
    pub fn from_ends(start: Point, end: Point, snap: bool, style: LineStyle) -> Option<Self> {
        let line = Self { start, end, style };
        let length = line.length();
        if !(length.is_finite() && length > 0.0) {
            return None;
        }
        if !snap {
            return Some(line);
        }
        let angle = (line.angle() / ANGLE_SNAP).round() * ANGLE_SNAP;
        Some(line.with_angle(angle))
    }

    pub fn length(&self) -> f64 {
        (self.end.x - self.start.x).hypot(self.end.y - self.start.y)
    }

    /// The direction from `start` to `end`, in radians.
    pub fn angle(&self) -> f64 {
        (self.end.y - self.start.y).atan2(self.end.x - self.start.x)
    }

    pub fn midpoint(&self) -> Point {
        Point::new(
            (self.start.x + self.end.x) * 0.5,
            (self.start.y + self.end.y) * 0.5,
        )
    }

    /// The line with `end` moved along its direction so that it is `length` long.
    pub fn with_length(&self, length: f64) -> Self {
        let mut line = self.clone();
        let (sin, cos) = self.angle().sin_cos();
        line.end = Point::new(self.start.x + length * cos, self.start.y + length * sin);
        line
    }

    /// The line turned around `start` to point at `angle`, keeping its length.
    pub fn with_angle(&self, angle: f64) -> Self {
        let mut line = self.clone();
        let length = self.length();
        let (sin, cos) = angle.sin_cos();
        line.end = Point::new(self.start.x + length * cos, self.start.y + length * sin);
        line
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        for point in [&mut self.start, &mut self.end] {
            point.x += dx;
            point.y += dy;
        }
    }

    pub fn handles(&self) -> Vec<Point> {
        vec![self.start, self.end]
    }

    /// Handle 0 is `start` and handle 1 is `end`.
    pub fn with_handle_at(&self, handle: usize, to: Point) -> Self {
        let mut line = self.clone();
        match handle {
            0 => line.start = to,
            1 => line.end = to,
            _ => {}
        }
        line
    }

    /// The distance from the point to the nearest point of the line.
    pub fn distance_to(&self, point: Point) -> f64 {
        let (dx, dy) = (self.end.x - self.start.x, self.end.y - self.start.y);
        let length_sq = dx * dx + dy * dy;
        let t = if length_sq > 0.0 {
            (((point.x - self.start.x) * dx + (point.y - self.start.y) * dy) / length_sq)
                .clamp(0.0, 1.0)
        } else {
            0.0
        };
        (self.start.x + dx * t - point.x).hypot(self.start.y + dy * t - point.y)
    }

    /// How many visible segments the style splits the line into.
    pub fn dash_count(&self) -> usize {
        let Some(pattern) = self.style.pattern() else {
            return 1;
        };
        let period: f64 = pattern.iter().sum();
        let visible = pattern.len().div_ceil(2);
        let periods = (self.length() / period).ceil();
        if periods.is_finite() {
            (periods as usize).saturating_mul(visible).max(1)
        } else {
            usize::MAX
        }
    }

    /// The visible segments of the line, in order from `start`; the last dash is cut at `end`.
    /// At most `MAX_DASHES_PER_LINE` are returned.
    pub fn dashes(&self) -> Vec<(Point, Point)> {
        let length = self.length();
        let pattern = match self.style.pattern() {
            Some(pattern) if self.style.is_valid() && length > 0.0 => pattern,
            _ => return vec![(self.start, self.end)],
        };
        let at = |distance: f64| {
            let t = distance / length;
            Point::new(
                self.start.x + (self.end.x - self.start.x) * t,
                self.start.y + (self.end.y - self.start.y) * t,
            )
        };
        let mut dashes = Vec::new();
        let mut distance = 0.0;
        for (index, part) in pattern.iter().cycle().enumerate() {
            if distance >= length || dashes.len() >= MAX_DASHES_PER_LINE {
                break;
            }
            let next = (distance + part).min(length);
            if index % 2 == 0 {
                dashes.push((at(distance), at(next)));
            }
            distance = next;
        }
        dashes
    }

    pub(crate) fn has_valid_size(&self) -> bool {
        let length = self.length();
        length.is_finite() && length > 1.0e-6 && self.style.is_valid()
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::assert_near;
    use super::*;

    fn line(x: f64, y: f64, style: LineStyle) -> Line {
        Line {
            start: Point::new(0.0, 0.0),
            end: Point::new(x, y),
            style,
        }
    }

    #[test]
    fn a_line_needs_length() {
        let point = Point::new(3.0, 4.0);
        assert_eq!(Line::from_ends(point, point, false, LineStyle::Solid), None);
    }

    #[test]
    fn snapping_turns_in_fifteen_degree_steps_and_keeps_the_length() {
        let start = Point::new(0.0, 0.0);
        let snapped =
            Line::from_ends(start, Point::new(10.0, 1.0), true, LineStyle::Solid).unwrap();
        assert_near(snapped.end, Point::new(10.0_f64.hypot(1.0), 0.0));
        let diagonal =
            Line::from_ends(start, Point::new(10.0, 9.0), true, LineStyle::Solid).unwrap();
        assert!((diagonal.angle() - TAU / 8.0).abs() < 1.0e-9);
    }

    #[test]
    fn handles_move_their_end() {
        let moved = line(10.0, 0.0, LineStyle::Solid).with_handle_at(1, Point::new(5.0, 5.0));
        assert_eq!(moved.end, Point::new(5.0, 5.0));
        assert_eq!(moved.start, Point::new(0.0, 0.0));
        let mut shifted = moved.clone();
        shifted.translate(1.0, 2.0);
        assert_eq!(
            shifted.handles(),
            vec![Point::new(1.0, 2.0), Point::new(6.0, 7.0)]
        );
    }

    #[test]
    fn resizing_keeps_the_start_and_the_direction() {
        let resized = line(3.0, 4.0, LineStyle::Solid).with_length(10.0);
        assert_near(resized.end, Point::new(6.0, 8.0));
    }

    #[test]
    fn a_solid_line_is_one_dash() {
        let solid = line(10.0, 0.0, LineStyle::Solid);
        assert_eq!(solid.dashes(), vec![(solid.start, solid.end)]);
        assert_eq!(solid.dash_count(), 1);
    }

    #[test]
    fn dashes_follow_the_pattern_and_stop_at_the_end() {
        let dashed = line(
            12.0,
            0.0,
            LineStyle::Dashed {
                dash: 5.0,
                gap: 2.0,
            },
        );
        let dashes = dashed.dashes();
        assert_eq!(dashes.len(), 2);
        assert_near(dashes[0].1, Point::new(5.0, 0.0));
        assert_near(dashes[1].0, Point::new(7.0, 0.0));
        assert_near(dashes[1].1, Point::new(12.0, 0.0));

        let dash_dot = line(
            20.0,
            0.0,
            LineStyle::DashDot {
                dash: 10.0,
                gap: 2.0,
            },
        );
        let dashes = dash_dot.dashes();
        // 10 dash, 2 gap, 1 dot, 2 gap, then a dash cut at 20.
        assert_eq!(dashes.len(), 3);
        assert_near(dashes[1].0, Point::new(12.0, 0.0));
        assert_near(dashes[1].1, Point::new(13.0, 0.0));
        assert_near(dashes[2].0, Point::new(15.0, 0.0));
        assert_near(dashes[2].1, Point::new(20.0, 0.0));
    }

    #[test]
    fn distance_is_measured_to_the_segment() {
        let segment = line(10.0, 0.0, LineStyle::Solid);
        assert!((segment.distance_to(Point::new(5.0, 3.0)) - 3.0).abs() < 1.0e-9);
        assert!((segment.distance_to(Point::new(13.0, 4.0)) - 5.0).abs() < 1.0e-9);
    }
}
