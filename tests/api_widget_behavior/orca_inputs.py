"""CTL-011: what Orca says for the input pieces (two number inputs, a switch,
a rating and a masked code input) as the user moves through them and types.

Each check reads only the speech logged after its own step. A phrase that
must not be spoken is judged only after the reader has had SETTLE seconds to
speak. Every step runs even when an earlier one failed, so one run names all
the violations; the run then fails with all of them in one AssertionError.
"""
import time

BACKSPACE, TAB, SPACE, RIGHT = 0xff08, 0xff09, 0x20, 0xff53
# Keypad Enter: Orca's Where Am I in its desktop layout.
WHERE_AM_I = 0xff8d
DIGITS = "0123456789"
# Orca logs each utterance in quotes, so a quote before a digit is a spoken
# number; the voice settings after it and the background App's "Background
# calls 0" never match.
SPOKEN_DIGITS = tuple(f"'{digit}" for digit in DIGITS)
# Seconds the reader gets to speak before a phrase is judged unspoken.
SETTLE = 1.5
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

    def unheard(step, start, *phrases):
        """None of the phrases is spoken after `start`, judged after SETTLE."""
        idle(SETTLE)
        spoken = [phrase for phrase in phrases if speech(phrase, start)]
        if spoken:
            failures.append(f"{step}: Orca said {', '.join(map(repr, spoken))}")

    def echoed(start, phrase):
        # Pacing only: wait for the reader's echo of a key, never a verdict.
        try:
            wait("key echo", lambda: speech(phrase, start), 2)
        except AssertionError:
            pass

    def type_digit(digit):
        start = speech_start()
        keyboard.key(ord(digit))
        echoed(start, f"'{digit}'")
        idle(PAUSE)

    def focus_moves(step, node):
        start = speech_start()
        keyboard.key(TAB)
        wait(f"{step}: focus did not arrive", lambda: state(node, Atspi.StateType.FOCUSED))
        return start

    volume = wait("Volume number input absent", lambda: named(root, "Volume"))
    year = wait("Year number input absent", lambda: named(root, "Year"))
    wifi = wait("Wi-Fi switch absent", lambda: named(root, "Wi-Fi"))
    stars = wait("Stars rating absent", lambda: named(root, "Stars"))
    wait("Volume focus absent", lambda: state(volume, Atspi.StateType.FOCUSED))
    # The page opens with the focus on Volume: its speech is all that came
    # before this point. Orca reads the value as its own utterance ('0.').
    heard("Volume given the focus", 0, "Volume", "spin button", "'0")
    idle(PAUSE)

    start = speech_start()
    keyboard.key(BACKSPACE)
    echoed(start, "backspace")
    unheard("Volume emptied", start, *SPOKEN_DIGITS)

    start = speech_start()
    type_digit("4")
    type_digit("2")
    heard("4 then 2 typed into Volume (0 to 10)", start, "42", "invalid entry")
    idle(PAUSE)
    # Emptied after the invalid 42, the box reads as empty. Emptying 4 sends
    # the reader no number to speak, so Orca's Where Am I asks for what the
    # box holds: its name, its role and blank, and no number, neither the 42
    # it showed nor the 4 the application still holds (the developer's
    # ruling on escalation 3b71c4b1).
    for _ in range(2):
        start = speech_start()
        keyboard.key(BACKSPACE)
        echoed(start, "backspace")
        idle(PAUSE)
    start = speech_start()
    keyboard.key(WHERE_AM_I)
    heard("Where Am I on Volume emptied after 42", start, "Volume", "spin button", "blank")
    unheard("Where Am I on Volume emptied after 42", start, *SPOKEN_DIGITS)
    idle(PAUSE)

    start = focus_moves("Tab to Year", year)
    echoed(start, "Year")
    idle(PAUSE)
    for _ in range(4):
        keyboard.key(BACKSPACE)
        idle(0.25)
    idle(SETTLE)

    start = speech_start()
    for digit in "2025":
        type_digit(digit)
    heard("2025 typed into Year (1900 to 2100)", start, "2025")
    unheard("2025 typed into Year (1900 to 2100)", start, "invalid entry")

    start = focus_moves("Tab to Wi-Fi", wifi)
    heard("Wi-Fi given the focus", start, "Wi-Fi", "not pressed")
    idle(PAUSE)
    start = speech_start()
    keyboard.key(SPACE)
    # The quote keeps "not pressed" from passing for "pressed".
    heard("Space on Wi-Fi", start, "'pressed")
    idle(PAUSE)

    start = focus_moves("Tab to Stars", stars)
    heard("Stars given the focus", start, "Stars", "slider", "3 of 5")
    idle(PAUSE)
    start = speech_start()
    keyboard.key(RIGHT)
    heard("Right on Stars", start, "4 of 5")
    idle(PAUSE)

    start = speech_start()
    keyboard.key(TAB)
    wait("Tab to Code: focus did not leave Stars", lambda: not state(stars, Atspi.StateType.FOCUSED))
    heard("Code given the focus", start, "Code, 6 digits", "password text")
    idle(PAUSE)
    start = speech_start()
    for digit in "12":
        keyboard.key(ord(digit))
        idle(PAUSE)
    unheard("1 and 2 typed into the masked Code", start, "'1", "'2")

    if failures:
        raise AssertionError("; ".join(failures))
    print("A11Y inputs: number input name, role, value, invalid and emptied entry; in-range year "
          "without invalid entry; switch, rating and masked code speech passed", flush=True)
