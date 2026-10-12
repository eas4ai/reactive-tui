//! How a resizable split shares its length among its panes (NAV-008) and
//! how a divider's move passes cells from pane to pane (NAV-009), in whole
//! cells.

use super::PaneSize;

/// What one pane asks for: its size, if any, and the fewest cells it takes.
pub(super) type Spec = (Option<PaneSize>, u16);

/// Whether two lists of specs ask for the same: a weight compares by its
/// bits, so a weight that is not a number still equals itself.
pub(super) fn same(a: &[Spec], b: &[Spec]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|((a, a_min), (b, b_min))| {
            a_min == b_min
                && match (a, b) {
                    (Some(PaneSize::Cells(a)), Some(PaneSize::Cells(b))) => a == b,
                    (Some(PaneSize::Weight(a)), Some(PaneSize::Weight(b))) => {
                        a.to_bits() == b.to_bits()
                    }
                    (None, None) => true,
                    _ => false,
                }
        })
}

/// The panes' first sizes in `length` cells: the `Cells` sizes first, then
/// the weights share what is left, a pane with no size counting as a weight
/// of one; the first panes with a positive weight take the cells the shares
/// leave over. No pane falls below its minimum while the minimums fit, and
/// the sizes sum to `length`.
pub(super) fn initial(specs: &[Spec], length: u16) -> Vec<u16> {
    let mins: Vec<u16> = specs.iter().map(|(_, min)| *min).collect();
    let mut sizes = vec![0u16; specs.len()];
    let mut fixed = 0u32;
    let mut weights: Vec<(usize, f64)> = Vec::new();
    for (index, (size, min)) in specs.iter().enumerate() {
        match size {
            Some(PaneSize::Cells(cells)) => {
                sizes[index] = (*cells).max(*min);
                fixed += u32::from(sizes[index]);
            }
            Some(PaneSize::Weight(weight)) => {
                let weight = f64::from(*weight);
                weights.push((
                    index,
                    if weight.is_finite() {
                        weight.max(0.0)
                    } else {
                        0.0
                    },
                ));
            }
            None => weights.push((index, 1.0)),
        }
    }
    let rest = u32::from(length).saturating_sub(fixed);
    let total: f64 = weights.iter().map(|(_, weight)| weight).sum();
    // Weights that are all zero share the rest equally.
    let even = total <= 0.0;
    let mut given = 0u32;
    for (index, weight) in &weights {
        let share = if even {
            1.0 / weights.len() as f64
        } else {
            weight / total
        };
        let cells = (f64::from(rest) * share).floor().min(f64::from(u16::MAX)) as u16;
        sizes[*index] = cells;
        given += u32::from(cells);
    }
    let mut left = rest.saturating_sub(given);
    while left > 0 {
        let before = left;
        for (index, weight) in &weights {
            if left > 0 && (even || *weight > 0.0) {
                sizes[*index] = sizes[*index].saturating_add(1);
                left -= 1;
            }
        }
        if left == before {
            break;
        }
    }
    for (size, min) in sizes.iter_mut().zip(&mins) {
        *size = (*size).max(*min);
    }
    let grow = weights
        .last()
        .map_or(specs.len().saturating_sub(1), |(index, _)| *index);
    fit(sizes, &mins, length, grow)
}

/// The sizes that keep each pane's share of `length` (NAV-008): each pane's
/// whole cells, the cells left over to the panes with the largest
/// remainders, the earlier pane first on a tie; then each pane below its
/// minimum raised to it, with the cells taken from the panes with the most
/// to spare. The sizes sum to `length`.
pub(super) fn rescale(shares: &[f64], mins: &[u16], length: u16) -> Vec<u16> {
    let ideal: Vec<f64> = shares
        .iter()
        .map(|share| {
            let share = if share.is_finite() {
                share.max(0.0)
            } else {
                0.0
            };
            share * f64::from(length)
        })
        .collect();
    let mut sizes: Vec<u16> = ideal
        .iter()
        .map(|cells| cells.floor().min(f64::from(u16::MAX)) as u16)
        .collect();
    let given: u32 = sizes.iter().map(|size| u32::from(*size)).sum();
    let mut left = u32::from(length).saturating_sub(given);
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by(|a, b| {
        let fraction = |index: usize| ideal[index] - ideal[index].floor();
        fraction(*b)
            .partial_cmp(&fraction(*a))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(b))
    });
    while left > 0 && !order.is_empty() {
        for index in &order {
            if left == 0 {
                break;
            }
            sizes[*index] = sizes[*index].saturating_add(1);
            left -= 1;
        }
    }
    let mut owed = 0u32;
    for (size, min) in sizes.iter_mut().zip(mins) {
        if *size < *min {
            owed += u32::from(*min - *size);
            *size = *min;
        }
    }
    while owed > 0 {
        let Some(richest) = (0..sizes.len())
            .filter(|index| sizes[*index] > mins[*index])
            .max_by_key(|index| (sizes[*index] - mins[*index], std::cmp::Reverse(*index)))
        else {
            break;
        };
        sizes[richest] -= 1;
        owed -= 1;
    }
    let last = sizes.len().saturating_sub(1);
    fit(sizes, mins, length, last)
}

/// `sizes` made to sum to `length`: cells over it are taken from the last
/// panes first, each down to its minimum and, when the minimums themselves
/// do not fit, below it; cells under it go to pane `grow`.
fn fit(mut sizes: Vec<u16>, mins: &[u16], length: u16, grow: usize) -> Vec<u16> {
    let total: u32 = sizes.iter().map(|size| u32::from(*size)).sum();
    let length = u32::from(length);
    if total > length {
        let mut excess = total - length;
        for keep_minimums in [true, false] {
            for index in (0..sizes.len()).rev() {
                let floor = if keep_minimums { mins[index] } else { 0 };
                let take = u32::from(sizes[index].saturating_sub(floor)).min(excess);
                sizes[index] -= take as u16;
                excess -= take;
            }
        }
    } else if let Some(size) = sizes.get_mut(grow) {
        let missing = (length - total).min(u32::from(u16::MAX)) as u16;
        *size = size.saturating_add(missing);
    }
    sizes
}

/// `sizes` after the divider after pane `divider` moves `delta` cells,
/// right or down when positive (NAV-009): the panes on the side it moves
/// toward give cells, the pane beside the divider down to its minimum and
/// then the next one beyond it, each down to its minimum; the pane on the
/// other side of the divider takes them all.
pub(super) fn moved(sizes: &[u16], mins: &[u16], divider: usize, delta: i32) -> Vec<u16> {
    let mut sizes = sizes.to_vec();
    if divider + 1 >= sizes.len() || delta == 0 {
        return sizes;
    }
    let (taker, givers): (usize, Vec<usize>) = if delta > 0 {
        (divider, (divider + 1..sizes.len()).collect())
    } else {
        (divider + 1, (0..=divider).rev().collect())
    };
    let mut wanted = delta.unsigned_abs();
    let mut taken = 0u16;
    for index in givers {
        let room = u32::from(sizes[index].saturating_sub(mins[index]));
        let take = room.min(wanted) as u16;
        sizes[index] -= take;
        wanted -= u32::from(take);
        taken = taken.saturating_add(take);
        if wanted == 0 {
            break;
        }
    }
    sizes[taker] = sizes[taker].saturating_add(taken);
    sizes
}

/// The least and the greatest size the pane before divider `divider` can
/// take by moves of that divider: down to its own minimum, as a move back
/// takes from it first, and up by every cell the panes after it can give.
pub(super) fn bounds(sizes: &[u16], mins: &[u16], divider: usize) -> (u16, u16) {
    let size = sizes.get(divider).copied().unwrap_or(0);
    let least = mins.get(divider).copied().unwrap_or(0).min(size);
    let room: u32 = (divider + 1..sizes.len())
        .map(|index| u32::from(sizes[index].saturating_sub(mins[index])))
        .sum();
    let greatest = (u32::from(size) + room).min(u32::from(u16::MAX)) as u16;
    (least, greatest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn specs(sizes: &[Option<PaneSize>]) -> Vec<Spec> {
        sizes.iter().map(|size| (*size, 4)).collect()
    }

    /// NAV-008: sizes first, then weights; equal shares otherwise, the
    /// first pane taking the odd cell; a minimum holds.
    #[test]
    fn nav_008_panes_share_by_sizes_weights_and_minimums() {
        assert_eq!(initial(&specs(&[None, None]), 99), vec![50, 49]);
        assert_eq!(
            initial(&specs(&[Some(PaneSize::Cells(49)), None]), 99),
            vec![49, 50]
        );
        assert_eq!(
            initial(
                &specs(&[
                    Some(PaneSize::Cells(20)),
                    Some(PaneSize::Weight(1.0)),
                    Some(PaneSize::Weight(2.0)),
                ]),
                98
            ),
            vec![20, 26, 52]
        );
        assert_eq!(
            initial(&specs(&[Some(PaneSize::Cells(2)), None]), 99),
            vec![4, 95]
        );
        assert_eq!(
            initial(&[(Some(PaneSize::Cells(5)), 10), (None, 4)], 99),
            vec![10, 89]
        );
        // Sizes that ask for more than the length give it back from the
        // last pane, down to its minimum.
        assert_eq!(
            initial(
                &specs(&[Some(PaneSize::Cells(80)), Some(PaneSize::Cells(80))]),
                99
            ),
            vec![80, 19]
        );
        // Sizes that ask for less leave the rest to the last pane.
        assert_eq!(
            initial(
                &specs(&[Some(PaneSize::Cells(10)), Some(PaneSize::Cells(10))]),
                99
            ),
            vec![10, 89]
        );
        let shared = initial(&specs(&[None, None, None]), 98);
        assert_eq!(shared.iter().sum::<u16>(), 98);
        assert!(shared.iter().max().unwrap() - shared.iter().min().unwrap() <= 1);
    }

    /// NAV-008: a pane of 30 of 99 cells is 60 or 61 of 199.
    #[test]
    fn nav_008_a_rescale_keeps_each_share() {
        let sizes = rescale(&[30.0 / 99.0, 69.0 / 99.0], &[4, 4], 199);
        assert_eq!(sizes.iter().sum::<u16>(), 199);
        assert!(sizes[0] == 60 || sizes[0] == 61, "{sizes:?}");
        // A share too small for a pane's minimum is raised to it.
        assert_eq!(rescale(&[0.01, 0.99], &[4, 4], 50), vec![4, 46]);
    }

    /// NAV-009: a move takes from the pane beside the divider, then the
    /// next, each down to its minimum.
    #[test]
    fn nav_009_a_move_takes_from_the_next_panes_down_to_their_minimums() {
        let mins = [4, 4, 4];
        assert_eq!(moved(&[49, 50], &mins[..2], 0, 10), vec![59, 40]);
        assert_eq!(moved(&[49, 50], &mins[..2], 0, 1000), vec![95, 4]);
        assert_eq!(moved(&[49, 50], &mins[..2], 0, -1000), vec![4, 95]);
        assert_eq!(moved(&[33, 32, 33], &mins, 0, 47), vec![80, 4, 14]);
        assert_eq!(moved(&[33, 32, 33], &mins, 0, 1000), vec![90, 4, 4]);
        assert_eq!(moved(&[33, 32, 33], &mins, 1, -1000), vec![4, 4, 90]);
        assert_eq!(bounds(&[49, 50], &mins[..2], 0), (4, 95));
        assert_eq!(bounds(&[33, 32, 33], &mins, 0), (4, 90));
    }
}
