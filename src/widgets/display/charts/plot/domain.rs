//! The value-axis domain of a cartesian chart from its props, shared by the
//! renderer and by the builders, whose tick formats run once at `build()`
//! over the ticks the axis will carry (CHT-011, CHT-034).

use super::scale::ScaleLinear;
use super::tick::{linear_ticks, nice_domain};
use crate::widgets::display::charts::{ChartAxis, ChartProps, ChartType, DataPoint};

/// The value-axis domain for `props` under `axis`'s explicit limits: the
/// data's range including zero, or for candles their lows to their highs,
/// and for stacked bars and areas the stacked totals at each index rather
/// than the single values. An empty or degenerate range becomes a span of
/// one around the value.
pub fn value_domain(props: &ChartProps, axis: &ChartAxis) -> Result<(f64, f64), &'static str> {
    value_domain_with(props, axis, None)
}

/// [`value_domain`] over `values` in place of the points' own: the values a
/// transition is showing this frame (four per point for candles), so the
/// automatic range follows the motion and no shape dips below its previous
/// rendering while the target range is wider (CHT-022). `None` reads the
/// points.
pub fn value_domain_with(
    props: &ChartProps,
    axis: &ChartAxis,
    values: Option<&[Vec<f64>]>,
) -> Result<(f64, f64), &'static str> {
    let candles = props.chart_type == ChartType::Candlestick;
    let stacked = props.stacked
        && matches!(
            props.chart_type,
            ChartType::BarVertical | ChartType::BarHorizontal | ChartType::Area
        );
    let visible = || props.series.iter().filter(|s| s.visible);
    let auto = if candles {
        let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
        for (s, point, i) in visible_points(props) {
            let (a, b) = match values.and_then(|v| v.get(s)) {
                Some(v) if v.len() == props.series[s].data.len() * 4 => {
                    (v[i * 4 + 2], v[i * 4 + 1])
                }
                Some(v) => (
                    v.get(i).copied().unwrap_or(point.value),
                    v.get(i).copied().unwrap_or(point.value),
                ),
                None => point
                    .candle
                    .map_or((point.value, point.value), |c| (c.low, c.high)),
            };
            if a.is_finite() && b.is_finite() {
                low = low.min(a);
                high = high.max(b);
            }
        }
        if low > high {
            (0.0, 1.0)
        } else if low == high {
            (low - 1.0, high + 1.0)
        } else {
            (low, high)
        }
    } else if stacked {
        // Positive and negative values stack separately from zero, so the
        // domain covers the positive total above and the negative one below.
        let count = visible().map(|s| s.data.len()).max().unwrap_or(0);
        let mut positive = vec![0.0f64; count];
        let mut negative = vec![0.0f64; count];
        for (s, point, i) in visible_points(props) {
            let value = value_of(values, s, i, point.value);
            if value.is_finite() {
                if value >= 0.0 {
                    positive[i] += value;
                } else {
                    negative[i] += value;
                }
            }
        }
        ScaleLinear::domain_including_zero(positive.into_iter().chain(negative))
    } else {
        ScaleLinear::domain_including_zero(
            visible_points(props).map(|(s, point, i)| value_of(values, s, i, point.value)),
        )
    };
    let mut low = axis.min.unwrap_or(auto.0);
    let mut high = axis.max.unwrap_or(auto.1);
    if low == high {
        let pad = low.abs().max(1.0) * 0.1;
        if axis.min.is_none() {
            low -= pad;
        }
        if axis.max.is_none() {
            high += pad;
        }
    }
    if low >= high || !(high - low).is_finite() {
        return Err("Axis range must have a finite positive span");
    }
    // A free end widens to a round step grid for the axis's tick count, so
    // exactly that many ticks land on round values (CHT-034).
    Ok(nice_domain(
        (low, high),
        (axis.min.is_some(), axis.max.is_some()),
        axis.tick_count,
    ))
}

/// Every point of every visible series as (series index, point, point index).
fn visible_points(props: &ChartProps) -> impl Iterator<Item = (usize, &DataPoint, usize)> {
    props
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| s.visible)
        .flat_map(|(s, series)| series.data.iter().enumerate().map(move |(i, p)| (s, p, i)))
}

/// The value shown for point `i` of series `s`: the motion's when given,
/// else `own`.
fn value_of(values: Option<&[Vec<f64>]>, s: usize, i: usize, own: f64) -> f64 {
    values
        .and_then(|v| v.get(s))
        .and_then(|v| v.get(i))
        .copied()
        .unwrap_or(own)
}

/// The values of the ticks the value axis of `props` carries under `axis`:
/// exactly `axis.tick_count` round values across its domain, so a tick
/// format run at `build()` labels the same ticks the renderer draws; never
/// more than the chart has cells on its longer side, or than a terminal side
/// has where the chart fills its box, whatever count was asked for
/// (CHT-040).
pub fn value_ticks(props: &ChartProps, axis: &ChartAxis) -> Vec<f64> {
    let Ok(domain) = value_domain(props, axis) else {
        return Vec::new();
    };
    let cells = if props.width == 0 || props.height == 0 {
        u16::MAX
    } else {
        props.width.max(props.height)
    };
    let scale = ScaleLinear::new(domain, (0.0, 1.0));
    linear_ticks(&scale, axis.tick_count.clamp(1, usize::from(cells)))
        .into_iter()
        .map(|tick| tick.value)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::display::charts::{DataPoint, DataSeries};

    fn props(kind: ChartType, stacked: bool, series: &[&[f64]]) -> ChartProps {
        ChartProps {
            chart_type: kind,
            stacked,
            series: series
                .iter()
                .map(|values| {
                    DataSeries::new("s", values.iter().map(|v| DataPoint::new(*v)).collect())
                })
                .collect(),
            ..Default::default()
        }
    }

    /// CHT-011: two stacked series of 8 end the axis at 16, not 8.
    #[test]
    fn stacked_totals_extend_the_domain() {
        let p = props(ChartType::BarVertical, true, &[&[8.0], &[8.0]]);
        assert_eq!(value_domain(&p, &p.y_axis), Ok((0.0, 16.0)));
        let grouped = props(ChartType::BarVertical, false, &[&[8.0], &[8.0]]);
        assert_eq!(value_domain(&grouped, &grouped.y_axis), Ok((0.0, 8.0)));
        let mixed = props(ChartType::Area, true, &[&[3.0, -2.0], &[4.0, -5.0]]);
        // Five ticks over -7 to 7 land on -8, -4, 0, 4 and 8 (CHT-034).
        assert_eq!(value_domain(&mixed, &mixed.y_axis), Ok((-8.0, 8.0)));
    }

    /// The ticks a format labels at build are the renderer's round ticks.
    #[test]
    fn value_ticks_are_round_values_across_the_domain() {
        let p = props(ChartType::Line, false, &[&[2.0, 8.0, 5.0]]);
        assert_eq!(value_ticks(&p, &p.y_axis), vec![0.0, 2.0, 4.0, 6.0, 8.0]);
    }
}
