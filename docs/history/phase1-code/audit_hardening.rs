//! ARCHIVED phase-1 implementation. Not compiled or used by the active flight.
#[cfg(feature = "phase0-audit")]
use super::{
    DVec3, FLOW_DESCENT_START, FLOW_PATH_END, FRAME, FRAME_BYTES, GRID_SQUARE_START,
    ORBIT_ENTRY, PLANET_CENTER, PLANET_INTRO, RING_BIRTH, STARFIELD_FILL, STARFIELD_HOLD,
    STAR_ORGANIZE, SURFACE_SAMPLE_CACHE, append, append_number, dvec_length_squared, dvec_sub,
    exit, flow_camera, flow_descent_end, phase0_basis_valid, process_cpu_ns,
    projected_surface_bounds, render_demo, surface_mesh_bounds, surface_mesh_step, write_all,
};

#[cfg(feature = "phase0-audit")]
pub(super) fn phase7_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase7 hardening audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase7_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase7_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase7_frame_hash() -> u64 {
    let frame = core::ptr::addr_of!(FRAME).cast::<u8>();
    let mut result = 14_695_981_039_346_656_037u64;
    let mut index = 0usize;
    while index < FRAME_BYTES {
        result ^= unsafe { frame.add(index).read() } as u64;
        result = result.wrapping_mul(1_099_511_628_211);
        index += 1;
    }
    result
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_phase7_hardening_audit() -> ! {
    let mut view = projected_surface_bounds(&flow_camera(29.0), 1.0);
    let radii = [0.0f32, 2.5, 4.0, 7.0, 10.0, 20.0];
    view.radius_x = 0.0;
    view.radius_y = 0.0;
    let mut previous_bounds = surface_mesh_bounds(view, surface_mesh_step());
    let mut index = 0usize;
    while index < radii.len() {
        view.radius_x = radii[index];
        view.radius_y = radii[index];
        let bounds = surface_mesh_bounds(view, surface_mesh_step());
        phase7_require(
            bounds.0 <= previous_bounds.0
                && bounds.1 >= previous_bounds.1
                && bounds.2 <= previous_bounds.2
                && bounds.3 >= previous_bounds.3,
            230,
            b"single surface lattice bounds are not deterministic and nested",
        );
        previous_bounds = bounds;
        index += 1;
    }
    let checkpoints = [0.0f32, 14.75, 21.0, 27.99, 28.0, 51.959_26, 65.0, 94.0, 137.0];
    index = 0;
    while index < checkpoints.len() {
        let first = flow_camera(checkpoints[index]);
        let second = flow_camera(checkpoints[index]);
        phase7_require(
            phase0_basis_valid(first)
                && dvec_length_squared(dvec_sub(first.position, second.position)) == 0.0
                && PLANET_CENTER == DVec3 { x: 0.0, y: 0.0, z: 0.0 },
            231,
            b"hardened sequence is not deterministic under seeking",
        );
        index += 1;
    }
    let frame_seek_checkpoints = [27.99f32, 28.0, 40.0, 42.0, 51.959_26, 53.0, 58.0, 65.0, 94.0, 122.0, 137.0];
    index = 0;
    while index < frame_seek_checkpoints.len() {
        let time = frame_seek_checkpoints[index];
        unsafe { core::ptr::addr_of_mut!(SURFACE_SAMPLE_CACHE).write_bytes(0, 1); }
        render_demo(time, 1.0);
        let first = phase7_frame_hash();
        render_demo((time + 0.731).min(FLOW_PATH_END as f32), 1.0);
        render_demo(time, 1.0);
        phase7_require(
            first == phase7_frame_hash(),
            233,
            b"seeking away and back does not reproduce the complete frame",
        );
        index += 1;
    }
    let performance_checkpoints = [
        0.0f32,
        STARFIELD_HOLD,
        STARFIELD_FILL,
        STAR_ORGANIZE,
        RING_BIRTH,
        GRID_SQUARE_START,
        PLANET_INTRO,
        ORBIT_ENTRY,
        FLOW_DESCENT_START as f32,
        32.0,
        34.0,
        38.0,
        40.0,
        41.0,
        42.0,
        53.0,
        58.0,
        65.0,
        78.0,
        86.0,
        94.0,
        100.0,
        122.0,
        flow_descent_end() as f32,
        FLOW_PATH_END as f32,
    ];
    let mut maximum_frame_ns = 0u64;
    let mut slowest_checkpoint = 0usize;
    index = 0;
    while index < performance_checkpoints.len() {
        unsafe { core::ptr::addr_of_mut!(SURFACE_SAMPLE_CACHE).write_bytes(0, 1); }
        let started = process_cpu_ns();
        render_demo(performance_checkpoints[index], 1.0);
        let frame_ns = process_cpu_ns().saturating_sub(started);
        if frame_ns > maximum_frame_ns {
            maximum_frame_ns = frame_ns;
            slowest_checkpoint = index;
        }
        index += 1;
    }
    if maximum_frame_ns > 33_333_334 {
        let mut report = [0u8; 192];
        let report_pointer = report.as_mut_ptr();
        let mut report_length = 0usize;
        append(
            report_pointer,
            &mut report_length,
            b"phase7 hardening audit failed: frame budget checkpoint-index=",
        );
        append_number(
            report_pointer,
            &mut report_length,
            slowest_checkpoint as u32,
        );
        append(report_pointer, &mut report_length, b" time-ms-x1000=");
        append_number(
            report_pointer,
            &mut report_length,
            (performance_checkpoints[slowest_checkpoint] * 1_000.0) as u32,
        );
        append(report_pointer, &mut report_length, b" frame-ns=");
        append_number(
            report_pointer,
            &mut report_length,
            maximum_frame_ns.min(u32::MAX as u64) as u32,
        );
        append(report_pointer, &mut report_length, b" maximum=33333334\n");
        write_all(&report[..report_length]);
        exit(232)
    }
    write_all(
        b"phase7 hardening audit ok: legacy=absent surface-lod=continuous seek=frame-deterministic controls=phase0 performance=all-checkpoints\n",
    );
    exit(0)
}
