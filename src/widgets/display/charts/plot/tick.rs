//! Tick generation and label formatting for axes.

use super::scale::{ScaleBand, ScaleLinear, ScalePoint};

/// One axis tick: the data value, its range position and its label.
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    /// Data value the tick marks.
    pub value: f64,
    /// Position on the axis in the scale's range units.
    pub position: f64,
    /// Text shown beside the tick.
    pub label: String,
}

/// A round step (1, 2 or 5 times a power of ten) that yields about `count`
/// ticks across `span`.
pub fn nice_step(span: f64, count: usize) -> f64 {
    if span.is_nan() || span <= 0.0 || !span.is_finite() {
        return 1.0;
    }
    let raw = span / count.max(1) as f64;
    let magnitude = 10f64.powf(raw.log10().floor());
    let residual = raw / magnitude;
    // d3's tickStep thresholds: sqrt(50), sqrt(10) and sqrt(2).
    let factor = if residual >= 7.07 {
        10.0
    } else if residual >= 3.16 {
        5.0
    } else if residual >= 1.41 {
        2.0
    } else {
        1.0
    };
    factor * magnitude
}

/// Format a tick value: integers without decimals, other values with one,
/// and scientific notation far outside the readable range.
pub fn format_tick(value: f64) -> String {
    if !value.is_finite() {
        return String::new();
    }
    if value.abs() >= 1e6 || (value != 0.0 && value.abs() < 0.001) {
        return format!("{value:.1e}");
    }
    if (value - value.round()).abs() < 1e-9 {
        format!("{:.0}", value.round() + 0.0)
    } else {
        format!("{value:.1}")
    }
}

/// Ticks at round values across a linear scale, about `count` of them,
/// always including the domain ends' nearest round values inside the domain.
pub fn linear_ticks(scale: &ScaleLinear, count: usize) -> Vec<Tick> {
    let (a, b) = scale.domain();
    let (low, high) = (a.min(b), a.max(b));
    if count == 0 || (high - low).is_nan() || high <= low {
        return vec![Tick {
            value: low,
            position: scale.map(low),
            label: format_tick(low),
        }];
    }
    let step = nice_step(high - low, count);
    let first = (low / step).ceil();
    let last = (high / step).floor();
    let mut ticks = Vec::new();
    let mut i = first;
    while i <= last && ticks.len() < 1000 {
        let value = i * step;
        let value = if value.abs() < step * 1e-9 {
            0.0
        } else {
            value
        };
        ticks.push(Tick {
            value,
            position: scale.map(value),
            label: format_tick(value),
        });
        i += 1.0;
    }
    if ticks.is_empty() {
        for value in [low, high] {
            ticks.push(Tick {
                value,
                position: scale.map(value),
                label: format_tick(value),
            });
        }
    }
    ticks
}

/// Ticks at fixed `labels` spread evenly across a linear scale, for axes with
/// custom labels.
pub fn labeled_ticks(scale: &ScaleLinear, labels: &[String]) -> Vec<Tick> {
    let (a, b) = scale.domain();
    let n = labels.len();
    labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let t = if n < 2 {
                0.5
            } else {
                i as f64 / (n - 1) as f64
            };
            let value = a + (b - a) * t;
            Tick {
                value,
                position: scale.map(value),
                label: label.clone(),
            }
        })
        .collect()
}

/// One tick per band, centred on the band, labelled from `labels` (falling
/// back to the index).
pub fn band_ticks(scale: &ScaleBand, labels: &[Option<String>]) -> Vec<Tick> {
    (0..scale.count())
        .map(|i| Tick {
            value: i as f64,
            position: scale.center(i),
            label: labels
                .get(i)
                .cloned()
                .flatten()
                .unwrap_or_else(|| i.to_string()),
        })
        .collect()
}

/// One tick per point, labelled from `labels` (falling back to the index).
pub fn point_ticks(scale: &ScalePoint, labels: &[Option<String>]) -> Vec<Tick> {
    (0..scale.count())
        .map(|i| Tick {
            value: i as f64,
            position: scale.map(i),
            label: labels
                .get(i)
                .cloned()
                .flatten()
                .unwrap_or_else(|| i.to_string()),
        })
        .collect()
}

/// The smallest stride `k` such that every `k`-th label, each `widths[i]`
/// cells wide at `positions[i]`, keeps at least `gap` cells to the next shown
/// label. `1` means every label fits. This is the reference's `tick_margin`,
/// computed from the text instead of supplied by hand.
pub fn label_skip(positions: &[f64], widths: &[usize], gap: usize) -> usize {
    let n = positions.len().min(widths.len());
    if n < 2 {
        return 1;
    }
    'stride: for stride in 1..n {
        let mut previous_end: Option<f64> = None;
        for i in (0..n).step_by(stride) {
            let start = positions[i] - widths[i] as f64 / 2.0;
            if let Some(end) = previous_end {
                if start < end + gap as f64 {
                    continue 'stride;
                }
            }
            previous_end = Some(positions[i] + widths[i] as f64 / 2.0);
        }
        return stride;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_steps_are_one_two_five() {
        assert_eq!(nice_step(10.0, 5), 2.0);
        assert_eq!(nice_step(100.0, 4), 20.0);
        assert_eq!(nice_step(7.0, 10), 0.5);
        assert_eq!(nice_step(100.0, 2), 50.0);
        assert_eq!(nice_step(0.3, 3), 0.1);
        assert_eq!(nice_step(0.0, 3), 1.0);
    }

    #[test]
    fn linear_ticks_land_on_round_values_inside_the_domain() {
        let scale = ScaleLinear::new((0.0, 10.0), (0.0, 100.0));
        let ticks = linear_ticks(&scale, 5);
        let values: Vec<f64> = ticks.iter().map(|t| t.value).collect();
        assert_eq!(values, vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!(ticks[1].position, 20.0);
        assert_eq!(ticks[0].label, "0");
        let negative = linear_ticks(&ScaleLinear::new((-5.0, 5.0), (0.0, 10.0)), 4);
        assert!(negative.iter().any(|t| t.value == 0.0 && t.label == "0"));
        assert!(negative.iter().all(|t| t.value >= -5.0 && t.value <= 5.0));
    }

    #[test]
    fn tick_formatting_keeps_integers_short() {
        assert_eq!(format_tick(3.0), "3");
        assert_eq!(format_tick(-0.0), "0");
        assert_eq!(format_tick(2.5), "2.5");
        assert_eq!(format_tick(1e7), "1.0e7");
        assert_eq!(format_tick(f64::NAN), "");
    }

    #[test]
    fn label_skip_hides_labels_that_would_overlap() {
        let positions: Vec<f64> = (0..10).map(|i| i as f64 * 3.0).collect();
        let widths = vec![4; 10];
        assert_eq!(label_skip(&positions, &widths, 1), 2);
        let wide: Vec<f64> = (0..10).map(|i| i as f64 * 10.0).collect();
        assert_eq!(label_skip(&wide, &widths, 1), 1);
        assert_eq!(label_skip(&[0.0], &[3], 1), 1);
    }

    #[test]
    fn band_and_point_ticks_use_labels_or_indices() {
        let band = ScaleBand::new(2, (0.0, 20.0));
        let ticks = band_ticks(&band, &[Some("a".into()), None]);
        assert_eq!(ticks[0].label, "a");
        assert_eq!(ticks[1].label, "1");
        assert_eq!(ticks[0].position, 5.0);
        let point = ScalePoint::new(3, (0.0, 20.0));
        assert_eq!(point_ticks(&point, &[])[2].position, 20.0);
        let custom = labeled_ticks(
            &ScaleLinear::new((0.0, 1.0), (0.0, 10.0)),
            &["s".into(), "e".into()],
        );
        assert_eq!(custom[1].position, 10.0);
    }
}
