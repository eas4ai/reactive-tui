use super::super::{BreadcrumbProps, OverflowStrategy};

pub(super) struct Plan {
    pub entries: Vec<(Option<usize>, Option<usize>)>,
    pub overflow: bool,
}

impl Plan {
    pub fn new(
        props: &BreadcrumbProps,
        widths: Option<&[usize]>,
        available: Option<usize>,
        separator: usize,
    ) -> Self {
        let all = || {
            (0..props.segments.len())
                .map(|index| (Some(index), None))
                .collect()
        };
        let (Some(widths), Some(available)) = (widths, available) else {
            return Self {
                entries: all(),
                overflow: false,
            };
        };
        let total = widths
            .iter()
            .fold(0usize, |sum, width| sum.saturating_add(*width))
            .saturating_add(separator.saturating_mul(widths.len().saturating_sub(1)));
        if total <= available
            || matches!(
                props.overflow_strategy,
                OverflowStrategy::Scroll | OverflowStrategy::Wrap
            )
        {
            return Self {
                entries: all(),
                overflow: total > available,
            };
        }
        if available == 0 || widths.is_empty() {
            return Self {
                entries: Vec::new(),
                overflow: total > available,
            };
        }
        let mut indices = match props.overflow_strategy {
            OverflowStrategy::MiddleEllipsis if widths.len() > 2 => {
                let mut visible = vec![Some(0), None, Some(widths.len() - 1)];
                let mut used = widths[0]
                    .saturating_add(3)
                    .saturating_add(widths[widths.len() - 1])
                    .saturating_add(separator.saturating_mul(2));
                for index in (1..widths.len() - 1).rev() {
                    let next = used.saturating_add(widths[index]).saturating_add(separator);
                    if next > available {
                        break;
                    }
                    visible.insert(2, Some(index));
                    used = next;
                }
                visible
            }
            OverflowStrategy::TruncateStart | OverflowStrategy::TruncateEnd => {
                let backwards = props.overflow_strategy == OverflowStrategy::TruncateStart;
                let mut visible = Vec::new();
                let mut used = 0usize;
                for position in 0..widths.len() {
                    let index = if backwards {
                        widths.len() - position - 1
                    } else {
                        position
                    };
                    let next = used
                        .saturating_add(widths[index])
                        .saturating_add(if visible.is_empty() { 0 } else { separator });
                    if next > available && !visible.is_empty() {
                        break;
                    }
                    visible.push(Some(index));
                    used = next;
                }
                if backwards {
                    visible.reverse();
                }
                visible
            }
            _ => (0..widths.len()).map(Some).collect(),
        };
        // A terminal narrower than the separators cannot show a complete path.
        // Keep the last selected segment and clip it to the available cells.
        if separator.saturating_mul(indices.len().saturating_sub(1)) >= available {
            indices = indices
                .into_iter()
                .rev()
                .find(Option::is_some)
                .into_iter()
                .collect();
        }
        let budget =
            available.saturating_sub(separator.saturating_mul(indices.len().saturating_sub(1)));
        let natural: Vec<_> = indices
            .iter()
            .map(|index| index.map_or(3, |index| widths[index]))
            .collect();
        let mut allocated = natural.clone();
        if natural
            .iter()
            .fold(0usize, |sum, width| sum.saturating_add(*width))
            > budget
        {
            let mut remaining = budget;
            for (index, width) in allocated.iter_mut().enumerate() {
                *width = (*width).min(remaining / (natural.len() - index));
                remaining -= *width;
            }
            for (width, natural) in allocated.iter_mut().zip(&natural) {
                let extra = remaining.min(natural - *width);
                *width += extra;
                remaining -= extra;
            }
        }
        Self {
            entries: indices
                .into_iter()
                .zip(allocated.into_iter().map(Some))
                .collect(),
            overflow: true,
        }
    }
}
