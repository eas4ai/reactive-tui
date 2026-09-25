use super::*;
use crate::{
    builder::ElementBuilder,
    component::{ElementType, LayoutInfo, LayoutType, LifecycleEvent},
    layout::style::StyleBuilder,
};
mod motion;
type Color = (f32, f32, f32, f32);

#[derive(Clone, PartialEq)]
pub(super) struct LiveProps {
    pub config: ProgressBarProps,
    pub seed: ProgressBarState,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub(super) struct LiveProgress {
    viewport: Option<LayoutInfo>,
    motion: motion::Motion,
}
impl Component for LiveProgress {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        Self {
            viewport: None,
            motion: motion::Motion::new(&props.seed),
        }
    }
    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut Self::State) -> bool {
        let changed = self
            .viewport
            .is_none_or(|old| old.content_size() != layout.content_size());
        self.viewport = Some(layout);
        changed
    }
    fn render(&self, props: &Self::Props, _: &Self::State) -> Element {
        let config = &props.config;
        let error = validate(config).err();
        let text = if error.is_none() {
            ProgressBar.format_text(config)
        } else {
            String::new()
        };
        let (width, height) = self.viewport.map_or((0, 0), |layout| {
            let (w, h) = layout.content_size();
            (w.max(0.0) as usize, h.max(0.0) as usize)
        });
        let sample = self
            .motion
            .sample(config, error.is_none() && width > 0 && height > 0);
        let insets = self.viewport.map_or([0.0; 4], |v| v.insets);
        let mut children = Vec::new();
        let mut row = 0;
        if let Some(error) = error {
            children.push(text_at(
                error.to_string(),
                0,
                0,
                width,
                "text-red-500",
                None,
                1.0,
            ));
        } else {
            if let Some(label) = &config.label {
                children.push(text_at(
                    label.clone(),
                    0,
                    row,
                    width,
                    config.text_style.as_deref().unwrap_or(""),
                    None,
                    1.0,
                ));
                row += 1;
            }
            let rows = (config.height as usize)
                .min(height.saturating_sub(row + usize::from(!text.is_empty())));
            let length = if config.orientation == ProgressBarOrientation::Vertical {
                rows
            } else {
                width
            };
            let foreground = config.color.as_deref().and_then(parse_color);
            let background = config.background_color.as_deref().and_then(parse_color);
            for y in 0..rows {
                let mut start = 0;
                while start < width {
                    let axis = if config.orientation == ProgressBarOrientation::Vertical {
                        rows - 1 - y
                    } else {
                        start
                    };
                    let (_, filled) = bar_char(config, sample.fraction, axis, length, sample.phase);
                    let mut end = start;
                    let mut line = String::new();
                    while end < width {
                        let axis = if config.orientation == ProgressBarOrientation::Vertical {
                            rows - 1 - y
                        } else {
                            end
                        };
                        let (mut mark, next_filled) =
                            bar_char(config, sample.fraction, axis, length, sample.phase);
                        if next_filled != filled {
                            break;
                        }
                        if next_filled
                            && config.striped
                            && (axis + y + (sample.phase * 12.0) as usize) % 4 >= 2
                        {
                            mark = '▌';
                        }
                        line.push(mark);
                        end += 1;
                    }
                    let color = if filled { foreground } else { background };
                    children.push(text_at(
                        line,
                        start,
                        row + y,
                        end - start,
                        config.bar_style.as_deref().unwrap_or(""),
                        color,
                        if filled { sample.alpha } else { 1.0 },
                    ));
                    start = end;
                }
            }
            row += rows;
            if !text.is_empty() && row < height {
                children.push(text_at(
                    text.clone(),
                    0,
                    row,
                    width,
                    config.text_style.as_deref().unwrap_or(""),
                    None,
                    1.0,
                ));
            }
        }
        let content = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(insets[0])
                    .inset_top(insets[1])
                    .width_px(width as f32)
                    .height_px(height as f32)
                    .overflow_hidden(),
            )
            .children(children)
            .build();
        let natural_height = if error.is_some() {
            1
        } else {
            config.height as usize
                + usize::from(config.label.is_some())
                + usize::from(!text.is_empty())
        };
        let mut style = StyleBuilder::new()
            .height_px(natural_height as f32 + insets[1] + insets[3])
            .max_height_percent(100.0)
            .max_width_percent(100.0)
            .overflow_hidden();
        style = if error.is_some() {
            style.width_px(40.0)
        } else if let Some(width) = config.width {
            style.width_px(width as f32)
        } else {
            style.width_percent(100.0)
        };
        let intrinsic = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .width_px(config.width.unwrap_or(24) as f32)
                    .height_px(0.0),
            )
            .build();
        let mut node =
            crate::accessibility::Node::new(crate::accessibility::Role::ProgressIndicator);
        node.set_label(config.label.as_deref().unwrap_or("Progress"));
        node.set_value(error.map_or_else(|| text.clone(), str::to_owned));
        if error.is_none() && !config.indeterminate {
            node.inner
                .set_numeric_value(config.value.clamp(config.min_value, config.max_value));
            node.inner.set_min_numeric_value(config.min_value);
            node.inner.set_max_numeric_value(config.max_value);
        }
        ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(style)
            .class(config.style.as_deref().unwrap_or(""))
            .children(vec![content, intrinsic])
            .build()
            .with_accessibility(node)
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut Self::State) {
        if event == LifecycleEvent::Unmount {
            self.motion.cancel();
        }
    }
}

fn text_at(
    text: String,
    x: usize,
    y: usize,
    width: usize,
    class: &str,
    color: Option<Color>,
    opacity: f32,
) -> Element {
    let mut style = StyleBuilder::new()
        .position_absolute()
        .inset_left(x as f32)
        .inset_top(y as f32)
        .width_px(width as f32)
        .height_px(1.0)
        .opacity(opacity)
        .overflow_hidden();
    if let Some((r, g, b, a)) = color {
        style = style.fg_rgba(r, g, b, a);
    }
    let text = text
        .chars()
        .map(|c| if c.is_control() { '�' } else { c })
        .collect::<String>();
    ElementBuilder::new(ElementType::Text(text))
        .styles(style)
        .class(&format!("whitespace-pre truncate {class}"))
        .build()
}

fn parse_color(value: &str) -> Option<Color> {
    if !value.is_ascii() {
        return None;
    }
    let value = value
        .strip_prefix("bg-")
        .or_else(|| value.strip_prefix("text-"))
        .unwrap_or(value);
    crate::layout::colors::parse_color_token(value)
        .or_else(|| {
            let token = if value.starts_with('[') {
                format!("bg-{value}")
            } else {
                format!("bg-[{value}]")
            };
            crate::layout::css::colors::apply_bg_color(&token, StyleBuilder::new())
                .and_then(|s| s.bg_rgba)
        })
        .filter(|(r, g, b, a)| {
            [r, g, b, a]
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(*v))
        })
}

pub(super) fn validate(props: &ProgressBarProps) -> Result<(), &'static str> {
    if !valid_range(props) {
        return Err("Invalid value or range");
    }
    if props.height == 0 || props.width == Some(0) {
        return Err("Invalid bar dimensions");
    }
    if props.segments == Some(0) {
        return Err("Invalid segment count");
    }
    if [&props.color, &props.background_color]
        .into_iter()
        .flatten()
        .any(|s| parse_color(s).is_none())
    {
        return Err("Invalid progress color");
    }
    Ok(())
}

pub(super) fn bar_char(
    props: &ProgressBarProps,
    fraction: f64,
    index: usize,
    length: usize,
    phase: f64,
) -> (char, bool) {
    if length == 0 {
        return (' ', false);
    }
    let filled = if props.indeterminate {
        let span = (length / 4).max(1);
        let start = (phase.fract() * (length + span) as f64) as isize - span as isize;
        (index as isize) >= start && (index as isize) < start + span as isize
    } else if let Some(segments) = props.segments {
        let segments = (segments as usize).min(length).max(1);
        let segment = index * segments / length;
        if index + 1 < length
            && (index + 1) * segments / length != segment
            && length / segments >= 2
        {
            return (' ', false);
        }
        segment < (fraction * segments as f64).floor() as usize
    } else {
        index < (fraction * length as f64).floor() as usize
    };
    (
        if filled {
            '█'
        } else if props.striped {
            '░'
        } else {
            '▒'
        },
        filled,
    )
}
