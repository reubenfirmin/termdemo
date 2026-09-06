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
    "1.0",
    "2.0",
    "3.0",
    "4.0",
    "5.0",
    "6.0",
    "7.0",
    "8.0",
    "9.0",
    "10.0",
    "11.0",
    "12.0",
    "13.0",
    "14.0",
    "14.74",
    "14.75",
    "14.80",
    "16.0",
    "18.0",
    "20.0",
    "20.063",
    "20.10",
    "21.0",
    "22.0",
    "23.0",
    "24.0",
    "25.0",
    "26.0",
    "26.9",
    "27.0",
    "27.10",
    "27.25",
    "27.50",
    "27.75",
    "27.90",
    "27.99",
    "28.0",
    "28.25",
    "28.5",
    "28.75",
    "29.0",
    "30.0",
    "31.0",
    "32.0",
    "33.0",
    "34.0",
    "34.5",
    "34.9",
    "34.97",
    "34.97835",
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


def check_pipeline_source(baseline_path: Path) -> None:
    source_path = baseline_path.parent.parent / "src" / "main.rs"
    source = source_path.read_text()
    forbidden = (
        "Option<FlowCamera>",
        "flow.is_none()",
        "flow.is_some()",
        "render_intro_surface",
        "journey_camera",
        "journey_position",
        "clear_clean_background",
        "render_ray_surface_lod",
        "fill_atmospheric_sky",
        "render_distant_surface_lod",
        "render_world_surface",
        "CameraPose",
        "const PLANET_CENTER: Vec3",
        "#[allow(dead_code)]",
        "fn sphere_roots",
        "SCENE_",
        "if flow_camera_is_straight",
        "planet_x_scale",
        "warp_grid_point",
        "constrain_planet_framing",
        "fn lift_grid_point",
        "fn flow_depth",
    )
    present = [token for token in forbidden if token in source]
    if present:
        fail("split camera/render pipeline remains: " + ", ".join(present))
    required = (
        "fn flow_camera(time: f32) -> FlowCamera",
        "clear_depth_buffer();",
        "fn flow_grid_world_point(",
        "fn flow_star_world_point(",
        "fn flow_field_transport(",
        "project_flow_star_segment(",
        "project_flow_grid_segment(",
        "fn projection_x_scale(",
        "render_continuous_grid(elapsed, &flow, x_scale)",
        "render_flow_surface(",
        "render_polygon_surface_lod(",
        "apply_atmospheric_shell(",
    )
    missing = [token for token in required if token not in source]
    if missing:
        fail("unified pipeline marker missing: " + ", ".join(missing))
    flyby_start = source.index("fn flyby_segment(")
    canonical_marker = '\n#[cfg(feature = "phase0-audit")]\nfn canonical_flyby_segment'
    flyby_end = source.index(canonical_marker, flyby_start)
    flyby_digest = hashlib.sha256(source[flyby_start:flyby_end].encode()).hexdigest()
    canonical_flyby_digest = "230a7e8db698a84157f95828cc314b84f38c4a342bfb637605288abc7a8666e5"
    if flyby_digest != canonical_flyby_digest:
        fail(
            "signed-off flyby law changed: "
            f"expected {canonical_flyby_digest}, got {flyby_digest}"
        )
    reference_start = source.index("fn canonical_flyby_segment(")
    reference_end = source.index("\nconst fn formation_gap", reference_start)
    reference_digest = hashlib.sha256(
        source[reference_start:reference_end].encode()
    ).hexdigest()
    canonical_reference_digest = "f4dee75ac62a1c1314117cae7baafcc28547932f9ce76c73c2d3ca134e7feb4a"
    if reference_digest != canonical_reference_digest:
        fail(
            "signed-off flyby oracle changed: "
            f"expected {canonical_reference_digest}, got {reference_digest}"
        )
    print("phase7 source ok: one FlowCamera, branch-free aspect-correct grid projection, polygon surface, shared shell/depth")


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

        # Deliberately fragment the escape sequence. Real terminals may split
        # these bytes across reads; Right Arrow must not be mistaken for a
        # standalone Escape when that happens.
        os.write(master, b"\x1b")
        time.sleep(0.005)
        os.write(master, b"[")
        time.sleep(0.005)
        os.write(master, b"C")
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
        check_pipeline_source(baseline_path)
        check_motion(audit_binary)
        check_controls(normal_binary)
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"phase0 audit failed: {error}", file=sys.stderr)
        return 1

    print("phase0 audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
