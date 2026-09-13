#!/usr/bin/env python3
"""Time the NORMAL executable through a PTY for the complete 0–77s journey.

Counts completed Kitty protocol transmissions and decodes only the existing
timestamp counter. This is not a Kitty display/compositor FPS measurement or
visual approval. No screenshots/assets, reference updates, or system tuning.
"""
import base64
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pty
import re
import select
import struct
import subprocess
import termios
import time

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/x86_64-unknown-linux-gnu/release/termdemo"
MARKER = b"\x1b[?2026l"
PACKET = re.compile(rb"\x1b_G[^;]*;([A-Za-z0-9+/=]+)\x1b\\")
DIGITS = [0b111101101101111, 0b010110010010111, 0b111001111100111,
          0b111001111001111, 0b101101111001001, 0b111100111001111,
          0b111100111101111, 0b111001010010010, 0b111101111101111,
          0b111101111001111]


def counter(frame):
    rgb = bytearray()
    for packet in PACKET.finditer(frame):
        rgb.extend(base64.b64decode(packet[1], validate=True))
        if len(rgb) >= 19 * 320 * 3:
            break
    assert len(rgb) >= 19 * 320 * 3, "Truncated frame counter"
    digits = []
    for left in (7, 15, 23, 34):
        bits = 0
        for row in range(5):
            for column in range(3):
                offset = ((7 + 2 * row) * 320 + left + 2 * column) * 3
                bits = (bits << 1) | int(rgb[offset] > 200)
        assert bits in DIGITS, f"Invalid timestamp digit {bits:015b}"
        digits.append(DIGITS.index(bits))
    return digits[0] * 100 + digits[1] * 10 + digits[2] + digits[3] / 10


def main():
    master, slave = pty.openpty()
    before = termios.tcgetattr(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 25, 80, 0, 0))
    process = subprocess.Popen([str(BINARY)], stdin=slave, stdout=slave, stderr=slave,
                               close_fds=True)
    frames, pending, tail = [], bytearray(), bytearray()
    started = time.monotonic()
    quit_sent = False
    try:
        while time.monotonic() - started < 120:
            if select.select([master], [], [], .05)[0]:
                chunk = os.read(master, 65536)
                pending.extend(chunk)
                tail.extend(chunk)
                del tail[:-1024]
                while (end := pending.find(MARKER)) >= 0:
                    end += len(MARKER)
                    stamp = counter(pending[:end])
                    frames.append({"wallSeconds": time.monotonic() - started,
                                   "animationSeconds": stamp})
                    del pending[:end]
                    if stamp == 77 and not quit_sent:
                        os.write(master, b"q")
                        quit_sent = True
            # Drain cleanup bytes even if exit races the reader's last chunk.
            if process.poll() is not None and not select.select([master], [], [], 0)[0]:
                break
        else:
            raise AssertionError("Playback did not reach its 77s endpoint within 120s")
        assert process.wait(timeout=5) == 0, "Normal playback failed"
        assert frames and frames[-1]["animationSeconds"] == 77, "Endpoint not reached"
        assert all(b["animationSeconds"] >= a["animationSeconds"]
                   for a, b in zip(frames, frames[1:])), "Counter reversed"
        assert termios.tcgetattr(slave) == before, "Terminal mode was not restored"
        assert b"\x1b[?1049l" in tail, "Alternate screen was not restored"
    finally:
        if process.poll() is None:
            os.write(master, b"q")
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        os.close(master)
        os.close(slave)
    gaps = sorted((b["wallSeconds"] - a["wallSeconds"]) * 1000
                  for a, b in zip(frames, frames[1:]))
    duration = frames[-1]["wallSeconds"] - frames[0]["wallSeconds"]
    summary = {"frames": len(frames), "transmissionSeconds": duration,
               "averageTransmissionsPerSecond": (len(frames) - 1) / duration,
               "p95TransmissionGapMs": gaps[int(len(gaps) * .95)],
               "maxTransmissionGapMs": gaps[-1], "finalCounter": 77}
    report = {"normalSha256": hashlib.sha256(BINARY.read_bytes()).hexdigest(),
              "note": __doc__.strip(), "summary": summary, "frames": frames}
    output = ROOT / "target/phase3-audit/playback-profile.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(summary, indent=2))
    print(f"Profile saved to {output.relative_to(ROOT)}.")


if __name__ == "__main__":
    main()
