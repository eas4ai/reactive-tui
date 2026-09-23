use super::{
    api::{AnimateParams, PropertyValue},
    AnimatedProperty, AnimationTarget, AnimationTargetError,
};
use std::collections::HashMap;

pub(super) struct TargetBinding {
    targets: Vec<AnimationTarget>,
    params: AnimateParams,
    tracks: Vec<Vec<(String, PropertyValue)>>,
    pub(super) capture_pending: bool,
}
impl TargetBinding {
    pub(super) fn new(
        targets: Vec<AnimationTarget>,
        params: AnimateParams,
    ) -> Result<Self, AnimationTargetError> {
        if targets.is_empty() {
            return Err(AnimationTargetError::MissingTarget(
                "empty target list".into(),
            ));
        }
        for (present, name) in [
            (params.keyframes.is_some(), "keyframes"),
            (params.color.is_some(), "color"),
            (params.size.is_some(), "size"),
            (params.position.is_some(), "position"),
            (params.css.is_some(), "CSS"),
            (params.transform.is_some(), "transform map"),
        ] {
            if present {
                return Err(AnimationTargetError::UnsupportedProperty(name));
            }
        }
        Ok(Self {
            targets,
            params,
            tracks: Vec::new(),
            capture_pending: true,
        })
    }
    pub(super) fn resolve(&mut self) -> Result<AnimatedProperty, AnimationTargetError> {
        let mut properties = Vec::new();
        let mut tracks = Vec::new();
        for target in &self.targets {
            let current = target.current()?;
            let mut params = self.params.clone();
            let mut track = Vec::new();
            for (name, value) in [
                ("opacity", &mut params.opacity),
                ("translateX", &mut params.translate_x),
                ("translateY", &mut params.translate_y),
                ("scale", &mut params.scale),
                ("rotate", &mut params.rotate),
            ] {
                if let Some(value) = value {
                    *value = resolve_value(name, value, &current)?;
                    track.push((name.into(), value.clone()));
                }
            }
            if let Some(custom) = &mut params.custom {
                let mut names = custom.keys().cloned().collect::<Vec<_>>();
                names.sort();
                for name in names {
                    if track.iter().any(|(existing, _)| existing == &name) {
                        return Err(AnimationTargetError::InvalidValue(
                            name,
                            "property specified twice".into(),
                        ));
                    }
                    let value = custom.get_mut(&name).expect("existing property");
                    *value = resolve_value(&name, value, &current)?;
                    track.push((name, value.clone()));
                }
            }
            properties.push(super::api::combine_properties(
                super::api::extract_animated_properties(&params),
            ));
            tracks.push(track);
        }
        self.tracks = tracks;
        Ok(super::api::combine_properties(properties))
    }
    pub(super) fn live(&self) -> Result<(), AnimationTargetError> {
        for target in &self.targets {
            target.current()?;
        }
        Ok(())
    }
    pub(super) fn sample(&self, progress: f32) -> Result<(), AnimationTargetError> {
        self.live()?;
        for (target, track) in self.targets.iter().zip(&self.tracks) {
            let values = track
                .iter()
                .map(|(name, value)| {
                    let (from, to, progress) = match value {
                        PropertyValue::FromTo { from, to } => (*from, *to, progress),
                        PropertyValue::Array(values) if values.len() > 2 => {
                            let position = progress.clamp(0.0, 1.0) * (values.len() - 1) as f32;
                            let segment = (position as usize).min(values.len() - 2);
                            (
                                values[segment],
                                values[segment + 1],
                                position - segment as f32,
                            )
                        }
                        PropertyValue::Array(values) => (values[0], values[1], progress),
                        _ => unreachable!("resolved current values"),
                    };
                    let value = from + (to - from) * progress;
                    if !value.is_finite() {
                        return Err(AnimationTargetError::InvalidValue(
                            name.clone(),
                            "non-finite sample".into(),
                        ));
                    }
                    Ok((name.clone(), value))
                })
                .collect::<Result<HashMap<_, _>, _>>()?;
            target.sample(values)?;
        }
        Ok(())
    }
}

pub(super) fn numeric_values(params: &AnimateParams) -> Vec<(&str, &PropertyValue)> {
    let mut result = Vec::new();
    for (name, value) in [
        ("opacity", &params.opacity),
        ("translateX", &params.translate_x),
        ("translateY", &params.translate_y),
        ("scale", &params.scale),
        ("rotate", &params.rotate),
    ] {
        if let Some(value) = value {
            result.push((name, value));
        }
    }
    if let Some(custom) = &params.custom {
        result.extend(custom.iter().map(|(name, value)| (name.as_str(), value)));
    }
    result
}
pub(super) fn resolve_value(
    name: &str,
    value: &PropertyValue,
    current: &HashMap<String, f32>,
) -> Result<PropertyValue, AnimationTargetError> {
    let invalid = |message: &str| AnimationTargetError::InvalidValue(name.into(), message.into());
    let (from, to) = match value {
        PropertyValue::FromTo { from, to } => (*from, *to),
        PropertyValue::Array(values) => {
            if values.len() < 2 || values.iter().any(|value| !value.is_finite()) {
                return Err(invalid("an array needs at least two finite values"));
            }
            return Ok(value.clone());
        }
        PropertyValue::Single(to) => (
            *current
                .get(name)
                .ok_or_else(|| AnimationTargetError::MissingProperty(name.into()))?,
            *to,
        ),
        PropertyValue::Relative(expression) => {
            let from = *current
                .get(name)
                .ok_or_else(|| AnimationTargetError::MissingProperty(name.into()))?;
            let expression = expression.trim();
            let (operator, operand) = expression
                .split_at_checked(1)
                .ok_or_else(|| invalid("relative expression needs an operator and number"))?;
            let operand = operand
                .parse::<f32>()
                .map_err(|_| invalid("relative operand is not a number"))?;
            if !operand.is_finite() {
                return Err(invalid("relative operand must be finite"));
            }
            let to = match operator {
                "+" => from + operand,
                "-" => from - operand,
                "*" => from * operand,
                "/" if operand != 0.0 => from / operand,
                "/" => return Err(invalid("division by zero")),
                _ => return Err(invalid("relative operator must be +, -, * or /")),
            };
            (from, to)
        }
    };
    if !from.is_finite() || !to.is_finite() {
        return Err(invalid("endpoints must be finite"));
    }
    Ok(PropertyValue::FromTo { from, to })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relative_binding_captures_current_values_when_delayed_playback_starts() {
        use crate::{
            animation::{
                api::{try_animate, DelayValue},
                TargetRegistry,
            },
            builder::core::div,
            layout::style::StyleBuilder,
            reactive::wake::AppWaker,
        };
        use std::time::{Duration, Instant};
        let mut owner = TargetRegistry::new(AppWaker::new());
        let element = |opacity| {
            div()
                .id("target")
                .styles(StyleBuilder::new().opacity(opacity))
                .build()
        };
        owner.publish(&element(0.25), None, 0).unwrap();
        let target = owner.context().target("target").unwrap();
        let mut animation = try_animate(
            target,
            AnimateParams {
                opacity: Some(PropertyValue::Relative("+0.25".into())),
                delay: Some(DelayValue::Fixed(10000.0)),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!animation.update(Duration::ZERO));
        owner.publish(&element(0.5), None, 0).unwrap();
        animation.start_time = Some(Instant::now() - Duration::from_secs(20));
        assert!(animation.update(Duration::ZERO));
        assert_eq!(animation.property, AnimatedProperty::Opacity(0.5, 0.75));
    }
}
