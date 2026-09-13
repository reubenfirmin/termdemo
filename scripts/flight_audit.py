#!/usr/bin/env python3
"""Independent live-flight gates and exact extraction comparison; no image assets.

The historical source/geometry assertions remain assertions, not new approvals.
One failed gate never prevents another independent gate from running.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess
import time

from phase0_audit import check_controls, check_pipeline_source, render_digest
from rust_sources import check_extraction

ROOT = Path(__file__).resolve().parent.parent
HEADER = struct.Struct("<8sIIId")
RECORD = struct.Struct("<25d")
SELECTORS = (
    ("playback-state", "_"), ("flight-requirements", "="),
    ("legacy-motion", "!"), ("precision", "@"), ("motion-derivatives", "#"),
    ("world-camera", "^"), ("surface", "%"), ("legacy-flight-fingerprint", "F"),
    ("refinement", "R"), ("atmosphere", "A"), ("hardening-performance", "+"),
    ("orbital-descent", "v"), ("grid-density", "d"), ("wormhole", "w"),
)


def independent_gates(gates):
    """Catch per-gate failures, including timeouts; preserve every outcome."""
    results = []
    for name, gate in gates:
        print(f"RUN {name}", flush=True)
        start = time.monotonic()
        try:
            output = gate()
            result = dict(name=name, status="PASS", output=output or "")
        except Exception as error:
            result = dict(name=name, status="FAIL", output=str(error))
        result["seconds"] = time.monotonic() - start
        results.append(result)
        print(f"{result['status']} {name}: {result['output']}", flush=True)
    return results


def selector_gate(binary, selector):
    result = subprocess.run([binary], input=selector.encode("ascii"),
                            capture_output=True, timeout=180, check=False)
    output = (result.stdout + result.stderr).decode("utf-8", errors="replace").strip()
    if result.returncode:
        raise RuntimeError(f"exit={result.returncode}\n{output}")
    return output


def suite(args):
    gates = [("extraction-tokens", lambda: check_extraction(ROOT)),
             ("legacy-source-locks", lambda: check_pipeline_source(
        ROOT / "tests/phase0_frames.sha256"))]
    gates += [(name, lambda selector=selector: selector_gate(args.audit_binary, selector))
              for name, selector in SELECTORS]
    gates.append(("playback-terminal", lambda: check_controls(args.binary)))
    results = independent_gates(gates)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(results, indent=2) + "\n")
    failed = [result["name"] for result in results if result["status"] == "FAIL"]
    print(f"SUMMARY {len(results) - len(failed)}/{len(results)} gates passed; "
          f"failures={','.join(failed) or 'none'}; report={args.output}")
    return int(bool(failed))


class Trace:
    def __init__(self, path):
        self.data = path.read_bytes()
        if len(self.data) < HEADER.size:
            raise ValueError("truncated camera trace header")
        magic, self.hz, self.count, columns, self.scale = HEADER.unpack_from(self.data)
        if magic != b"TDMOTION" or columns != 25 or self.hz != 1920:
            raise ValueError("unsupported camera trace format")
        if len(self.data) != HEADER.size + self.count * RECORD.size or self.count != 140 * self.hz + 1:
            raise ValueError("truncated or incomplete 0–140s camera trace")
        if not math.isfinite(self.scale) or self.scale <= 0:
            raise ValueError("invalid world-to-metre conversion")

    def row(self, index):
        return RECORD.unpack_from(self.data, HEADER.size + index * RECORD.size)

    def derivative(self, index, stride=1):
        left = self.row(max(0, index - stride))
        center = self.row(index)
        right = self.row(min(self.count - 1, index + stride))
        return position_derivative(left, center, right)


def position_derivative(left, center, right):
    """Differentiate actual positions at their actual, possibly unequal times."""
    dl, dr = center[0] - left[0], right[0] - center[0]
    if dl == 0:
        return tuple((right[k] - center[k]) / dr for k in range(1, 4))
    if dr == 0:
        return tuple((center[k] - left[k]) / dl for k in range(1, 4))
    return tuple((dr * (center[k] - left[k]) / dl
                  + dl * (right[k] - center[k]) / dr) / (dl + dr) for k in range(1, 4))


def measure(trace, output):
    output.mkdir(parents=True, exist_ok=True)
    speeds = [math.hypot(*trace.derivative(i)) * trace.scale for i in range(trace.count)]
    exit_speed = speeds[5 * trace.hz]
    rows = []
    distance = 0.0
    max_error = (0.0, 0.0)
    max_coarse_error = (0.0, 0.0)
    max_short_loss = (0.0, 0.0)
    max_second_loss = (0.0, 0.0)
    max_second_gain = (0.0, 0.0)
    crossings = []
    next_half_lap = 0.5
    previous = trace.row(0)
    with (output / "camera-motion.csv").open("w", newline="") as file:
        writer = csv.writer(file)
        columns = ("time_s", "actual_position_speed_mps", "reported_speed_mps",
                   "exit_speed_remaining_pct", "last_second_loss_pct", "altitude_agl_m",
                   "orbits", "distance_since_5s_m", "velocity_vector_error_relative",
                   "tangential_acceleration_mps2", "fractional_braking_per_s")
        writer.writerow(columns)
        for i, speed in enumerate(speeds):
            sample = trace.row(i)
            if not all(math.isfinite(value) for value in sample):
                raise ValueError(f"nonfinite camera state at sample {i}")
            if i and sample[0] <= previous[0]:
                raise ValueError(f"nonmonotonic camera timestamp at sample {i}")
            if i > 5 * trace.hz:
                distance += (speed + speeds[i - 1]) * 0.5 * (sample[0] - previous[0])
            reported = sample[20] * trace.scale
            error = math.hypot(*(a - b for a, b in zip(trace.derivative(i), sample[4:7])))
            error /= max(math.hypot(*sample[4:7]), 1e-30)
            accel = sum(a * b for a, b in zip(sample[4:7], sample[7:10]))
            accel = accel / max(math.hypot(*sample[4:7]), 1e-30) * trace.scale
            loss = 100 * (1 - speed / speeds[i - trace.hz]) if i >= trace.hz else None
            if 5 * trace.hz <= i < trace.count - 1:
                max_error = max(max_error, (error, sample[0]))
                coarse = trace.derivative(i, 16)
                coarse_error = math.hypot(*(a - b for a, b in zip(coarse, sample[4:7])))
                coarse_error /= max(math.hypot(*sample[4:7]), 1e-30)
                max_coarse_error = max(max_coarse_error, (coarse_error, sample[0]))
            if i >= 6 * trace.hz:
                max_second_loss = max(max_second_loss, (loss, sample[0]))
                max_second_gain = max(max_second_gain, (-loss, sample[0]))
            if i >= 5 * trace.hz + 192:
                short = 100 * (1 - speed / speeds[i - 192])
                max_short_loss = max(max_short_loss, (short, sample[0]))
            while i and sample[22] >= next_half_lap > previous[22]:
                fraction = (next_half_lap - previous[22]) / (sample[22] - previous[22])
                event_time = previous[0] + fraction * (sample[0] - previous[0])
                crossings.append(dict(orbits=next_half_lap, time_s=event_time))
                next_half_lap += 0.5
            row = (sample[0], speed, reported, 100 * speed / exit_speed, loss,
                   sample[19], sample[22], distance if i >= 5 * trace.hz else None,
                   error, accel, -accel / max(reported, 1e-30))
            writer.writerow(row)
            if i % trace.hz == 0 and i >= 5 * trace.hz:
                rows.append(row)
            previous = sample
    summary = dict(samples=trace.count, hz=trace.hz, world_unit_metres=trace.scale,
                   trace_sha256=hashlib.sha256(trace.data).hexdigest(),
                   max_position_velocity_relative_error=max_error,
                   max_coarse_position_velocity_relative_error=max_coarse_error,
                   max_100ms_speed_loss_pct_and_end_time=max_short_loss,
                   max_1s_speed_loss_pct_and_end_time=max_second_loss,
                   max_1s_speed_gain_pct_and_end_time=max_second_gain,
                   measured_half_lap_crossings=crossings)
    (output / "motion-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    lines = ["# Actual current camera motion — not the proposed flight", "",
             "Speed is independently differentiated from production camera positions at 1,920 Hz.",
             "Reported velocity is compared separately; finite-difference error is reported, not waived.",
             "The 0–140s CSV contains every sample. Distance integrates measured speed from 5s.",
             "Percent loss compares with one second earlier; negative loss means reacceleration.",
             "No motion, geometry or visual approval is implied by an extraction match.", "",
             "| Time s | Actual m/s | Reported m/s | % exit | Last 1s loss % | AGL m | Orbits |",
             "|---:|---:|---:|---:|---:|---:|---:|"]
    for row in rows:
        lines.append(f"| {row[0]:.0f} | {row[1]:,.3f} | {row[2]:,.3f} | {row[3]:.8f} | "
                     f"{row[4]:.5f} | {row[5]:,.3f} | {row[6]:.7f} |")
    lines += ["", "## Measured diagnostics", "", "```json", json.dumps(summary, indent=2), "```", ""]
    (output / "camera-motion.md").write_text("\n".join(lines))
    print(json.dumps(summary, indent=2), flush=True)


def capture(args):
    args.output.mkdir(parents=True, exist_ok=True)
    path = args.output / "camera-state.bin"
    with path.open("wb") as output:
        result = subprocess.run([args.audit_binary], input=b"s", stdout=output,
                                stderr=subprocess.PIPE, timeout=180, check=False)
    if result.returncode:
        raise RuntimeError(f"trace failed: exit={result.returncode} {result.stderr!r}")
    measure(Trace(path), args.output)
    return 0


def compare(args):
    before, after = Trace(args.before), Trace(args.after)
    if before.data != after.data:
        if before.data[:HEADER.size] != after.data[:HEADER.size]:
            raise RuntimeError("camera trace header/scale changed")
        for index in range(before.count):
            offset = HEADER.size + index * RECORD.size
            if before.data[offset:offset + RECORD.size] != after.data[offset:offset + RECORD.size]:
                raise RuntimeError(f"camera-state BIT MISMATCH at {before.row(index)[0]}s (sample {index})")
        raise RuntimeError("camera trace length changed")
    print(f"EXACT MATCH: {before.count} camera states, all 25 fields, 0–140s at {before.hz} Hz")
    return 0


def compare_render(args):
    # RGB stays in subprocess RAM and is immediately hashed. No image is saved,
    # displayed or used as an asset; the protected image baseline is untouched.
    times = [0.001, 0.5, 1.0, 2.0, 3.0, 4.0, 4.001, 5.0, 7.6, 11.0,
             20.5, 21.0, 23.0, 26.0, 28.0, 29.0, 31.0, 32.0, 34.0, 35.0,
             36.0, 38.0, 40.0, 42.0, 53.0, 58.0, 90.0, 122.0, 137.0, 140.0]
    if args.dense:
        times = sorted({*times, *(i / 60 for i in range(1, 241)),
                        *(float(i) for i in range(5, 141)), 20.499, 20.501,
                        25.658333, 27.999, 28.001, 31.999, 32.001, 34.999,
                        35.001, 37.999, 38.001, 39.999, 40.001, 58.001})
    mismatches = []
    for index, value in enumerate(times):
        stamp = f"{value:.6f}"
        if render_digest(args.before, stamp) != render_digest(args.after, stamp):
            mismatches.append(stamp)
        if index % 5 == 0:
            print(f"render byte comparison {index + 1}/{len(times)}", flush=True)
    if mismatches:
        raise RuntimeError(f"render byte mismatches at {','.join(mismatches)}")
    print(f"EXACT MATCH: {len(times)} normal-render RGB digests, 0–140s checkpoints; no baseline rewritten")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("suite", "capture"):
        sub = commands.add_parser(name)
        sub.add_argument("--audit-binary", type=lambda value: Path(value).resolve(),
                         default=ROOT / "target/phase0-audit/x86_64-unknown-linux-gnu/release/termdemo")
        sub.add_argument("--output", type=Path, required=True)
        if name == "suite":
            sub.add_argument("--binary", type=lambda value: Path(value).resolve(),
                             default=ROOT / "target/x86_64-unknown-linux-gnu/release/termdemo")
    for name in ("compare", "compare-render"):
        sub = commands.add_parser(name)
        sub.add_argument("before", type=lambda value: Path(value).resolve())
        sub.add_argument("after", type=lambda value: Path(value).resolve())
        if name == "compare-render":
            sub.add_argument("--dense", action="store_true", help="slow: 60-Hz opening plus every whole second")
    args = parser.parse_args()
    return {"suite": suite, "capture": capture, "compare": compare,
            "compare-render": compare_render}[args.command](args)


if __name__ == "__main__":
    raise SystemExit(main())
