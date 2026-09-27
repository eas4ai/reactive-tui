#!/usr/bin/env python3
"""Verify dialog HTTPS against an ephemeral trusted and untrusted certificate."""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import argparse
import hashlib
import json
import os
import signal
import ssl
import subprocess
import sys
import tempfile
import threading

ROOT = Path(__file__).resolve().parents[1]
PROBE = "widgets::dialog::http::tests::tls_certificate_probe"


def captured(command, *, environment=None, timeout):
    # Match the existing maintenance gates: a timeout also stops compiler children.
    with tempfile.TemporaryFile() as output:
        process = subprocess.Popen(
            command, cwd=ROOT, env=environment, stdout=output,
            stderr=subprocess.STDOUT, start_new_session=os.name != "nt",
        )
        try:
            status = process.wait(timeout=timeout)
        except BaseException:
            if os.name == "nt":
                subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                               check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            else:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            process.wait()
            raise
        output.seek(0)
        data = output.read(4 * 1024 * 1024 + 1)
        if len(data) > 4 * 1024 * 1024:
            raise RuntimeError("HTTPS fixture command output exceeds 4 MiB")
        return subprocess.CompletedProcess(command, status, data.decode("utf-8", errors="replace"))


def main(dedicated_desktop=False):
    if os.name == "nt" and not dedicated_desktop:
        raise RuntimeError("Windows HTTPS trust setup requires --dedicated-desktop")
    with tempfile.TemporaryDirectory(prefix="rtui-dialog-tls-") as directory:
        certificate = Path(directory) / "certificate.pem"
        key = Path(directory) / "key.pem"
        generated = captured(
            ["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes",
             "-keyout", str(key), "-out", str(certificate), "-days", "1",
             "-subj", "/CN=localhost", "-addext", "subjectAltName=DNS:localhost"],
            timeout=30,
        )
        if generated.returncode:
            raise RuntimeError(f"Could not generate the HTTPS fixture certificate: {generated.stdout}")
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(certificate, key)
        requests = []
        failures = []

        class Handler(BaseHTTPRequestHandler):
            def handle(self):
                try:
                    super().handle()
                except ConnectionResetError:
                    # A client rejecting the certificate can close before an HTTP request.
                    pass

            def do_POST(self):
                try:
                    length = int(self.headers["Content-Length"])
                    if not 0 <= length <= 1024 * 1024:
                        raise ValueError("unbounded fixture request")
                    body = json.loads(self.rfile.read(length))
                    if self.path != "/validate" or body != {"value": "tls-fixture"}:
                        raise ValueError("unexpected fixture request")
                    requests.append(body)
                    response = b'{"valid":true}'
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(response)))
                    self.end_headers()
                    self.wfile.write(response)
                except Exception as error:
                    failures.append(str(error))
                    self.close_connection = True

            def log_message(self, *_args):
                pass

        class Server(ThreadingHTTPServer):
            daemon_threads = False

            def handle_error(self, _request, _address):
                failures.append(str(sys.exception()))

            def get_request(self):
                stream, address = super().get_request()
                stream.settimeout(2)
                try:
                    return context.wrap_socket(stream, server_side=True), address
                except Exception:
                    stream.close()
                    raise

        server = Server(("127.0.0.1", 0), Handler)
        worker = threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.01})
        worker.start()
        try:
            for trust in ("trusted", "untrusted"):
                environment = os.environ.copy()
                environment.update({
                    "CARGO_INCREMENTAL": "0",
                    "CARGO_TERM_COLOR": "never",
                    "RTUI_DIALOG_TLS_URL": f"https://localhost:{server.server_port}/validate",
                    "RTUI_DIALOG_TLS_TRUST": trust,
                    "NO_PROXY": "localhost,127.0.0.1,::1",
                    "no_proxy": "localhost,127.0.0.1,::1",
                })
                for name in ("CURL_CA_BUNDLE", "SSL_CERT_FILE", "SSL_CERT_DIR"):
                    environment.pop(name, None)
                if trust == "trusted":
                    environment["CURL_CA_BUNDLE"] = str(certificate)
                # Schannel ignores CURL_CA_BUNDLE. Exercise its native trust
                # machine store, then remove this owned certificate before the
                # untrusted case. This requires a dedicated disposable desktop.
                native_trust = os.name == "nt" and trust == "trusted"
                if native_trust:
                    certutil = str(Path(os.environ["SystemRoot"]) / "System32/certutil.exe")
                    thumbprint = hashlib.sha1(
                        ssl.PEM_cert_to_DER_cert(certificate.read_text())
                    ).hexdigest()
                try:
                    if native_trust:
                        added = captured([certutil, "-f", "-addstore", "Root", str(certificate)], timeout=30)
                        if added.returncode:
                            raise RuntimeError(f"Could not trust the HTTPS fixture certificate: {added.stdout}")
                    result = captured(
                        ["cargo", "test", "--locked", "--lib", PROBE, "--", "--ignored", "--exact", "--nocapture"],
                        environment=environment, timeout=60,
                    )
                finally:
                    if native_trust:
                        removed = captured([certutil, "-delstore", "Root", thumbprint], timeout=30)
                        if removed.returncode:
                            raise RuntimeError(f"Could not remove the HTTPS fixture certificate: {removed.stdout}")
                print(result.stdout, end="", flush=True)
                if result.returncode or f"test {PROBE} ... ok" not in result.stdout:
                    raise RuntimeError(f"The {trust} HTTPS probe did not execute and pass")
                if failures or len(requests) != 1:
                    raise RuntimeError(f"Unexpected HTTPS fixture traffic: requests={len(requests)}, failures={failures}")
                print(f"Dialog HTTPS {trust}: passed", flush=True)
        finally:
            server.shutdown()
            server.server_close()
            worker.join(timeout=3)
            if worker.is_alive():
                raise RuntimeError("HTTPS fixture did not stop")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dedicated-desktop", action="store_true")
    main(parser.parse_args().dedicated_desktop)
