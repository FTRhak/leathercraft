use super::{ExportError, HoleOutput, centerlines, number, stitch_holes, validate_export};
use crate::model::{Bounds, Point, Project, ring_bounds};
use crate::shapes::Shape;

pub fn to_svg(project: &Project) -> Result<String, ExportError> {
    let regions = validate_export(project)?;
    // Centerlines and stitch holes may reach past their pieces, so they widen the drawing.
    let line_bounds =
        centerlines(project).filter_map(|(_, _, line)| Shape::Line(line.clone()).bounds());
    let holes = stitch_holes(project);
    let hole_bounds = holes.iter().filter_map(|(_, _, hole)| match hole {
        HoleOutput::Circle(center, radius) => ring_bounds(&[
            Point::new(center.x - radius, center.y - radius),
            Point::new(center.x + radius, center.y + radius),
        ]),
        HoleOutput::Outline(ring) => ring_bounds(ring),
    });
    let bounds = regions
        .iter()
        .filter_map(|region| region.bounds())
        .chain(line_bounds)
        .chain(hole_bounds)
        .reduce(Bounds::union)
        .ok_or(ExportError::EmptyProject)?;
    let (min_x, min_y) = (bounds.min_x, bounds.min_y);
    let width = bounds.width();
    let height = bounds.height();
    let mut svg = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}mm\" height=\"{}mm\" viewBox=\"0 0 {} {}\">\n",
        number(width),
        number(height),
        number(width),
        number(height)
    );
    svg.push_str(
        "  <g fill=\"none\" stroke=\"#111827\" stroke-width=\"0.25\" stroke-linejoin=\"round\">\n",
    );
    for (piece, region) in project.pieces.iter().zip(&regions) {
        svg.push_str(&format!("    <path data-piece=\"{}\" d=\"", piece.id));
        let mut rings = Vec::new();
        for ring in region.rings() {
            let mut path = String::new();
            for (index, point) in ring.iter().enumerate() {
                let command = if index == 0 { 'M' } else { 'L' };
                path.push_str(&format!(
                    "{command} {} {} ",
                    number(point.x - min_x),
                    number(point.y - min_y)
                ));
            }
            path.push('Z');
            rings.push(path);
        }
        svg.push_str(&rings.join(" "));
        svg.push_str("\" />\n");
    }
    svg.push_str("  </g>\n");
    let mut lines = centerlines(project).peekable();
    if lines.peek().is_some() {
        svg.push_str("  <g fill=\"none\" stroke=\"#111827\" stroke-width=\"0.15\">\n");
        for (piece, child, line) in lines {
            let dash = line.style.pattern().map_or_else(String::new, |pattern| {
                let lengths: Vec<String> = pattern.into_iter().map(number).collect();
                format!(" stroke-dasharray=\"{}\"", lengths.join(" "))
            });
            svg.push_str(&format!(
                "    <line data-piece=\"{piece}\" data-child=\"{child}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"{dash} />\n",
                number(line.start.x - min_x),
                number(line.start.y - min_y),
                number(line.end.x - min_x),
                number(line.end.y - min_y),
            ));
        }
        svg.push_str("  </g>\n");
    }
    if !holes.is_empty() {
        svg.push_str("  <g fill=\"none\" stroke=\"#111827\" stroke-width=\"0.1\">\n");
        for (piece, child, hole) in &holes {
            match hole {
                HoleOutput::Circle(center, radius) => svg.push_str(&format!(
                    "    <circle data-piece=\"{piece}\" data-child=\"{child}\" cx=\"{}\" cy=\"{}\" r=\"{}\" />\n",
                    number(center.x - min_x),
                    number(center.y - min_y),
                    number(*radius),
                )),
                HoleOutput::Outline(ring) => {
                    let points: Vec<String> = ring
                        .iter()
                        .map(|point| {
                            format!("{},{}", number(point.x - min_x), number(point.y - min_y))
                        })
                        .collect();
                    svg.push_str(&format!(
                        "    <polygon data-piece=\"{piece}\" data-child=\"{child}\" points=\"{}\" />\n",
                        points.join(" ")
                    ));
                }
            }
        }
        svg.push_str("  </g>\n");
    }
    svg.push_str("</svg>\n");
    Ok(svg)
}
