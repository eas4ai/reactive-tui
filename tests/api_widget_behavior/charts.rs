use super::{app_input, key, run, Control};
use reactive_tui::{
    builder,
    component::Element,
    event::types::{Event, KeyCode, MouseEvent, MouseEventKind, Position},
    widgets::display::{
        Chart, ChartAxis, ChartProps, ChartType, ChartsBuilder, DataPoint, DataSeries, FillStyle,
        LegendPosition, SizeClass,
    },
};

fn axis() -> ChartAxis {
    ChartAxis {
        show_labels: false,
        show_grid: false,
        ..Default::default()
    }
}
fn props(kind: ChartType, size: (u16, u16)) -> ChartProps {
    ChartProps {
        chart_type: kind,
        width: size.0,
        height: size.1,
        series: vec![DataSeries::new(
            "Measurements",
            vec![
                DataPoint::with_label(2.0, "Low"),
                DataPoint::with_label(8.0, "High").with_metadata("unit", "ms"),
            ],
        )
        .with_color("#ff0000")
        .with_fill_style(FillStyle::Solid)],
        x_axis: axis(),
        y_axis: ChartAxis {
            min: Some(0.0),
            max: Some(10.0),
            ..axis()
        },
        legend: reactive_tui::widgets::display::ChartLegend {
            visible: false,
            ..Default::default()
        },
        ..Default::default()
    }
}
fn last(element: Element, size: (u16, u16)) -> app_input::Snapshot {
    run(Control(element), size, vec![(2, None)]).pop().unwrap()
}
fn count(frame: &app_input::Snapshot, mark: char) -> usize {
    frame.text.chars().filter(|c| *c == mark).count()
}
fn is_braille(c: char) -> bool {
    ('\u{2800}'..='\u{28FF}').contains(&c)
}
fn braille(frame: &app_input::Snapshot) -> usize {
    frame.text.chars().filter(|c| is_braille(*c)).count()
}
fn hover(x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(MouseEvent::new(
        MouseEventKind::Move,
        Position::cell(x, y),
    )))
}
/// Every cell holding `mark`, as (row, column).
fn cells_with(frame: &app_input::Snapshot, mark: &str) -> Vec<(u16, u16)> {
    let (rows, cols) = frame.screen.size();
    (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .filter(|(r, c)| {
            frame
                .screen
                .cell(*r, *c)
                .is_some_and(|cell| cell.contents() == mark)
        })
        .collect()
}

#[test]
fn charts_render_distinct_geometry_at_two_measured_sizes() {
    for size in [(20, 10), (36, 14)] {
        let line = last(Element::typed::<Chart>(props(ChartType::Line, size)), size);
        let area = last(Element::typed::<Chart>(props(ChartType::Area, size)), size);
        let scatter = last(
            Element::typed::<Chart>(props(ChartType::Scatter, size)),
            size,
        );
        assert!(braille(&line) > 5, "{}", line.text);
        assert_eq!(count(&scatter, '•'), 2, "{}", scatter.text);
        assert_eq!(count(&line, '●'), 2, "{}", line.text);
        assert!(count(&area, '█') > 5, "{}", area.text);
        assert_eq!(count(&line, '█'), 0);
        let pie = last(Element::typed::<Chart>(props(ChartType::Pie, size)), size);
        let donut = last(Element::typed::<Chart>(props(ChartType::Donut, size)), size);
        assert!(count(&pie, '█') > count(&donut, '█'));
        let center = (size.1 / 2, size.0 / 2);
        assert_eq!(pie.screen.cell(center.0, center.1).unwrap().contents(), "█");
        assert_eq!(
            donut.screen.cell(center.0, center.1).unwrap().contents(),
            " "
        );
        let vertical = last(
            Element::typed::<Chart>(props(ChartType::BarVertical, size)),
            size,
        );
        let mut horizontal_props = props(ChartType::BarHorizontal, size);
        horizontal_props.x_axis = ChartAxis {
            min: Some(0.0),
            max: Some(10.0),
            ..axis()
        };
        horizontal_props.y_axis = axis();
        let horizontal = last(Element::typed::<Chart>(horizontal_props), size);
        assert!(count(&vertical, '█') > 0 && count(&horizontal, '█') > 0);
        assert_ne!(vertical.text, horizontal.text);
        for frame in [&line, &area, &scatter, &pie, &donut, &vertical, &horizontal] {
            assert!(!frame.text.contains("Chart"), "{}", frame.text);
            assert!(frame.text.lines().count() <= size.1 as usize);
        }
    }
}

#[test]
fn charts_use_exact_values_colors_and_point_overrides() {
    for size in [(20, 10), (36, 14)] {
        let mut config = props(ChartType::Scatter, size);
        config.series[0].data[1].color = Some("#00ff00".into());
        let frame = last(
            ChartsBuilder::scatter()
                .with_series(config.series.clone())
                .size(size.0, size.1)
                .x_axis(config.x_axis.clone())
                .y_axis(config.y_axis.clone())
                .no_legend()
                .render(),
            size,
        );
        let marks = cells_with(&frame, "•");
        assert_eq!(marks.len(), 2, "{}", frame.text);
        let low = *marks.iter().max_by_key(|(r, _)| *r).unwrap();
        let high = *marks.iter().min_by_key(|(r, _)| *r).unwrap();
        // The first point sits on the first column, the last on the last
        // column, and the lower value is drawn lower on the grid.
        assert_eq!(low.1, 0, "{}", frame.text);
        assert_eq!(high.1, size.0 - 1, "{}", frame.text);
        assert!(low.0 > high.0, "{}", frame.text);
        assert_eq!(
            frame.screen.cell(low.0, low.1).unwrap().fgcolor(),
            vt100::Color::Rgb(255, 0, 0)
        );
        assert_eq!(
            frame.screen.cell(high.0, high.1).unwrap().fgcolor(),
            vt100::Color::Rgb(0, 255, 0)
        );
    }
}

#[test]
fn chart_tooltips_follow_painted_points_and_keyboard_and_can_be_disabled() {
    // A 44 by 12 chart is the medium class, which draws the tooltip box;
    // the mini class keeps its shapes and speaks the value instead (CHT-024).
    for size in [(48, 14), (64, 16)] {
        let config = props(ChartType::Scatter, (44, 12));
        let frames = run(
            Control(Element::typed::<Chart>(config.clone()).auto_focus()),
            size,
            vec![
                (2, hover(43, 2)),
                (3, key(KeyCode::Home)),
                (4, key(KeyCode::End)),
                (5, key(KeyCode::Escape)),
                (6, None),
            ],
        );
        assert!(
            frames
                .iter()
                .any(|f| f.text.contains("High: 8;") && f.text.contains("unit=ms")),
            "{:?}",
            frames.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(frames.iter().any(|f| f.text.contains("Low: 2")));
        assert!(!frames.last().unwrap().text.contains("unit=ms"));
        let mut disabled = config;
        disabled.show_tooltips = false;
        let frames = run(
            Control(Element::typed::<Chart>(disabled).auto_focus()),
            size,
            vec![(2, hover(23, 2)), (2, key(KeyCode::End)), (2, None)],
        );
        assert!(frames.iter().all(|f| !f.text.contains("unit=ms")));
    }
}

#[test]
fn charts_display_axis_titles_grid_custom_ticks_and_all_legend_positions() {
    // Both sizes are the medium class, which draws axes and the legend.
    for size in [(40, 14), (48, 18)] {
        for position in [
            LegendPosition::Top,
            LegendPosition::Bottom,
            LegendPosition::Left,
            LegendPosition::Right,
            LegendPosition::Floating(1, 1),
        ] {
            let mut config = props(ChartType::Line, size);
            config.title = Some("Telemetry".into());
            config.x_axis = ChartAxis {
                title: Some("Elapsed".into()),
                custom_labels: vec!["start".into(), "end".into()],
                tick_count: 2,
                ..Default::default()
            };
            config.y_axis = ChartAxis {
                title: Some("Latency".into()),
                min: Some(0.0),
                max: Some(10.0),
                tick_count: 2,
                ..Default::default()
            };
            config.legend.visible = true;
            config.legend.position = position;
            let frame = last(Element::typed::<Chart>(config), size);
            for label in [
                "Telemetry",
                "Elapsed",
                "Latency",
                "Measurements",
                "start",
                "end",
            ] {
                assert!(
                    frame.text.contains(label),
                    "missing {label}: {}",
                    frame.text
                );
            }
            // The grid belongs to the large class; medium draws axes,
            // ticks and the legend only (CHT-024).
            assert_eq!(count(&frame, '·'), 0, "{}", frame.text);
        }
    }
}

#[test]
fn charts_reject_invalid_values_ranges_and_handle_empty_data() {
    for size in [(32, 10), (60, 16)] {
        let mut empty = props(ChartType::Line, size);
        empty.series.clear();
        assert!(last(Element::typed::<Chart>(empty), size)
            .text
            .contains("No data to display"));
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut config = props(ChartType::Line, size);
            config.series[0].data[0].value = invalid;
            assert!(last(Element::typed::<Chart>(config), size)
                .text
                .contains("values must be finite"));
        }
        let mut config = props(ChartType::Pie, size);
        config.series[0].data[0].value = -1.0;
        assert!(last(Element::typed::<Chart>(config), size)
            .text
            .contains("must be nonnegative"));
        let mut config = props(ChartType::Line, size);
        config.y_axis.min = Some(10.0);
        assert!(last(Element::typed::<Chart>(config), size)
            .text
            .contains("minimum must be below"));
        let mut config = props(ChartType::Line, size);
        config.series[0].color = Some("#界".into());
        assert!(last(Element::typed::<Chart>(config), size)
            .text
            .contains("Invalid series color"));
    }
}

#[test]
fn chart_builder_and_macro_draw_data_instead_of_descriptions() {
    for size in [(32, 12), (60, 16)] {
        for element in [
            builder::chart()
                .line_chart()
                .simple_series("Signal", vec![1.0, 3.0, 2.0])
                .size(24, 10)
                .legend(false, LegendPosition::Right)
                .build(),
            reactive_tui::chart![line: "Signal" => [1.0, 3.0, 2.0]],
        ] {
            let frame = last(element, size);
            assert!(count(&frame, '●') > 0, "{}", frame.text);
            assert!(!frame.text.contains("Chart:"));
        }
    }
}

#[test]
fn charts_honor_line_and_fill_styles_and_visible_series() {
    use reactive_tui::widgets::display::LineStyle;
    for size in [(24, 12), (40, 18)] {
        let mut none = props(ChartType::Line, size);
        none.series[0].line_style = LineStyle::None;
        let frame = last(Element::typed::<Chart>(none), size);
        assert_eq!(count(&frame, '●'), 2, "{}", frame.text);
        assert_eq!(braille(&frame), 0, "{}", frame.text);
        let mut solid = props(ChartType::Line, size);
        solid.series[0].data[0].value = 5.0;
        solid.series[0].data[1].value = 5.0;
        let full = last(Element::typed::<Chart>(solid.clone()), size);
        solid.series[0].line_style = LineStyle::Dashed;
        let dashed = last(Element::typed::<Chart>(solid.clone()), size);
        assert!(
            braille(&dashed) > 0 && braille(&dashed) < braille(&full),
            "dashed {} vs solid {}:\n{}",
            braille(&dashed),
            braille(&full),
            dashed.text
        );
        solid.series[0].line_style = LineStyle::Dotted;
        let dotted = last(Element::typed::<Chart>(solid), size);
        assert!(
            braille(&dotted) > 0 && braille(&dotted) < braille(&full),
            "{}",
            dotted.text
        );
        for (style, mark) in [
            (FillStyle::Solid, '█'),
            (FillStyle::Gradient, '█'),
            (FillStyle::Pattern("/".into()), '/'),
            (FillStyle::Pattern("diagonal".into()), '/'),
            (FillStyle::Pattern("dots".into()), '·'),
            (FillStyle::Pattern("ab".into()), 'b'),
        ] {
            let mut config = props(ChartType::Area, size);
            config.series[0].fill_style = style.clone();
            let frame = last(Element::typed::<Chart>(config), size);
            assert!(count(&frame, mark) > 0, "{style:?}: {}", frame.text);
            if style == FillStyle::Gradient {
                // The ramp shades toward the baseline: one column carries
                // more than one color.
                let (rows, cols) = frame.screen.size();
                let shades = (0..cols)
                    .map(|c| {
                        (0..rows)
                            .filter_map(|r| frame.screen.cell(r, c))
                            .filter(|cell| cell.contents() == "█")
                            .map(|cell| format!("{:?}", cell.fgcolor()))
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                    })
                    .max()
                    .unwrap_or(0);
                assert!(shades >= 2, "gradient must shade: {}", frame.text);
            }
        }
        let mut config = props(ChartType::Area, size);
        config.series[0].fill_style = FillStyle::None;
        assert_eq!(count(&last(Element::typed::<Chart>(config), size), '█'), 0);
        let mut config = props(ChartType::Scatter, size);
        let mut hidden = config.series[0].clone();
        hidden.name = "Hidden".into();
        hidden.visible = false;
        hidden.data[0].value = f64::NAN;
        config.series.push(hidden);
        config.legend.visible = true;
        let frame = last(Element::typed::<Chart>(config), size);
        assert_eq!(count(&frame, '•'), 2);
        assert!(!frame.text.contains("Hidden"));
        assert!(!frame.text.contains("finite"));
    }
}

#[test]
fn charts_clip_scatter_values_outside_explicit_axes_and_color_pie_sectors() {
    for size in [(24, 12), (40, 18)] {
        let mut config = props(ChartType::Scatter, size);
        config.series[0].data = vec![
            DataPoint::new(-10.0),
            DataPoint::new(5.0),
            DataPoint::new(20.0),
        ];
        let frame = last(Element::typed::<Chart>(config.clone()), size);
        assert_eq!(count(&frame, '•'), 1, "{}", frame.text);
        config.x_axis.min = Some(0.0);
        config.x_axis.max = Some(0.5);
        assert_eq!(count(&last(Element::typed::<Chart>(config), size), '•'), 0);
        let mut config = props(ChartType::Pie, size);
        config.series[0].color = None;
        config.series[0].data = vec![DataPoint::new(1.0), DataPoint::new(1.0)];
        config.color_palette = vec!["#ff0000".into(), "#00ff00".into()];
        let frame = last(Element::typed::<Chart>(config), size);
        // The first slice sweeps clockwise from twelve o'clock over the
        // right half, the second over the left half.
        assert_eq!(
            frame
                .screen
                .cell(size.1 / 2, size.0 * 5 / 8)
                .unwrap()
                .fgcolor(),
            vt100::Color::Rgb(255, 0, 0),
            "{}",
            frame.text
        );
        assert_eq!(
            frame
                .screen
                .cell(size.1 / 2, size.0 * 3 / 8)
                .unwrap()
                .fgcolor(),
            vt100::Color::Rgb(0, 255, 0),
            "{}",
            frame.text
        );
    }
}

#[test]
fn charts_update_props_and_resize_inside_a_padded_parent() {
    use reactive_tui::{app::RootComponent, event::types::ResizeEvent};
    struct Updating {
        config: std::sync::Mutex<ChartProps>,
    }
    impl RootComponent for Updating {
        fn render(&self) -> Element {
            builder::div()
                .class("w-full h-full p-0.5")
                .child(Element::typed::<Chart>(self.config.lock().unwrap().clone()).auto_focus())
                .build()
        }
        fn handle_event(&self, event: &Event) -> reactive_tui::event::router::EventResult {
            if matches!(event,Event::Key(k) if k.code==KeyCode::Char('u')) {
                self.config.lock().unwrap().series[0].data[1].value = 5.0;
                return reactive_tui::event::router::EventResult::Consumed;
            }
            reactive_tui::event::router::EventResult::Ignored
        }
        fn wake_driven(&self) -> bool {
            true
        }
    }
    let frames = run(
        Updating {
            config: std::sync::Mutex::new(props(ChartType::Scatter, (80, 20))),
        },
        // Both terminal sizes keep the padded chart in the medium class,
        // which draws the tooltip box (CHT-024). A scatter hover selects the
        // nearest point, so any cell in the right half picks the high point.
        (48, 14),
        vec![
            (2, hover(45, 3)),
            (3, key(KeyCode::Char('u'))),
            (4, Some(Event::Resize(ResizeEvent::new(44, 12)))),
            (6, hover(41, 5)),
            (7, None),
        ],
    );
    assert!(
        frames.iter().any(|f| f.text.contains("High: 8;")),
        "{:?}",
        frames.iter().map(|f| &f.text).collect::<Vec<_>>()
    );
    assert!(
        frames.last().unwrap().text.contains("High:") && frames.last().unwrap().text.contains("5;"),
        "{}",
        frames.last().unwrap().text
    );
    assert!(
        frames.last().unwrap().text.contains("unit=ms"),
        "{}",
        frames.last().unwrap().text
    );
    assert!(frames
        .last()
        .unwrap()
        .screen
        .cell(0, 0)
        .unwrap()
        .contents()
        .trim()
        .is_empty());
}

#[test]
fn chart_line_segments_intersect_axes_without_inventing_edge_points() {
    for size in [(25, 13), (49, 25)] {
        let mut config = props(ChartType::Line, size);
        config.series[0].data = vec![DataPoint::new(-10.0), DataPoint::new(20.0)];
        config.curve = reactive_tui::widgets::display::Curve::Linear;
        let frame = last(Element::typed::<Chart>(config), size);
        assert_eq!(count(&frame, '●'), 0, "{}", frame.text);
        assert!(braille(&frame) > 0, "{}", frame.text);
        // The segment enters the axis range a third of the way across and
        // leaves it two thirds across; outside those the cells stay empty.
        let near = |row: u16, col: u16| {
            (col.saturating_sub(1)..=col + 1).any(|c| {
                frame
                    .screen
                    .cell(row, c)
                    .is_some_and(|cell| cell.contents().chars().any(is_braille))
            })
        };
        assert!(near(size.1 - 1, (size.0 - 1) / 3), "{}", frame.text);
        assert!(near(0, (size.0 - 1) * 2 / 3), "{}", frame.text);
        assert!(frame
            .screen
            .cell(size.1 - 1, 0)
            .unwrap()
            .contents()
            .trim()
            .is_empty());
        assert!(frame
            .screen
            .cell(0, size.0 - 1)
            .unwrap()
            .contents()
            .trim()
            .is_empty());
    }
}

#[test]
fn chart_animation_paints_intermediate_frames_and_reduced_motion_is_immediate() {
    for size in [(24, 12), (40, 18)] {
        let mut config = props(ChartType::BarVertical, size);
        let complete = last(Element::typed::<Chart>(config.clone()), size);
        config.animated = true;
        config.animation_duration = 800;
        let frames = run(
            Control(Element::typed::<Chart>(config.clone())),
            size,
            vec![(8, None)],
        );
        assert!(
            frames
                .iter()
                .any(|f| count(f, '█') > 0 && count(f, '█') < count(&complete, '█')),
            "{:?}",
            frames.iter().map(|f| count(f, '█')).collect::<Vec<_>>()
        );
        config.class = Some("reduced-motion".into());
        let reduced = last(Element::typed::<Chart>(config), size);
        assert_eq!(count(&reduced, '█'), count(&complete, '█'));
    }
}

#[test]
fn chart_legend_placement_and_width_follow_the_configuration() {
    for size in [(24, 12), (40, 18)] {
        for (position, x, y) in [
            (LegendPosition::Top, 0, 0),
            (LegendPosition::Bottom, 0, size.1 - 1),
            (LegendPosition::Left, 0, 0),
            (LegendPosition::Right, size.0 - 3, 0),
            (LegendPosition::Floating(2, 3), 2, 3),
        ] {
            let mut config = props(ChartType::Scatter, size);
            config.series[0].name = "S".into();
            config.legend.visible = true;
            config.legend.position = position;
            // The mini class hides the legend; force the medium layout.
            config.size_class = Some(SizeClass::Medium);
            let frame = last(Element::typed::<Chart>(config), size);
            assert_eq!(
                frame.screen.cell(y, x).unwrap().contents(),
                "■",
                "{}",
                frame.text
            );
            assert_eq!(frame.screen.cell(y, x + 2).unwrap().contents(), "S");
        }
        let mut config = props(ChartType::Scatter, size);
        config.legend.visible = true;
        config.legend.max_width = Some(2);
        config.size_class = Some(SizeClass::Medium);
        let frame = last(Element::typed::<Chart>(config), size);
        assert!(!frame.text.contains("Measurements"));
        assert!(frame.text.contains('■'));
        // An unset width or height fills the allotted rectangle instead of
        // being an error.
        for (width, height) in [(0, 5), (10, 0), (0, 0)] {
            let mut config = props(ChartType::Line, size);
            config.width = width;
            config.height = height;
            let frame = last(Element::typed::<Chart>(config), size);
            assert!(!frame.text.contains("Chart width"), "{}", frame.text);
            assert!(count(&frame, '●') > 0, "{}", frame.text);
        }
    }
}

#[test]
fn chart_convenience_builders_and_disabled_controls_keep_their_behavior() {
    for size in [(32, 12), (60, 18)] {
        let data = DataSeries::new("Series", vec![DataPoint::new(2.0), DataPoint::new(8.0)]);
        for (builder, mark) in [
            (ChartsBuilder::bar(), '█'),
            (ChartsBuilder::bar_horizontal(), '█'),
            (ChartsBuilder::line(), '●'),
            (ChartsBuilder::area(), '█'),
            (ChartsBuilder::pie(), '█'),
            (ChartsBuilder::donut(), '█'),
            (ChartsBuilder::scatter(), '•'),
        ] {
            let frame = last(
                builder
                    .series(data.clone())
                    .title("Graph")
                    .width(24)
                    .height(10)
                    .no_legend()
                    .color_palette(vec!["#00ff00".into()])
                    .class("text-blue-500")
                    .render(),
                size,
            );
            assert!(frame.text.contains("Graph"));
            assert!(count(&frame, mark) > 0, "{}", frame.text);
        }
        for builder in [
            builder::chart().bar_chart(),
            builder::chart().horizontal_bar_chart(),
            builder::chart().area_chart(),
            builder::chart().pie_chart(),
            builder::chart().scatter_plot(),
            builder::chart().chart_type(ChartType::Donut),
        ] {
            let frame = last(
                builder
                    .series_list(vec![data.clone()])
                    .size(24, 10)
                    .colors(vec!["#00ff00".into()])
                    .axes(Some("X".into()), Some("Y".into()))
                    .title("Named")
                    .legend(false, LegendPosition::Bottom)
                    .class("text-blue-500")
                    .build_with_name("Charts"),
                size,
            );
            assert!(frame.text.contains("Named"));
            assert!(
                count(&frame, '█') + count(&frame, '•') > 0,
                "{}",
                frame.text
            );
        }
        let element = Element::typed::<Chart>(props(ChartType::Scatter, (24, 10)))
            .disabled(true)
            .auto_focus();
        let frames = run(
            Control(element),
            size,
            vec![(2, hover(23, 2)), (2, key(KeyCode::End)), (2, None)],
        );
        assert!(frames.iter().all(|f| !f.text.contains("unit=ms")));
    }
}

#[test]
fn chart_bars_keep_signed_values_and_do_not_paint_values_outside_the_scale() {
    for size in [(24, 12), (40, 18)] {
        let mut config = props(ChartType::BarVertical, size);
        config.series[0].data = vec![DataPoint::new(-5.0), DataPoint::new(5.0)];
        config.y_axis.min = Some(-10.0);
        let frame = last(Element::typed::<Chart>(config.clone()), size);
        let blocks = cells_with(&frame, "█");
        assert!(!blocks.is_empty(), "{}", frame.text);
        let left_col = blocks.iter().map(|(_, c)| *c).min().unwrap();
        let right_col = blocks.iter().map(|(_, c)| *c).max().unwrap();
        let negative: Vec<u16> = blocks
            .iter()
            .filter(|(_, c)| *c == left_col)
            .map(|(r, _)| *r)
            .collect();
        let positive: Vec<u16> = blocks
            .iter()
            .filter(|(_, c)| *c == right_col)
            .map(|(r, _)| *r)
            .collect();
        // The negative bar hangs below the baseline, the positive one rises
        // above it, and neither reaches the other's side.
        assert!(
            negative.iter().min() >= positive.iter().max(),
            "negative rows {negative:?} must lie below positive rows {positive:?}:\n{}",
            frame.text
        );
        config.y_axis.min = Some(6.0);
        config.y_axis.max = Some(10.0);
        assert_eq!(count(&last(Element::typed::<Chart>(config), size), '█'), 0);
        let mut config = props(ChartType::BarHorizontal, size);
        config.series[0].data = vec![DataPoint::new(0.0), DataPoint::new(0.0)];
        assert_eq!(count(&last(Element::typed::<Chart>(config), size), '█'), 0);
    }
}

#[test]
fn chart_own_padding_positions_plot_and_tooltips_in_the_content_box() {
    // A 44 by 12 chart is the medium class, which draws the tooltip box.
    for size in [(48, 14), (60, 18)] {
        let mut config = props(ChartType::Scatter, (44, 12));
        config.class = Some("p-0.5".into());
        let plain = last(Element::typed::<Chart>(config.clone()), size);
        let marks = cells_with(&plain, "•");
        assert_eq!(marks.len(), 2, "{}", plain.text);
        // Padding keeps the plot off the first cell.
        assert!(plain
            .screen
            .cell(0, 0)
            .unwrap()
            .contents()
            .trim()
            .is_empty());
        // Hovering the higher point shows its tooltip inside the content box.
        let (row, col) = *marks.iter().min_by_key(|(r, _)| *r).unwrap();
        let frames = run(
            Control(Element::typed::<Chart>(config)),
            size,
            vec![(2, hover(col, row)), (3, None)],
        );
        let frame = frames.last().unwrap();
        assert!(
            frame.text.contains("High:")
                && frame.text.contains("8;")
                && frame.text.contains("unit=ms"),
            "{}",
            frame.text
        );
    }
}
