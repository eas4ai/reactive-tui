//! Min/max decimation: a series with more points than the plot has columns
//! keeps each column's minimum and maximum (CHT-027), and every kept sample
//! remembers its original index so the tooltip can report it.

/// One kept sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Index into the original series.
    pub index: usize,
    /// The value at that index.
    pub value: f64,
}

/// Reduce `values` to at most two samples per column across `columns`
/// columns, in index order. Series that already fit are returned unchanged.
/// Non-finite values are skipped.
pub fn decimate_min_max(values: &[f64], columns: usize) -> Vec<Sample> {
    let finite = |(index, value): (usize, &f64)| {
        value.is_finite().then_some(Sample {
            index,
            value: *value,
        })
    };
    if columns == 0 || values.len() <= columns.saturating_mul(2) {
        return values.iter().enumerate().filter_map(finite).collect();
    }
    let per_column = values.len() as f64 / columns as f64;
    let mut out = Vec::with_capacity(columns * 2);
    for column in 0..columns {
        let start = (column as f64 * per_column).floor() as usize;
        let end = (((column + 1) as f64 * per_column).floor() as usize).min(values.len());
        let mut low: Option<Sample> = None;
        let mut high: Option<Sample> = None;
        for sample in values[start..end.max(start)]
            .iter()
            .enumerate()
            .filter_map(|(i, v)| finite((start + i, v)))
        {
            if low.is_none_or(|l| sample.value < l.value) {
                low = Some(sample);
            }
            if high.is_none_or(|h| sample.value > h.value) {
                high = Some(sample);
            }
        }
        match (low, high) {
            (Some(a), Some(b)) if a.index != b.index => {
                if a.index < b.index {
                    out.push(a);
                    out.push(b);
                } else {
                    out.push(b);
                    out.push(a);
                }
            }
            (Some(a), _) => out.push(a),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_series_are_untouched_and_nan_is_dropped() {
        let out = decimate_min_max(&[1.0, f64::NAN, 3.0], 10);
        assert_eq!(out.len(), 2);
        assert_eq!(
            out[1],
            Sample {
                index: 2,
                value: 3.0
            }
        );
    }

    #[test]
    fn large_series_keep_two_per_column_with_original_indices() {
        let values: Vec<f64> = (0..10_000).map(|i| ((i as f64) * 0.01).sin()).collect();
        let out = decimate_min_max(&values, 200);
        assert!(out.len() <= 400, "{}", out.len());
        assert!(out.windows(2).all(|w| w[0].index < w[1].index));
        let max = out.iter().map(|s| s.value).fold(f64::MIN, f64::max);
        assert!(
            (max - 1.0).abs() < 0.01,
            "column maxima must survive: {max}"
        );
        assert!(out.iter().all(|s| values[s.index] == s.value));
    }

    #[test]
    fn zero_columns_returns_everything() {
        assert_eq!(decimate_min_max(&[1.0, 2.0], 0).len(), 2);
    }
}
