//! VDOM inline declarations share the native layout and paint representation.

use super::*;
use crate::error::{ReactiveError, Result};

fn invalid(declaration: &str) -> ReactiveError {
    ReactiveError::layout(format!(
        "Invalid terminal inline style declaration: {declaration:?}"
    ))
}

fn number(value: &str) -> Option<f32> {
    value.parse::<f32>().ok().filter(|n| n.is_finite())
}

fn cells(value: &str) -> Option<f32> {
    number(
        value
            .strip_suffix("px")
            .or_else(|| value.strip_suffix("ch"))
            .unwrap_or(value),
    )
}

fn dimension(value: &str) -> Option<Dimension> {
    if value == "auto" {
        Some(Dimension::auto())
    } else if let Some(percent) = value.strip_suffix('%') {
        number(percent)
            .filter(|n| *n >= 0.0)
            .map(|n| Dimension::percent(n / 100.0))
    } else {
        cells(value).filter(|n| *n >= 0.0).map(Dimension::length)
    }
}

impl StyleBuilder {
    pub(crate) fn with_inline_declarations(mut self, declarations: &str) -> Result<Self> {
        for declaration in declarations
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let (name, value) = declaration
                .split_once(':')
                .ok_or_else(|| invalid(declaration))?;
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim().to_ascii_lowercase();
            self = self
                .inline_property(&name, &value)
                .ok_or_else(|| invalid(declaration))?;
        }
        Ok(self)
    }

    fn inline_property(self, name: &str, value: &str) -> Option<Self> {
        match name {
            "color" | "background-color" | "background" | "opacity" | "z-index" | "font-weight"
            | "font-style" | "text-decoration" => self.inline_paint(name, value),
            "padding" | "margin" | "padding-top" | "padding-right" | "padding-bottom"
            | "padding-left" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left"
            | "top" | "right" | "bottom" | "left" | "gap" | "row-gap" | "column-gap" => {
                self.inline_spacing(name, value)
            }
            _ => self.inline_layout(name, value),
        }
    }

    fn inline_layout(mut self, name: &str, value: &str) -> Option<Self> {
        match name {
            "width" => self.style.size.width = dimension(value)?,
            "height" => self.style.size.height = dimension(value)?,
            "min-width" => self.style.min_size.width = dimension(value)?,
            "min-height" => self.style.min_size.height = dimension(value)?,
            "max-width" => self.style.max_size.width = dimension(value)?,
            "max-height" => self.style.max_size.height = dimension(value)?,
            "display" => {
                self.style.display = match value {
                    "flex" => Display::Flex,
                    "grid" => Display::Grid,
                    "block" => Display::Block,
                    "none" => Display::None,
                    _ => return None,
                }
            }
            "position" => {
                self.style.position = match value {
                    "relative" => taffy::style::Position::Relative,
                    "absolute" => taffy::style::Position::Absolute,
                    _ => return None,
                }
            }
            "flex-direction" => {
                self.style.flex_direction = match value {
                    "row" => FlexDirection::Row,
                    "column" => FlexDirection::Column,
                    "row-reverse" => FlexDirection::RowReverse,
                    "column-reverse" => FlexDirection::ColumnReverse,
                    _ => return None,
                }
            }
            "flex-wrap" => {
                self.style.flex_wrap = match value {
                    "nowrap" => taffy::style::FlexWrap::NoWrap,
                    "wrap" => taffy::style::FlexWrap::Wrap,
                    "wrap-reverse" => taffy::style::FlexWrap::WrapReverse,
                    _ => return None,
                }
            }
            "align-items" => {
                self.style.align_items = Some(match value {
                    "start" | "flex-start" => TAlign::Start,
                    "end" | "flex-end" => TAlign::End,
                    "center" => TAlign::Center,
                    "stretch" => TAlign::Stretch,
                    "baseline" => TAlign::Baseline,
                    _ => return None,
                })
            }
            "justify-content" => {
                self.style.justify_content = Some(match value {
                    "start" | "flex-start" => TJustify::Start,
                    "end" | "flex-end" => TJustify::End,
                    "center" => TJustify::Center,
                    "space-between" => TJustify::SpaceBetween,
                    "space-around" => TJustify::SpaceAround,
                    "space-evenly" => TJustify::SpaceEvenly,
                    _ => return None,
                })
            }
            "flex-grow" => self.style.flex_grow = number(value).filter(|n| *n >= 0.0)?,
            "flex-shrink" => self.style.flex_shrink = number(value).filter(|n| *n >= 0.0)?,
            "flex-basis" => self.style.flex_basis = dimension(value)?,
            "overflow" | "overflow-x" | "overflow-y" => {
                let overflow = match value {
                    "visible" => Overflow::Visible,
                    "hidden" => Overflow::Hidden,
                    "scroll" | "auto" => Overflow::Scroll,
                    _ => return None,
                };
                if name != "overflow-y" {
                    self.style.overflow.x = overflow;
                }
                if name != "overflow-x" {
                    self.style.overflow.y = overflow;
                }
            }
            _ => return None,
        }
        Some(self)
    }

    fn inline_paint(mut self, name: &str, value: &str) -> Option<Self> {
        match name {
            "color" | "background-color" | "background" => {
                let color = crate::layout::colors::parse_color_token(value)?;
                if name == "color" {
                    self.fg_rgba = Some(color);
                } else {
                    self.bg_rgba = Some(color);
                }
            }
            "opacity" => self.opacity = Some(number(value).filter(|n| (0.0..=1.0).contains(n))?),
            "z-index" => self.z_index = Some(value.parse().ok()?),
            "font-weight" => {
                self = self.bold(match value {
                    "bold" => true,
                    "normal" => false,
                    _ => number(value).filter(|n| (1.0..=1000.0).contains(n))? >= 600.0,
                })
            }
            "font-style" => {
                self = self.italic(match value {
                    "italic" => true,
                    "normal" => false,
                    _ => return None,
                })
            }
            "text-decoration" => {
                let mut underline = false;
                let mut strike = false;
                for token in value.split_whitespace() {
                    match token {
                        "none" => {}
                        "underline" => underline = true,
                        "line-through" => strike = true,
                        _ => return None,
                    }
                }
                if value.is_empty() {
                    return None;
                }
                self = self.underline(underline).strike(strike);
            }
            _ => return None,
        }
        Some(self)
    }

    fn inline_spacing(mut self, name: &str, value: &str) -> Option<Self> {
        match name {
            "padding" | "margin" => {
                if name == "margin" && value == "auto" {
                    return Some(self.margin_auto());
                }
                let lengths = value
                    .split_whitespace()
                    .map(cells)
                    .collect::<Option<Vec<_>>>()?;
                let (top, right, bottom, left) = match lengths.as_slice() {
                    [a] => (*a, *a, *a, *a),
                    [a, b] => (*a, *b, *a, *b),
                    [a, b, c] => (*a, *b, *c, *b),
                    [a, b, c, d] => (*a, *b, *c, *d),
                    _ => return None,
                };
                if name == "padding" {
                    if lengths.iter().any(|n| *n < 0.0) {
                        return None;
                    }
                    self = self
                        .padding_t_px(top)
                        .padding_r_px(right)
                        .padding_b_px(bottom)
                        .padding_l_px(left);
                } else {
                    self = self
                        .margin_t_px(top)
                        .margin_r_px(right)
                        .margin_b_px(bottom)
                        .margin_l_px(left);
                }
            }
            "padding-top" | "padding-right" | "padding-bottom" | "padding-left" | "margin-top"
            | "margin-right" | "margin-bottom" | "margin-left" | "top" | "right" | "bottom"
            | "left" => {
                let length = cells(value)?;
                if name.starts_with("padding-") && length < 0.0 {
                    return None;
                }
                self = match name {
                    "padding-top" => self.padding_t_px(length),
                    "padding-right" => self.padding_r_px(length),
                    "padding-bottom" => self.padding_b_px(length),
                    "padding-left" => self.padding_l_px(length),
                    "margin-top" => self.margin_t_px(length),
                    "margin-right" => self.margin_r_px(length),
                    "margin-bottom" => self.margin_b_px(length),
                    "margin-left" => self.margin_l_px(length),
                    "top" => self.inset_top(length),
                    "right" => self.inset_right(length),
                    "bottom" => self.inset_bottom(length),
                    _ => self.inset_left(length),
                };
            }
            "gap" | "row-gap" | "column-gap" => {
                let lengths = value
                    .split_whitespace()
                    .map(cells)
                    .collect::<Option<Vec<_>>>()?;
                if lengths.iter().any(|n| *n < 0.0) {
                    return None;
                }
                let (row, column) = match lengths.as_slice() {
                    [a] => (*a, *a),
                    [a, b] if name == "gap" => (*a, *b),
                    _ => return None,
                };
                if name != "column-gap" {
                    self.style.gap.height = LengthPercentage::length(row);
                }
                if name != "row-gap" {
                    self.style.gap.width = LengthPercentage::length(column);
                }
            }
            _ => return None,
        }
        Some(self)
    }
}
