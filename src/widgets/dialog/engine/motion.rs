use super::*;
use crate::widgets::display::modal::Motion;

/// One terminal-cell animation sample for a named dialog animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DialogAnimationFrame {
    /// Opacity in 0..=1.
    pub opacity: f32,
    /// Horizontal displacement in terminal cells.
    pub x: f32,
    /// Vertical displacement in terminal cells.
    pub y: f32,
    /// Nonnegative horizontal scale.
    pub scale_x: f32,
    /// Nonnegative vertical scale.
    pub scale_y: f32,
}
impl Default for DialogAnimationFrame {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}
impl DialogAnimationFrame {
    fn valid(self) -> bool {
        [self.opacity, self.x, self.y, self.scale_x, self.scale_y]
            .into_iter()
            .all(f32::is_finite)
            && (0.0..=1.0).contains(&self.opacity)
            && self.scale_x >= 0.0
            && self.scale_y >= 0.0
    }
}

fn frame(animation: &DialogAnimation, progress: f32) -> DialogAnimationFrame {
    let mut frame = DialogAnimationFrame::default();
    match animation {
        DialogAnimation::None => {}
        DialogAnimation::Fade | DialogAnimation::Custom(_) => frame.opacity = progress,
        DialogAnimation::Slide(direction) => {
            let offset = 3.0 * (1.0 - progress).powi(3);
            match direction {
                SlideDirection::Up => frame.y = -offset,
                SlideDirection::Down => frame.y = offset,
                SlideDirection::Left => frame.x = -offset,
                SlideDirection::Right => frame.x = offset,
                SlideDirection::Center => {
                    frame.scale_x = 0.8 + progress * 0.2;
                    frame.scale_y = frame.scale_x;
                }
            }
        }
        DialogAnimation::Scale => {
            frame.scale_x = 0.8 + progress * 0.2;
            frame.scale_y = frame.scale_x;
        }
        DialogAnimation::Bounce => {
            let scale = if progress < 0.5 {
                4.0 * progress * progress
            } else {
                1.0 - (1.0 - progress).powi(2) * (progress * std::f32::consts::TAU).sin().abs()
            };
            frame.scale_x = scale;
            frame.scale_y = scale;
        }
    }
    frame
}

pub(super) fn motion(
    core: &Arc<Core>,
    id: DialogId,
    config: &DialogEngineConfig,
    custom: Option<AnimationCallback>,
) -> Motion {
    let owner = Arc::downgrade(core);
    let animation = config.default_theme.animation.clone();
    Motion {
        duration: config.animation_duration,
        apply: Arc::new(move |progress, style| {
            let value = custom.as_ref().map_or_else(
                || frame(&animation, progress),
                |callback| callback(progress),
            );
            if !value.valid() {
                let error =
                    "Dialog animation returned invalid opacity, displacement or scale".to_string();
                if let Some(core) = owner.upgrade() {
                    core.close(id, DialogResult::Error(error.clone()));
                }
                return Err(error);
            }
            style.opacity = Some(value.opacity);
            style.motion.transform.x = value.x;
            style.motion.transform.y = value.y;
            style.motion.transform.scale_x = value.scale_x;
            style.motion.transform.scale_y = value.scale_y;
            Ok(())
        }),
    }
}
