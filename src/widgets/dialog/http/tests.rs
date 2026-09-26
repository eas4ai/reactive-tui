use super::*;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn listener() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    (listener, url)
}

fn accept(listener: TcpListener) -> TcpStream {
    // A hang guard, not a timing check: generous so a busy machine cannot
    // fail a correct test by running it slowly.
    let end = Instant::now() + Duration::from_secs(30);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                // Windows accepted sockets inherit the listener's nonblocking mode.
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                return stream;
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < end =>
            {
                thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("loopback accept: {error}"),
        }
    }
}

fn receive(stream: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        let count = stream.read(&mut buffer).unwrap();
        assert_ne!(count, 0, "client closed before sending its request");
        bytes.extend_from_slice(&buffer[..count]);
        assert!(bytes.len() <= MAX_REQUEST);
        let Some(end) = bytes.windows(4).position(|bytes| bytes == b"\r\n\r\n") else {
            continue;
        };
        let header = std::str::from_utf8(&bytes[..end]).unwrap();
        let length = header
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().unwrap())
            })
            .unwrap();
        if bytes.len() >= end + 4 + length {
            return bytes;
        }
    }
}

#[test]
fn request_preserves_json_escaping_and_custom_headers() {
    let (listener, url) = listener();
    let server = thread::spawn(move || {
        let mut stream = accept(listener);
        let bytes = receive(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
            .unwrap();
        bytes
    });
    let value = "\"\\\n界e\u{301} @fixture";
    let headers = HashMap::from([("Authorization".into(), "Bearer fixture-token".into())]);
    let configuration = configuration(&url, "value", value, Some(&headers)).unwrap();
    assert_eq!(request(&configuration, TIMEOUT, || false).unwrap(), b"{}");
    let bytes = server.join().unwrap();
    let end = bytes
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .unwrap();
    assert!(std::str::from_utf8(&bytes[..end])
        .unwrap()
        .contains("Authorization: Bearer fixture-token"));
    let body: serde_json::Value = serde_json::from_slice(&bytes[end + 4..]).unwrap();
    assert_eq!(body, serde_json::json!({"value":value}));
}

#[test]
fn cancellation_joins_worker_and_closes_the_actual_connection() {
    let (listener, url) = listener();
    let job = Job::start(&url, "value", "pending", None).unwrap();
    let mut stream = accept(listener);
    receive(&mut stream);
    let started = Instant::now();
    drop(job);
    // The behavior under test: dropping cancels at once instead of waiting
    // for the unanswered request.
    assert!(started.elapsed() < Duration::from_secs(1));
    let mut remaining = Vec::new();
    if let Err(error) = stream.read_to_end(&mut remaining) {
        assert_eq!(error.kind(), std::io::ErrorKind::ConnectionReset, "{error}");
    }
    assert!(
        remaining.is_empty(),
        "request already arrived before cancellation"
    );
}

#[test]
fn a_silent_endpoint_hits_the_owned_deadline() {
    let (_listener, url) = listener();
    let configuration = configuration(&url, "value", "pending", None).unwrap();
    let started = Instant::now();
    let checks = std::cell::RefCell::new(Vec::new());
    let error = request(&configuration, Duration::from_millis(200), || {
        checks.borrow_mut().push(started.elapsed());
        false
    })
    .unwrap_err();
    assert!(error.contains("timed out"), "{error}");
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(1),
        "request took {elapsed:?}; deadline checks: {:?}",
        checks.into_inner()
    );
}

#[test]
fn chunked_response_cannot_bypass_the_size_limit() {
    let (listener, url) = listener();
    let server = thread::spawn(move || {
        let mut stream = accept(listener);
        receive(&mut stream);
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        let body = "x".repeat(MAX_RESPONSE as usize + 1);
        let response = format!("{:x}\r\n{body}\r\n0\r\n\r\n", body.len());
        let _ = stream.write_all(response.as_bytes());
    });
    let configuration = configuration(&url, "value", "bounded", None).unwrap();
    let error = request(&configuration, TIMEOUT, || false).unwrap_err();
    assert!(
        error.contains("63") || error.contains("transfer limit"),
        "{error}"
    );
    server.join().unwrap();
}

#[test]
fn unsafe_configuration_and_oversized_requests_fail_before_spawn() {
    for url in [
        "file:///etc/passwd",
        "ftp://example.test",
        "http://?",
        "http://example.test\nurl=elsewhere",
    ] {
        assert!(configuration(url, "value", "x", None).is_err());
    }
    for (name, value) in [("X-Test", "ok\r\nInjected: yes"), ("@file", "x")] {
        assert!(configuration(
            "https://example.test",
            "value",
            "x",
            Some(&HashMap::from([(name.into(), value.into())]))
        )
        .is_err());
    }
    assert!(configuration(
        "https://example.test",
        "value",
        &"x".repeat(MAX_REQUEST),
        None
    )
    .unwrap_err()
    .contains("1 MiB"));
}

#[cfg(unix)]
fn xis_001_validate_curl_captures(captures: &[serde_json::Value]) -> Result<(), String> {
    if captures.len() != 2 {
        return Err(format!(
            "expected two curl processes, got {}",
            captures.len()
        ));
    }
    let expected_arguments = [
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
    ];
    let expected_environment = [
        ("https_proxy", "http://proxy.invalid:8080"),
        ("NO_PROXY", "example.invalid"),
        ("CURL_CA_BUNDLE", "/fixture/ca.pem"),
    ];
    for capture in captures {
        let environment = capture["environment"]
            .as_object()
            .ok_or_else(|| "curl capture has no environment".to_string())?;
        for name in [
            "HOME",
            "RTUI_XIS_SECRET",
            "RTUI_XIS_CHILD",
            "RTUI_XIS_CAPTURE",
        ] {
            if environment.contains_key(name) {
                return Err(format!("curl inherited unrelated environment value {name}"));
            }
        }
        if environment
            .get("PATH")
            .and_then(serde_json::Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err("curl lost PATH needed to locate the executable".into());
        }
        for (name, expected) in expected_environment {
            if environment.get(name).and_then(serde_json::Value::as_str) != Some(expected) {
                return Err(format!("curl lost supported environment value {name}"));
            }
        }
    }
    if captures[0]["arguments"] != serde_json::json!(["--disable", "--version"]) {
        return Err("curl version arguments changed".into());
    }
    if captures[1]["arguments"] != serde_json::json!(expected_arguments) {
        return Err("curl request arguments changed".into());
    }
    let request = captures[1]["stdin"]
        .as_str()
        .ok_or_else(|| "request capture has no stdin".to_string())?;
    if !request.contains("url = \"https://example.invalid/validate\"")
        || !request.contains("data-raw = \"{\\\"value\\\":\\\"private\\\"}\"")
    {
        return Err("curl request configuration did not stay on private stdin".into());
    }
    Ok(())
}

#[test]
#[cfg(unix)]
fn xis_001_capture_validator_rejects_unsafe_observations() {
    let environment = serde_json::json!({
        "PATH": "/fixture/bin",
        "https_proxy": "http://proxy.invalid:8080",
        "NO_PROXY": "example.invalid",
        "CURL_CA_BUNDLE": "/fixture/ca.pem"
    });
    let valid = vec![
        serde_json::json!({
            "arguments": ["--disable", "--version"],
            "stdin": "",
            "environment": environment
        }),
        serde_json::json!({
            "arguments": ["--disable", "--globoff", "--proto", "=http,https", "--silent",
                "--show-error", "--max-time", "5", "--max-filesize", "65536", "--write-out",
                "\n%{http_code}", "--config", "-"],
            "stdin": "url = \"https://example.invalid/validate\"\ndata-raw = \"{\\\"value\\\":\\\"private\\\"}\"\n",
            "environment": environment
        }),
    ];
    xis_001_validate_curl_captures(&valid).unwrap();

    let mut inherited = valid.clone();
    inherited[1]["environment"]["RTUI_XIS_SECRET"] = "must-not-leak".into();
    assert!(xis_001_validate_curl_captures(&inherited).is_err());
    let mut missing_proxy = valid.clone();
    missing_proxy[0]["environment"]
        .as_object_mut()
        .unwrap()
        .remove("https_proxy");
    assert!(xis_001_validate_curl_captures(&missing_proxy).is_err());
    let mut redirect = valid.clone();
    redirect[1]["arguments"]
        .as_array_mut()
        .unwrap()
        .push("--location".into());
    assert!(xis_001_validate_curl_captures(&redirect).is_err());
    let mut exposed = valid.clone();
    exposed[1]["stdin"] = "".into();
    assert!(xis_001_validate_curl_captures(&exposed).is_err());
    assert!(xis_001_validate_curl_captures(&valid[..1]).is_err());
}

#[test]
#[cfg(unix)]
fn xis_001_request_process_receives_only_documented_environment() {
    const CHILD: &str = "RTUI_XIS_CHILD";
    const CAPTURE: &str = "RTUI_XIS_CAPTURE";
    if std::env::var_os(CHILD).is_some() {
        let path = std::path::PathBuf::from(std::env::var_os(CAPTURE).unwrap());
        let configuration =
            configuration("https://example.invalid/validate", "value", "private", None).unwrap();
        assert_eq!(request(&configuration, TIMEOUT, || false).unwrap(), b"{}");
        let captures = std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect::<Vec<_>>();
        xis_001_validate_curl_captures(&captures).unwrap();
        return;
    }

    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let capture = directory.path().join("capture.jsonl");
    let executable = directory.path().join("curl");
    let script = format!(
        r#"#!/usr/bin/python3
import json, os, sys
capture = {{"arguments": sys.argv[1:], "stdin": "", "environment": dict(os.environ)}}
if sys.argv[1:] != ["--disable", "--version"]:
    capture["stdin"] = sys.stdin.read()
with open({}, "a", encoding="utf-8") as output:
    output.write(json.dumps(capture, sort_keys=True) + "\n")
if sys.argv[1:] == ["--disable", "--version"]:
    print("curl 8.4.0 fixture")
else:
    sys.stdout.write("{{}}\n200")
"#,
        serde_json::to_string(&capture.to_string_lossy()).unwrap()
    );
    std::fs::write(&executable, script).unwrap();
    let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&executable, permissions).unwrap();

    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "widgets::dialog::http::tests::xis_001_request_process_receives_only_documented_environment",
            "--nocapture",
        ])
        .env_clear()
        .env(CHILD, "1")
        .env(CAPTURE, &capture)
        .env("PATH", directory.path())
        .env("https_proxy", "http://proxy.invalid:8080")
        .env("NO_PROXY", "example.invalid")
        .env("CURL_CA_BUNDLE", "/fixture/ca.pem")
        .env("HOME", "/fixture/home")
        .env("RTUI_XIS_SECRET", "must-not-leak")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}

#[test]
#[cfg(unix)]
fn missing_curl_is_an_explicit_error_in_an_isolated_process() {
    const MARKER: &str = "RTUI_HTTP_MISSING_CURL_CHILD";
    if std::env::var_os(MARKER).is_some() {
        let configuration = configuration("https://example.invalid", "value", "x", None).unwrap();
        let error = request(&configuration, TIMEOUT, || false).unwrap_err();
        assert!(
            error.contains("requires curl 8.4") && error.contains("Could not start curl"),
            "{error}"
        );
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "widgets::dialog::http::tests::missing_curl_is_an_explicit_error_in_an_isolated_process", "--nocapture"])
        .env(MARKER, "1")
        .env("PATH", directory.path())
        .output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}

#[test]
#[ignore = "invoked with a trusted/untrusted HTTPS fixture by scripts/check-dialog-http.py"]
fn tls_certificate_probe() {
    let url = std::env::var("RTUI_DIALOG_TLS_URL").expect("HTTPS fixture URL");
    let trust = std::env::var("RTUI_DIALOG_TLS_TRUST").expect("HTTPS fixture trust mode");
    let configuration = configuration(&url, "value", "tls-fixture", None).unwrap();
    let result = request(&configuration, TIMEOUT, || false);
    match trust.as_str() {
        "trusted" => {
            let response: serde_json::Value = serde_json::from_slice(&result.unwrap()).unwrap();
            assert_eq!(response, serde_json::json!({"valid": true}));
        }
        "untrusted" => {
            let error = result.unwrap_err();
            assert!(
                error.contains("60"),
                "expected a certificate verification failure: {error}"
            );
        }
        _ => panic!("unknown HTTPS fixture trust mode"),
    }
}
