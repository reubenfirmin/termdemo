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

try:
    from .rust_sources import PipelineSource
except ImportError:
    from rust_sources import PipelineSource


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
    source_root = baseline_path.parent.parent / "src"
    pipeline = PipelineSource(sorted(path for path in source_root.glob("*.rs")
                                     if path.name != "flight_speed.rs"))
    source = pipeline.text
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
        "fn flow_grid_world_point(",
        "fn flow_star_world_point(",
        "fn flow_field_transport(",
        "fn streakflight_displacement(",
        "fn field_streak_volume_visibility(",
        "particle_passes",
        "fill_star_space(",
        "fn star_space_color(",
        "project_flow_star_segment(",
        "project_flow_grid_segment(",
        "fn field_particle_point(",
        "fn field_particle_organization(",
        "fn field_particle_point_regularized(",
        "fn render_subpixel_surface_lod(",
        "fn surface_polygon_weight(",
        "fn surface_unresolved_depth_bias(",
    )
    present = [token for token in forbidden if token in source]
    if present:
        fail("split camera/render pipeline remains: " + ", ".join(present))
    required = (
        "fn flow_camera(time: f32) -> FlowCamera",
        "clear_depth_buffer();",
        "fn field_point(layout: FieldLayout, s: f64, theta: f64)",
        "fn field_ring_identity(ordinal: i32) -> FieldRingIdentity",
        "fn field_ring_s(layout: FieldLayout, identity: FieldRingIdentity) -> f64",
        "const FIELD_RING_DISTANCE_TABLE_LEN: usize = 512;",
        "const fn field_ring_distance_table()",
        "const FIELD_RING_DISTANCE_TABLE:",
        "fn field_first_ring_ordinal(layout: FieldLayout) -> i32",
        "fn field_last_ring_ordinal(layout: FieldLayout) -> i32",
        "fn field_first_physical_ring_ordinal(layout: FieldLayout) -> i32",
        "struct StarCellIdentity",
        "fn star_cell_identity(ordinal: u32) -> StarCellIdentity",
        "fn star_cell_ordinal(identity: StarCellIdentity) -> u32",
        "fn star_cell_bounds(layout: FieldLayout, identity: StarCellIdentity)",
        "fn field_mesh_vertex(",
        "fn field_edge_points(layout: FieldLayout, identity: FieldEdgeIdentity)",
        "struct StarObjectIdentity",
        "struct RingObjectIdentity",
        "struct RailObjectIdentity",
        "fn star_object_position(",
        "fn star_object_exists(",
        "struct BackgroundStarIdentity",
        "fn background_star_position(",
        "fn local_star_field_visibility(",
        "fn local_star_object_visibility(",
        "fn field_exterior_visibility(",
        "fn ring_object_segment(",
        "fn rail_object_segment(",
        "fn rail_lane_start_ordinal(",
        "fn rail_object_exists(",
        "const UNIVERSE_FIELD: FieldLayout",
        "struct Universe",
        "const UNIVERSE: Universe",
        "fn universe() -> Universe",
        "fn universe_field_layout() -> FieldLayout",
        "fn fill_universe_vacuum()",
        "fn universe_space_visibility(camera: &FlowCamera) -> f32",
        "fn field_mesh_visibility(",
        "const FIELD_RING_START_S: f64 = FIELD_ORDERING_START_S + 64.0;",
        "const FIELD_MESH_FADE_START_WORLD: f64 = 80.0;",
        "const FIELD_MESH_MAX_DISTANCE_WORLD: f64 = 180.0;",
        "const FIELD_MOTION_BLUR_EXPOSURE: f32 = 0.032;",
        "fn field_star_exposure(layout: FieldLayout, position: DVec3) -> f32",
        "let exposure_start = field_star_exposure_camera(elapsed, &flow);",
        "const LIGHT_YEAR_MILES: f64 = 5_878_625_373_183.608;",
        "const STARFLIGHT_START_DISTANCE_WORLD: f64 = miles_to_world(LIGHT_YEAR_MILES * 4.0);",
        "fn starflight_displacement(time: f64) -> (f64, f64, f64)",
        "fn decelerating_approach_component(",
        "struct WorldCurveNode",
        "fn world_curve_capture_geometry(",
        "fn world_curve_spiral_base_radius(",
        "fn world_curve_spiral_speed(phase: f64, time: f64) -> f64",
        "fn world_curve_orbit_normal(",
        "fn generate_world_curve()",
        "let starflight = starflight_displacement(time);",
        "fn project_field_segment(",
        "fn clip_exposure_half_space(",
        "fn project_field_star_exposure(",
        "fn canonical_flow_grid_world_point(",
        "fn canonical_flow_star_world_point(",
        "fn canonical_flow_field_transport(",
        "fn canonical_star_particle_world_position(",
        "fn canonical_star_exposure_times(",
        "fn fixed_candidate_from_rays(",
        "project_canonical_flow_star_segment(",
        "project_canonical_flow_grid_segment(",
        "fn projection_x_scale(",
        "render_universe_field(&exposure_start, &flow, x_scale)",
        "render_flow_surface(",
        "fn surface_mesh_bounds(",
        "render_polygon_surface_lod(",
        "apply_atmospheric_shell(",
    )
    missing = [token for token in required if token not in source]
    if missing:
        fail("unified pipeline marker missing: " + ", ".join(missing))
    star_identity = pipeline.items[("struct", "StarObjectIdentity")]
    if (
        "cell: StarCellIdentity" not in star_identity
        or "object: u8" not in star_identity
        or "FieldRingIdentity" in star_identity
        or "lane:" in star_identity
        or "sample:" in star_identity
    ):
        fail("star identity remains coupled to ring lanes or mesh samples")
    star_position = pipeline.function("star_object_position")
    forbidden_star_mesh_dependencies = (
        "field_point(",
        "field_mesh_vertex(",
        "field_edge_points(",
        "field_ring_",
        "FieldRingIdentity",
        "RingObjectIdentity",
        "RailObjectIdentity",
    )
    coupled = [
        token for token in forbidden_star_mesh_dependencies if token in star_position
    ]
    if coupled:
        fail("star position remains coupled to mesh geometry: " + ", ".join(coupled))
    field_point_start = source.index("fn field_point(")
    field_point_signature = source[field_point_start:source.index("{", field_point_start)]
    forbidden_field_inputs = ("time", "camera", "depth", "ring")
    present_field_inputs = [
        token for token in forbidden_field_inputs if token in field_point_signature
    ]
    if present_field_inputs:
        fail(
            "anchored field_point has dynamic placement inputs: "
            + ", ".join(present_field_inputs)
        )
    for function_name in (
        "field_ring_s",
        "field_mesh_vertex",
        "field_edge_points",
        "star_object_position",
        "star_object_exists",
        "star_cell_identity",
        "star_cell_ordinal",
        "star_cell_bounds",
        "ring_object_segment",
        "rail_object_segment",
        "rail_lane_start_ordinal",
        "rail_object_exists",
    ):
        function_start = source.index(f"fn {function_name}(")
        signature = source[function_start:source.index("{", function_start)]
        dynamic_inputs = [
            token for token in ("time", "camera", "depth", "age") if token in signature
        ]
        if dynamic_inputs:
            fail(
                f"anchored {function_name} has dynamic placement inputs: "
                + ", ".join(dynamic_inputs)
            )
    particle_witness = pipeline.block("canonical_star_particle_reference",
        "canonical_star_particle_local_position", "canonical_star_particle_world_position",
        "canonical_star_exposure_times")
    forbidden_particle_dependencies = (
        "flow_camera(",
        "trajectory_state(",
        "canonical_flow_field_transport(",
    )
    present_particle_dependencies = [
        token for token in forbidden_particle_dependencies if token in particle_witness
    ]
    if present_particle_dependencies:
        fail(
            "canonical particle witness directly reads camera/transport state: "
            + ", ".join(present_particle_dependencies)
        )
    approach_digest = hashlib.sha256(pipeline.block("approach_camera_reference").encode()).hexdigest()
    canonical_approach_digest = "c8a78854eef5bc5429084fed6f4544adc9130f342eef0ce3c92858166ea6c58e"
    if approach_digest != canonical_approach_digest:
        fail(
            "approved decelerating near-approach law changed: "
            f"expected {canonical_approach_digest}, got {approach_digest}"
        )
    opening_locks = (
        (
            "approved downward capture geometry (braking retiming only)",
            ("world_curve_capture_frame", "world_curve_capture_geometry"),
            "",
            "dee81eb58966291c12df694caadb92858507f67273a2c8431b5ddfcea17879c3",
        ),
        (
            "approved orbital radius and altitude geometry",
            ("world_curve_spiral_base_radius", "world_curve_spiral_radius_value"),
            "\n",
            "62d3fc1020badd48b5072c0e3b6afd321c47ead0c45db806238f3f1409af412d",
        ),
        (
            "approved orbital and regional spatial curve",
            ("world_curve_orbit_normal", "world_curve_frame_rotation", "world_curve_regional_frame",
             "world_curve_regional_phase", "world_curve_late_normal"),
            "\n",
            "ac3efd7b45c6d7dee75c3cb4b5f513225d95f6deb0878a1c6cc85c043d2a3773",
        ),
        (
            "opening starflight law",
            ("starflight_displacement",),
            "\n",
            "28598df5ea34b8bcb482441ba49afb50a866230f260afff83f7f99e8c27c634a",
        ),
        (
            "18-to-13.258 decelerating 4-to-28-second approach law",
            ("decelerating_approach_component",),
            "\n",
            "27a4002f04ecbb72dbb381015725c8632b52a040494315d3f85ab6eed718987a",
        ),
    )
    for label, names, trailing, expected_digest in opening_locks:
        actual_digest = hashlib.sha256(pipeline.block(*names, trailing=trailing).encode()).hexdigest()
        if actual_digest != expected_digest:
            fail(
                f"approved {label} changed: expected {expected_digest}, "
                f"got {actual_digest}"
            )
    active_state = pipeline.function("trajectory_state")
    if "WORLD_CURVE" not in active_state:
        fail("active trajectory does not evaluate the unified world curve")
    forbidden_active_controls = (
        "approach_maneuver_state(",
        "capture_impact_acceleration(",
        "orbit_brake_amount(",
        "capture_bank_tangent_state(",
    )
    present_active_controls = [
        token for token in forbidden_active_controls if token in active_state
    ]
    if present_active_controls:
        fail(
            "active trajectory still composes rejected capture controls: "
            + ", ".join(present_active_controls)
        )
    capture_geometry = pipeline.block("world_curve_capture_geometry", "world_curve_capture_arc_length",
                                      "world_curve_capture_advance")
    capture_signature = capture_geometry[:capture_geometry.index("{")]
    if "time" in capture_signature:
        fail("capture centerline geometry depends on demo time")
    active_camera = pipeline.function("flow_camera")
    if "trajectory_state(time as f64)" not in active_camera:
        fail("camera does not consume the unified trajectory state")
    forbidden_camera_controls = (
        "FLOW_CAPTURE_",
        "capture_bank_tangent_state(",
        "capture_impact_acceleration(",
    )
    present_camera_controls = [
        token for token in forbidden_camera_controls if token in active_camera
    ]
    if present_camera_controls:
        fail(
            "camera still consumes an independent capture law: "
            + ", ".join(present_camera_controls)
        )
    # Hash only the fixed background-position function.  Local-star visibility
    # helpers are separate policy and may evolve without changing this geometry.
    background_definition = pipeline.block("background_star_position")
    background_digest = hashlib.sha256(background_definition.encode()).hexdigest()
    canonical_background_digest = "16444f827f2daf49f9c7f1a90ff382e2ed12975a6f87da632d2b2cee9bcf89dc"
    if background_digest != canonical_background_digest:
        fail(
            "exterior background-star geometry changed: "
            f"expected {canonical_background_digest}, got {background_digest}"
        )
    background_signature = background_definition[:background_definition.index("{")]
    if "camera" in background_signature or "time" in background_signature:
        fail("background star positions depend on the camera or timeline")
    universe_start = source.index("const UNIVERSE_FIELD: FieldLayout")
    universe_end = source.index("\nfn universe_field_layout()", universe_start)
    universe_definition = source[universe_start:universe_end]
    forbidden_universe_dependencies = (
        "trajectory_",
        "starflight_",
        "camera",
        "time",
        "STARFIELD_HOLD",
        "STAR_ORGANIZE",
    )
    present_universe_dependencies = [
        token for token in forbidden_universe_dependencies
        if token in universe_definition
    ]
    if present_universe_dependencies:
        fail(
            "universe definition depends on the current flight: "
            + ", ".join(present_universe_dependencies)
        )
    renderer = pipeline.function("render_universe_field")
    forbidden_renderer_dependencies = (
        "canonical_flow_",
        "ring_state(",
        "ring_birth(",
        "flyby_segment(",
        "travel_row(",
        "field_region(",
        "FieldRegion::",
        "STARFIELD_HOLD",
        "STAR_ORGANIZE",
        "field_particle_point_layered(",
        "field_particle_organization(",
        "particle_visibility",
        "ring_visibility",
        "flow_camera(",
        "time:",
        "connectivity",
    )
    present_renderer_dependencies = [
        token for token in forbidden_renderer_dependencies if token in renderer
    ]
    if present_renderer_dependencies:
        fail(
            "anchored renderer reads compatibility/time-placement code: "
            + ", ".join(present_renderer_dependencies)
        )
    required_renderer_dependencies = (
        "let universe = universe();",
        "universe_space_visibility(flow)",
        "field_ring_identity(",
        "field_ring_s(",
        "star_cell_identity(",
        "star_cell_may_project(",
        "field_first_physical_ring_ordinal(",
        "field_last_ring_ordinal(",
        "star_object_position(",
        "star_object_exists(",
        "background_star_position(",
        "local_star_field_visibility(",
        "local_star_object_visibility(",
        "field_exterior_visibility(",
        "draw_background_star(",
        "ring_object_segment(",
        "rail_object_segment(",
        "rail_object_exists(",
        "project_field_star_exposure(",
        "field_mesh_visibility(",
        "project_field_segment(",
    )
    missing_renderer_dependencies = [
        token for token in required_renderer_dependencies if token not in renderer
    ]
    if missing_renderer_dependencies:
        fail(
            "anchored renderer dependency missing: "
            + ", ".join(missing_renderer_dependencies)
        )
    if "star_cell_may_project(layout, cell, exposure_start)\n            || star_cell_may_project(layout, cell, flow)" not in renderer:
        fail("renderer can discard a star cell visible at one shutter endpoint")
    exposure_projection = pipeline.function("project_field_star_exposure")
    exposure_digest = hashlib.sha256(pipeline.block("clip_exposure_half_space",
        "project_field_star_exposure").encode()).hexdigest()
    canonical_exposure_digest = "2f871bef070aabc9cd67d99e91652c2f67fc1bc05e10a5339b00e7aa31522b97"
    if exposure_digest != canonical_exposure_digest:
        fail(
            "fixed-star shutter clipping changed: "
            f"expected {canonical_exposure_digest}, got {exposure_digest}"
        )
    for token in (
        "project_flow_depth(exposure_start, point)",
        "project_flow_depth(exposure_end, point)",
        "clip_exposure_half_space(",
    ):
        if token not in exposure_projection:
            fail("physical streak exposure dependency missing: " + token)
    exposure_signature = exposure_projection[:exposure_projection.index("{")]
    if "time" in exposure_signature or "trajectory" in exposure_signature:
        fail("star exposure clipping reads a timeline or trajectory instead of two poses")
    for compatibility_function in (
        "canonical_flow_grid_world_point",
        "canonical_flow_field_transport",
        "canonical_flow_star_world_point",
        "project_canonical_flow_star_segment",
        "project_canonical_flow_grid_segment",
    ):
        marker = f'#[cfg(feature = "phase0-audit")]\nfn {compatibility_function}('
        if marker not in source:
            fail(f"compatibility oracle is not audit-only: {compatibility_function}")
    flyby_digest = hashlib.sha256(pipeline.block("flyby_segment").encode()).hexdigest()
    canonical_flyby_digest = "230a7e8db698a84157f95828cc314b84f38c4a342bfb637605288abc7a8666e5"
    if flyby_digest != canonical_flyby_digest:
        fail(
            "signed-off flyby law changed: "
            f"expected {canonical_flyby_digest}, got {flyby_digest}"
        )
    reference_digest = hashlib.sha256(pipeline.block("canonical_flyby_segment").encode()).hexdigest()
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
    restored_screen = threading.Event()
    finished = threading.Event()
    updates = threading.Condition()
    frame_updates = 0
    protocol_tail = b""

    def drain() -> None:
        nonlocal frame_updates, protocol_tail
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
            if b"\x1b[?1049l" in captured_end:
                restored_screen.set()
            # Count completed protocol updates only. No frame decoding,
            # screenshots, image files or visual baselines are involved.
            marker = b"\x1b[?2026l"
            stream = protocol_tail + chunk
            count = stream.count(marker)
            protocol_tail = stream[-(len(marker) - 1):]
            if count:
                with updates:
                    frame_updates += count
                    updates.notify_all()

    def wait_for_updates(target: int) -> None:
        deadline = time.monotonic() + 3.0
        with updates:
            while frame_updates < target:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    fail(f"playback did not produce update {target}; got {frame_updates}")
                updates.wait(remaining)

    def require_paused() -> int:
        # Permit the one frame already being transmitted when Space arrived.
        deadline = time.monotonic() + 3.0
        while time.monotonic() < deadline:
            with updates:
                count = frame_updates
                changed = updates.wait_for(lambda: frame_updates != count, timeout=0.35)
            if not changed:
                if process.poll() is not None:
                    fail("paused process exited")
                return count
        fail("Space did not freeze frame transmission")
        raise AssertionError("unreachable")

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
        if not entered_screen.wait(30.0):
            fail("interactive process did not enter the alternate screen")
        wait_for_updates(3)

        os.write(master, b" ")
        paused_count = require_paused()

        # Deliberately fragment the escape sequence. Real terminals may split
        # these bytes across reads; Right Arrow must not be mistaken for a
        # standalone Escape when that happens.
        os.write(master, b"\x1b")
        time.sleep(0.005)
        os.write(master, b"[")
        time.sleep(0.005)
        os.write(master, b"C")
        wait_for_updates(paused_count + 1)
        if require_paused() != paused_count + 1:
            fail("Right Arrow does not render exactly one paused seek frame")
        if process.poll() is not None:
            fail("Right Arrow exited the program")

        # A checkpoint can be inspected without unexpectedly resuming motion.
        paused_count = frame_updates
        os.write(master, b"4")
        wait_for_updates(paused_count + 1)
        if require_paused() != paused_count + 1:
            fail("checkpoint seek resumes paused playback")

        # Space must resume, then be able to pause again. Verify the actual
        # terminal loop, not merely that it accepted the key without exiting.
        paused_count = frame_updates
        os.write(master, b" ")
        wait_for_updates(paused_count + 3)
        os.write(master, b" ")
        require_paused()

        os.write(master, b"\x1b")
        try:
            return_code = process.wait(timeout=3.0)
        except subprocess.TimeoutExpired:
            fail("Escape did not exit within three seconds")
        if return_code != 0:
            fail(f"interactive process exited with status {return_code}")
        if not restored_screen.wait(1.0):
            fail("paused Escape did not restore the alternate screen")
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
    print("phase0 controls ok: Space pause/resume, frozen output, paused arrow/checkpoint seeks, Escape, terminal cleanup")


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
