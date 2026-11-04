use super::{ExportError, HoleOutput, centerlines, number, stitch_holes, validate_export};
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
    // Each dash is its own LINE, so the pattern survives without a linetype table.
    for (_, _, line) in centerlines(project) {
        for (a, b) in line.dashes() {
            dxf.push_str(&format!(
                "0\nLINE\n8\nCENTERLINE\n10\n{}\n20\n{}\n11\n{}\n21\n{}\n",
                number(a.x),
                number(a.y),
                number(b.x),
                number(b.y)
            ));
        }
    }
    for (_, _, hole) in stitch_holes(project) {
        match hole {
            HoleOutput::Circle(center, radius) => dxf.push_str(&format!(
                "0\nCIRCLE\n8\nSTITCH_HOLES\n10\n{}\n20\n{}\n40\n{}\n",
                number(center.x),
                number(center.y),
                number(radius)
            )),
            HoleOutput::Outline(ring) => {
                dxf.push_str(&format!(
                    "0\nLWPOLYLINE\n8\nSTITCH_HOLES\n90\n{}\n70\n1\n",
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
        }
    }
    dxf.push_str("0\nENDSEC\n0\nEOF\n");
    Ok(dxf)
}
