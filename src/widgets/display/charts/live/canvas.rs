use super::*;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

mod cartesian;
mod pie;
use cartesian::cartesian;
use pie::pie;

type Color = (f32, f32, f32, f32);
type Point = (usize, usize);
#[derive(Clone, Default)]
struct Cell {
    text: String,
    color: Option<Color>,
    point: Option<Point>,
}
pub(super) struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}
#[derive(Clone, Copy)]
struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Canvas {
    pub(super) fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); width.saturating_mul(height)],
        }
    }
    fn put(&mut self, x: usize, y: usize, text: &str, color: Option<Color>, point: Option<Point>) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = Cell {
                text: text.into(),
                color,
                point,
            };
        }
    }
    fn text(&mut self, x: usize, y: usize, width: usize, text: &str, color: Option<Color>) {
        let mut offset = 0;
        for grapheme in text.graphemes(true) {
            let safe = if grapheme.chars().any(char::is_control) {
                "�"
            } else {
                grapheme
            };
            let cells = UnicodeWidthStr::width(safe);
            if cells == 0 {
                continue;
            }
            if offset + cells > width || x + offset + cells > self.width {
                break;
            }
            self.put(x + offset, y, safe, color, None);
            for continuation in 1..cells {
                self.put(x + offset + continuation, y, "", color, None);
            }
            offset += cells;
        }
    }
    pub(super) fn point_at(&self, x: usize, y: usize) -> Option<Point> {
        (x < self.width && y < self.height)
            .then(|| self.cells[y * self.width + x].point)
            .flatten()
    }
    fn wrapped(&mut self, text: &str, y: usize) {
        let lines = wrapped_lines(text, self.width);
        for (row, line) in lines.iter().take(self.height.saturating_sub(y)).enumerate() {
            self.text(0, y + row, self.width, line, None);
        }
    }
    pub(super) fn tooltip(&mut self, text: &str, position: Option<(usize, usize)>) {
        if self.height == 0 || self.width == 0 {
            return;
        }
        let lines = wrapped_lines(text, self.width);
        let height = lines.len().min(self.height);
        let y = position.map_or(self.height - height, |(_, y)| {
            (y + 1).min(self.height - height)
        });
        for row in y..y + height {
            for x in 0..self.width {
                self.put(x, row, " ", None, None);
            }
        }
        for (row, line) in lines.iter().take(height).enumerate() {
            self.text(0, y + row, self.width, line, None);
        }
    }
    pub(super) fn elements(&self) -> Vec<Element> {
        let mut elements = Vec::new();
        for y in 0..self.height {
            let mut x = 0;
            while x < self.width {
                let start = x;
                let color = self.cells[y * self.width + x].color;
                let mut text = String::new();
                while x < self.width && self.cells[y * self.width + x].color == color {
                    let cell = &self.cells[y * self.width + x];
                    if cell.text.is_empty() {
                        // A wide glyph already occupies its continuation cell.
                        if x == 0
                            || UnicodeWidthStr::width(
                                self.cells[y * self.width + x - 1].text.as_str(),
                            ) < 2
                        {
                            text.push(' ');
                        }
                    } else {
                        text.push_str(&cell.text);
                    }
                    x += 1;
                }
                let mut style = StyleBuilder::new()
                    .position_absolute()
                    .inset_left(start as f32)
                    .inset_top(y as f32)
                    .width_px((x - start) as f32)
                    .height_px(1.0)
                    .overflow_hidden();
                if let Some((r, g, b, a)) = color {
                    style = style.fg_rgba(r, g, b, a);
                }
                elements.push(
                    ElementBuilder::new(ElementType::Text(text))
                        .styles(style)
                        .class("whitespace-pre")
                        .build(),
                );
            }
        }
        elements
    }
}

fn wrapped_lines(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty()
            && UnicodeWidthStr::width(line.as_str()) + 1 + UnicodeWidthStr::width(word) > width
        {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        for grapheme in word.graphemes(true) {
            if UnicodeWidthStr::width(line.as_str()) + UnicodeWidthStr::width(grapheme) > width
                && !line.is_empty()
            {
                lines.push(std::mem::take(&mut line));
            }
            line.push_str(grapheme);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn color(value: &str) -> Option<Color> {
    value
        .is_ascii()
        .then(|| crate::layout::colors::parse_color_token(value))
        .flatten()
}

pub(super) fn validate(props: &ChartProps) -> Result<(), &'static str> {
    if props.width == 0 || props.height == 0 {
        return Err("Chart width and height must be positive");
    }
    for axis in [&props.x_axis, &props.y_axis] {
        if axis.min.is_some_and(|v| !v.is_finite()) || axis.max.is_some_and(|v| !v.is_finite()) {
            return Err("Axis limits must be finite");
        }
        if axis
            .min
            .zip(axis.max)
            .is_some_and(|(min, max)| min >= max || !(max - min).is_finite())
        {
            return Err("Axis minimum must be below maximum with a finite span");
        }
    }
    for series in &props.series {
        if !series.visible {
            continue;
        }
        if series.color.as_deref().is_some_and(|s| color(s).is_none()) {
            return Err("Invalid series color");
        }
        for point in &series.data {
            if !point.value.is_finite() {
                return Err("Chart values must be finite");
            }
            if matches!(props.chart_type, ChartType::Pie | ChartType::Donut) && point.value < 0.0 {
                return Err("Pie values must be nonnegative");
            }
            if point.color.as_deref().is_some_and(|s| color(s).is_none()) {
                return Err("Invalid point color");
            }
        }
    }
    if props.color_palette.iter().any(|s| color(s).is_none()) {
        return Err("Invalid palette color");
    }
    Ok(())
}

fn point_color(props: &ChartProps, series: usize, point: usize) -> Option<Color> {
    let data = &props.series[series];
    data.data[point]
        .color
        .as_deref()
        .or(data.color.as_deref())
        .or_else(|| {
            props
                .color_palette
                .get(series % props.color_palette.len().max(1))
                .map(String::as_str)
        })
        .and_then(color)
}

pub(super) fn draw(props: &ChartProps, width: usize, height: usize, progress: f64) -> Canvas {
    // Bound allocation independently of a malicious or unconstrained parent layout.
    let (width, height) = (
        width.min(if props.width == 0 || props.height == 0 {
            24
        } else {
            props.width as usize
        }),
        height.min(if props.width == 0 || props.height == 0 {
            2
        } else {
            props.height as usize
        }),
    );
    let mut canvas = Canvas::new(width, height.min(1_000_000 / width.max(1)));
    let height = canvas.height;
    if let Err(error) = validate(props) {
        canvas.wrapped(error, 0);
        return canvas;
    }
    let mut area = Rect {
        x: 0,
        y: 0,
        w: width,
        h: height,
    };
    if let Some(title) = &props.title {
        canvas.text(0, 0, width, title, None);
        area.y += 1;
        area.h = area.h.saturating_sub(1);
    }
    let visible: Vec<_> = props
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| s.visible)
        .collect();
    if visible.iter().all(|(_, s)| s.data.is_empty()) {
        canvas.text(area.x, area.y, area.w, "No data to display", None);
        return canvas;
    }
    let legend = legend_area(props, &visible, &mut area);
    if area.w > 0 && area.h > 0 {
        if matches!(props.chart_type, ChartType::Pie | ChartType::Donut) {
            pie(&mut canvas, props, area, progress);
        } else if let Err(error) = cartesian(&mut canvas, props, area, progress) {
            canvas.text(area.x, area.y, area.w, error, None);
        }
    }
    if let Some(rect) = legend {
        for (row, (index, series)) in visible.iter().take(rect.h).enumerate() {
            let tint = series
                .color
                .as_deref()
                .or_else(|| {
                    props
                        .color_palette
                        .get(index % props.color_palette.len().max(1))
                        .map(String::as_str)
                })
                .and_then(color);
            canvas.text(
                rect.x,
                rect.y + row,
                rect.w,
                &format!("■ {}", series.name),
                tint,
            );
        }
    }
    canvas
}

fn legend_area(
    props: &ChartProps,
    visible: &[(usize, &DataSeries)],
    area: &mut Rect,
) -> Option<Rect> {
    if !props.legend.visible || visible.is_empty() {
        return None;
    }
    let width = visible
        .iter()
        .map(|(_, s)| UnicodeWidthStr::width(s.name.as_str()) + 2)
        .max()
        .unwrap_or(0)
        .min(props.legend.max_width.unwrap_or(u16::MAX) as usize)
        .min(area.w);
    let height = visible.len().min(area.h);
    let rect = match props.legend.position {
        LegendPosition::Top => {
            let r = Rect {
                w: width,
                h: height,
                ..*area
            };
            area.y += height;
            area.h -= height;
            r
        }
        LegendPosition::Bottom => {
            let r = Rect {
                y: area.y + area.h - height,
                w: width,
                h: height,
                ..*area
            };
            area.h -= height;
            r
        }
        LegendPosition::Left => {
            let w = width.min(area.w / 2);
            let r = Rect {
                w,
                h: height,
                ..*area
            };
            area.x += w;
            area.w -= w;
            r
        }
        LegendPosition::Right => {
            let w = width.min(area.w / 2);
            let r = Rect {
                x: area.x + area.w - w,
                w,
                h: height,
                ..*area
            };
            area.w -= w;
            r
        }
        LegendPosition::Floating(x, y) => Rect {
            x: area.x + x as usize,
            y: area.y + y as usize,
            w: width.min(area.w.saturating_sub(x as usize)),
            h: height.min(area.h.saturating_sub(y as usize)),
        },
    };
    Some(rect)
}
