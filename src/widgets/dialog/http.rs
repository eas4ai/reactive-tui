//! Owned HTTP requests for dialog validation and suggestions.

use crate::core::owned_process::{self, Options};
use std::{
    collections::HashMap,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const MAX_REQUEST: usize = 1024 * 1024;
const MAX_RESPONSE: u64 = 64 * 1024;
const TIMEOUT: Duration = Duration::from_secs(5);
const CURL_ENVIRONMENT: &[&str] = &[
    "PATH",
    "PATHEXT",
    "SystemRoot",
    "WINDIR",
    "http_proxy",
    "https_proxy",
    "all_proxy",
    "no_proxy",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "CURL_CA_BUNDLE",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
];

#[cfg(test)]
mod tests;

pub(super) struct Job {
    cancelled: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    result: Mutex<Receiver<Result<Vec<u8>, String>>>,
}

impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogHttpJob")
            .field("running", &self.worker.is_some())
            .finish()
    }
}

impl Job {
    pub(super) fn start(
        url: &str,
        field: &str,
        value: &str,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<Self, String> {
        let configuration = configuration(url, field, value, headers)?;
        let cancelled = Arc::new(AtomicBool::new(false));
        let stop = cancelled.clone();
        let (sender, result) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("dialog-http".into())
            .spawn(move || {
                let result = request(&configuration, TIMEOUT, || stop.load(Ordering::Acquire));
                let _ = sender.send(result);
            })
            .map_err(|error| format!("Could not start dialog HTTP worker: {error}"))?;
        Ok(Self {
            cancelled,
            worker: Some(worker),
            result: Mutex::new(result),
        })
    }

    pub(super) fn poll(&mut self) -> Option<Result<Vec<u8>, String>> {
        let result = self.result.lock().unwrap().try_recv();
        let result = match result {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err("Dialog HTTP worker stopped without a result".into())
            }
        };
        if let Some(worker) = self.worker.take() {
            if worker.join().is_err() {
                return Some(Err("Dialog HTTP worker panicked".into()));
            }
        }
        Some(result)
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            if worker.join().is_err() {
                log::error!("Dialog HTTP worker panicked during cancellation");
            }
        }
    }
}

fn option(configuration: &mut String, name: &str, value: &str) -> Result<(), String> {
    configuration.push_str(name);
    configuration.push_str(" = \"");
    for character in value.chars() {
        match character {
            '\\' => configuration.push_str("\\\\"),
            '"' => configuration.push_str("\\\""),
            '\n' => configuration.push_str("\\n"),
            '\r' => configuration.push_str("\\r"),
            '\t' => configuration.push_str("\\t"),
            character if character.is_control() => {
                return Err("HTTP configuration contains an unsupported control character".into())
            }
            character => configuration.push(character),
        }
        if configuration.len() > MAX_REQUEST {
            return Err("Dialog HTTP request exceeds the 1 MiB limit".into());
        }
    }
    configuration.push_str("\"\n");
    if configuration.len() > MAX_REQUEST {
        return Err("Dialog HTTP request exceeds the 1 MiB limit".into());
    }
    Ok(())
}

fn configuration(
    url: &str,
    field: &str,
    value: &str,
    headers: Option<&HashMap<String, String>>,
) -> Result<String, String> {
    if value.len() > MAX_REQUEST {
        return Err("Dialog HTTP request exceeds the 1 MiB limit".into());
    }
    if !url.split_once("://").is_some_and(|(scheme, rest)| {
        (scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https"))
            && !rest
                .split(['/', '?', '#'])
                .next()
                .unwrap_or_default()
                .is_empty()
    }) || url
        .chars()
        .any(|character| character.is_whitespace() || character.is_control())
    {
        return Err("Dialog HTTP endpoint must be an HTTP or HTTPS URL".into());
    }
    let mut result = String::new();
    option(&mut result, "url", url)?;
    option(&mut result, "request", "POST")?;
    option(&mut result, "header", "Content-Type: application/json")?;
    option(&mut result, "header", "Accept: application/json")?;
    if let Some(headers) = headers {
        for (name, value) in headers {
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
                || value.chars().any(|character| character.is_control())
            {
                return Err("Dialog HTTP header has an invalid name or value".into());
            }
            if name.len().saturating_add(value.len()) > MAX_REQUEST {
                return Err("Dialog HTTP request exceeds the 1 MiB limit".into());
            }
            option(&mut result, "header", &format!("{name}: {value}"))?;
        }
    }
    let payload = serde_json::to_string(&HashMap::from([(field, value)]))
        .map_err(|error| format!("Could not encode dialog request: {error}"))?;
    option(&mut result, "data-raw", &payload)?;
    Ok(result)
}

fn curl_command() -> Command {
    let mut command = Command::new("curl");
    command.env_clear();
    for name in CURL_ENVIRONMENT {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command
}

fn request(
    configuration: &str,
    timeout: Duration,
    cancelled: impl Fn() -> bool,
) -> Result<Vec<u8>, String> {
    let started = Instant::now();
    let options = Options {
        purpose: "dialog HTTP",
        timeout,
        max_input: MAX_REQUEST as u64,
        // The body is followed by a newline and the three-digit HTTP status.
        max_output: MAX_RESPONSE + 4,
        capture_output: true,
        allow_background_after_success: false,
    };
    let mut version = curl_command();
    version.args(["--disable", "--version"]);
    let version = owned_process::run(version, None, options, &cancelled)
        .map_err(|error| format!("Dialog HTTP requires curl 8.4 or newer: {error}"))?;
    let version = String::from_utf8_lossy(&version);
    let mut parts = version
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .split('.');
    let major = parts.next().and_then(|part| part.parse::<u32>().ok());
    let minor = parts.next().and_then(|part| part.parse::<u32>().ok());
    if !matches!((major, minor), (Some(major), Some(minor)) if major > 8 || major == 8 && minor >= 4)
    {
        return Err("Dialog HTTP requires curl 8.4 or newer".into());
    }
    let mut command = curl_command();
    command.args([
        "--disable",
        "--globoff",
        "--proto",
        "=http,https",
        "--silent",
        "--show-error",
        "--max-time",
        "5",
        "--max-filesize",
        "65536",
        "--write-out",
        "\n%{http_code}",
        "--config",
        "-",
    ]);
    let output = owned_process::run(
        command,
        Some(configuration.as_bytes()),
        Options {
            timeout: timeout.saturating_sub(started.elapsed()),
            ..options
        },
        cancelled,
    )?;
    let Some(separator) = output.iter().rposition(|byte| *byte == b'\n') else {
        return Err("Dialog HTTP response has no status".into());
    };
    let status = std::str::from_utf8(&output[separator + 1..])
        .ok()
        .and_then(|status| status.parse::<u16>().ok())
        .filter(|_| output.len() - separator == 4)
        .ok_or_else(|| "Dialog HTTP response has an invalid status".to_string())?;
    if !(200..300).contains(&status) {
        return Err(format!("Dialog HTTP endpoint returned status {status}"));
    }
    Ok(output[..separator].to_vec())
}
