use super::*;
use crate::widgets::dialog::http;

#[derive(Default)]
pub(super) struct State {
    pending: Option<Pending>,
    accepted: Option<Accepted>,
}

struct Snapshot {
    value: String,
    url: String,
}

impl Snapshot {
    fn matches(&self, owner: &Runtime) -> bool {
        self.value == owner.value.get() && endpoint(&owner.options()) == Some(self.url.as_str())
    }
}

struct Pending {
    snapshot: Snapshot,
    job: http::Job,
    submit: bool,
}

struct Accepted {
    snapshot: Snapshot,
    warnings: Vec<String>,
}

fn endpoint(options: &InputDialogOptions) -> Option<&str> {
    options
        .validation
        .as_ref()
        .and_then(|validation| validation.async_validation_url.as_deref())
}

pub(super) fn same_endpoint(a: &InputDialogOptions, b: &InputDialogOptions) -> bool {
    endpoint(a) == endpoint(b)
}

#[derive(serde::Deserialize)]
struct Response {
    valid: bool,
    message: Option<String>,
    #[serde(default)]
    warnings: Vec<String>,
}

impl Runtime {
    pub(super) fn cancel_remote(&self) {
        if let (Some(scheduler), Some(timer)) =
            (&self.scheduler, self.remote_timer.lock().unwrap().take())
        {
            scheduler.cancel_timer(timer);
        }
        // Drop outside the state lock; cancellation joins the worker and reaps curl.
        let state = std::mem::take(&mut *self.remote.lock().unwrap());
        drop(state);
        self.remote_pending.set(false);
    }

    pub(super) fn remote_validation(self: &Arc<Self>, submit: bool) -> bool {
        let options = self.options();
        let Some(url) = options
            .validation
            .as_ref()
            .and_then(|validation| validation.async_validation_url.as_deref())
        else {
            return true;
        };
        let mut state = self.remote.lock().unwrap();
        if let Some(accepted) = state
            .accepted
            .as_ref()
            .filter(|accepted| accepted.snapshot.matches(self))
        {
            let mut warnings = self.warnings.get();
            warnings.extend(accepted.warnings.clone());
            self.warnings.set(warnings);
            return true;
        }
        if let Some(pending) = state
            .pending
            .as_mut()
            .filter(|pending| pending.snapshot.matches(self))
        {
            pending.submit |= submit;
            return false;
        }
        drop(state);
        self.cancel_remote();
        let Some(scheduler) = &self.scheduler else {
            self.error
                .set(Some("Remote validation needs an App scheduler".into()));
            return false;
        };
        let value = self.value.get();
        let job = match http::Job::start(url, "value", &value, None) {
            Ok(job) => job,
            Err(error) => {
                self.error.set(Some(error));
                return false;
            }
        };
        self.remote.lock().unwrap().pending = Some(Pending {
            snapshot: Snapshot {
                value,
                url: url.to_owned(),
            },
            job,
            submit,
        });
        self.remote_pending.set(true);
        let owner = Arc::downgrade(self);
        let mut timer = self.remote_timer.lock().unwrap();
        *timer = Some(
            scheduler.schedule_interval(Duration::from_millis(5), move || {
                if let Some(owner) = owner.upgrade() {
                    owner.poll_remote();
                }
            }),
        );
        false
    }

    fn poll_remote(self: &Arc<Self>) {
        let (pending, result) = {
            let mut state = self.remote.lock().unwrap();
            let Some(pending) = state.pending.as_mut() else {
                return;
            };
            let Some(result) = pending.job.poll() else {
                return;
            };
            (state.pending.take().unwrap(), result)
        };
        if let (Some(scheduler), Some(timer)) =
            (&self.scheduler, self.remote_timer.lock().unwrap().take())
        {
            scheduler.cancel_timer(timer);
        }
        self.remote_pending.set(false);
        if !self.visible.get() || !self.activity.active() || !pending.snapshot.matches(self) {
            return;
        }
        let response = result.and_then(|bytes| {
            let response: Response = serde_json::from_slice(&bytes).map_err(|error| {
                format!(
                    "Remote validation returned invalid JSON at line {}, column {}",
                    error.line(),
                    error.column()
                )
            })?;
            if response.warnings.len() > 64 {
                return Err("Remote validation returned more than 64 warnings".into());
            }
            Ok(response)
        });
        let response = match response {
            Ok(response) => response,
            Err(error) => {
                self.error.set(Some(error));
                return;
            }
        };
        if !self.validate_local() {
            return;
        }
        let mut warnings = self.warnings.get();
        warnings.extend(response.warnings.clone());
        self.warnings.set(warnings);
        if !response.valid {
            self.error.set(Some(
                response
                    .message
                    .filter(|message| !message.trim().is_empty())
                    .unwrap_or_else(|| "Remote validation rejected this value".into()),
            ));
            return;
        }
        self.error.set(None);
        self.remote.lock().unwrap().accepted = Some(Accepted {
            snapshot: pending.snapshot,
            warnings: response.warnings,
        });
        if pending.submit {
            self.complete_submission();
        }
    }
}
