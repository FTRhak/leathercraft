use super::{ExportError, number, validate_export};
use crate::model::Project;

pub fn to_dxf(project: &Project) -> Result<String, ExportError> {
    validate_export(project)?;
    let mut dxf = String::from(
        "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n9\n$MEASUREMENT\n70\n1\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n",
    );
    for piece in &project.pieces {
        dxf.push_str(&format!(
            "0\nLWPOLYLINE\n8\nPATTERN\n90\n{}\n70\n1\n",
            piece.points.len()
        ));
        for point in &piece.points {
            dxf.push_str(&format!(
                "10\n{}\n20\n{}\n",
                number(point.x),
                number(point.y)
            ));
        }
    }
    dxf.push_str("0\nENDSEC\n0\nEOF\n");
    Ok(dxf)
}
