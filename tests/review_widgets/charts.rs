use crate::common::app_input;
use reactive_tui::{
    app::RootComponent,
    component::Element,
    widgets::display::{
        Chart, ChartAxis, ChartLegend, ChartProps, ChartType, DataPoint, DataSeries,
    },
};

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
}

/// How many cells of the last painted frame hold `mark`.
fn painted(props: ChartProps, size: (u16, u16), mark: char) -> usize {
    columns_of(props, size, mark).len()
}

/// The columns of the cells of the last painted frame that hold `mark`.
fn columns_of(props: ChartProps, size: (u16, u16), mark: char) -> Vec<usize> {
    let frames = app_input::run(Root(Element::typed::<Chart>(props)), size, vec![(2, None)]);
    frames
        .last()
        .unwrap()
        .text
        .lines()
        .flat_map(|line| {
            line.chars()
                .enumerate()
                .filter(|(_, c)| *c == mark)
                .map(|(column, _)| column)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// A scatter chart with two points at x 10 and x 20.
fn scatter(size: (u16, u16), min: Option<f64>, max: Option<f64>) -> ChartProps {
    let axis = ChartAxis {
        show_labels: false,
        show_grid: false,
        ..Default::default()
    };
    let point = |x: f64, value: f64| {
        let mut point = DataPoint::new(value);
        point.x = Some(x);
        point
    };
    ChartProps {
        chart_type: ChartType::Scatter,
        width: size.0,
        height: size.1,
        series: vec![DataSeries::new(
            "s",
            vec![point(10.0, 1.0), point(20.0, 2.0)],
        )],
        x_axis: ChartAxis {
            min,
            max,
            ..axis.clone()
        },
        y_axis: ChartAxis {
            min: Some(0.0),
            max: Some(3.0),
            ..axis
        },
        legend: ChartLegend {
            visible: false,
            ..Default::default()
        },
        dots: Some(true),
        ..Default::default()
    }
}

/// CHT-041: a numeric x axis's limits are x values, not indices.
#[test]
fn cht_041_scatter_x_limits_bound_by_x_value_not_index() {
    for size in [(24, 12), (40, 18)] {
        assert_eq!(painted(scatter(size, None, None), size, '•'), 2);
        assert_eq!(
            painted(scatter(size, Some(10.0), Some(20.0)), size, '•'),
            2,
            "limits that hold both x values keep both points at {size:?}"
        );
        assert_eq!(
            painted(scatter(size, Some(0.0), Some(1.0)), size, '•'),
            0,
            "limits below both x values draw no point at {size:?}"
        );
    }
}

/// CHT-041, finding 2 of the review: a limit set on one end alone pins that
/// end, so the point at the limit sits at the plot's edge and the other
/// point is clipped; a minimum above every x leaves no point and no
/// invented domain. The plot's edges are where both limits pinned at the
/// points put them.
#[test]
fn cht_041_a_limit_on_one_end_alone_pins_that_end() {
    for size in [(24, 12), (40, 18)] {
        let edges = columns_of(scatter(size, Some(10.0), Some(20.0)), size, '•');
        assert_eq!(edges.len(), 2, "{edges:?}");
        let (left_edge, right_edge) = (edges[0].min(edges[1]), edges[0].max(edges[1]));
        assert!(left_edge < right_edge, "{edges:?}");
        let from_twenty = columns_of(scatter(size, Some(20.0), None), size, '•');
        assert_eq!(
            from_twenty,
            vec![left_edge],
            "a minimum of 20 alone puts the point at 20 on the plot's left edge at {size:?}"
        );
        let to_ten = columns_of(scatter(size, None, Some(10.0)), size, '•');
        assert_eq!(
            to_ten,
            vec![right_edge],
            "a maximum of 10 alone puts the point at 10 on the plot's right edge at {size:?}"
        );
        assert_eq!(painted(scatter(size, Some(30.0), None), size, '•'), 0);
    }
}
