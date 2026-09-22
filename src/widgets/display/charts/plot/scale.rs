//! Scales map data values to positions on one axis. Every chart renderer
//! obtains cell and dot positions only through these types (CHT-010).
//!
//! Ranges are expressed in fractional units chosen by the caller: the
//! renderers pass dot coordinates (two per column, four per row) so the mask
//! canvas can place edges at eighths of a cell.

/// A continuous linear scale from a numeric domain to a numeric range.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleLinear {
    domain: (f64, f64),
    range: (f64, f64),
}

impl ScaleLinear {
    /// Build a scale over `domain` mapped onto `range`. A range whose start
    /// is greater than its end inverts the axis, which is how a y axis that
    /// grows upward is expressed on a grid whose rows grow downward.
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Self {
        Self { domain, range }
    }

    /// The domain that covers `values`, widened to include zero when the
    /// values do not cross it (CHT-011), and given a small span when every
    /// value is equal so the scale never divides by zero.
    pub fn domain_including_zero(values: impl IntoIterator<Item = f64>) -> (f64, f64) {
        let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
        for value in values {
            if value.is_finite() {
                low = low.min(value);
                high = high.max(value);
            }
        }
        if low > high {
            return (0.0, 1.0);
        }
        if low > 0.0 {
            low = 0.0;
        }
        if high < 0.0 {
            high = 0.0;
        }
        if low == high {
            let pad = low.abs().max(1.0);
            return (low - pad * 0.1, high + pad);
        }
        (low, high)
    }

    /// The domain start and end.
    pub fn domain(&self) -> (f64, f64) {
        self.domain
    }

    /// The range start and end.
    pub fn range(&self) -> (f64, f64) {
        self.range
    }

    /// Replace the domain, keeping the range.
    pub fn with_domain(mut self, domain: (f64, f64)) -> Self {
        self.domain = domain;
        self
    }

    /// Map a data value to its range position. Values outside the domain
    /// extrapolate; callers clip in data space first when that matters.
    pub fn map(&self, value: f64) -> f64 {
        let span = self.domain.1 - self.domain.0;
        if span == 0.0 || !span.is_finite() {
            return self.range.0;
        }
        let t = (value - self.domain.0) / span;
        self.range.0 + t * (self.range.1 - self.range.0)
    }

    /// Map a range position back to a data value.
    pub fn invert(&self, position: f64) -> f64 {
        let span = self.range.1 - self.range.0;
        if span == 0.0 {
            return self.domain.0;
        }
        let t = (position - self.range.0) / span;
        self.domain.0 + t * (self.domain.1 - self.domain.0)
    }

    /// Whether `value` lies inside the domain (inclusive).
    pub fn contains(&self, value: f64) -> bool {
        let (low, high) = (
            self.domain.0.min(self.domain.1),
            self.domain.0.max(self.domain.1),
        );
        value >= low && value <= high
    }

    /// The range position of zero, clamped into the range, which every bar
    /// grows from (CHT-011).
    pub fn baseline(&self) -> f64 {
        let (low, high) = (
            self.range.0.min(self.range.1),
            self.range.0.max(self.range.1),
        );
        self.map(0.0).clamp(low, high)
    }
}

/// A band scale places `count` categories side by side with inner padding
/// between bands and outer padding at both ends, as d3's scaleBand does.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleBand {
    count: usize,
    range: (f64, f64),
    padding_inner: f64,
    padding_outer: f64,
}

impl ScaleBand {
    /// A band scale for `count` categories over `range`, with no padding.
    pub fn new(count: usize, range: (f64, f64)) -> Self {
        Self {
            count,
            range,
            padding_inner: 0.0,
            padding_outer: 0.0,
        }
    }

    /// Fraction of each step left empty between neighbouring bands, 0 to 1.
    pub fn padding_inner(mut self, padding: f64) -> Self {
        self.padding_inner = padding.clamp(0.0, 1.0);
        self
    }

    /// Fraction of a step left empty before the first and after the last band.
    pub fn padding_outer(mut self, padding: f64) -> Self {
        self.padding_outer = padding.max(0.0);
        self
    }

    /// Number of categories.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The range start and end.
    pub fn range(&self) -> (f64, f64) {
        self.range
    }

    /// Distance from the start of one band to the start of the next.
    pub fn step(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        let span = (self.range.1 - self.range.0).abs();
        let divisor = self.count as f64 - self.padding_inner + self.padding_outer * 2.0;
        if divisor <= 0.0 {
            span
        } else {
            span / divisor
        }
    }

    /// Width of one band.
    pub fn bandwidth(&self) -> f64 {
        self.step() * (1.0 - self.padding_inner)
    }

    /// Start position of band `index`.
    pub fn start(&self, index: usize) -> f64 {
        let offset = self.step() * (self.padding_outer + index as f64);
        if self.range.1 >= self.range.0 {
            self.range.0 + offset
        } else {
            self.range.0 - offset - self.bandwidth()
        }
    }

    /// Start and end of band `index`.
    pub fn band(&self, index: usize) -> (f64, f64) {
        let start = self.start(index);
        (start, start + self.bandwidth())
    }

    /// Centre of band `index`.
    pub fn center(&self, index: usize) -> f64 {
        let (a, b) = self.band(index);
        (a + b) / 2.0
    }

    /// The band whose step contains `position`, clamped to the ends, so a
    /// pointer anywhere inside a step selects that band (CHT-019).
    pub fn nearest(&self, position: f64) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        let step = self.step();
        if step <= 0.0 {
            return Some(0);
        }
        let origin = self.range.0.min(self.range.1) + step * self.padding_outer;
        let index = ((position - origin) / step).floor();
        Some(index.clamp(0.0, (self.count - 1) as f64) as usize)
    }
}

/// A point scale places `count` categories at evenly spaced positions with
/// the first and last on the range ends.
#[derive(Debug, Clone, PartialEq)]
pub struct ScalePoint {
    count: usize,
    range: (f64, f64),
}

impl ScalePoint {
    /// A point scale for `count` categories over `range`.
    pub fn new(count: usize, range: (f64, f64)) -> Self {
        Self { count, range }
    }

    /// Number of categories.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The range start and end.
    pub fn range(&self) -> (f64, f64) {
        self.range
    }

    /// Distance between neighbouring points.
    pub fn step(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            (self.range.1 - self.range.0) / (self.count - 1) as f64
        }
    }

    /// Position of category `index`; a single category sits at the centre.
    pub fn map(&self, index: usize) -> f64 {
        if self.count < 2 {
            (self.range.0 + self.range.1) / 2.0
        } else {
            self.range.0 + self.step() * index as f64
        }
    }

    /// Position of a fractional category index, for interpolated positions.
    pub fn map_fraction(&self, index: f64) -> f64 {
        if self.count < 2 {
            (self.range.0 + self.range.1) / 2.0
        } else {
            self.range.0 + self.step() * index
        }
    }

    /// The category nearest to `position` (CHT-019).
    pub fn nearest(&self, position: f64) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        let step = self.step();
        if step == 0.0 {
            return Some(0);
        }
        let index = ((position - self.range.0) / step).round();
        Some(index.clamp(0.0, (self.count - 1) as f64) as usize)
    }
}

/// An ordinal scale maps a category index to one of a fixed set of values,
/// cycling when there are more categories than values. Charts use it for
/// series colors.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleOrdinal<T> {
    range: Vec<T>,
}

impl<T: Clone> ScaleOrdinal<T> {
    /// An ordinal scale over `range`.
    pub fn new(range: Vec<T>) -> Self {
        Self { range }
    }

    /// The value for category `index`, or `None` when the range is empty.
    pub fn map(&self, index: usize) -> Option<T> {
        if self.range.is_empty() {
            None
        } else {
            self.range.get(index % self.range.len()).cloned()
        }
    }

    /// The values this scale cycles through.
    pub fn range(&self) -> &[T] {
        &self.range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_maps_and_inverts_with_an_inverted_range() {
        let scale = ScaleLinear::new((0.0, 10.0), (40.0, 0.0));
        assert_eq!(scale.map(0.0), 40.0);
        assert_eq!(scale.map(10.0), 0.0);
        assert_eq!(scale.map(2.5), 30.0);
        assert_eq!(scale.invert(30.0), 2.5);
        assert_eq!(scale.baseline(), 40.0);
    }

    #[test]
    fn linear_domain_includes_zero_only_when_values_do_not_cross_it() {
        assert_eq!(
            ScaleLinear::domain_including_zero([5.0, 6.0, 7.0]),
            (0.0, 7.0)
        );
        assert_eq!(
            ScaleLinear::domain_including_zero([-3.0, -1.0]),
            (-3.0, 0.0)
        );
        assert_eq!(ScaleLinear::domain_including_zero([-2.0, 4.0]), (-2.0, 4.0));
        assert_eq!(ScaleLinear::domain_including_zero([4.0, 4.0]), (0.0, 4.0));
        let flat = ScaleLinear::domain_including_zero([0.0, 0.0]);
        assert!(flat.0 < 0.0 && flat.1 > 0.0);
        assert_eq!(ScaleLinear::domain_including_zero([f64::NAN]), (0.0, 1.0));
    }

    #[test]
    fn band_padding_separates_bands_and_nearest_snaps_to_steps() {
        let scale = ScaleBand::new(4, (0.0, 40.0))
            .padding_inner(0.5)
            .padding_outer(0.25);
        let step = scale.step();
        assert!((step - 10.0).abs() < 1e-9, "{step}");
        assert!((scale.bandwidth() - 5.0).abs() < 1e-9);
        let (a, b) = scale.band(0);
        let (c, _) = scale.band(1);
        assert!(
            b < c,
            "bands must not touch with inner padding: {a} {b} {c}"
        );
        assert_eq!(scale.nearest(0.0), Some(0));
        assert_eq!(scale.nearest(14.0), Some(1));
        assert_eq!(scale.nearest(39.9), Some(3));
        assert_eq!(scale.nearest(1000.0), Some(3));
        assert_eq!(ScaleBand::new(0, (0.0, 1.0)).nearest(0.5), None);
    }

    #[test]
    fn band_with_no_padding_tiles_the_range() {
        let scale = ScaleBand::new(4, (0.0, 40.0));
        assert_eq!(scale.band(0), (0.0, 10.0));
        assert_eq!(scale.band(3), (30.0, 40.0));
    }

    #[test]
    fn point_scale_puts_ends_on_the_range_and_finds_the_nearest() {
        let scale = ScalePoint::new(5, (0.0, 40.0));
        assert_eq!(scale.map(0), 0.0);
        assert_eq!(scale.map(4), 40.0);
        assert_eq!(scale.map(2), 20.0);
        assert_eq!(scale.nearest(12.0), Some(1));
        assert_eq!(scale.nearest(-9.0), Some(0));
        assert_eq!(ScalePoint::new(1, (0.0, 40.0)).map(0), 20.0);
        assert_eq!(ScalePoint::new(0, (0.0, 40.0)).nearest(3.0), None);
    }

    #[test]
    fn ordinal_cycles_and_is_empty_safe() {
        let scale = ScaleOrdinal::new(vec!["a", "b"]);
        assert_eq!(scale.map(0), Some("a"));
        assert_eq!(scale.map(3), Some("b"));
        assert_eq!(ScaleOrdinal::<&str>::new(Vec::new()).map(0), None);
    }
}
