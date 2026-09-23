#!/usr/bin/env python3
"""Exercise terminal setup, resizing, normal exit, and signal cleanup in a PTY."""

import argparse
import fcntl
import os
import pathlib
import pty
import select
import signal
import struct
import subprocess
import termios
import time


def resize(fd: int, columns: int, rows: int) -> None:
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, columns, 0, 0))


def drain(fd: int, seconds: float) -> bytes:
    deadline = time.monotonic() + seconds
    chunks = []
    while time.monotonic() < deadline:
        readable, _, _ = select.select([fd], [], [], min(0.05, deadline - time.monotonic()))
        if not readable:
            continue
        try:
            chunks.append(os.read(fd, 65536))
        except OSError:
            break
    return b"".join(chunks)


def run_case(binary: pathlib.Path, stop_with_signal: bool) -> bytes:
    master, slave = pty.openpty()
    before = termios.tcgetattr(slave)
    resize(slave, 80, 24)
    process = subprocess.Popen(
        [
            str(binary),
            "--fps",
            "12",
            "--interval",
            "1",
            "--seed",
            "42",
            "--ascii",
        ],
        stdin=slave,
        stdout=slave,
        stderr=slave,
        close_fds=True,
        env={**os.environ, "TERM": "xterm-256color"},
    )
    output = drain(master, 0.3)
    resize(slave, 100, 30)
    os.kill(process.pid, signal.SIGWINCH)
    output += drain(master, 0.2)
    if stop_with_signal:
        os.kill(process.pid, signal.SIGTERM)
    else:
        os.write(master, b"q")
    output += drain(master, 0.4)
    process.wait(timeout=3)
    after = termios.tcgetattr(slave)
    os.close(master)
    os.close(slave)

    assert process.returncode == 0, (process.returncode, output[-2000:])
    assert before == after, "terminal attributes were not restored"
    for sequence in (b"\x1b[?1049h", b"\x1b[?2026h", b"\x1b[?1049l", b"\x1b[?25h"):
        assert sequence in output, f"missing terminal sequence {sequence!r}"
    return output


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", nargs="?", default="target/debug/term-screensaver")
    args = parser.parse_args()
    binary = pathlib.Path(args.binary).resolve()
    if not binary.is_file():
        raise SystemExit(f"binary not found: {binary}; run cargo build first")

    normal = run_case(binary, stop_with_signal=False)
    signaled = run_case(binary, stop_with_signal=True)
    assert b"\x1b[2J" in normal
    assert b"\x1b[2J" in signaled
    print("PTY smoke test passed: normal exit, resize, and SIGTERM cleanup")


if __name__ == "__main__":
    main()
