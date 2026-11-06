use super::{Edge, Item};

/// Moves each item's edge or center onto the reference's.
pub(super) fn align(items: &[Item], at: Edge, (low, high): (f64, f64)) -> Vec<(usize, f64)> {
    let target = match at {
        Edge::Min => low,
        Edge::Center => (low + high) * 0.5,
        Edge::Max => high,
    };
    items
        .iter()
        .enumerate()
        .map(|(index, item)| (index, item.min + target - item.at(at)))
        .collect()
}
