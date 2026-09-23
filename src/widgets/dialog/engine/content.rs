use super::*;
use std::sync::Weak;

#[derive(Clone)]
pub(super) struct Input {
    options: InputDialogOptions,
    revision: u64,
}
impl From<InputDialogOptions> for Input {
    fn from(options: InputDialogOptions) -> Self {
        Self {
            options,
            revision: 0,
        }
    }
}

#[derive(Clone)]
pub(super) struct Progress {
    options: ProgressDialogOptions,
    value: f32,
}
impl From<ProgressDialogOptions> for Progress {
    fn from(options: ProgressDialogOptions) -> Self {
        Self {
            options,
            value: 0.0,
        }
    }
}
#[derive(Clone)]
pub(super) struct Wizard {
    options: WizardDialogOptions,
    data: HashMap<String, String>,
}
impl From<WizardDialogOptions> for Wizard {
    fn from(options: WizardDialogOptions) -> Self {
        Self {
            options,
            data: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub(super) enum Content {
    Confirmation(ConfirmationDialogOptions),
    Input(Input),
    Autocomplete(AutocompleteDialogOptions),
    Progress(Progress),
    Toast(ToastOptions),
    Wizard(Wizard),
}
impl Content {
    pub(super) fn priority(&self) -> u16 {
        if matches!(self, Self::Toast(_)) {
            2000
        } else {
            1000
        }
    }
    pub(super) fn update(&mut self, update: DialogUpdate) -> Result<(), DialogEngineError> {
        match (self, update) {
            (Self::Progress(progress), DialogUpdate::Progress(value)) => {
                if !value.is_finite() {
                    return Err(DialogEngineError::InvalidProgress);
                }
                progress.value = value.clamp(0.0, 1.0);
            }
            (Self::Input(input), DialogUpdate::InputValue(value)) => {
                input.options.input.default_value = Some(value);
                input.revision = input.revision.wrapping_add(1);
            }
            (Self::Wizard(wizard), DialogUpdate::WizardData(data)) => wizard.data = data,
            _ => return Err(DialogEngineError::WrongType),
        }
        Ok(())
    }
    pub(super) fn connect(&mut self, core: Weak<Core>, id: DialogId) -> Option<CloseCallback> {
        let finished: CloseCallback = Arc::new(move |result| {
            if let Some(core) = core.upgrade() {
                core.finish(id, result, Some(Arc::downgrade(&core)));
            }
        });
        match self {
            Self::Confirmation(options) => options.on_close.replace(finished),
            Self::Input(input) => input.options.on_close.replace(finished),
            Self::Autocomplete(options) => options.on_close.replace(finished),
            Self::Progress(progress) => {
                let callback = progress.options.on_cancel.take();
                progress.options.on_cancel =
                    Some(Arc::new(move || finished(DialogResult::Cancelled)));
                callback.map(|callback| {
                    Arc::new(move |result| {
                        if matches!(result, DialogResult::Cancelled) {
                            callback();
                        }
                    }) as CloseCallback
                })
            }
            Self::Toast(options) => {
                let callback = options.on_close.take();
                options.on_close = Some(Arc::new(move || finished(DialogResult::Confirmed(None))));
                callback.map(|callback| Arc::new(move |_| callback()) as CloseCallback)
            }
            Self::Wizard(wizard) => {
                let completed = wizard.options.on_complete.take();
                let closed = finished.clone();
                wizard.options.on_complete = Some(Arc::new(move |data| {
                    if completed.as_ref().is_some_and(|callback| !callback(data)) {
                        return false;
                    }
                    closed(DialogResult::Confirmed(None));
                    true
                }));
                let cancelled = wizard.options.on_cancel.take();
                wizard.options.on_cancel =
                    Some(Arc::new(move || finished(DialogResult::Cancelled)));
                cancelled.map(|callback| {
                    Arc::new(move |result| {
                        if matches!(result, DialogResult::Cancelled) {
                            callback();
                        }
                    }) as CloseCallback
                })
            }
        }
    }
    pub(super) fn render(&self, id: DialogId, theme: &DialogTheme) -> Element {
        let bounds = Rect::default();
        match self {
            Self::Confirmation(options) => {
                ConfirmationDialog::new(id, options.clone()).render(bounds, theme)
            }
            Self::Input(input) => InputDialog::new(id, input.options.clone()).render_with_revision(
                bounds,
                theme,
                input.revision,
            ),
            Self::Autocomplete(options) => {
                AutocompleteDialog::new(id, options.clone()).render(bounds, theme)
            }
            Self::Progress(progress) => {
                let mut dialog = ProgressDialog::new(id, progress.options.clone());
                dialog.set_progress(progress.value);
                dialog.render(bounds, theme)
            }
            Self::Toast(options) => Toast::new(id, options.clone()).render(bounds, theme),
            Self::Wizard(wizard) => {
                let mut dialog = WizardDialog::new(id, wizard.options.clone());
                dialog.set_data(wizard.data.clone());
                dialog.render(bounds, theme)
            }
        }
    }
}
