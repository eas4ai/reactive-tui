"""Real overlay role, text, focus restoration and dismissal checks."""
import time


def exercise(root, keyboard, wait, named, state, click, speech, Atspi, speech_start):
    launch = wait("Modal launcher absent", lambda: named(root, "Open details"))
    assert named(root, "Project details") is None
    start = speech_start()
    click(launch)
    dialog = wait("Modal dialog absent", lambda: named(root, "Project details"))
    assert dialog.get_role() == Atspi.Role.DIALOG, dialog.get_role_name()
    wait("Orca did not announce the dialog", lambda: speech("Project details", start) and speech("dialog", start))
    entry = wait("Dialog input absent", lambda: named(root, "Dialog name"))
    start = speech_start()
    assert entry.get_component_iface().grab_focus()
    wait("Dialog entry focus absent", lambda: state(entry, Atspi.StateType.FOCUSED))
    wait("Orca did not read dialog input", lambda: speech("Dialog name", start) and speech("draft", start))
    launch.get_component_iface().grab_focus()
    time.sleep(.2)
    assert state(entry, Atspi.StateType.FOCUSED), "Modal focus trap let assistive focus escape"
    keyboard.key(0xff57)
    keyboard.key(ord('x'))
    wait("Dialog editing failed", lambda: Atspi.Text.get_text(entry, 0, -1) == "draftx")
    start = speech_start()
    keyboard.key(0xff1b)
    wait("Closed dialog remains exposed", lambda: named(root, "Dialog name") is None)
    wait("Modal did not restore launcher focus", lambda: state(launch, Atspi.StateType.FOCUSED))
    wait("Orca did not announce restored focus", lambda: speech("Open details", start))
    click(launch)
    close = wait("Modal Close action absent", lambda: named(root, "Close"))
    assert close.get_role() == Atspi.Role.PUSH_BUTTON
    click(close)
    wait("Assistive Close failed", lambda: named(root, "Dialog name") is None)
    print("A11Y Modal: dialog speech, trapped assistive focus, editing, restoration and close passed", flush=True)

    keyboard.key(0xffc9)
    trigger = wait("Popover trigger absent", lambda: named(root, "Show help"))
    assert named(root, "Help search") is None
    start = speech_start()
    click(trigger)
    entry = wait("Popover content absent", lambda: named(root, "Help search"))
    wait("Popover autofocus absent", lambda: state(entry, Atspi.StateType.FOCUSED))
    wait("Orca did not read popover input", lambda: speech("Help search", start) and speech("topic", start))
    keyboard.key(0xff57)
    keyboard.key(ord('x'))
    wait("Popover editing failed", lambda: Atspi.Text.get_text(entry, 0, -1) == "topicx")
    start = speech_start()
    keyboard.key(0xff1b)
    wait("Dismissed popover remains exposed", lambda: named(root, "Help search") is None)
    wait("Popover did not restore trigger focus", lambda: state(trigger, Atspi.StateType.FOCUSED))
    wait("Orca did not announce popover restoration", lambda: speech("Show help", start))
    print("A11Y Popover: input speech, editing, Escape removal and focus restoration passed", flush=True)
