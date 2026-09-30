//! The color roles of a theme (THM-001) and the color a theme that leaves
//! one out gets for it (THM-002).

use super::{presets, Theme};
use std::sync::OnceLock;

type Rgba = (f32, f32, f32, f32);

/// The fills. Each has a text role of its own, named by the fill with
/// `-foreground` added: the text on `primary` is `primary-foreground`.
pub const FILLS: [&str; 8] = [
    "primary",
    "secondary",
    "accent",
    "success",
    "warning",
    "error",
    "info",
    "selection",
];

/// The roles that are neither a fill nor the text on one.
pub const PLAIN: [&str; 10] = [
    "background",
    "surface",
    "foreground",
    "text-muted",
    "border",
    "input",
    "ring",
    "hover",
    "overlay",
    "shadow",
];

const BLACK: Rgba = (0.0, 0.0, 0.0, 1.0);
const WHITE: Rgba = (1.0, 1.0, 1.0, 1.0);

/// Relative luminance as WCAG 2.1 defines it.
fn luminance(color: Rgba) -> f32 {
    let channel = |value: f32| {
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.0) + 0.7152 * channel(color.1) + 0.0722 * channel(color.2)
}

/// The contrast ratio of two colors as WCAG 2.1 defines it, from 1 to 21.
pub fn contrast(one: Rgba, other: Rgba) -> f32 {
    let (one, other) = (luminance(one), luminance(other));
    (one.max(other) + 0.05) / (one.min(other) + 0.05)
}

/// Black or white, whichever contrasts more with `fill`. One of the two
/// always reaches 4.5 to 1, because their ratios multiply to 21.
pub fn text_on(fill: Rgba) -> Rgba {
    if contrast(BLACK, fill) > contrast(WHITE, fill) {
        BLACK
    } else {
        WHITE
    }
}

/// The built-in light or dark preset, built once.
fn preset(light: bool) -> &'static Theme {
    static DARK: OnceLock<Theme> = OnceLock::new();
    static LIGHT: OnceLock<Theme> = OnceLock::new();
    if light {
        LIGHT.get_or_init(presets::light_theme)
    } else {
        DARK.get_or_init(presets::dark_theme)
    }
}

impl Theme {
    /// The color of a role that neither this theme nor a theme it extends
    /// defines (THM-002): the text on a fill is black or white, whichever
    /// contrasts more with the fill; `selection` and `ring` are the theme's
    /// `primary`; `input` is its `surface`; `hover` is seven parts of its
    /// `surface` and one of its `foreground`; and every other role is the
    /// built-in light preset's when the theme's `background` is light, and
    /// the dark preset's otherwise. `None` when `role` is not a role.
    pub(super) fn fallback(&self, role: &str) -> Option<Rgba> {
        if let Some(fill) = role.strip_suffix("-foreground") {
            return FILLS
                .contains(&fill)
                .then(|| self.resolve_variable(fill))
                .flatten()
                .map(text_on);
        }
        match role {
            "selection" | "ring" => self.resolve_variable("primary"),
            "input" => self.resolve_variable("surface"),
            "hover" => {
                let surface = self.resolve_variable("surface")?;
                let foreground = self.resolve_variable("foreground")?;
                let mixed = |ground: f32, text: f32| (7.0 * ground + text) / 8.0;
                Some((
                    mixed(surface.0, foreground.0),
                    mixed(surface.1, foreground.1),
                    mixed(surface.2, foreground.2),
                    1.0,
                ))
            }
            role if PLAIN.contains(&role) || FILLS.contains(&role) => {
                let light = self
                    .defined("background")
                    .is_some_and(|ground| contrast(BLACK, ground) > contrast(WHITE, ground));
                preset(light).defined(role)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_or_white_reaches_the_contrast_of_normal_text_on_any_fill() {
        for level in 0..=255u8 {
            let grey = f32::from(level) / 255.0;
            for fill in [
                (grey, grey, grey, 1.0),
                (grey, 0.0, 0.0, 1.0),
                (0.0, grey, 0.0, 1.0),
                (0.0, 0.0, grey, 1.0),
                (grey, grey, 0.0, 1.0),
            ] {
                let ratio = contrast(text_on(fill), fill);
                assert!(ratio >= 4.5, "{fill:?}: {ratio}");
            }
        }
    }

    #[test]
    fn a_name_that_is_no_role_has_no_fallback() {
        let theme = Theme::new("empty");
        assert_eq!(theme.resolve_variable("chart-9"), None);
        assert_eq!(theme.resolve_variable("nothing-foreground"), None);
        assert_eq!(theme.resolve_variable("blue-500"), None);
    }
}
