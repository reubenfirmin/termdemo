//! ARCHIVED phase-1 implementation. Not compiled or used by the active flight.
//! Contains the rejected speed caps AND late phase/altitude clocks until phase 3.
use super::{
    DVec3, FLOW_AIRPLANE_ALTITUDE_MILES, FLOW_FINAL_ALTITUDE_WORLD, ORBIT_ENTRY,
    TRAJECTORY_ENTRY_RATE, TrajectoryState, WORLD_CURVE_ORBIT_RADIUS, WORLD_CURVE_STEP,
    WORLD_ORBIT_COMPLETION_TIME, WORLD_ORBIT_INTERVALS, approach_camera_reference, brake_exp,
    brake_log, cubic_component, dvec_length, dvec_normalize, dvec_sub, exit,
    flow_high_pass_end, miles_to_world, smootherstep_f64, sqrt_f64,
    world_curve_capture_arc_length, world_curve_spiral_metric, world_curve_spiral_position,
    write_all,
};
#[cfg(feature = "phase0-audit")]
use super::{
    world_curve_orbit_length,
};

// One extended log-speed envelope, starting 3.5 seconds before the old
// far-side brake. The distance integral follows the unchanged capture and
// the candidate extended spiral, reaching aircraft speed/height at 40 seconds.
pub(super) const WORLD_BRAKE_START: f64 = 25.658_333_333_333_335;
pub(super) const WORLD_BRAKE_END: f64 = 40.0;
pub(super) const WORLD_BRAKE_TURN_LIMIT: f64 = 6.3;
// dv/ds bounds fractional braking (d ln(v)/dt), including approaching a
// tighter bend. The look-ahead limit is integrated in world arc length.
pub(super) const WORLD_BRAKE_SPATIAL_SLOPE: f64 = 1.5;
pub(super) const WORLD_BRAKE_START_TICK: usize = 24_632;
pub(super) const WORLD_BRAKE_END_TICK: usize = 38_400;
pub(super) const WORLD_BRAKE_SUBSTEPS: usize = 1;
pub(super) const WORLD_BRAKE_SAMPLES: usize = (WORLD_BRAKE_END_TICK - WORLD_BRAKE_START_TICK) * WORLD_BRAKE_SUBSTEPS + 1;
pub(super) const WORLD_BRAKE_STEP: f64 = WORLD_CURVE_STEP / WORLD_BRAKE_SUBSTEPS as f64;
// Fast-jet motion at airline height, followed by a smooth reduction to
// canyon cruise during descent. Neither target changes the world scale.
pub(super) const WORLD_CURVE_AIRCRAFT_PHASE_RATE: f64 = 0.004_5;
pub(super) const WORLD_CURVE_GROUND_PHASE_RATE: f64 = 0.001;
pub(super) fn world_curve_braked_speed(time: f64, reference_speed: f64) -> f64 {
    if time <= WORLD_BRAKE_START { return reference_speed; }
    let duration = WORLD_BRAKE_END - WORLD_BRAKE_START;
    let elapsed = (time - WORLD_BRAKE_START).clamp(0.0, duration);
    let bias = unsafe { core::ptr::addr_of!(WORLD_BRAKE_BIAS).read() };
    let u = elapsed / duration;
    // One asymmetric rational log-speed envelope. Both endpoint powers are
    // above two, so speed, acceleration and their first derivative meet the
    // neighboring motion continuously. Bias moves the broad pulse, rather
    // than increasing a power that concentrates braking into a narrow cliff.
    let progress = if u >= 1.0 { 1.0 } else {
        let a = brake_exp(2.8 * brake_log(u));
        let b = bias * brake_exp(2.05 * brake_log(1.0 - u));
        a / (a + b)
    };
    let log_ratio = unsafe { core::ptr::addr_of!(WORLD_BRAKE_LOG_RATIO).read() };
    reference_speed * brake_exp(log_ratio * progress)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn world_curve_approach_speed(time: f64) -> f64 {
    world_curve_controlled_speed(time, 0.0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn world_curve_spiral_speed(phase: f64, time: f64) -> f64 {
    let remaining = unsafe { core::ptr::addr_of!(WORLD_BRAKE_CAPTURE_DISTANCE).read() };
    world_curve_controlled_speed(time, remaining + world_curve_orbit_length(phase))
}

#[derive(Clone, Copy)]
pub(super) struct WorldArcNode {
    pub(super) distance: f64,
    pub(super) metric: f64,
    pub(super) derivative: f64,
    pub(super) speed_limit: f64,
    pub(super) limit_derivative: f64,
}
pub(super) static mut WORLD_ORBIT_ARC: [WorldArcNode; WORLD_ORBIT_INTERVALS + 1] =
    [WorldArcNode { distance: 0.0, metric: 0.0, derivative: 0.0,
        speed_limit: 0.0, limit_derivative: 0.0 }; WORLD_ORBIT_INTERVALS + 1];
pub(super) static mut WORLD_BRAKE_DISTANCE: [f64; WORLD_BRAKE_SAMPLES] = [0.0; WORLD_BRAKE_SAMPLES];
pub(super) static mut WORLD_BRAKE_CAPTURE_DISTANCE: f64 = 0.0;
pub(super) static mut WORLD_BRAKE_BIAS: f64 = 8.0;
pub(super) static mut WORLD_BRAKE_LOG_RATIO: f64 = 0.0;

pub(super) fn generate_world_orbit_arc() {
    let tangent = |p: f64| dvec_normalize(dvec_sub(
        world_curve_spiral_position(p + 0.01), world_curve_spiral_position(p - 0.01)));
    let mut distance = 0.0;
    let mut previous_metric = world_curve_spiral_metric(0.0);
    let mut curvature_max = 1.0 / WORLD_CURVE_ORBIT_RADIUS;
    let mut index = 0;
    while index <= WORLD_ORBIT_INTERVALS {
        let phase = index as f64;
        let metric = world_curve_spiral_metric(phase);
        if index > 0 {
            // Integrate inside the canonical sine cells; endpoint-only
            // quadrature aliases their quintic derivatives.
            let mut sum = previous_metric + metric;
            let mut sample = 1;
            while sample < 8 {
                let weight = if sample & 1 == 0 { 2.0 } else { 4.0 };
                sum += weight * world_curve_spiral_metric(phase - 1.0 + sample as f64 / 8.0);
                sample += 1;
            }
            distance += sum / 24.0;
        }
        let derivative = (world_curve_spiral_metric(phase + 0.05)
            - world_curve_spiral_metric(phase - 0.05)) / 0.1;
        // Anticipate the next few samples instead of reacting after a turn.
        let ahead = (phase + 4.0).min(WORLD_ORBIT_INTERVALS as f64);
        let curvature = dvec_length(dvec_sub(tangent(ahead + 0.05), tangent(ahead - 0.05)))
            / (0.1 * world_curve_spiral_metric(ahead));
        curvature_max = curvature_max.max(curvature);
        unsafe { core::ptr::addr_of_mut!(WORLD_ORBIT_ARC[index]).write(WorldArcNode {
            distance, metric, derivative,
            speed_limit: WORLD_BRAKE_TURN_LIMIT / curvature_max,
            limit_derivative: 0.0,
        }); }
        previous_metric = metric;
        index += 1;
    }
    // Plan the velocity reduction BEFORE tighter curvature. This backward
    // bound limits dv/ds; the forward minimum prevents accelerating again
    // after passing the tighter part of the same curve.
    index = WORLD_ORBIT_INTERVALS;
    while index > 0 {
        let after = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[index]).read() };
        let mut before = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[index - 1]).read() };
        before.speed_limit = before.speed_limit.min(after.speed_limit
            + WORLD_BRAKE_SPATIAL_SLOPE * (after.distance - before.distance));
        unsafe { core::ptr::addr_of_mut!(WORLD_ORBIT_ARC[index - 1]).write(before); }
        index -= 1;
    }
    let mut minimum = f64::MAX;
    index = 0;
    while index <= WORLD_ORBIT_INTERVALS {
        let mut node = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[index]).read() };
        minimum = minimum.min(node.speed_limit);
        node.speed_limit = minimum;
        unsafe { core::ptr::addr_of_mut!(WORLD_ORBIT_ARC[index]).write(node); }
        index += 1;
    }
    index = 1;
    while index < WORLD_ORBIT_INTERVALS {
        let a = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[index - 1]).read() };
        let mut b = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[index]).read() };
        let c = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[index + 1]).read() };
        let left = (b.speed_limit - a.speed_limit) / (b.distance - a.distance);
        let right = (c.speed_limit - b.speed_limit) / (c.distance - b.distance);
        b.limit_derivative = if left < 0.0 && right < 0.0 {
            2.0 * left * right / (left + right)
        } else { 0.0 };
        unsafe { core::ptr::addr_of_mut!(WORLD_ORBIT_ARC[index]).write(b); }
        index += 1;
    }
}

pub(super) fn world_curve_speed_limit(distance: f64) -> f64 {
    let remaining = unsafe { core::ptr::addr_of!(WORLD_BRAKE_CAPTURE_DISTANCE).read() };
    let s = (distance - remaining).max(0.0);
    let mut lower = 0;
    let mut upper = WORLD_ORBIT_INTERVALS;
    while upper - lower > 1 {
        let middle = (lower + upper) / 2;
        let node = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[middle]).read() };
        if node.distance < s { lower = middle; } else { upper = middle; }
    }
    let a = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[lower]).read() };
    let b = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[upper]).read() };
    let span = b.distance - a.distance;
    cubic_component((s - a.distance) / span, span,
        a.speed_limit, b.speed_limit, a.limit_derivative, b.limit_derivative).0
}

pub(super) fn world_curve_controlled_speed(time: f64, distance: f64) -> f64 {
    let reference = if time < ORBIT_ENTRY as f64 {
        -approach_camera_reference(time).1
    } else { -TRAJECTORY_ENTRY_RATE };
    if time <= WORLD_BRAKE_START { return reference; }
    let requested = world_curve_braked_speed(time, reference);
    let cap = world_curve_speed_limit(distance);
    let ratio = requested / cap;
    let r2 = ratio * ratio;
    let r4 = r2 * r2;
    let r8 = r4 * r4;
    let r16 = r8 * r8;
    let activation = smootherstep_f64(time - WORLD_BRAKE_START);
    // Smooth minimum. Both operands are nonincreasing along this flight;
    // there is no hard speed clamp, release kick or independent late brake.
    requested / sqrt_f64(sqrt_f64(sqrt_f64(sqrt_f64(1.0 + activation * r16))))
}

pub(super) fn integrate_world_brake(record: bool) -> f64 {
    let mut distance = 0.0;
    let mut index = 0;
    while index + 1 < WORLD_BRAKE_SAMPLES {
        if record { unsafe { core::ptr::addr_of_mut!(WORLD_BRAKE_DISTANCE[index]).write(distance); } }
        let t = WORLD_BRAKE_START + index as f64 * WORLD_BRAKE_STEP;
        let dt = WORLD_BRAKE_STEP;
        let a = world_curve_controlled_speed(t, distance);
        let b = world_curve_controlled_speed(t + dt * 0.5, distance + a * dt * 0.5);
        let c = world_curve_controlled_speed(t + dt * 0.5, distance + b * dt * 0.5);
        let d = world_curve_controlled_speed(t + dt, distance + c * dt);
        distance += dt * (a + 2.0 * b + 2.0 * c + d) / 6.0;
        index += 1;
    }
    if record { unsafe { core::ptr::addr_of_mut!(WORLD_BRAKE_DISTANCE[index]).write(distance); } }
    distance
}

pub(super) fn configure_world_brake(parameter: f64, start: TrajectoryState, normal: DVec3, tangent: DVec3) {
    let remaining = world_curve_capture_arc_length(parameter, 1.0, start, normal, tangent);
    let end = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[WORLD_ORBIT_INTERVALS]).read() };
    let target = remaining + end.distance;
    let terminal_speed = end.metric * WORLD_CURVE_AIRCRAFT_PHASE_RATE;
    unsafe {
        core::ptr::addr_of_mut!(WORLD_BRAKE_CAPTURE_DISTANCE).write(remaining);
        core::ptr::addr_of_mut!(WORLD_BRAKE_LOG_RATIO).write(brake_log(terminal_speed / -TRAJECTORY_ENTRY_RATE));
    }
    // The curve stays fixed. Solve just the broad log-speed envelope so its
    // integrated distance reaches that curve's endpoint at aircraft speed.
    let mut lower = 0.1;
    let mut upper = 16.0;
    let mut iteration = 0;
    while iteration < 48 {
        let bias = (lower + upper) * 0.5;
        unsafe { core::ptr::addr_of_mut!(WORLD_BRAKE_BIAS).write(bias); }
        if integrate_world_brake(false) < target { lower = bias; } else { upper = bias; }
        iteration += 1;
    }
    unsafe { core::ptr::addr_of_mut!(WORLD_BRAKE_BIAS).write((lower + upper) * 0.5); }
    let covered = integrate_world_brake(true);
    // Never clamp to an infeasible fit and then jump to the endpoint at 40 s.
    if (covered - target).abs() > 1.0e-9 {
        write_all(b"trajectory initialization failed: orbital braking distance cannot reach its endpoint\n");
        exit(181);
    }
    unsafe { core::ptr::addr_of_mut!(WORLD_ORBIT_COMPLETION_TIME).write(WORLD_BRAKE_END); }
}

pub(super) fn world_curve_regional_phase(elapsed: f64) -> f64 {
    let time = elapsed.max(0.0);
    let u = ((time - 2.0) / 16.0).clamp(0.0, 1.0);
    let u2 = u * u;
    let integral = 16.0 * u2 * u2 * (2.5 + u * (-3.0 + u)) + (time - 18.0).max(0.0);
    WORLD_CURVE_AIRCRAFT_PHASE_RATE * time
        + (WORLD_CURVE_GROUND_PHASE_RATE - WORLD_CURVE_AIRCRAFT_PHASE_RATE) * integral
}

pub(super) fn world_curve_late_clearance(time: f64) -> f64 {
    let airplane = miles_to_world(FLOW_AIRPLANE_ALTITUDE_MILES);
    // Two seconds at aircraft height, then a continuous sixteen-second
    // descent to the same displaced terrain used by the renderer.
    let descent = smootherstep_f64((time - flow_high_pass_end()) / 16.0);
    airplane + (FLOW_FINAL_ALTITUDE_WORLD - airplane) * descent
}
