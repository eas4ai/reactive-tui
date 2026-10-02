//! Project named animation values onto layout and terminal paint properties.

use crate::{
    animation::{keyframes::KeyframeValue, AnimatedProperty, AnimationValue, TransformProperty},
    error::{ReactiveError, Result},
    layout::style::StyleBuilder,
};

pub(super) fn apply_property(
    style: &mut StyleBuilder,
    property: &AnimatedProperty,
    t: f32,
    viewport: (u16, u16),
) -> Result<()> {
    let lerp = |from: f32, to: f32| from + (to - from) * t;
    match property {
        AnimatedProperty::Opacity(from, to) => style.opacity = Some(lerp(*from, *to)),
        AnimatedProperty::Scale(from, to)
        | AnimatedProperty::Transform(TransformProperty::Scale(from, to)) => {
            style.motion.transform.scale_x = lerp(*from, *to);
            style.motion.transform.scale_y = lerp(*from, *to);
        }
        AnimatedProperty::Rotation(from, to) => style.motion.transform.rotation = lerp(*from, *to),
        AnimatedProperty::Transform(TransformProperty::Rotate(from, to)) => {
            style.motion.transform.rotation = lerp(*from, *to).to_degrees()
        }
        AnimatedProperty::Transform(TransformProperty::TranslateX(from, to)) => {
            style.motion.transform.x_percent = 0.0;
            style.motion.transform.x = lerp(*from, *to)
        }
        AnimatedProperty::Transform(TransformProperty::TranslateY(from, to)) => {
            style.motion.transform.y_percent = 0.0;
            style.motion.transform.y = lerp(*from, *to)
        }
        AnimatedProperty::Position(fx, fy, tx, ty) => {
            style.motion.transform.x = lerp(f32::from(*fx), f32::from(*tx));
            style.motion.transform.y = lerp(f32::from(*fy), f32::from(*ty));
        }
        AnimatedProperty::Transform(TransformProperty::Translate(fx, fy, tx, ty)) => {
            style.motion.transform.x = lerp(*fx, *tx);
            style.motion.transform.y = lerp(*fy, *ty);
        }
        AnimatedProperty::Transform(TransformProperty::ScaleX(from, to)) => {
            style.motion.transform.scale_x = lerp(*from, *to);
        }
        AnimatedProperty::Transform(TransformProperty::ScaleY(from, to)) => {
            style.motion.transform.scale_y = lerp(*from, *to);
        }
        AnimatedProperty::Transform(TransformProperty::SkewX(from, to)) => {
            style.motion.transform.skew_x = lerp(*from, *to);
        }
        AnimatedProperty::Transform(TransformProperty::SkewY(from, to)) => {
            style.motion.transform.skew_y = lerp(*from, *to);
        }
        AnimatedProperty::Transform(TransformProperty::Matrix(from, to)) => {
            style.motion.transform.matrix = [
                lerp(from.a, to.a),
                lerp(from.b, to.b),
                lerp(from.c, to.c),
                lerp(from.d, to.d),
                lerp(from.e, to.e),
                lerp(from.f, to.f),
            ];
        }
        AnimatedProperty::Size(fw, fh, tw, th) => {
            *style = std::mem::take(style)
                .width_px(lerp(f32::from(*fw), f32::from(*tw)))
                .height_px(lerp(f32::from(*fh), f32::from(*th)));
        }
        AnimatedProperty::Color(from, to) => {
            style.fg_rgba = Some((
                lerp(f32::from(from.0), f32::from(to.0)) / 255.0,
                lerp(f32::from(from.1), f32::from(to.1)) / 255.0,
                lerp(f32::from(from.2), f32::from(to.2)) / 255.0,
                1.0,
            ));
        }
        AnimatedProperty::Multiple(properties) => {
            for property in properties {
                apply_property(style, property, t, viewport)?;
            }
        }
        AnimatedProperty::Property(name, from, to) | AnimatedProperty::Custom(name, from, to) => {
            apply_value(
                style,
                name,
                &AnimationValue::Number(lerp(*from, *to)),
                viewport,
            )?;
        }
        AnimatedProperty::CssProperty(name, from, to) => {
            apply_value(
                style,
                name,
                &AnimatedProperty::interpolate_css_value(from, to, t),
                viewport,
            )?;
        }
        AnimatedProperty::PropertySet(properties) => {
            for property in properties {
                let start = property.duration_offset.clamp(0.0, 1.0);
                let progress = if start == 1.0 {
                    if t >= 1.0 {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    ((t - start) / (1.0 - start)).clamp(0.0, 1.0)
                };
                let progress = property
                    .easing_override
                    .as_ref()
                    .map_or(progress, |easing| easing.apply(progress));
                let value = AnimatedProperty::interpolate_animation_value(
                    &property.from,
                    &property.to,
                    progress,
                );
                apply_value(style, &property.name, &value, viewport)?;
            }
        }
        AnimatedProperty::Keyframes(sequence) => {
            let mut values: Vec<_> = sequence.sample(t).into_iter().collect();
            values.sort_by(|a, b| a.0.cmp(&b.0));
            for (name, value) in values {
                if let KeyframeValue::Color(r, g, b, a) = value {
                    apply_color(
                        style,
                        &name,
                        (
                            f32::from(r) / 255.0,
                            f32::from(g) / 255.0,
                            f32::from(b) / 255.0,
                            f32::from(a) / 255.0,
                        ),
                    )?;
                } else {
                    apply_value(style, &name, &value.to_animation_value(), viewport)?;
                }
            }
        }
    }
    Ok(())
}

fn invalid(name: &str, reason: &str) -> ReactiveError {
    ReactiveError::invalid_parameter(format!("Cannot paint animated property {name:?}: {reason}"))
}

fn apply_color(style: &mut StyleBuilder, name: &str, color: (f32, f32, f32, f32)) -> Result<()> {
    match name {
        "color" | "foreground" | "textColor" | "text-color" => style.fg_rgba = Some(color),
        "background" | "backgroundColor" | "background-color" => style.bg_rgba = Some(color),
        _ => {
            return Err(invalid(
                name,
                "expected a foreground or background color property",
            ))
        }
    }
    Ok(())
}

fn apply_value(
    style: &mut StyleBuilder,
    name: &str,
    value: &AnimationValue,
    viewport: (u16, u16),
) -> Result<()> {
    match value {
        AnimationValue::Number(value) => apply_number(style, name, *value),
        AnimationValue::Color { r, g, b } => apply_color(
            style,
            name,
            (
                f32::from(*r) / 255.0,
                f32::from(*g) / 255.0,
                f32::from(*b) / 255.0,
                1.0,
            ),
        ),
        AnimationValue::Unit(value, unit) => match unit.as_str() {
            "px" | "em" | "rem" => apply_number(style, name, *value),
            "vw" => apply_number(style, name, *value * f32::from(viewport.0) / 100.0),
            "vh" => apply_number(style, name, *value * f32::from(viewport.1) / 100.0),
            "%" => apply_percent(style, name, *value),
            "deg" if matches!(name, "rotate" | "rotation" | "skewX" | "skewY") => {
                apply_number(style, name, *value)
            }
            "rad" if matches!(name, "rotate" | "rotation" | "skewX" | "skewY") => {
                apply_number(style, name, value.to_degrees())
            }
            _ => Err(invalid(name, &format!("unsupported unit {unit:?}"))),
        },
        AnimationValue::Transform(matrix) if name == "transform" => {
            style.motion.transform.matrix =
                [matrix.a, matrix.b, matrix.c, matrix.d, matrix.e, matrix.f];
            Ok(())
        }
        AnimationValue::Map(values) => {
            let mut values: Vec<_> = values.iter().collect();
            values.sort_by(|a, b| a.0.cmp(b.0));
            for (name, value) in values {
                apply_value(style, name, value, viewport)?;
            }
            Ok(())
        }
        AnimationValue::Array(values) if name == "transform" && values.len() == 6 => {
            style.motion.transform.matrix.copy_from_slice(values);
            Ok(())
        }
        AnimationValue::Array(values) if name == "translate" && values.len() == 2 => {
            style.motion.transform.x = values[0];
            style.motion.transform.y = values[1];
            Ok(())
        }
        AnimationValue::Array(values) if name == "scale" && values.len() == 2 => {
            style.motion.transform.scale_x = values[0];
            style.motion.transform.scale_y = values[1];
            Ok(())
        }
        AnimationValue::Boolean(value) => {
            let current = std::mem::take(style);
            *style = match name {
                "bold" => current.bold(*value),
                "italic" => current.italic(*value),
                "underline" => current.underline(*value),
                "strike" => current.strike(*value),
                _ => {
                    *style = current;
                    return Err(invalid(name, "unsupported boolean style"));
                }
            };
            Ok(())
        }
        AnimationValue::String(value) => apply_string(style, name, value),
        _ => Err(invalid(
            name,
            "value type does not match a terminal style property",
        )),
    }
}

fn apply_number(style: &mut StyleBuilder, name: &str, value: f32) -> Result<()> {
    if !value.is_finite() {
        return Err(invalid(name, "value must be finite"));
    }
    match name {
        "opacity" => {
            style.opacity = Some(value.clamp(0.0, 1.0));
            return Ok(());
        }
        "x" | "translateX" | "translate-x" => {
            style.motion.transform.x_percent = 0.0;
            style.motion.transform.x = value;
            return Ok(());
        }
        "y" | "translateY" | "translate-y" => {
            style.motion.transform.y_percent = 0.0;
            style.motion.transform.y = value;
            return Ok(());
        }
        "rotate" | "rotation" => {
            style.motion.transform.rotation = value;
            return Ok(());
        }
        "scaleX" | "scale-x" => {
            style.motion.transform.scale_x = value;
            return Ok(());
        }
        "scaleY" | "scale-y" => {
            style.motion.transform.scale_y = value;
            return Ok(());
        }
        "scale" => {
            style.motion.transform.scale_x = value;
            style.motion.transform.scale_y = value;
            return Ok(());
        }
        "skewX" | "skew-x" => {
            style.motion.transform.skew_x = value;
            return Ok(());
        }
        "skewY" | "skew-y" => {
            style.motion.transform.skew_y = value;
            return Ok(());
        }
        "lineHeight" | "line-height" => {
            style.text.line_height = Some(value.round().clamp(1.0, f32::from(u16::MAX)) as usize);
            return Ok(());
        }
        _ => {}
    }
    let current = std::mem::take(style);
    *style = match name {
        "width" => current.width_px(value.max(0.0)),
        "height" => current.height_px(value.max(0.0)),
        "minWidth" | "min-width" => current.min_width_px(value.max(0.0)),
        "minHeight" | "min-height" => current.min_height_px(value.max(0.0)),
        "maxWidth" | "max-width" => current.max_width_px(value.max(0.0)),
        "maxHeight" | "max-height" => current.max_height_px(value.max(0.0)),
        "left" => current.inset_left(value),
        "right" => current.inset_right(value),
        "top" => current.inset_top(value),
        "bottom" => current.inset_bottom(value),
        "padding" => current.padding_all_px(value.max(0.0)),
        "paddingLeft" | "padding-left" => current.padding_l_px(value.max(0.0)),
        "paddingRight" | "padding-right" => current.padding_r_px(value.max(0.0)),
        "paddingTop" | "padding-top" => current.padding_t_px(value.max(0.0)),
        "paddingBottom" | "padding-bottom" => current.padding_b_px(value.max(0.0)),
        "margin" => current.margin_all_px(value),
        "marginLeft" | "margin-left" => current.margin_l_px(value),
        "marginRight" | "margin-right" => current.margin_r_px(value),
        "marginTop" | "margin-top" => current.margin_t_px(value),
        "marginBottom" | "margin-bottom" => current.margin_b_px(value),
        "gap" => current.gap_px(value.max(0.0), value.max(0.0)),
        "flexGrow" | "flex-grow" => current.flex_grow(value.max(0.0)),
        "flexShrink" | "flex-shrink" => current.flex_shrink(value.max(0.0)),
        "zIndex" | "z-index" => current.z_index(value.round() as i32),
        "fontWeight" | "font-weight" => current.bold(value >= 600.0),
        _ => {
            *style = current;
            return Err(invalid(name, "unknown numeric style property"));
        }
    };
    Ok(())
}

fn apply_percent(style: &mut StyleBuilder, name: &str, value: f32) -> Result<()> {
    if !value.is_finite() {
        return Err(invalid(name, "percentage must be finite"));
    }
    if matches!(name, "opacity" | "scale" | "scaleX" | "scaleY") {
        return apply_number(style, name, value / 100.0);
    }
    if matches!(name, "x" | "translateX" | "translate-x") {
        style.motion.transform.x = 0.0;
        style.motion.transform.x_percent = value / 100.0;
        return Ok(());
    }
    if matches!(name, "y" | "translateY" | "translate-y") {
        style.motion.transform.y = 0.0;
        style.motion.transform.y_percent = value / 100.0;
        return Ok(());
    }
    let current = std::mem::take(style);
    *style = match name {
        "width" => current.width_percent(value.max(0.0)),
        "height" => current.height_percent(value.max(0.0)),
        "minWidth" | "min-width" => current.min_width_percent(value.max(0.0)),
        "minHeight" | "min-height" => current.min_height_percent(value.max(0.0)),
        "maxWidth" | "max-width" => current.max_width_percent(value.max(0.0)),
        "maxHeight" | "max-height" => current.max_height_percent(value.max(0.0)),
        _ => {
            *style = current;
            return Err(invalid(
                name,
                "percentage requires a size, opacity or scale property",
            ));
        }
    };
    Ok(())
}

fn apply_string(style: &mut StyleBuilder, name: &str, value: &str) -> Result<()> {
    if value == "auto" && matches!(name, "width" | "height" | "margin") {
        let current = std::mem::take(style);
        *style = match name {
            "width" => current.width_auto(),
            "height" => current.height_auto(),
            _ => current.margin_auto(),
        };
        return Ok(());
    }
    let class = match (name, value) {
        ("fontWeight" | "font-weight", "bold") => "font-bold",
        ("fontWeight" | "font-weight", "normal") => "font-normal",
        ("fontStyle" | "font-style", "italic") => "italic",
        ("fontStyle" | "font-style", "normal") => "not-italic",
        ("textTransform" | "text-transform", "uppercase") => "uppercase",
        ("textTransform" | "text-transform", "lowercase") => "lowercase",
        ("textTransform" | "text-transform", "capitalize") => "capitalize",
        ("textTransform" | "text-transform", "none") => "normal-case",
        ("overflow", "hidden") => "overflow-hidden",
        ("overflow", "visible") => "overflow-visible",
        _ => return Err(invalid(name, "unsupported string style value")),
    };
    *style = crate::layout::css::apply_utility_classes(class, std::mem::take(style));
    Ok(())
}
