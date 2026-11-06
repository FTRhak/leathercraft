use super::{AlignTo, Item};

/// Equal gaps between the outer items, or between the edges of the project or the parent.
pub(super) fn space(
    items: &[Item],
    order: &[usize],
    to: AlignTo,
    (low, high): (f64, f64),
) -> Vec<(usize, f64)> {
    if order.len() < 2 {
        return Vec::new();
    }
    let (start, end) = if to.is_outer() {
        (low, high)
    } else {
        (
            items[order[0]].min,
            order
                .iter()
                .map(|&index| items[index].max)
                .fold(f64::MIN, f64::max),
        )
    };
    let total: f64 = order.iter().map(|&index| items[index].size()).sum();
    let gap = (end - start - total) / (order.len() - 1) as f64;
    let mut position = start;
    order
        .iter()
        .map(|&index| {
            let target = position;
            position += items[index].size() + gap;
            (index, target)
        })
        .collect()
}

/// The key piece stays, and the others are packed away from it with `gap` between them.
pub(super) fn pack(items: &[Item], order: &[usize], key: u64, gap: f64) -> Vec<(usize, f64)> {
    let Some(center) = order.iter().position(|&index| items[index].id == key) else {
        return Vec::new();
    };
    let mut targets = Vec::with_capacity(order.len());
    let mut position = items[order[center]].max + gap;
    for &index in &order[center + 1..] {
        targets.push((index, position));
        position += items[index].size() + gap;
    }
    let mut position = items[order[center]].min - gap;
    for &index in order[..center].iter().rev() {
        let min = position - items[index].size();
        targets.push((index, min));
        position = min - gap;
    }
    targets
}
