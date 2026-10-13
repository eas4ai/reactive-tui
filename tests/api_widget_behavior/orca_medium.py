"""NAV-011: what Orca says for the medium widgets (a resizable split, a
sidebar, a list and a log view) as the user moves through them, and for a
row appended to the log while the focus is elsewhere. Each check reads only
the speech logged after its own step. Every step runs even when an earlier
one failed, so one run names all the violations; the run then fails with
all of them in one AssertionError.
"""

import time

TAB, RIGHT, DOWN, END, ENTER = 0xff09, 0xff53, 0xff54, 0xff57, 0xff0d
# Orca drops an announcement that follows another from the same App within
# 100 ms; keep each user action clear of the last announcement.
PAUSE = 0.3


def exercise(root, keyboard, wait, named, state, click, speech, Atspi, speech_start):
    failures = []

    def idle(seconds):
        # Keep the AT-SPI main loop turning while the reader speaks.
        ready = time.monotonic() + seconds
        wait("reader pacing failed", lambda: time.monotonic() >= ready, seconds + 2)

    def heard(step, start, *phrases, timeout=3):
        """Every phrase is spoken after `start`; each missing one is recorded."""
        try:
            wait(step, lambda: all(speech(phrase, start) for phrase in phrases), timeout)
        except AssertionError:
            missing = [phrase for phrase in phrases if not speech(phrase, start)]
            failures.append(f"{step}: Orca did not say {', '.join(map(repr, missing))}")

    def press(key):
        start = speech_start()
        keyboard.key(key)
        return start

    panes = wait("Panes divider absent", lambda: named(root, "Panes"))
    wait("Main sidebar absent", lambda: named(root, "Main"))
    fruits = wait("Fruits list absent", lambda: named(root, "Fruits"))
    log = wait("Build log absent", lambda: named(root, "Build log"))
    wait("Panes focus absent", lambda: state(panes, Atspi.StateType.FOCUSED))

    # The page opens with the focus on the divider: its speech is all that
    # came before this point.
    heard("Panes given the focus", 0, "Panes")
    idle(PAUSE)
    heard("Right on Panes", press(RIGHT), "41")
    idle(PAUSE)

    heard("Tab to Main", press(TAB), "Home")
    idle(PAUSE)
    heard("Down onto Projects", press(DOWN), "Projects", "collapsed")
    idle(PAUSE)
    heard("Right on Projects", press(RIGHT), "expanded")
    idle(PAUSE)

    # The sidebar's toggle is the next stop; the list is the one after it.
    press(TAB)
    idle(PAUSE)
    heard("Tab to Fruits", press(TAB), "Item 1")
    idle(PAUSE)
    heard("End on Fruits", press(END), "Item 70000")
    idle(PAUSE)

    # The probe appends a row to the log when the list confirms an item; the
    # log announces it politely, and the focus stays on the list.
    heard("A row appended to Build log", press(ENTER), "Compiling widgets", timeout=5)
    last = named(root, "Item 70000")
    if state(log, Atspi.StateType.FOCUSED) or not (
            state(fruits, Atspi.StateType.FOCUSED) or (last and state(last, Atspi.StateType.FOCUSED))):
        failures.append("A row appended to Build log: the focus left the list")

    if failures:
        raise AssertionError("; ".join(failures))
    print("A11Y medium widgets: divider name and value, sidebar items and group state, list options "
          "and the log's polite row passed", flush=True)
