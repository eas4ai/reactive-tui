"""Display values delivered through the platform reader interface."""
import time


def exercise(root, keyboard, wait, named, state, click, speech, Atspi, speech_start):
    bar = wait("Progress bar absent", lambda: named(root, "Download progress"))
    assert bar.get_role() == Atspi.Role.PROGRESS_BAR, bar.get_role_name()
    value = bar.get_value_iface()
    assert value is not None, "Progress numeric range absent"
    assert value.get_minimum_value() == 0
    assert value.get_maximum_value() == 200
    assert value.get_current_value() == 50
    assert not state(bar, Atspi.StateType.FOCUSABLE)
    pending = named(root, "Waiting for download")
    assert pending.get_role() == Atspi.Role.PROGRESS_BAR
    assert state(pending, Atspi.StateType.INDETERMINATE)
    assert pending.get_value_iface() is None
    assert named(root, "Invalid download").get_value_iface() is None
    assert named(root, "Completed download").get_value_iface().get_current_value() == 200
    advance = named(root, "Advance download")
    keyboard.key(0xff53)
    wait("Download action focus absent", lambda: state(advance, Atspi.StateType.FOCUSED))
    wait("Orca did not announce download action", lambda: speech("Advance download"))
    start = speech_start()
    keyboard.reader_next_object(previous=True)
    wait("Orca did not read initial progress", lambda: speech("Download progress", start) and speech("25 percent", start))
    keyboard.reader_next_object()
    wait("Orca did not return to download action", lambda: speech("Advance download", start))
    start = speech_start()
    click(advance)
    wait("Progress value change absent", lambda: value.get_current_value() == 150)
    keyboard.reader_next_object(previous=True)
    wait("Orca did not read progress label", lambda: speech("Download progress", start))
    wait("Orca did not announce progress", lambda: speech("75 percent", start))
    print("A11Y ProgressBar: role, range, changed value and Orca progress speech passed", flush=True)
    keyboard.key(0xffc9)
    chart = wait("Chart absent", lambda: named(root, "Request latency"))
    assert chart.get_role() == Atspi.Role.IMAGE
    wait("Chart autofocus absent", lambda: state(chart, Atspi.StateType.FOCUSED))
    wait("Orca did not read chart label", lambda: speech("Request latency"))
    start = speech_start()
    keyboard.key(0xff50)
    wait("Orca did not read first chart point", lambda: speech("Latency / Low: 2", start))
    # Orca suppresses repeated announcements within its short event interval.
    # Let the reader consume the first point before navigating to the next.
    time.sleep(.3)
    start = speech_start()
    keyboard.key(0xff57)
    wait("Orca did not read last chart point", lambda: speech("Latency / High: 8; unit=ms", start))
    keyboard.key(0xff1b)
    wait("Dismissed chart point remains exposed", lambda: named(root, "Latency / High: 8; unit=ms") is None)
    print("A11Y Chart: image role, title, focus and keyboard point speech passed", flush=True)
    keyboard.key(0xffc9)
    pane = wait("ScrollView absent", lambda: named(root, "Report viewport"))
    assert pane.get_role() == Atspi.Role.SCROLL_PANE, pane.get_role_name()
    wait("ScrollView focus absent", lambda: state(pane, Atspi.StateType.FOCUSED))
    wait("Orca did not read scroll label", lambda: speech("Report viewport"))
    offscreen = named(root, "Report annotation")
    if offscreen is not None:
        component = offscreen.get_component_iface()
        assert component is None or not component.grab_focus(), "Clipped input accepted assistive focus"
    keyboard.key(0xff57)
    entry = wait("Scrolled input absent", lambda: named(root, "Report annotation"))
    start = speech_start()
    assert entry.get_component_iface().grab_focus()
    wait("Scrolled input focus absent", lambda: state(entry, Atspi.StateType.FOCUSED))
    wait("Orca did not read scrolled input", lambda: speech("Report annotation", start) and speech("draft", start))
    keyboard.key(0xff57)
    keyboard.key(ord('x'))
    wait("Scrolled input editing failed", lambda: Atspi.Text.get_text(entry, 0, -1) == "draftx")
    print("A11Y ScrollView: pane role, label, scrolling, child focus, speech and editing passed", flush=True)
    start = speech_start()
    keyboard.key(0xffc9)
    change = wait("Image action absent", lambda: named(root, "Change sample"))
    wait("Image action focus absent", lambda: state(change, Atspi.StateType.FOCUSED))
    # The App's focus state can arrive before Orca processes its focus event.
    wait("Orca did not announce image action", lambda: speech("Change sample", start))
    sample = named(root, "Black square sample")
    assert sample.get_role() == Atspi.Role.IMAGE
    assert sample.get_child_count() == 0, "Image fallback characters leaked to the reader"
    start = speech_start()
    keyboard.reader_next_object()
    wait("Orca did not read image label and role", lambda: speech("Black square sample", start) and speech("image", start))
    keyboard.reader_next_object(previous=True)
    start = speech_start()
    click(change)
    wait("Image props did not update", lambda: named(root, "White square sample"))
    assert named(root, "Black square sample") is None
    # Allow Orca to consume the asynchronous name-change event before navigation.
    time.sleep(.3)
    keyboard.reader_next_object()
    wait("Orca did not read changed image label", lambda: speech("White square sample", start))
    print("A11Y Image: image role, alternative label, hidden fallback cells and changed-prop speech passed", flush=True)
    keyboard.key(0xffc9)
    terminal = wait("Embedded terminal absent", lambda: named(root, "Build terminal"))
    assert terminal.get_role() == Atspi.Role.TERMINAL, terminal.get_role_name()
    wait("Embedded terminal focus absent", lambda: state(terminal, Atspi.StateType.FOCUSED))
    wait("Embedded terminal text absent", lambda: "Ready for command" in Atspi.Text.get_text(terminal, 0, -1))
    keyboard.key(0xff8d)  # Orca Where Am I includes a terminal's title and role.
    wait("Orca did not read embedded terminal title", lambda: speech("Build terminal"))
    start = speech_start()
    keyboard.key(ord('x'))
    keyboard.key(0xff0d)
    wait("Embedded terminal input/output absent", lambda: "Received:x" in Atspi.Text.get_text(terminal, 0, -1))
    wait("Orca did not read embedded output", lambda: speech("Received:x", start))
    # Terminal keys belong to its child. Move focus outside before App shutdown.
    leave = named(root, "Outside terminal")
    assert leave.get_component_iface().grab_focus()
    wait("Could not leave embedded terminal", lambda: state(leave, Atspi.StateType.FOCUSED))
    print("A11Y Terminal: role, title, focus, readable screen and real child input/output speech passed", flush=True)
