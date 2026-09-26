"""ASCII screen reconstruction for the existing real-PTY acceptance probes."""
import re

SYNC_END = b"\x1b[?2026l"


def completed(data):
    """Ignore an unfinished frame, including partial escape sequences."""
    end = data.rfind(SYNC_END)
    return bytes(data[:end + len(SYNC_END)]) if end >= 0 else b""


# The renderer emits sparse cells, so stripping ANSI cannot recover a screen:
# unchanged spaces and letters are deliberately absent. This ASCII-only probe
# follows cursor positioning; Rust tests independently check Unicode via vt100.
def screen_text(data):
    cells = {}
    x = y = 0
    tokens = re.finditer(rb"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|[^\x1b]", bytes(data))
    for match in tokens:
        token = match.group()
        if token.startswith(b"\x1b["):
            params, command = token[2:-1], token[-1:]
            if command in (b"H", b"f"):
                numbers = params.split(b";")
                y = int(numbers[0] or b"1") - 1
                x = int(numbers[1] or b"1") - 1 if len(numbers) > 1 else 0
            elif command == b"J":
                if params in (b"2",b"3"):
                    cells.clear()
                elif params in (b"",b"0"):
                    cells = {p:c for p,c in cells.items() if p < (y,x)}
            continue
        if token.startswith(b"\x1b]"):
            continue
        if token == b"\r": x = 0
        elif token == b"\n": y += 1
        elif token == b"\b": x = max(0,x-1)
        elif len(token) == 1 and 32 <= token[0] < 127:
            if 0 <= x < 200 and 0 <= y < 100: cells[y,x] = chr(token[0])
            x += 1
    return "\n".join("".join(cells.get((row,col)," ") for col in range(200)) for row in range(100))
