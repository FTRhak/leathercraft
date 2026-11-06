use super::{AlignTo, Edge, Item};

/// The outer items stay and the rest are spread between them; against the project or the
/// parent, the outer items move to its edges first.
pub(super) fn distribute(
    items: &[Item],
    order: &[usize],
    at: Edge,
    to: AlignTo,
    (low, high): (f64, f64),
) -> Vec<(usize, f64)> {
    let (Some(&first), Some(&last)) = (order.first(), order.last()) else {
        return Vec::new();
    };
    if order.len() < 2 {
        return Vec::new();
    }
    let (start, end) = if to.is_outer() {
        (
            low + items[first].at(at) - items[first].min,
            high - (items[last].max - items[last].at(at)),
        )
    } else {
        (items[first].at(at), items[last].at(at))
    };
    let steps = (order.len() - 1) as f64;
    order
        .iter()
        .enumerate()
        .map(|(position, &index)| {
            let target = start + (end - start) * position as f64 / steps;
            (index, items[index].min + target - items[index].at(at))
        })
        .collect()
}
