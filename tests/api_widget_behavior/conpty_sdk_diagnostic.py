"""Compare raw OS and official redistributable ConPTY cleanup on Windows.

Diagnostic only: this does not exercise or certify Reactive-TUI's PTY backend.
"""
import ctypes as c
from ctypes import wintypes as w
import hashlib
import io
import os
from pathlib import Path
import time
import urllib.request
import zipfile

VERSION = "1.24.260710001"
DIGEST = "175640566a3b59c4b132070ee96c2c77e5ab7edd2e92732a5eb3610bbf63d90e"
URL = ("https://api.nuget.org/v3-flatcontainer/microsoft.windows.console.conpty/"
       f"{VERSION}/microsoft.windows.console.conpty.{VERSION}.nupkg")


class Coord(c.Structure):
    _fields_ = [("x", c.c_short), ("y", c.c_short)]


kernel = c.WinDLL("kernel32", use_last_error=True)
kernel.CreatePipe.argtypes = [c.POINTER(w.HANDLE), c.POINTER(w.HANDLE), c.c_void_p, w.DWORD]
kernel.CreatePipe.restype = w.BOOL
kernel.CloseHandle.argtypes = [w.HANDLE]
kernel.CloseHandle.restype = w.BOOL
kernel.GetCurrentProcess.restype = w.HANDLE
kernel.GetProcessHandleCount.argtypes = [w.HANDLE, c.POINTER(w.DWORD)]
kernel.GetProcessHandleCount.restype = w.BOOL


def handles():
    count = w.DWORD()
    if not kernel.GetProcessHandleCount(kernel.GetCurrentProcess(), c.byref(count)):
        raise c.WinError(c.get_last_error())
    return count.value


def cycles(dll, prefix, label):
    create = getattr(dll, prefix + "CreatePseudoConsole")
    create.argtypes = [Coord, w.HANDLE, w.HANDLE, w.DWORD, c.POINTER(w.HANDLE)]
    create.restype = c.c_long
    close = getattr(dll, prefix + "ClosePseudoConsole")
    close.argtypes = [w.HANDLE]
    close.restype = None
    counts = [handles()]
    for cycle in range(6):
        pipes = []
        console = w.HANDLE()
        try:
            for _ in range(2):
                read, write = w.HANDLE(), w.HANDLE()
                if not kernel.CreatePipe(c.byref(read), c.byref(write), None, 0):
                    raise c.WinError(c.get_last_error())
                pipes.extend([read, write])
            result = create(Coord(40, 8), pipes[0], pipes[3], 0, c.byref(console))
            if result < 0:
                raise RuntimeError(f"{label}: CreatePseudoConsole HRESULT {result:#x}")
        finally:
            for pipe in pipes:
                if not kernel.CloseHandle(pipe):
                    raise c.WinError(c.get_last_error())
            if console:
                close(console)
        counts.append(handles())
        print(f"{label} raw cycle {cycle}: {counts[-1]} handles", flush=True)
    time.sleep(0.5)
    print(f"{label}: before/cycles/after500ms {counts + [handles()]}", flush=True)
    return counts


print("Diagnostic: direct ConPTY API only; no Reactive-TUI sessions", flush=True)
cycles(kernel, "", "Windows OS")
with urllib.request.urlopen(URL, timeout=30) as response:
    package = response.read(8 * 1024 * 1024)
assert hashlib.sha256(package).hexdigest() == DIGEST, "SDK package digest mismatch"
directory = Path(os.environ["RUNNER_TEMP"]) / "conpty-sdk-diagnostic"
directory.mkdir()
with zipfile.ZipFile(io.BytesIO(package)) as archive:
    for entry, name in [("runtimes/win-x64/native/conpty.dll", "conpty.dll"),
                        ("build/native/runtimes/x64/OpenConsole.exe", "OpenConsole.exe")]:
        (directory / name).write_bytes(archive.read(entry))
sdk = c.WinDLL(str(directory / "conpty.dll"), winmode=0x1100)
print(f"Microsoft.Windows.Console.ConPTY {VERSION} sha256:{DIGEST}", flush=True)
counts = cycles(sdk, "Conpty", "Microsoft SDK")
# Exclude one-time DLL initialization; every subsequent console must return to
# exactly the same count. A per-cycle OS leak cannot pass this comparison.
assert len(set(counts[1:])) == 1, f"SDK still leaks per-console handles: {counts}"
print("PASS diagnostic: official SDK has no per-console handle growth", flush=True)
