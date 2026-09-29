"""Exercise the real TUI through a pseudo-terminal on Unix CI runners.

This checks terminal escape output and key handling under common TERM profiles.
It does not claim to emulate GNOME Terminal, Terminal.app or Windows Terminal.
"""

import fcntl
import os
import pty
import re
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time
from pathlib import Path


def read_until(fd: int, expected: bytes, timeout: float = 8.0, raw: bool = False) -> bytes:
    data = bytearray()
    deadline = time.monotonic() + timeout
    while expected not in (data if raw else re.sub(rb"\x1b\[[0-?]*[ -/]*[@-~]", b"", data)):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise AssertionError(f"Missing {expected!r}; output head: {bytes(data[:800])!r}")
        readable, _, _ = select.select([fd], [], [], remaining)
        if readable:
            try:
                chunk = os.read(fd, 65536)
            except OSError as error:
                raise AssertionError(f"Terminal closed before {expected!r}: {error}; tail: {bytes(data[-500:])!r}") from error
            if not chunk:
                raise AssertionError(f"Terminal closed before {expected!r}")
            data.extend(chunk)
    return bytes(data)


def check_profile(binary: Path, profile: str, root: Path) -> None:
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 100, 0, 0))
    with tempfile.TemporaryDirectory(prefix="forge-tui-") as progress_dir:
        env = os.environ.copy()
        env.update(
            TERM=profile,
            FORGE_HOME=str(root),
            FORGE_PROGRESS_DIR=progress_dir,
        )
        def make_controlling_terminal() -> None:
            os.setsid()
            fcntl.ioctl(slave, termios.TIOCSCTTY, 0)

        process = subprocess.Popen(
            [str(binary), "tui"],
            stdin=slave,
            stdout=slave,
            stderr=slave,
            env=env,
            preexec_fn=make_controlling_terminal,
        )
        os.close(slave)
        try:
            read_until(master, b"Choisis ton parcours")
            time.sleep(0.2)
            os.write(master, b"h")
            read_until(master, b"Accueil :")
            os.write(master, b"\x1b")
            read_until(master, b"Choisis ton parcours")
            os.write(master, b"\r")
            read_until(master, b"D\xc3\xa9tails de l'exercice")
            os.write(master, b"\r")
            read_until(master, b"Diagnostics")
            os.write(master, b"\x11")  # Ctrl+Q, available in every screen.
            tail = read_until(master, b"\x1b[?1049l", raw=True)
            process.wait(timeout=5)
            assert process.returncode == 0, (profile, process.returncode)
            assert b"\x1b[?25h" in tail, "cursor was not restored"
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)
            os.close(master)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: python3 tests/tui_pty.py /path/to/forge")
    binary = Path(sys.argv[1]).resolve()
    root = Path(__file__).resolve().parents[1]
    for profile in ("xterm", "xterm-256color", "screen-256color"):
        check_profile(binary, profile, root)
        print(f"PTY {profile}: OK")


if __name__ == "__main__":
    main()
