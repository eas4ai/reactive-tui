#!/usr/bin/env python3
"""Exercise the real App in a private X server and D-Bus session with Orca.

Run with /usr/bin/python3 so the distribution's GI/AT-SPI bindings are available.
Run serially: Orca rejects another process owned by the same user, even on a
separate D-Bus session. This runner never uses --replace.
All activation environments are set before D-Bus starts. Never launch a desktop
application until the fixture's window has been located on the private display.
"""
import argparse
import ctypes
import json
import os
from pathlib import Path
import re
import select
import signal
import subprocess
import sys
import tempfile
import time

ACCOUNT = "Account preferences accessible label"
LOCKED = "Unavailable preferences accessible label"
PRIVACY = "Privacy preferences accessible label"
ENTRY = "Nested account entry"
HOME = "Breadcrumb home accessible label"
DOCS = "Breadcrumb docs accessible label"
CURRENT = "Breadcrumb current accessible label"
UNAVAILABLE = "Breadcrumb unavailable accessible label"


def stop_group(child):
    try:
        os.killpg(child.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        child.wait(timeout=3)
    except subprocess.TimeoutExpired:
        pass
    # D-Bus services can outlive their launcher. Reap the whole owned group.
    try:
        os.killpg(child.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    child.wait(timeout=3)


def isolated(args):
    run = Path(tempfile.mkdtemp(prefix="rtui-orca-"))
    env = os.environ.copy()
    for key in ["DBUS_SESSION_BUS_ADDRESS", "DBUS_STARTER_ADDRESS",
                "AT_SPI_BUS_ADDRESS", "WAYLAND_DISPLAY", "SESSION_MANAGER",
                "XAUTHORITY"]:
        env.pop(key, None)
    for name in ["CONFIG", "CACHE", "DATA", "RUNTIME"]:
        directory = run / name.lower()
        directory.mkdir(mode=0o700)
        env[f"XDG_{name}_HOME" if name != "RUNTIME" else "XDG_RUNTIME_DIR"] = str(directory)
    env.update(XDG_CURRENT_DESKTOP="GNOME", XDG_SESSION_DESKTOP="gnome",
               XDG_SESSION_TYPE="x11", GDK_BACKEND="x11", GTK_USE_PORTAL="0")
    orca_data = Path(env["XDG_DATA_HOME"]) / "orca"
    orca_data.mkdir()
    # Flush genuine reader diagnostics as they happen. Do not alter its event
    # handling, speech generation, or settings to manufacture announcements.
    (orca_data / "orca-customizations.py").write_text(
        "from orca import debug\n"
        "if debug.debugFile is not None:\n"
        "    debug.debugFile.reconfigure(line_buffering=True)\n"
    )
    read, write = os.pipe()
    with (run / "xvfb.log").open("w") as log:
        xvfb = subprocess.Popen(
            ["Xvfb", "-displayfd", str(write), "-screen", "0", "1024x768x24", "-nolisten", "tcp"],
            pass_fds=(write,), stdout=log, stderr=log, start_new_session=True, env=env,
        )
    os.close(write)
    try:
        if not select.select([read], [], [], 5)[0]:
            raise RuntimeError("Xvfb did not become ready")
        display = os.read(read, 64).decode().strip()
        if not display.isdecimal():
            raise RuntimeError("Xvfb returned an invalid display number")
        env["DISPLAY"] = ":" + display
        command = ["dbus-run-session", "--", "/usr/bin/python3", str(Path(__file__).resolve()),
                   "--inside", str(run), "--binary", str(Path(args.binary).resolve()),
                   "--geometry", args.geometry]
        if args.catalog:
            command.extend(["--catalog", args.catalog])
        if args.negative:
            command.append("--negative")
        if args.negative_css:
            command.append("--negative-css")
        with (run / "session.log").open("w") as log:
            session = subprocess.Popen(command, stdout=log, stderr=log, env=env, start_new_session=True)
        try:
            status = session.wait(timeout=65)
        finally:
            stop_group(session)
        output = (run / "session.log").read_text()
        for line in output.splitlines():
            if line.startswith("A11Y "):
                print(line, flush=True)
        print(f"Orca diagnostic logs: {run}", flush=True)
        if not (args.negative or args.negative_css):
            reader_log = (run / "orca-launch.log").read_text()
            if re.search(r"(?:AddAccessible with unknown signature|Unknown signature .* for RemoveAccessible)",
                         output + reader_log):
                raise RuntimeError("AT-SPI reader rejected malformed cache signal signatures")
        if args.negative or args.negative_css:
            expected = "CSS control absent" if args.negative_css else "missing semantic account label"
            if status == 0 or expected not in output:
                raise AssertionError("negative control did not fail at the semantic-label assertion")
            print("A11Y negative control rejected painted labels without semantic labels", flush=True)
        elif status:
            raise RuntimeError(f"Orca workflow failed with exit {status}; see {run / 'session.log'}")
    finally:
        os.close(read)
        stop_group(xvfb)


class Keyboard:
    def __init__(self, window):
        self.x11 = ctypes.CDLL("libX11.so.6")
        self.x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
        self.x11.XOpenDisplay.restype = ctypes.c_void_p
        self.x11.XSetInputFocus.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_ulong]
        self.x11.XFlush.argtypes = [ctypes.c_void_p]
        self.x11.XCloseDisplay.argtypes = [ctypes.c_void_p]
        self.x11.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
        self.x11.XDefaultRootWindow.restype = ctypes.c_ulong
        self.x11.XKeysymToKeycode.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
        self.x11.XKeysymToKeycode.restype = ctypes.c_uint
        self.xtst = ctypes.CDLL("libXtst.so.6")
        self.xtst.XTestFakeKeyEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong]
        self.display = self.x11.XOpenDisplay(os.environ["DISPLAY"].encode())
        if not self.display:
            raise RuntimeError("cannot open the private X display")
        self.window = window
        self.focus(True)

    def focus(self, focused):
        target = self.window if focused else self.x11.XDefaultRootWindow(self.display)
        self.x11.XSetInputFocus(self.display, target, 1, 0)
        self.x11.XFlush(self.display)

    def key(self, keysym):
        code = self.x11.XKeysymToKeycode(self.display, keysym)
        if not code:
            raise RuntimeError(f"no X keycode for {keysym}")
        for pressed in [1, 0]:
            if not self.xtst.XTestFakeKeyEvent(self.display, code, pressed, 0):
                raise RuntimeError("isolated keyboard injection failed")
        self.x11.XFlush(self.display)

    def reader_next_object(self, previous=False):
        # Orca's standard Ctrl+Orca+Right object-navigation command. It reads
        # inert content without assigning application keyboard focus to it.
        modifiers = [self.x11.XKeysymToKeycode(self.display, sym) for sym in [0xffe3, 0xff63]]
        if not all(modifiers):
            raise RuntimeError("missing Orca modifier keycodes")
        try:
            for code in modifiers:
                assert self.xtst.XTestFakeKeyEvent(self.display, code, 1, 0)
            self.key(0xff51 if previous else 0xff53)
        finally:
            for code in reversed(modifiers):
                self.xtst.XTestFakeKeyEvent(self.display, code, 0, 0)
            self.x11.XFlush(self.display)

    def close(self):
        self.x11.XCloseDisplay(self.display)


def inside(args):
    import gi
    gi.require_version("Atspi", "2.0")
    from gi.repository import Atspi, GLib
    Atspi.set_timeout(1000, 1000)
    run = Path(args.inside)
    children, logs = [], []
    keyboard = None

    def spawn(command, filename):
        log = (run / filename).open("w")
        logs.append(log)
        child = subprocess.Popen(command, stdout=log, stderr=log)
        children.append(child)
        return child

    def wait(reason, predicate, timeout=5):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            while GLib.MainContext.default().pending():
                GLib.MainContext.default().iteration(False)
            try:
                value = predicate()
            except GLib.Error as error:
                # A valid cache removal can retire a node during traversal.
                # Retry the observation, never the user action or its deadline.
                if not application_disappeared(error):
                    raise
                value = None
            if value:
                return value
            time.sleep(0.03)
        raise AssertionError(reason)

    def descendants(node, depth=0):
        if depth > 20:
            raise AssertionError("unexpected accessibility tree depth")
        node.clear_cache()
        yield node
        count = node.get_child_count()
        if count > 200:
            raise AssertionError("unexpected accessibility tree width")
        for index in range(count):
            child = node.get_child_at_index(index)
            if child:
                yield from descendants(child, depth + 1)

    def application_disappeared(error):
        return (error.domain == "atspi_error" and error.code == 0
                and error.message == "The application no longer exists")

    def applications():
        desktop = Atspi.get_desktop(0)
        desktop.clear_cache()
        for index in range(desktop.get_child_count()):
            try:
                app = desktop.get_child_at_index(index)
                if app and app.get_name() == "accessibility_probe":
                    yield app
            except GLib.Error as error:
                if not application_disappeared(error):
                    raise

    def application():
        return next(applications(), None)

    def find_window(name):
        for app in applications():
            try:
                for node in descendants(app):
                    if node.get_name() == name:
                        return node
            except GLib.Error as error:
                # App removal can race this remote traversal. The caller's
                # existing deadline still requires its replacement to appear.
                if not application_disappeared(error):
                    raise
        return None

    def window(round_number):
        return find_window(f"Reactive TUI App accessibility round {round_number}")

    def background_window():
        return find_window("Independent background App")

    def named(root, label, role=None):
        return next((node for node in descendants(root) if node.get_name() == label
                     and (role is None or node.get_role() == role)), None)

    def state(node, flag):
        node.clear_cache()
        return node.get_state_set().contains(flag)

    def click(node):
        action = node.get_action_iface()
        assert action is not None and action.get_n_actions() == 1
        assert action.get_action_name(0) == "click"
        assert action.do_action(0)

    def speech(text, start=0):
        path = run / "orca.log"
        if not path.exists():
            return False
        return any("SPEECH OUTPUT:" in line and text in line.split("SPEECH OUTPUT:", 1)[1]
                   for line in path.read_text().splitlines()[start:])

    try:
        for schema, key in [("org.gnome.desktop.interface", "toolkit-accessibility"),
                            ("org.gnome.desktop.a11y.applications", "screen-reader-enabled")]:
            subprocess.run(["gsettings", "set", schema, key, "true"], check=True, timeout=5)
        for command in [["gnome-terminal", "--version"], ["orca", "--version"]]:
            version = subprocess.run(command, capture_output=True, text=True, check=True, timeout=10)
            print("A11Y " + version.stdout.strip(), flush=True)
            (run / (command[0] + "-version.log")).write_text(version.stdout + version.stderr)
        print("A11Y isolated X11/Xvfb viewport " + args.geometry, flush=True)
        orca = spawn(["orca", "--debug-file", str(run / "orca.log")], "orca-launch.log")

        def reader_ready():
            if orca.poll() is not None:
                raise RuntimeError(f"Orca exited during startup ({orca.returncode}); see orca-launch.log")
            return (run / "orca.log").exists() and "Starting Atspi main event loop" in (run / "orca.log").read_text()

        wait("Orca did not register with AT-SPI", reader_ready, timeout=10)
        command = ["gnome-terminal", "--display=" + os.environ["DISPLAY"],
                   "--geometry=" + args.geometry, "--wait", "--", args.binary, str(run / "callbacks.jsonl")]
        if args.negative:
            command.append("--without-semantic-labels")
        if args.negative_css:
            command.append("--without-semantic-styles")
        if args.catalog in ["tabs", "overlays", "display", "dialogs"]:
            command.append("--" + args.catalog)
        terminal = spawn(command, "terminal.log")

        def isolated_window():
            output = subprocess.run(["xwininfo", "-root", "-tree"], capture_output=True,
                                    text=True, check=True, timeout=3).stdout
            match = re.search(r'(0x[0-9a-f]+) "Terminal": \("org.gnome.Terminal" "Gnome-terminal"\)', output)
            if match:
                attributes = subprocess.run(["xwininfo", "-id", match.group(1)], capture_output=True,
                                            text=True, check=True, timeout=3).stdout
                if "Map State: IsViewable" not in attributes:
                    return None
                (run / "xvfb-windows.log").write_text(output)
                return int(match.group(1), 16)
            return None

        # This must precede any synthesized input or accessibility action.
        xid = wait("GNOME Terminal is absent from the private X display", isolated_window)
        keyboard = Keyboard(xid)
        root = wait("first App accessible window absent", lambda: window(1))
        if args.catalog:
            if args.catalog == "tabs":
                from orca_tabs import exercise
            elif args.catalog == "overlays":
                from orca_overlays import exercise
            elif args.catalog == "display":
                from orca_display import exercise
            elif args.catalog == "dialogs":
                from orca_dialogs import exercise
            else:
                from orca_data import exercise
            exercise(root, keyboard, wait, named, state, click, speech, Atspi,
                     lambda: len((run / "orca.log").read_text().splitlines()))
            keyboard.key(0xffc6)
            second = wait("second App absent", lambda: window(2))
            wait("first App remains attached", lambda: window(1) is None)
            # Window registration precedes its first rendered controls. Confirm
            # that the replacement event loop accepts focus before sending exit.
            control = wait("second App controls absent", lambda: next(
                (node for node in descendants(second)
                 if node.get_role() != Atspi.Role.FRAME
                 and state(node, Atspi.StateType.FOCUSABLE)
                 and state(node, Atspi.StateType.ENABLED)), None))
            assert control.get_component_iface().grab_focus()
            # As for the first App, host input establishes that this newly
            # attached adapter belongs to the active terminal window.
            keyboard.key(0xff53)
            wait(f"second App did not accept focus: {control.get_role_name()} {control.get_name()}",
                 lambda: any(state(node, Atspi.StateType.FOCUSED) for node in descendants(second)))
            keyboard.key(0xffc6)
            terminal.wait(timeout=5)
            assert terminal.returncode == 0
            if args.catalog == "dialogs":
                reports = [json.loads(line) for line in (run / "callbacks.jsonl").read_text().splitlines()]
                assert reports[0]["menu_calls"] == ["Confirmation complete", "Input complete: demo",
                    "Autocomplete complete: Alpine", "Progress cancelled", "Wizard complete", "Toast closed"], reports
            wait("closed App remains attached", lambda: application() is None)
            assert orca.poll() is None
            print("A11Y data controls: reader delivery and adapter removal passed", flush=True)
            return
        account = wait("missing semantic account label", lambda: named(root, ACCOUNT))
        locked = named(root, LOCKED)
        privacy = named(root, PRIVACY)
        assert account.get_role() == Atspi.Role.PUSH_BUTTON
        assert locked is not None and privacy is not None
        assert state(account, Atspi.StateType.EXPANDABLE)
        assert not state(account, Atspi.StateType.EXPANDED)
        assert not state(locked, Atspi.StateType.ENABLED)
        assert not state(locked, Atspi.StateType.SENSITIVE)
        locked_action = locked.get_action_iface()
        assert locked_action is None or locked_action.get_n_actions() == 0
        assert named(root, ENTRY) is None
        keyboard.key(0xff53)  # Right; establish host input focus without activation.
        wait("account focus not delivered", lambda: state(account, Atspi.StateType.FOCUSED))
        wait("Orca did not announce account label", lambda: speech(ACCOUNT))
        wait("Orca did not announce collapsed button role", lambda: speech("collapsed button"))
        keyboard.focus(False)
        wait("leaving the terminal did not clear accessible focus", lambda: not state(account, Atspi.StateType.FOCUSED))
        keyboard.focus(True)
        wait("returning to the terminal did not restore accessible focus", lambda: state(account, Atspi.StateType.FOCUSED))
        click(account)
        wait("assistive click did not expand account", lambda: state(account, Atspi.StateType.EXPANDED))
        wait("Orca did not announce expansion", lambda: speech("'expanded'"))
        entry = wait("expanded child input absent", lambda: named(root, ENTRY))
        assert entry.get_role() == Atspi.Role.ENTRY
        assert entry.get_component_iface().grab_focus()
        wait("nested child focus not delivered", lambda: state(entry, Atspi.StateType.FOCUSED))
        wait("Orca did not announce child label", lambda: speech(ENTRY))
        assert privacy.get_component_iface().grab_focus()
        wait("virtual header focus not delivered", lambda: state(privacy, Atspi.StateType.FOCUSED))
        wait("Orca did not announce privacy label", lambda: speech(PRIVACY))
        keyboard.key(0xff0d)  # Return; activate the selected virtual header.
        wait("privacy keyboard activation failed", lambda: state(privacy, Atspi.StateType.EXPANDED))
        wait("collapsed child remains accessible", lambda: named(root, ENTRY) is None)
        assert not state(account, Atspi.StateType.EXPANDED)
        click(privacy)
        wait("assistive collapse failed", lambda: not state(privacy, Atspi.StateType.EXPANDED))
        keyboard.key(0xff50)  # Home; focus the first header.
        wait("Home did not focus first header", lambda: state(account, Atspi.StateType.FOCUSED))
        keyboard.key(0xff0d)
        wait("keyboard expansion failed", lambda: state(account, Atspi.StateType.EXPANDED))
        keyboard.key(0xffc5)  # F8; remove the widget through the root component.
        wait("removed widget remains accessible", lambda: named(root, ACCOUNT) is None)
        assert named(root, ENTRY) is None
        print("A11Y round 1: labels, roles, focus, disabled actions, expansion, nested input and removal passed", flush=True)
        home = wait("breadcrumb replacement absent", lambda: named(root, HOME))
        docs = named(root, DOCS)
        current = named(root, CURRENT)
        unavailable = named(root, UNAVAILABLE)
        assert docs is not None and current is not None and unavailable is not None
        assert all(node.get_role() == Atspi.Role.LINK for node in [home, docs, current, unavailable])
        for inert in [current, unavailable]:
            action = inert.get_action_iface()
            assert action is None or action.get_n_actions() == 0
        assert not state(unavailable, Atspi.StateType.ENABLED)
        assert state(current, Atspi.StateType.ACTIVE), "current breadcrumb lacks active state"
        assert current.get_attributes().get("current") == "page", "current breadcrumb lacks page attribute"
        assert home.get_component_iface().grab_focus()
        wait("breadcrumb assistive focus absent", lambda: state(home, Atspi.StateType.FOCUSED))
        wait("Orca did not announce breadcrumb home", lambda: speech(HOME))
        keyboard.key(0xff0d)
        speech_start = len((run / "orca.log").read_text().splitlines())
        keyboard.key(0xff53)
        wait("breadcrumb keyboard focus did not skip inert segment", lambda: state(docs, Atspi.StateType.FOCUSED))
        wait("Orca did not announce breadcrumb docs after Right", lambda: speech(DOCS, speech_start))
        click(docs)
        speech_start = len((run / "orca.log").read_text().splitlines())
        keyboard.reader_next_object()
        wait("Orca did not read inert current breadcrumb", lambda: speech(CURRENT, speech_start))
        wait("Orca did not announce current page state", lambda: speech("Current page", speech_start))
        keyboard.key(0xffc3)  # F6; remove the breadcrumb.
        wait("removed breadcrumb remains accessible", lambda: named(root, HOME) is None)
        assert named(root, CURRENT) is None
        print("A11Y breadcrumb: labels, link roles, current page announcement, inert actions, assistive and keyboard focus, activation and removal passed", flush=True)
        keyboard.key(0xffc2)  # F5; show CSS semantic controls in the real terminal.
        css_action = wait("CSS control absent", lambda: named(root, "Independent App action"))
        assert css_action.get_role() == Atspi.Role.TOGGLE_BUTTON
        assert css_action.get_description() == "CSS description delivered"
        assert not state(css_action, Atspi.StateType.PRESSED)
        assert state(named(root, "CSS expanded control"), Atspi.StateType.EXPANDED)
        assert named(root, "Must stay hidden from reader") is None
        assert named(root, "Reader-only nested detail") is not None
        assert named(root, "CSS restored reader text") is not None
        assert named(root, "CSS direct label").get_role() == Atspi.Role.HEADING
        option = named(root, "CSS selected option")
        assert state(option, Atspi.StateType.SELECTED)
        assert state(option, Atspi.StateType.CHECKED)
        assert css_action.get_component_iface().grab_focus()
        wait("CSS tabindex did not permit assistive focus", lambda: state(css_action, Atspi.StateType.FOCUSED))
        wait("Orca did not announce CSS reference label", lambda: speech("Independent App action"))
        # Orca 50 suppresses announcements from the same App within 100 ms.
        # Separate the initial live-region announcement from the user action.
        ready = time.monotonic() + 0.15
        wait("reader action pacing failed", lambda: time.monotonic() >= ready)
        click(css_action)
        wait("CSS action was not delivered", lambda: named(root, "Background calls 1"))
        wait("CSS pressed update did not reach AT-SPI", lambda: state(css_action, Atspi.StateType.PRESSED))
        wait("Orca did not announce CSS pressed update", lambda: speech("'pressed'"))
        wait("Orca did not announce CSS live content", lambda: speech("Background calls 1"))
        print("A11Y CSS: reference labels, description, role, focus, pressed update, live announcement and hidden state passed", flush=True)
        keyboard.key(0xffc0)  # F3; show retained checkbox controls.
        checkbox = wait("checkbox label absent", lambda: named(root, "Notification preference"))
        assert checkbox.get_role() == Atspi.Role.CHECK_BOX, "checkbox semantic role absent"
        assert state(checkbox, Atspi.StateType.INDETERMINATE)
        locked_checkbox = named(root, "Locked preference")
        assert locked_checkbox is not None
        assert locked_checkbox.get_role() == Atspi.Role.CHECK_BOX
        assert state(locked_checkbox, Atspi.StateType.CHECKED)
        assert not state(locked_checkbox, Atspi.StateType.ENABLED)
        action = locked_checkbox.get_action_iface()
        assert action is None or action.get_n_actions() == 0
        assert checkbox.get_component_iface().grab_focus()
        wait("checkbox focus absent", lambda: state(checkbox, Atspi.StateType.FOCUSED))
        wait("Orca did not announce checkbox label", lambda: speech("Notification preference"))
        wait("Orca did not announce mixed checkbox", lambda: speech("partially checked"))
        speech_start = len((run / "orca.log").read_text().splitlines())
        click(checkbox)
        wait("assistive checkbox toggle absent", lambda: state(checkbox, Atspi.StateType.CHECKED))
        assert not state(checkbox, Atspi.StateType.INDETERMINATE)
        wait("Orca did not announce checkbox checked", lambda: speech("'checked'", speech_start))
        speech_start = len((run / "orca.log").read_text().splitlines())
        keyboard.key(0x20)
        wait("keyboard checkbox toggle absent", lambda: not state(checkbox, Atspi.StateType.CHECKED))
        wait("Orca did not announce checkbox unchecked", lambda: speech("not checked", speech_start))
        print("A11Y checkbox: role, distinct label, mixed/checked/unchecked speech, focus, assistive/keyboard toggles and disabled actions passed", flush=True)
        slider = wait("slider label absent", lambda: named(root, "Playback volume"))
        assert slider.get_role() == Atspi.Role.SLIDER, "slider semantic role absent"
        numeric = slider.get_value_iface()
        assert numeric is not None
        assert numeric.get_minimum_value() == 0.0 and numeric.get_maximum_value() == 100.0
        assert numeric.get_current_value() == 25.0
        assert slider.get_component_iface().grab_focus()
        wait("slider focus absent", lambda: state(slider, Atspi.StateType.FOCUSED))
        wait("Orca did not announce slider label", lambda: speech("Playback volume"))
        speech_start = len((run / "orca.log").read_text().splitlines())
        keyboard.key(0xff53)  # Right increases by the authored step.
        wait("slider value did not update", lambda: numeric.get_current_value() == 30.0)
        wait("Orca did not announce changed slider value", lambda: speech("30", speech_start))
        keyboard.key(0xff57)  # End reaches the upper bound.
        wait("slider maximum not published", lambda: numeric.get_current_value() == 100.0)
        print("A11Y slider: label, role, focus, numeric bounds, keyboard step and changed value speech passed", flush=True)
        keyboard.key(0xffbf)  # F2; retained radio group.
        group = wait("radio group absent", lambda: named(root, "Delivery method"))
        assert group.get_role() == Atspi.Role.PANEL, "radio group semantic role absent"
        standard = wait("radio option label absent", lambda: named(root, "Standard delivery"))
        express = named(root, "Express delivery")
        unavailable = named(root, "Unavailable delivery")
        assert express is not None and unavailable is not None
        assert all(node.get_role() == Atspi.Role.RADIO_BUTTON for node in [standard, express, unavailable])
        assert state(standard, Atspi.StateType.CHECKED)
        assert not state(express, Atspi.StateType.CHECKED)
        assert not state(unavailable, Atspi.StateType.ENABLED)
        action = unavailable.get_action_iface()
        assert action is None or action.get_n_actions() == 0
        assert standard.get_component_iface().grab_focus()
        wait("radio focus absent", lambda: state(standard, Atspi.StateType.FOCUSED))
        wait("Orca did not announce radio label", lambda: speech("Standard delivery"))
        speech_start = len((run / "orca.log").read_text().splitlines())
        keyboard.key(0xff54)  # Down skips the disabled option.
        wait("radio keyboard focus absent", lambda: state(express, Atspi.StateType.FOCUSED))
        wait("Orca did not announce radio focus change", lambda: speech("Express delivery", speech_start))
        keyboard.key(0x20)
        wait("radio keyboard selection absent", lambda: state(express, Atspi.StateType.CHECKED))
        wait("Orca did not announce radio selection", lambda: speech("'selected'", speech_start))
        assert not state(standard, Atspi.StateType.CHECKED)
        click(standard)
        wait("radio assistive selection absent", lambda: state(standard, Atspi.StateType.CHECKED))
        assert not state(express, Atspi.StateType.CHECKED)
        print("A11Y radio: group/option roles, labels, focus, disabled skipping, exclusive keyboard and assistive selection passed", flush=True)
        keyboard.key(0xffbe)  # F1; independently keyed named radios.
        alpha = wait("named radio alpha absent", lambda: named(root, "Named alpha"))
        beta = named(root, "Named beta")
        assert beta is not None
        assert alpha.get_role() == Atspi.Role.RADIO_BUTTON, "named radio role absent"
        assert beta.get_role() == Atspi.Role.RADIO_BUTTON
        assert state(alpha, Atspi.StateType.CHECKED)
        assert not state(beta, Atspi.StateType.CHECKED)
        assert beta.get_component_iface().grab_focus()
        wait("named radio focus absent", lambda: state(beta, Atspi.StateType.FOCUSED))
        wait("Orca did not announce named radio", lambda: speech("Named beta"))
        click(beta)
        wait("named radio selection absent", lambda: state(beta, Atspi.StateType.CHECKED))
        assert not state(alpha, Atspi.StateType.CHECKED)
        print("A11Y named radios: roles, labels, focus and exclusive group selection passed", flush=True)
        keyboard.key(0xffc4)  # F7; single-choice dropdown.
        combo = wait("select absent", lambda: named(root, "Shipping speed"))
        assert combo.get_role() == Atspi.Role.COMBO_BOX, "select combo role absent"
        assert not state(combo, Atspi.StateType.EXPANDED)
        assert combo.get_component_iface().grab_focus()
        click(combo)
        wait("select did not expand", lambda: state(combo, Atspi.StateType.EXPANDED))
        click(combo)
        wait("expanded combo assistive activation did not close it", lambda: not state(combo, Atspi.StateType.EXPANDED))
        click(combo)
        wait("select did not reopen after assistive collapse", lambda: state(combo, Atspi.StateType.EXPANDED))
        standard = wait("select option absent", lambda: named(root, "Standard shipping"))
        express = named(root, "Express shipping")
        unavailable = named(root, "Unavailable shipping")
        assert standard.get_role() == Atspi.Role.LIST_ITEM
        assert state(standard, Atspi.StateType.SELECTED)
        assert not state(express, Atspi.StateType.SELECTED)
        assert not state(unavailable, Atspi.StateType.ENABLED)
        unavailable_actions = unavailable.get_action_iface()
        assert unavailable_actions is None or unavailable_actions.get_n_actions() == 0
        assert express.get_component_iface().grab_focus()
        wait("select option focus absent", lambda: state(express, Atspi.StateType.FOCUSED))
        wait("Orca did not announce select option", lambda: speech("Express shipping"))
        click(express)
        wait("select did not close after assistive choice", lambda: not state(combo, Atspi.StateType.EXPANDED))
        click(combo)
        express = wait("selected option absent after reopen", lambda: named(root, "Express shipping"))
        assert state(express, Atspi.StateType.SELECTED)
        standard = named(root, "Standard shipping")
        speech_start = len((run / "orca.log").read_text().splitlines())
        keyboard.key(0xff52)  # Up skips the disabled option.
        wait("select keyboard focus absent", lambda: state(standard, Atspi.StateType.FOCUSED))
        wait("Orca did not announce select keyboard navigation", lambda: speech("Standard shipping", speech_start))
        keyboard.key(0xff0d)
        wait("select keyboard choice did not close", lambda: not state(combo, Atspi.StateType.EXPANDED))
        keyboard.key(0xffc4)  # F7; multiple-choice dropdown.
        multiple = wait("multiple select absent", lambda: named(root, "Notice channels"))
        assert multiple.get_role() == Atspi.Role.COMBO_BOX
        assert state(multiple, Atspi.StateType.MULTISELECTABLE)
        click(multiple)
        email = wait("multiple select option absent", lambda: named(root, "Email notices"))
        phone = named(root, "Phone notices")
        wait("Orca did not announce multiple option focus", lambda: speech("Email notices"))
        speech_start = len((run / "orca.log").read_text().splitlines())
        keyboard.key(0x20)
        wait("multiple select first choice absent", lambda: state(email, Atspi.StateType.SELECTED))
        wait("Orca did not announce multiple selection", lambda: speech("'selected'", speech_start))
        click(phone)
        wait("multiple select second choice absent", lambda: state(phone, Atspi.StateType.SELECTED))
        assert state(email, Atspi.StateType.SELECTED)
        assert state(multiple, Atspi.StateType.EXPANDED)
        click(email)
        wait("multiple select deselection absent", lambda: not state(email, Atspi.StateType.SELECTED))
        assert state(phone, Atspi.StateType.SELECTED)
        keyboard.key(0xff1b)
        wait("multiple select Escape did not close", lambda: not state(multiple, Atspi.StateType.EXPANDED))
        print("A11Y select: combo/list/option roles, labels, expansion, focus, disabled skipping, single/multiple assistive and keyboard selection with selection speech passed", flush=True)
        from orca_menus import exercise as exercise_menus
        exercise_menus(root, keyboard, wait, named, state, click, speech, Atspi,
                       lambda: len((run / "orca.log").read_text().splitlines()))
        assert named(root, "Notification preference") is None, "removed checkbox remains accessible"
        background = wait("independent App did not attach", background_window)
        background_action = named(background, "Independent App action")
        assert background_action is not None
        click(background_action)
        wait("independent App action was not delivered", lambda: named(background, "Background calls 1"))
        keyboard.key(0xffc6)  # F9; close the first App and construct a second.
        second = wait("second App did not attach", lambda: window(2))
        wait("first App adapter remains attached", lambda: window(1) is None)
        assert background_window() is not None, "closing one App detached the other App"
        click(background_action)
        wait("surviving App action was not delivered", lambda: named(background, "Background calls 2"))
        fresh = named(second, ACCOUNT)
        assert fresh is not None and not state(fresh, Atspi.StateType.EXPANDED)
        keyboard.key(0xff53)
        wait("second App focus did not attach", lambda: state(fresh, Atspi.StateType.FOCUSED))
        click(fresh)
        wait("second App action failed", lambda: state(fresh, Atspi.StateType.EXPANDED))
        keyboard.key(0xffc6)
        terminal.wait(timeout=5)
        assert terminal.returncode == 0
        wait("closed App adapter remains attached", lambda: application() is None)
        reports = [json.loads(line) for line in (run / "callbacks.jsonl").read_text().splitlines()]
        expected = [[("account", True), ("privacy", True), ("privacy", False), ("account", True)],
                    [("account", True)]]
        assert len(reports) == 3
        assert reports[2] == {"background_calls": 2}, reports
        for index, (report, changes) in enumerate(zip(reports[:2], expected), 1):
            assert report["round"] == index
            assert report["css_calls"] == (1 if index == 1 else 0), reports
            assert report["menu_calls"] == (["1:checked:true", "1:document", "2:checked:true", "2:document", "3:document", "4:confirmed:notify,document"] if index == 1 else []), reports
            assert [(event["section_id"], event["expanded"]) for event in report["changes"]] == changes, reports
            expected_navigation = [("home", "/", "Home"), ("docs", "/docs", "Docs")] if index == 1 else []
            assert [(event["segment_id"], event["path"], event["label"])
                    for event in report["navigation"]] == expected_navigation, reports
        assert orca.poll() is None, "Orca exited before the workflow completed"
        print("A11Y round 2: simultaneous and successive App isolation, action delivery, exactly-once callbacks and adapter cleanup passed", flush=True)
        for line in (run / "orca.log").read_text().splitlines():
            if "SPEECH OUTPUT:" in line and any(text in line for text in [ACCOUNT, PRIVACY, ENTRY, HOME, DOCS, CURRENT, "Notification preference", "Standard delivery", "Express delivery", "Named beta", "radio button", "'selected'", "Playback volume", "partially checked", "not checked", "Menu file", "Enable notifications", "Recent menu", "Recent document", "check menu item checked", "'checked'", "Current page", "collapsed button", "'expanded'", "Independent App action", "Background calls 1", "'pressed'"]):
                print("A11Y " + line, flush=True)
    finally:
        if keyboard:
            keyboard.close()
        for child in reversed(children):
            if child.poll() is None:
                child.terminate()
                try:
                    child.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=2)
        for log in logs:
            log.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    parser.add_argument("--geometry", default="60x16")
    parser.add_argument("--inside")
    parser.add_argument("--catalog", choices=["tables", "tabs", "overlays", "display", "dialogs"])
    parser.add_argument("--negative", action="store_true")
    parser.add_argument("--negative-css", action="store_true")
    arguments = parser.parse_args()
    if arguments.inside:
        inside(arguments)
    else:
        isolated(arguments)
