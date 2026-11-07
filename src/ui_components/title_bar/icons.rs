//! The painted icons of the title bar. The icon font has no glyphs for them yet, so they are
//! drawn from geometry, in the stroke they are given (see `docs/icons.md`).

use bevy_egui::egui::{self, Painter, Rect, Stroke, pos2};

/// Open scissors, blades to the right: the logo of the app.
pub(super) fn scissors(painter: &Painter, rect: Rect, stroke: Stroke) {
    let ring = rect.height() * 0.2;
    let left = rect.left() + ring + stroke.width / 2.0;
    let upper = pos2(left, rect.top() + ring + stroke.width / 2.0);
    let lower = pos2(left, rect.bottom() - ring - stroke.width / 2.0);
    painter.circle_stroke(upper, ring, stroke);
    painter.circle_stroke(lower, ring, stroke);
    // Each blade starts at the edge of its ring and crosses the other one.
    let towards = |from: egui::Pos2, to: egui::Pos2| from + (to - from).normalized() * ring;
    let upper_tip = pos2(rect.right(), rect.bottom() - rect.height() * 0.1);
    let lower_tip = pos2(rect.right(), rect.top() + rect.height() * 0.1);
    painter.line_segment([towards(upper, upper_tip), upper_tip], stroke);
    painter.line_segment([towards(lower, lower_tip), lower_tip], stroke);
}

/// A square with a handle at each corner: the 2D editor.
pub(super) fn bounding_box(painter: &Painter, rect: Rect, stroke: Stroke) {
    let rect = rect.shrink(stroke.width / 2.0);
    let handle = rect.width() * 0.26;
    let corners = [
        rect.left_top(),
        rect.right_top() - egui::vec2(handle, 0.0),
        rect.left_bottom() - egui::vec2(0.0, handle),
        rect.right_bottom() - egui::vec2(handle, handle),
    ];
    for corner in corners {
        let square = Rect::from_min_size(corner, egui::Vec2::splat(handle));
        painter.rect_stroke(square, 0.0, stroke, egui::StrokeKind::Middle);
    }
    let inner = rect.shrink(handle / 2.0);
    let sides = [
        [
            pos2(rect.left() + handle, inner.top()),
            pos2(rect.right() - handle, inner.top()),
        ],
        [
            pos2(rect.left() + handle, inner.bottom()),
            pos2(rect.right() - handle, inner.bottom()),
        ],
        [
            pos2(inner.left(), rect.top() + handle),
            pos2(inner.left(), rect.bottom() - handle),
        ],
        [
            pos2(inner.right(), rect.top() + handle),
            pos2(inner.right(), rect.bottom() - handle),
        ],
    ];
    for side in sides {
        painter.line_segment(side, stroke);
    }
}

/// A cube seen from above a corner: the 3D preview.
pub(super) fn cube(painter: &Painter, rect: Rect, stroke: Stroke) {
    let rect = rect.shrink(stroke.width / 2.0);
    let (top, bottom) = (rect.top(), rect.bottom());
    let quarter = rect.height() / 4.0;
    let center = pos2(rect.center().x, top + 2.0 * quarter);
    let outline = vec![
        pos2(rect.center().x, top),
        pos2(rect.right(), top + quarter),
        pos2(rect.right(), bottom - quarter),
        pos2(rect.center().x, bottom),
        pos2(rect.left(), bottom - quarter),
        pos2(rect.left(), top + quarter),
    ];
    painter.add(egui::Shape::closed_line(outline, stroke));
    painter.line_segment([pos2(rect.left(), top + quarter), center], stroke);
    painter.line_segment([pos2(rect.right(), top + quarter), center], stroke);
    painter.line_segment([center, pos2(rect.center().x, bottom)], stroke);
}

pub(super) fn minimize(painter: &Painter, rect: Rect, stroke: Stroke) {
    painter.hline(rect.x_range(), rect.center().y, stroke);
}

pub(super) fn maximize(painter: &Painter, rect: Rect, stroke: Stroke) {
    painter.rect_stroke(rect, 1.0, stroke, egui::StrokeKind::Inside);
}

/// Two overlapping squares: the window goes back to its size before it was maximized.
pub(super) fn restore(painter: &Painter, rect: Rect, stroke: Stroke) {
    let offset = (rect.width() * 0.22).round();
    let front = Rect::from_min_max(
        pos2(rect.left(), rect.top() + offset),
        pos2(rect.right() - offset, rect.bottom()),
    );
    painter.rect_stroke(front, 1.0, stroke, egui::StrokeKind::Inside);
    let back = front
        .translate(egui::vec2(offset, -offset))
        .shrink(stroke.width / 2.0);
    painter.add(egui::Shape::line(
        vec![
            pos2(back.left(), front.top()),
            back.left_top(),
            back.right_top(),
            back.right_bottom(),
            pos2(front.right(), back.bottom()),
        ],
        stroke,
    ));
}

pub(super) fn close(painter: &Painter, rect: Rect, stroke: Stroke) {
    painter.line_segment([rect.left_top(), rect.right_bottom()], stroke);
    painter.line_segment([rect.right_top(), rect.left_bottom()], stroke);
}
