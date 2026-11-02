use super::{ExportError, number, validate_export};
use crate::model::Project;

pub fn to_svg(project: &Project) -> Result<String, ExportError> {
    validate_export(project)?;
    let bounds = project.bounds().ok_or(ExportError::EmptyProject)?;
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
    for piece in &project.pieces {
        svg.push_str(&format!("    <path data-piece=\"{}\" d=\"", piece.id));
        let mut rings = Vec::new();
        for ring in piece.rings() {
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
    svg.push_str("  </g>\n</svg>\n");
    Ok(svg)
}
