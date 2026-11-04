use super::{ExportError, centerlines, number, validate_export};
use crate::model::{Bounds, Project};
use crate::shapes::Shape;

pub fn to_svg(project: &Project) -> Result<String, ExportError> {
    let regions = validate_export(project)?;
    // Centerlines may reach past their pieces, so they widen the drawing.
    let line_bounds =
        centerlines(project).filter_map(|(_, _, line)| Shape::Line(line.clone()).bounds());
    let bounds = regions
        .iter()
        .filter_map(|region| region.bounds())
        .chain(line_bounds)
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
    svg.push_str("</svg>\n");
    Ok(svg)
}
