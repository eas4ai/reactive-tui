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
    let magnitude = libm::pow(10.0, libm::log10(raw).floor());
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

/// Format a tick value: integers without decimals, other values with the
/// fewest decimals (up to three) that show them exactly, and scientific
/// notation far outside the readable range.
pub fn format_tick(value: f64) -> String {
    if !value.is_finite() {
        return String::new();
    }
    if value.abs() >= 1e6 || (value != 0.0 && value.abs() < 0.001) {
        return format!("{value:.1e}");
    }
    if (value - value.round()).abs() < 1e-9 {
        return format!("{:.0}", value.round() + 0.0);
    }
    let decimals: usize = (1..=3)
        .find(|d: &usize| {
            let scaled = value * 10f64.powi(*d as i32);
            (scaled - scaled.round()).abs() < 1e-6
        })
        .unwrap_or(2);
    format!("{value:.decimals$}")
}

/// The tick steps an automatic axis may use, times a power of ten: round
/// values that keep the headroom past the data small.
const STEPS: [f64; 5] = [1.0, 2.0, 2.5, 4.0, 5.0];

/// The smallest round step (1, 2, 2.5, 4 or 5 times a power of ten) that covers
/// `span` in `intervals` steps, so an axis of `intervals + 1` ticks lands on
/// round values (CHT-034).
pub fn tick_step(span: f64, intervals: usize) -> f64 {
    if span.is_nan() || span <= 0.0 || !span.is_finite() {
        return 1.0;
    }
    let raw = span / intervals.max(1) as f64;
    let magnitude = libm::pow(10.0, libm::log10(raw).floor());
    let step = STEPS
        .iter()
        .map(|s| s * magnitude)
        .find(|step| *step >= raw * (1.0 - 1e-9))
        .unwrap_or(10.0 * magnitude);
    // A span of subnormal size divides, or its power of ten rounds, to
    // zero: the smallest normal step still covers it, and every step is a
    // finite positive normal number the widening below can grow (CHT-026).
    if step.is_normal() && step > 0.0 {
        step
    } else {
        f64::MIN_POSITIVE.max(raw)
    }
}

/// The next round step after `step`.
fn next_step(step: f64) -> f64 {
    let magnitude = libm::pow(10.0, libm::log10(step).floor());
    let residual = step / magnitude;
    STEPS
        .iter()
        .map(|s| s * magnitude)
        .find(|next| *next > residual * magnitude * (1.0 + 1e-9))
        .unwrap_or(10.0 * magnitude)
}

/// `domain` widened so that `count` ticks fall on round values: a free end
/// moves out to a multiple of a round step such that `count - 1`
/// steps cover the data (CHT-034). `pinned` says which ends the builder set;
/// both pinned, or fewer than two ticks, leaves the domain alone.
pub fn nice_domain(domain: (f64, f64), pinned: (bool, bool), count: usize) -> (f64, f64) {
    let (low, high) = domain;
    if count < 2
        || (pinned.0 && pinned.1)
        || high.is_nan()
        || high <= low
        || !(high - low).is_finite()
    {
        return domain;
    }
    let intervals = (count - 1) as f64;
    let mut step = tick_step(high - low, count - 1);
    match pinned {
        (true, false) => (low, low + intervals * step),
        (false, true) => (high - intervals * step, high),
        _ => {
            // Each round step is larger than the last, so the domain is
            // covered within a few hundred steps of a positive normal step;
            // a step that stops growing, or a bound reached, leaves the
            // domain as it is rather than looping on (CHT-026).
            for _ in 0..1024 {
                let start = (low / step).floor() * step;
                let end = start + intervals * step;
                if !start.is_finite() || !end.is_finite() {
                    return domain;
                }
                if end >= high * (1.0 - 1e-9) - step * 1e-9 {
                    // Zero stays zero and the ends read as the step's multiples.
                    let start = if start.abs() < step * 1e-9 {
                        0.0
                    } else {
                        start
                    };
                    return (start, end);
                }
                let next = next_step(step);
                if !(next > step && next.is_finite()) {
                    return domain;
                }
                step = next;
            }
            domain
        }
    }
}

/// Exactly `count` ticks spread evenly from the domain's low end to its high
/// end, labels formatted with [`format_tick`]; a `count` under two gives the
/// low end alone. An automatic domain is widened by [`nice_domain`] first so
/// these land on round values; a pinned one divides as asked (CHT-034).
pub fn linear_ticks(scale: &ScaleLinear, count: usize) -> Vec<Tick> {
    let (a, b) = scale.domain();
    let (low, high) = (a.min(b), a.max(b));
    let tick = |value: f64| Tick {
        value,
        position: scale.map(value),
        label: format_tick(value),
    };
    if count < 2 || (high - low).is_nan() || high <= low {
        return vec![tick(low)];
    }
    let intervals = count - 1;
    let step = (high - low) / intervals as f64;
    (0..=intervals)
        .map(|i| {
            let value = if i == intervals {
                high
            } else {
                low + step * i as f64
            };
            tick(snap(value, step))
        })
        .collect()
}

/// `value` rounded to two decimals past `step`'s magnitude, so a 0.1 step
/// reads 0.3 rather than 0.30000000000000004, and a value within rounding of
/// zero is zero.
fn snap(value: f64, step: f64) -> f64 {
    if step.is_nan() || step <= 0.0 || !step.is_finite() {
        return value;
    }
    let decimals = ((-libm::log10(step).floor()).max(0.0) as i32 + 2).min(12);
    let factor = 10f64.powi(decimals);
    let snapped = (value * factor).round() / factor;
    if snapped.abs() < step * 1e-9 {
        0.0
    } else {
        snapped
    }
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
///
/// Only the first `shown` bands get a tick: a chart laid out for more slots
/// than it has data labels its data alone, whatever the slot count asks
/// (CHT-040).
pub fn band_ticks(scale: &ScaleBand, labels: &[Option<String>], shown: usize) -> Vec<Tick> {
    (0..scale.count().min(shown))
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

/// One tick per point, labelled from `labels` (falling back to the index),
/// for the first `shown` points (CHT-040).
pub fn point_ticks(scale: &ScalePoint, labels: &[Option<String>], shown: usize) -> Vec<Tick> {
    (0..scale.count().min(shown))
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

/// At most `count` of `ticks`, spread evenly from the first to the last: an
/// axis's `label_count` (CHT-034). A `count` of zero, or one at least the
/// number of ticks, keeps every tick.
pub fn spread_ticks(ticks: Vec<Tick>, count: usize) -> Vec<Tick> {
    if count == 0 || ticks.len() <= count {
        return ticks;
    }
    let last = ticks.len() - 1;
    (0..count)
        .map(|k| {
            let i = if count == 1 {
                0
            } else {
                (k * last + (count - 1) / 2) / (count - 1)
            };
            ticks[i].clone()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ticks(n: usize) -> Vec<Tick> {
        (0..n)
            .map(|i| Tick {
                value: i as f64,
                position: i as f64 * 10.0,
                label: i.to_string(),
            })
            .collect()
    }

    #[test]
    fn spread_ticks_keeps_the_ends_and_spaces_the_rest() {
        let picked: Vec<String> = spread_ticks(ticks(5), 3)
            .into_iter()
            .map(|t| t.label)
            .collect();
        assert_eq!(picked, ["0", "2", "4"]);
        let picked: Vec<String> = spread_ticks(ticks(10), 4)
            .into_iter()
            .map(|t| t.label)
            .collect();
        assert_eq!(picked, ["0", "3", "6", "9"]);
        let one: Vec<String> = spread_ticks(ticks(6), 1)
            .into_iter()
            .map(|t| t.label)
            .collect();
        assert_eq!(one, ["0"]);
    }

    #[test]
    fn spread_ticks_leaves_a_short_or_unlimited_axis_alone() {
        assert_eq!(spread_ticks(ticks(3), 5).len(), 3);
        assert_eq!(spread_ticks(ticks(3), 3).len(), 3);
        assert_eq!(spread_ticks(ticks(7), 0).len(), 7);
    }

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
    fn linear_ticks_are_exactly_the_count_asked() {
        let scale = ScaleLinear::new((0.0, 10.0), (0.0, 100.0));
        let ticks = linear_ticks(&scale, 5);
        let values: Vec<f64> = ticks.iter().map(|t| t.value).collect();
        assert_eq!(values, vec![0.0, 2.5, 5.0, 7.5, 10.0]);
        assert_eq!(ticks[1].position, 25.0);
        assert_eq!(ticks[1].label, "2.5");
        // A tick count of 3 over 0 to 8 is three ticks, never five
        // (CHT-034's falsifier).
        let three = linear_ticks(&ScaleLinear::new((0.0, 8.0), (0.0, 1.0)), 3);
        let values: Vec<f64> = three.iter().map(|t| t.value).collect();
        assert_eq!(values, vec![0.0, 4.0, 8.0]);
        let negative = linear_ticks(&ScaleLinear::new((-5.0, 5.0), (0.0, 10.0)), 3);
        assert_eq!(negative[1].value, 0.0);
        assert_eq!(negative[1].label, "0");
        assert_eq!(linear_ticks(&scale, 1).len(), 1);
        let tenths = linear_ticks(&ScaleLinear::new((0.0, 0.3), (0.0, 1.0)), 4);
        assert_eq!(tenths[1].value, 0.1);
        assert_eq!(tenths[3].label, "0.3");
    }

    #[test]
    fn nice_domain_widens_a_free_end_to_round_steps_for_the_count() {
        assert_eq!(tick_step(8.0, 4), 2.0);
        assert_eq!(tick_step(10.0, 4), 2.5);
        assert_eq!(tick_step(13.0, 4), 4.0);
        assert_eq!(tick_step(100.0, 2), 50.0);
        assert_eq!(tick_step(0.3, 3), 0.1);
        assert_eq!(nice_domain((0.0, 8.0), (false, false), 5), (0.0, 8.0));
        assert_eq!(nice_domain((0.0, 10.0), (false, false), 5), (0.0, 10.0));
        assert_eq!(nice_domain((0.0, 13.0), (false, false), 5), (0.0, 16.0));
        assert_eq!(nice_domain((-3.0, 8.0), (false, false), 5), (-4.0, 12.0));
        assert_eq!(nice_domain((-7.0, 7.0), (false, false), 5), (-8.0, 8.0));
        assert_eq!(nice_domain((0.0, 7.0), (true, false), 5), (0.0, 8.0));
        assert_eq!(nice_domain((0.0, 7.0), (false, true), 5), (-1.0, 7.0));
        assert_eq!(nice_domain((0.0, 7.0), (true, true), 5), (0.0, 7.0));
        assert_eq!(nice_domain((0.0, 7.0), (false, false), 1), (0.0, 7.0));
    }

    #[test]
    fn tick_formatting_keeps_integers_short() {
        assert_eq!(format_tick(3.0), "3");
        assert_eq!(format_tick(-0.0), "0");
        assert_eq!(format_tick(2.5), "2.5");
        assert_eq!(format_tick(0.25), "0.25");
        assert_eq!(format_tick(8.0 / 3.0), "2.67");
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
        let ticks = band_ticks(&band, &[Some("a".into()), None], usize::MAX);
        assert_eq!(ticks[0].label, "a");
        assert_eq!(ticks[1].label, "1");
        assert_eq!(ticks[0].position, 5.0);
        let point = ScalePoint::new(3, (0.0, 20.0));
        assert_eq!(point_ticks(&point, &[], usize::MAX)[2].position, 20.0);
        let custom = labeled_ticks(
            &ScaleLinear::new((0.0, 1.0), (0.0, 10.0)),
            &["s".into(), "e".into()],
        );
        assert_eq!(custom[1].position, 10.0);
    }
}
