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
    let end = Instant::now() + Duration::from_secs(2);
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
