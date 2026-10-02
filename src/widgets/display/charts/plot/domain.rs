//! The value-axis domain of a cartesian chart from its props, shared by the
//! renderer and by the builders, whose tick formats run once at `build()`
//! over the ticks the axis will carry (CHT-011, CHT-034).

use super::scale::ScaleLinear;
use super::tick::linear_ticks;
use crate::widgets::display::charts::{ChartAxis, ChartProps, ChartType};

/// The value-axis domain for `props` under `axis`'s explicit limits: the
/// data's range including zero, or for candles their lows to their highs,
/// and for stacked bars and areas the stacked totals at each index rather
/// than the single values. An empty or degenerate range becomes a span of
/// one around the value.
pub fn value_domain(props: &ChartProps, axis: &ChartAxis) -> Result<(f64, f64), &'static str> {
    let candles = props.chart_type == ChartType::Candlestick;
    let stacked = props.stacked
        && matches!(
            props.chart_type,
            ChartType::BarVertical | ChartType::BarHorizontal | ChartType::Area
        );
    let visible = || props.series.iter().filter(|s| s.visible);
    let auto = if candles {
        let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
        for point in visible().flat_map(|s| s.data.iter()) {
            let (a, b) = point
                .candle
                .map_or((point.value, point.value), |c| (c.low, c.high));
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
        for series in visible() {
            for (i, point) in series.data.iter().enumerate() {
                if point.value.is_finite() {
                    if point.value >= 0.0 {
                        positive[i] += point.value;
                    } else {
                        negative[i] += point.value;
                    }
                }
            }
        }
        ScaleLinear::domain_including_zero(positive.into_iter().chain(negative))
    } else {
        ScaleLinear::domain_including_zero(visible().flat_map(|s| s.data.iter().map(|p| p.value)))
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
    Ok((low, high))
}

/// The values of the ticks the value axis of `props` carries under `axis`:
/// round values across its domain, `axis.tick_count` of them or so, so a
/// tick format run at `build()` labels the same ticks the renderer draws.
pub fn value_ticks(props: &ChartProps, axis: &ChartAxis) -> Vec<f64> {
    let Ok(domain) = value_domain(props, axis) else {
        return Vec::new();
    };
    let scale = ScaleLinear::new(domain, (0.0, 1.0));
    linear_ticks(&scale, axis.tick_count.max(1))
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
        assert_eq!(value_domain(&mixed, &mixed.y_axis), Ok((-7.0, 7.0)));
    }

    /// The ticks a format labels at build are the renderer's round ticks.
    #[test]
    fn value_ticks_are_round_values_across_the_domain() {
        let p = props(ChartType::Line, false, &[&[2.0, 8.0, 5.0]]);
        assert_eq!(value_ticks(&p, &p.y_axis), vec![0.0, 2.0, 4.0, 6.0, 8.0]);
    }
}
