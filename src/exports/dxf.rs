use super::{ExportError, number, validate_export};
use crate::boolean::Region;
use crate::model::Project;

pub fn to_dxf(project: &Project) -> Result<String, ExportError> {
    let regions = validate_export(project)?;
    let mut dxf = String::from(
        "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n9\n$MEASUREMENT\n70\n1\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n",
    );
    for ring in regions.iter().flat_map(Region::rings) {
        dxf.push_str(&format!(
            "0\nLWPOLYLINE\n8\nPATTERN\n90\n{}\n70\n1\n",
            ring.len()
        ));
        for point in ring {
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
