//! Strict JSON inputs are converted to existing dialog options.
use crate::component::Element;
use crate::widgets::dialog::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Open {
    Confirmation {
        title: String,
        #[serde(default)]
        message: String,
    },
    Input {
        title: String,
        #[serde(default)]
        prompt: String,
        #[serde(default)]
        value: Option<String>,
        #[serde(default)]
        placeholder: Option<String>,
    },
    Toast {
        title: String,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        duration: Option<u32>,
    },
    Progress {
        title: String,
        #[serde(default)]
        message: String,
        #[serde(default)]
        progress: f32,
    },
    Autocomplete {
        title: String,
        #[serde(default)]
        prompt: String,
        #[serde(default)]
        suggestions: Vec<String>,
    },
    Wizard {
        title: String,
        #[serde(default)]
        steps: Vec<Step>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Step {
    id: String,
    title: String,
    content: String,
    #[serde(default)]
    optional: bool,
}

impl Open {
    pub fn open(self, engine: &mut DialogEngine) -> Result<DialogId, DialogEngineError> {
        match self {
            Self::Confirmation { title, message } => {
                engine.try_show_confirmation(ConfirmationDialogOptions {
                    title,
                    message,
                    ..Default::default()
                })
            }
            Self::Input {
                title,
                prompt,
                value,
                placeholder,
            } => engine.try_show_input(InputDialogOptions {
                title,
                prompt,
                input: InputFieldConfig {
                    default_value: value,
                    placeholder,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Self::Toast {
                title,
                message,
                duration,
            } => engine.try_show_toast(ToastOptions {
                message: message.unwrap_or(title),
                duration: duration.map(|ms| Duration::from_millis(ms.into())),
                ..Default::default()
            }),
            Self::Progress {
                title,
                message,
                progress,
            } => {
                if !progress.is_finite() {
                    return Err(DialogEngineError::InvalidProgress);
                }
                let id = engine.try_show_progress(ProgressDialogOptions {
                    title,
                    message,
                    ..Default::default()
                })?;
                engine.update(id, DialogUpdate::Progress(progress))?;
                Ok(id)
            }
            Self::Autocomplete {
                title,
                prompt,
                suggestions,
            } => engine.try_show_autocomplete(AutocompleteDialogOptions {
                title,
                prompt,
                autocomplete: AutocompleteConfig {
                    static_suggestions: suggestions,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Self::Wizard { title, steps } => engine.try_show_wizard(WizardDialogOptions {
                title,
                steps: steps
                    .into_iter()
                    .map(|step| WizardStep {
                        id: step.id,
                        title: step.title,
                        content: Element::text(step.content),
                        can_skip: step.optional,
                        validator: None,
                    })
                    .collect(),
                ..Default::default()
            }),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub(super) enum Update {
    Progress {
        progress: f32,
    },
    Input {
        input: String,
    },
    Wizard {
        #[serde(rename = "wizardData")]
        wizard_data: HashMap<String, String>,
    },
    Layer {
        #[serde(rename = "zIndex")]
        z_index: u16,
    },
}
impl From<Update> for DialogUpdate {
    fn from(value: Update) -> Self {
        match value {
            Update::Progress { progress } => Self::Progress(progress),
            Update::Input { input } => Self::InputValue(input),
            Update::Wizard { wizard_data } => Self::WizardData(wizard_data),
            Update::Layer { z_index } => Self::ZIndex(z_index),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Completion {
    Confirmed { data: Option<String> },
    Cancelled,
    Selected { data: String },
    Custom { data: String },
    Error { data: String },
}
impl From<Completion> for DialogResult {
    fn from(value: Completion) -> Self {
        match value {
            Completion::Confirmed { data } => Self::Confirmed(data),
            Completion::Cancelled => Self::Cancelled,
            Completion::Selected { data } => Self::Selected(data),
            Completion::Custom { data } => Self::Custom(data),
            Completion::Error { data } => Self::Error(data),
        }
    }
}
impl From<DialogResult> for Completion {
    fn from(value: DialogResult) -> Self {
        match value {
            DialogResult::Confirmed(data) => Self::Confirmed { data },
            DialogResult::Cancelled => Self::Cancelled,
            DialogResult::Selected(data) => Self::Selected { data },
            DialogResult::Custom(data) => Self::Custom { data },
            DialogResult::Error(data) => Self::Error { data },
        }
    }
}
