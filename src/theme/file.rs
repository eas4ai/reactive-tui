//! Themes loaded from JSON documents (THM-006).

use super::presets::{
    dark_theme, gruvbox_dark_theme, high_contrast_theme, light_theme, solarized_dark_theme,
};
use super::{Theme, ThemeVariables};
use serde_json::Value;
use std::fmt;
use std::path::{Path, PathBuf};

/// Why a theme document could not be loaded (THM-006). Each message names
/// the file, key or preset at fault.
#[derive(Debug)]
pub enum ThemeFileError {
    /// The file could not be read.
    Read {
        /// The path that was read.
        path: PathBuf,
        /// The io error from the read.
        source: std::io::Error,
    },
    /// The text is not valid JSON.
    Json(serde_json::Error),
    /// The document is not a JSON object.
    NotAnObject,
    /// A key outside `name`, `extends` and `variables`.
    UnknownKey {
        /// The offending key.
        key: String,
    },
    /// `extends` names no built-in preset.
    UnknownPreset {
        /// The name that matched no preset.
        name: String,
    },
    /// A value that must be a string is not one.
    NotAString {
        /// The key that holds the value.
        key: String,
    },
    /// `variables` is present but is not an object.
    VariablesNotAnObject,
    /// The document has no `name`.
    MissingName,
}

impl fmt::Display for ThemeFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeFileError::Read { path, source } => {
                write!(f, "could not read theme file {}: {source}", path.display())
            }
            ThemeFileError::Json(err) => write!(f, "theme document is not valid JSON: {err}"),
            ThemeFileError::NotAnObject => {
                write!(f, "theme document must be a JSON object")
            }
            ThemeFileError::UnknownKey { key } => write!(
                f,
                "unknown key `{key}` in theme document; only name, extends and variables are allowed"
            ),
            ThemeFileError::UnknownPreset { name } => write!(
                f,
                "unknown preset `{name}` in extends; expected dark, light, high-contrast, solarized-dark or gruvbox-dark"
            ),
            ThemeFileError::NotAString { key } => {
                write!(f, "the value of `{key}` must be a string")
            }
            ThemeFileError::VariablesNotAnObject => {
                write!(f, "`variables` must be a JSON object")
            }
            ThemeFileError::MissingName => write!(f, "theme document has no `name`"),
        }
    }
}

impl std::error::Error for ThemeFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ThemeFileError::Read { source, .. } => Some(source),
            ThemeFileError::Json(err) => Some(err),
            _ => None,
        }
    }
}

/// The built-in preset a document's `extends` names, in either the spec's
/// spelling (`high-contrast`) or the code's (`high_contrast`).
fn preset_named(name: &str) -> Option<Theme> {
    match name {
        "dark" => Some(dark_theme()),
        "light" => Some(light_theme()),
        "high-contrast" | "high_contrast" => Some(high_contrast_theme()),
        "solarized-dark" | "solarized_dark" => Some(solarized_dark_theme()),
        "gruvbox-dark" | "gruvbox_dark" => Some(gruvbox_dark_theme()),
        _ => None,
    }
}

impl Theme {
    /// Load a theme from a JSON document: an object with `name` (a string),
    /// an optional `extends` (a built-in preset name) and `variables` (an
    /// object of variable names to string values). Any other key, an
    /// unknown preset, or a non-string value is refused with an error that
    /// names it (THM-006).
    pub fn from_json(text: &str) -> Result<Theme, ThemeFileError> {
        let value: Value = serde_json::from_str(text).map_err(ThemeFileError::Json)?;
        let Value::Object(map) = value else {
            return Err(ThemeFileError::NotAnObject);
        };
        for key in map.keys() {
            if !matches!(key.as_str(), "name" | "extends" | "variables") {
                return Err(ThemeFileError::UnknownKey { key: key.clone() });
            }
        }
        let name = match map.get("name") {
            None => return Err(ThemeFileError::MissingName),
            Some(Value::String(name)) => name.clone(),
            Some(_) => {
                return Err(ThemeFileError::NotAString {
                    key: "name".to_string(),
                })
            }
        };
        let parent = match map.get("extends") {
            None => None,
            Some(Value::String(preset)) => {
                Some(
                    preset_named(preset).ok_or_else(|| ThemeFileError::UnknownPreset {
                        name: preset.clone(),
                    })?,
                )
            }
            Some(_) => {
                return Err(ThemeFileError::NotAString {
                    key: "extends".to_string(),
                })
            }
        };
        let mut variables = ThemeVariables::new();
        match map.get("variables") {
            None => {}
            Some(Value::Object(entries)) => {
                for (key, value) in entries {
                    let Value::String(value) = value else {
                        return Err(ThemeFileError::NotAString { key: key.clone() });
                    };
                    variables = variables.set(key.clone(), value.clone());
                }
            }
            Some(_) => return Err(ThemeFileError::VariablesNotAnObject),
        }
        let theme = Theme::new(name).with_variables(variables);
        Ok(match parent {
            Some(base) => theme.extend(base),
            None => theme,
        })
    }

    /// Load a theme from the JSON document at `path` (THM-006). The error
    /// names the path when the file cannot be read.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Theme, ThemeFileError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|source| ThemeFileError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json(&text)
    }

    /// Write this theme as a JSON document that [`Theme::from_json`] loads
    /// back with the same name and variables (THM-006). `extends` is written
    /// only when the theme extends another, by that theme's name, and the
    /// variables are sorted by name so the output is deterministic.
    pub fn to_json(&self) -> String {
        fn quoted(text: &str) -> String {
            Value::String(text.to_owned()).to_string()
        }
        let mut out = String::from("{\n");
        out.push_str(&format!("  \"name\": {},\n", quoted(&self.name)));
        if let Some(parent) = &self.extends {
            out.push_str(&format!("  \"extends\": {},\n", quoted(&parent.name)));
        }
        let mut entries: Vec<(&String, &String)> = self.variables.all().iter().collect();
        entries.sort();
        out.push_str("  \"variables\": {");
        for (index, (key, value)) in entries.iter().enumerate() {
            let separator = if index == 0 { "\n" } else { ",\n" };
            out.push_str(&format!(
                "{separator}    {}: {}",
                quoted(key),
                quoted(value)
            ));
        }
        if !entries.is_empty() {
            out.push('\n');
            out.push_str("  ");
        }
        out.push_str("}\n}");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{
        dark_theme, gruvbox_dark_theme, high_contrast_theme, light_theme, solarized_dark_theme,
    };

    const OCEAN: &str =
        r##"{"name":"ocean","extends":"dark","variables":{"--color-primary":"#0077aa"}}"##;

    fn close(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
        let tol = 1.0 / 255.0 + 1e-6;
        (a.0 - b.0).abs() <= tol
            && (a.1 - b.1).abs() <= tol
            && (a.2 - b.2).abs() <= tol
            && (a.3 - b.3).abs() <= tol
    }

    #[test]
    fn thm_006_a_theme_loads_from_json_and_resolves_its_roles() {
        let theme = Theme::from_json(OCEAN).expect("the ocean document loads");
        assert_eq!(theme.name, "ocean");
        let primary = theme.resolve_color("primary").expect("primary resolves");
        assert!(close(
            primary,
            (0.0, 0x77 as f32 / 255.0, 0xaa as f32 / 255.0, 1.0)
        ));
        let surface = theme.resolve_color("surface").expect("surface resolves");
        let dark_surface = dark_theme()
            .resolve_color("surface")
            .expect("dark surface resolves");
        assert!(close(surface, dark_surface));
    }

    #[test]
    fn thm_006_an_unknown_preset_is_refused_by_name() {
        let err = Theme::from_json(r#"{"name":"x","extends":"ocean"}"#)
            .expect_err("an unknown preset is refused");
        assert!(err.to_string().contains("ocean"), "message was: {err}");
    }

    #[test]
    fn thm_006_an_unknown_key_is_refused_by_name() {
        let err =
            Theme::from_json(r#"{"name":"x","colors":{}}"#).expect_err("an unknown key is refused");
        assert!(err.to_string().contains("colors"), "message was: {err}");
    }

    #[test]
    fn thm_006_a_non_string_value_is_refused_by_key() {
        let err = Theme::from_json(r#"{"name":"x","variables":{"--color-primary":7}}"#)
            .expect_err("a non-string value is refused");
        assert!(
            err.to_string().contains("--color-primary"),
            "message was: {err}"
        );
    }

    #[test]
    fn thm_006_every_preset_round_trips_through_json() {
        let presets = [
            dark_theme(),
            light_theme(),
            high_contrast_theme(),
            solarized_dark_theme(),
            gruvbox_dark_theme(),
        ];
        for preset in presets {
            let loaded = Theme::from_json(&preset.to_json())
                .unwrap_or_else(|e| panic!("{} does not load back: {e}", preset.name));
            assert_eq!(loaded.name, preset.name);
            assert_eq!(loaded.variables.all(), preset.variables.all());
        }
    }

    #[test]
    fn thm_006_from_file_reads_a_document() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path =
            std::env::temp_dir().join(format!("thm006-ocean-{}-{nanos}.json", std::process::id()));
        std::fs::write(&path, OCEAN).expect("write the temp theme file");
        let loaded = Theme::from_file(&path);
        let _ = std::fs::remove_file(&path);
        let theme = loaded.expect("the file loads");
        assert_eq!(theme.name, "ocean");

        let missing = std::env::temp_dir().join("thm006-no-such-file-does-not-exist.json");
        let err = Theme::from_file(&missing).expect_err("a missing file fails");
        assert!(
            err.to_string().contains(&missing.display().to_string()),
            "message was: {err}"
        );
    }
}
