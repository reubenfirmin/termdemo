#!/usr/bin/env python3
"""Headless regression gate for the accepted pre-planet and orbit path."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import pty
import select
import subprocess
import sys
import termios
import threading
import time


FRAME_BYTES = 320 * 200 * 3
FRAME_TIMES = (
    "0.5",
    "4.0",
    "7.0",
    "11.0",
    "14.75",
    "20.063",
    "21.0",
    "24.0",
    "26.9",
    "27.0",
)


def fail(message: str) -> None:
    raise RuntimeError(message)


def render_digest(binary: Path, timestamp: str) -> str:
    result = subprocess.run(
        [binary],
        input=timestamp.encode("ascii"),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=15,
        check=False,
    )
    if result.returncode != 0:
        fail(
            f"frame {timestamp}s exited {result.returncode}: "
            f"{result.stderr.decode(errors='replace')}"
        )
    if len(result.stdout) != FRAME_BYTES:
        fail(
            f"frame {timestamp}s returned {len(result.stdout)} bytes; "
            f"expected {FRAME_BYTES}"
        )
    return hashlib.sha256(result.stdout).hexdigest()


def frame_hashes(binary: Path) -> dict[str, str]:
    return {timestamp: render_digest(binary, timestamp) for timestamp in FRAME_TIMES}


def load_baseline(path: Path) -> dict[str, str]:
    baseline: dict[str, str] = {}
    for line_number, raw_line in enumerate(path.read_text().splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        fields = line.split()
        if len(fields) != 2:
            fail(f"{path}:{line_number}: expected '<time> <sha256>'")
        timestamp, digest = fields
        baseline[timestamp] = digest
    return baseline


def compare_frames(binary: Path, baseline_path: Path) -> None:
    expected = load_baseline(baseline_path)
    actual = frame_hashes(binary)
    if set(expected) != set(FRAME_TIMES):
        fail("frame baseline timestamps do not match the protected checkpoint list")
    failures = [
        f"{timestamp}s expected {expected[timestamp]}, got {actual[timestamp]}"
        for timestamp in FRAME_TIMES
        if expected[timestamp] != actual[timestamp]
    ]
    if failures:
        fail("pre-planet frame regression:\n" + "\n".join(failures))
    print(f"phase0 frames ok: {len(FRAME_TIMES)} protected checkpoints")


def record_frames(binary: Path, baseline_path: Path) -> None:
    hashes = frame_hashes(binary)
    lines = [
        "# Protected 320x200 RGB24 frames. Regenerate only after explicit visual approval.",
        *(f"{timestamp} {hashes[timestamp]}" for timestamp in FRAME_TIMES),
        "",
    ]
    baseline_path.write_text("\n".join(lines))
    print(f"recorded {len(FRAME_TIMES)} frame hashes in {baseline_path}")


def check_motion(audit_binary: Path) -> None:
    result = subprocess.run(
        [audit_binary],
        input=b"!",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=15,
        check=False,
    )
    if result.returncode != 0:
        fail(
            f"motion audit exited {result.returncode}: "
            f"{result.stdout.decode(errors='replace')}"
            f"{result.stderr.decode(errors='replace')}"
        )
    output = result.stdout.decode("ascii", errors="replace").strip()
    if not output.startswith("phase0 motion audit ok:"):
        fail(f"unexpected motion audit output: {output!r}")
    print(output)


def check_controls(binary: Path) -> None:
    master, slave = pty.openpty()
    before = termios.tcgetattr(slave)
    captured_start = bytearray()
    captured_end = bytearray()
    entered_screen = threading.Event()
    finished = threading.Event()

    def drain() -> None:
        while not finished.is_set():
            readable, _, _ = select.select([master], [], [], 0.05)
            if not readable:
                continue
            try:
                chunk = os.read(master, 65_536)
            except OSError:
                break
            if not chunk:
                break
            if len(captured_start) < 262_144:
                captured_start.extend(chunk[: 262_144 - len(captured_start)])
            captured_end.extend(chunk)
            if len(captured_end) > 262_144:
                del captured_end[: len(captured_end) - 262_144]
            if b"\x1b[?1049h" in captured_start:
                entered_screen.set()

    process = subprocess.Popen(
        [binary],
        stdin=slave,
        stdout=slave,
        stderr=slave,
        close_fds=True,
    )
    reader = threading.Thread(target=drain, daemon=True)
    reader.start()
    try:
        if not entered_screen.wait(2.0):
            fail("interactive process did not enter the alternate screen")

        os.write(master, b"\x1b[C")
        time.sleep(0.15)
        if process.poll() is not None:
            fail("Right Arrow exited the program")

        os.write(master, b" ")
        time.sleep(0.15)
        if process.poll() is not None:
            fail("Space exited the program")

        os.write(master, b"\x1b")
        try:
            return_code = process.wait(timeout=3.0)
        except subprocess.TimeoutExpired:
            fail("Escape did not exit within three seconds")
        if return_code != 0:
            fail(f"interactive process exited with status {return_code}")
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=1.0)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=1.0)
        finished.set()
        reader.join(timeout=1.0)

    time.sleep(0.02)
    after = termios.tcgetattr(slave)
    if before != after:
        fail("terminal attributes were not restored after Escape")
    output_edges = bytes(captured_start) + bytes(captured_end)
    if b"\x1b[?25h" not in output_edges or b"\x1b[?1049l" not in output_edges:
        fail("cursor or alternate-screen cleanup sequence was not emitted")
    os.close(master)
    os.close(slave)
    print("phase0 controls ok: Right Arrow, Space, Escape, terminal cleanup")


def main() -> int:
    if len(sys.argv) not in (4, 5):
        print(
            "usage: phase0_audit.py NORMAL_BIN AUDIT_BIN BASELINE [--record]",
            file=sys.stderr,
        )
        return 2
    normal_binary = Path(sys.argv[1]).resolve()
    audit_binary = Path(sys.argv[2]).resolve()
    baseline_path = Path(sys.argv[3]).resolve()
    record = len(sys.argv) == 5 and sys.argv[4] == "--record"
    if len(sys.argv) == 5 and not record:
        print(f"unknown option: {sys.argv[4]}", file=sys.stderr)
        return 2

    try:
        if record:
            record_frames(normal_binary, baseline_path)
            return 0
        compare_frames(normal_binary, baseline_path)
        check_motion(audit_binary)
        check_controls(normal_binary)
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"phase0 audit failed: {error}", file=sys.stderr)
        return 1

    print("phase0 audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
