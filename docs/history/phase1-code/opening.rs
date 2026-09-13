//! ARCHIVED phase-1 implementation. Not compiled or used by the active flight.
//! Legacy-named equations here are STILL PRODUCTION dependencies, not audit-only.
use super::{
    DVec3, FLOW_AIRPLANE_ALTITUDE_MILES, FLOW_FINAL_ALTITUDE_WORLD, FLOW_PLANET_RADIUS_WORLD,
    GRID_APPROACH_AXIS_OFFSET, METERS_PER_MILE, ORBIT_ENTRY, STARFIELD_HOLD,
    STARFLIGHT_START_DISTANCE_WORLD, TrajectoryState, WORLD_CURVE_START, dvec_add, dvec_dot,
    dvec_length, dvec_normalize, dvec_scale, dvec_sub, flow_aircraft_arrival, flow_descent_end,
    flow_field_basis, flow_high_pass_end, flow_valley_corridor, flow_valley_overhead,
    generate_world_curve, hermite_acceleration, miles_to_world, quintic_component,
    sine_sample_c2_state, smootherstep_f64, smootherstep_state, sqrt_f64, vec3_from_dvec,
};

pub(super) const STARFLIGHT_JOIN_TIME: f64 = STARFIELD_HOLD as f64;
pub const FLOW_DESCENT_START: f64 = 35.0;
pub(super) const FLOW_CAPTURE_IMPACT_PEAK: f64 = 31.5;
pub(super) const FLOW_CAPTURE_IMPACT_END: f64 = FLOW_ORBIT_TWO_END;
pub(super) const FLOW_BRAKE_CONTROL: f64 = 38.0;
pub const FLOW_ORBIT_TWO_END: f64 = 42.0;
pub(super) const TRAJECTORY_HZ: usize = 120;
pub(super) const TRAJECTORY_STEP: f64 = 1.0 / TRAJECTORY_HZ as f64;
pub(super) const TRAJECTORY_TABLE_END: usize = 140;
pub(super) const TRAJECTORY_SAMPLES: usize = TRAJECTORY_TABLE_END * TRAJECTORY_HZ + 1;
#[derive(Clone, Copy)]
pub(super) struct TrajectoryNode {
    pub(super) clearance: f64,
    pub(super) clearance_rate: f64,
    pub(super) phase: f64,
    pub(super) phase_rate: f64,
    pub(super) meridian: f64,
    pub(super) meridian_rate: f64,
    pub(super) impact: f64,
    pub(super) impact_rate: f64,
}

pub(super) const EMPTY_TRAJECTORY_NODE: TrajectoryNode = TrajectoryNode {
    clearance: 0.0,
    clearance_rate: 0.0,
    phase: 0.0,
    phase_rate: 0.0,
    meridian: 0.0,
    meridian_rate: 0.0,
    impact: 0.0,
    impact_rate: 0.0,
};

pub(super) static mut TRAJECTORY: [TrajectoryNode; TRAJECTORY_SAMPLES] =
    [EMPTY_TRAJECTORY_NODE; TRAJECTORY_SAMPLES];

// zero and continuous force controls; no later milestone installs a position,
// velocity, camera frame, or alternate path equation.
pub const TRAJECTORY_ENTRY_CLEARANCE: f64 = 6.832_841_017_962_964;
pub(super) const TRAJECTORY_ENTRY_RATE: f64 = -13.258_229_140_114_77;
pub(super) const TRAJECTORY_ENTRY_MERIDIAN: f64 = -19.695_327_202_909_073;
pub(super) const TRAJECTORY_CAPTURE_ASYMPTOTE: f64 = 0.140_199_667_262_364_5;
pub(super) const TRAJECTORY_CAPTURE_CLEARANCE: f64 = 0.3375;
pub(super) const TRAJECTORY_CAPTURE_RATE: f64 = -0.042_701_732_517_378_695;
pub(super) const TRAJECTORY_CAPTURE_ACCELERATION: f64 = 0.011_677_464_087_222_941;

pub(super) fn trajectory_radial_reference(time: f64) -> (f64, f64, f64) {
    let airplane = miles_to_world(FLOW_AIRPLANE_ALTITUDE_MILES);
    let overhead = miles_to_world(1_500.0 / METERS_PER_MILE);
    let corridor = miles_to_world(200.0 / METERS_PER_MILE);
    let endpoint = FLOW_FINAL_ALTITUDE_WORLD;
    let orbit_descent = quintic_component(
        ((time - FLOW_DESCENT_START) / (flow_aircraft_arrival() - FLOW_DESCENT_START))
            .clamp(0.0, 1.0),
        flow_aircraft_arrival() - FLOW_DESCENT_START,
        TRAJECTORY_CAPTURE_CLEARANCE,
        airplane,
        TRAJECTORY_CAPTURE_RATE,
        0.0,
        TRAJECTORY_CAPTURE_ACCELERATION,
        0.0,
    );
    let overhead_descent = smootherstep_state(time, flow_high_pass_end(), flow_valley_overhead());
    let corridor_descent = smootherstep_state(time, flow_valley_overhead(), flow_valley_corridor());
    let final_descent = smootherstep_state(time, flow_valley_corridor(), flow_descent_end());
    (
        orbit_descent.0
            + (overhead - airplane) * overhead_descent.0
            + (corridor - overhead) * corridor_descent.0
            + (endpoint - corridor) * final_descent.0,
        orbit_descent.1
            + (overhead - airplane) * overhead_descent.1
            + (corridor - overhead) * corridor_descent.1
            + (endpoint - corridor) * final_descent.1,
        orbit_descent.2
            + (overhead - airplane) * overhead_descent.2
            + (corridor - overhead) * corridor_descent.2
            + (endpoint - corridor) * final_descent.2,
    )
}

// Legacy controls below remain only inside the pre-Phase-3L integrator that
// reproduces the approved opening state. The active world curve and camera do
// not read them after their exact 20.5-second join.
pub(super) fn capture_impact_acceleration(time: f64) -> f64 {
    let velocity_rise = smootherstep_state(
        time,
        ORBIT_ENTRY as f64,
        FLOW_CAPTURE_IMPACT_PEAK,
    );
    let velocity_fall = smootherstep_state(
        time,
        FLOW_CAPTURE_IMPACT_PEAK,
        FLOW_CAPTURE_IMPACT_END,
    );
    let peak_velocity = 2.0 * GRID_APPROACH_AXIS_OFFSET
        / (FLOW_CAPTURE_IMPACT_END - ORBIT_ENTRY as f64);
    peak_velocity * (velocity_rise.1 - velocity_fall.1)
}

pub(super) fn orbit_brake_amount(phase: f64) -> f64 {
    smootherstep_f64((phase - 512.0) / 1_536.0)
}

pub(super) fn trajectory_acceleration(time: f64, state: TrajectoryNode) -> (f64, f64, f64, f64) {
    let capture_acceleration = 2.0 * state.clearance_rate * state.clearance_rate
        / (state.clearance + FLOW_PLANET_RADIUS_WORLD - TRAJECTORY_CAPTURE_ASYMPTOTE)
            .max(1.0e-9);
    let intro_acceleration = approach_camera_reference(time).2;
    let capture = smootherstep_state(time, ORBIT_ENTRY as f64, 28.2).0;
    let approach_acceleration =
        intro_acceleration + (capture_acceleration - intro_acceleration) * capture;
    let reference = trajectory_radial_reference(time);
    let tracking_acceleration = reference.2
        - 6.0 * (state.clearance_rate - reference.1)
        - 9.0 * (state.clearance - reference.0);
    let tracking = smootherstep_state(time, 35.0, FLOW_BRAKE_CONTROL).0;
    let radial_acceleration =
        approach_acceleration + (tracking_acceleration - approach_acceleration) * tracking;

    let capture_spin = smootherstep_state(time, ORBIT_ENTRY as f64, 29.5).0;
    let first_orbit_brake = orbit_brake_amount(state.phase);
    let second_brake = smootherstep_state(time, 42.0, 60.0).0;
    let regional_brake = smootherstep_state(time, 55.0, 65.0).0;
    let second_rate = 78.500;
    let third_rate = 20.000;
    let regional_rate = 1.281_8;
    const PHASE_RADIANS_PER_UNIT: f64 = 0.006_135_923_151_542_565;
    let orbit_speed = 1.30 + (0.16 - 1.30) * first_orbit_brake;
    let orbit_rate = orbit_speed
        / ((FLOW_PLANET_RADIUS_WORLD + state.clearance) * PHASE_RADIANS_PER_UNIT);
    let later_rate = second_rate
        + (third_rate - second_rate) * second_brake
        + (regional_rate - third_rate) * regional_brake;
    let later_control = smootherstep_state(time, 42.0, 44.0).0;
    let phase_target = capture_spin
        * (orbit_rate + (later_rate - orbit_rate) * later_control);
    let braking_response = smootherstep_f64((state.phase - 512.0) / 512.0);
    let phase_gain = 5.0 + (2.5 - 5.0) * braking_response;
    let phase_acceleration = phase_gain * (phase_target - state.phase_rate);

    let meridian_acceleration = hermite_acceleration(
        time,
        flow_aircraft_arrival(),
        flow_descent_end(),
        TRAJECTORY_ENTRY_MERIDIAN,
        -6.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );
    let impact_acceleration = capture_impact_acceleration(time);
    (
        radial_acceleration,
        phase_acceleration,
        meridian_acceleration,
        impact_acceleration,
    )
}

pub(super) fn trajectory_rk4(time: f64, state: TrajectoryNode, step: f64) -> TrajectoryNode {
    let derivative = |sample_time: f64, sample: TrajectoryNode| {
        let acceleration = trajectory_acceleration(sample_time, sample);
        TrajectoryNode {
            clearance: sample.clearance_rate,
            clearance_rate: acceleration.0,
            phase: sample.phase_rate,
            phase_rate: acceleration.1,
            meridian: sample.meridian_rate,
            meridian_rate: acceleration.2,
            impact: sample.impact_rate,
            impact_rate: acceleration.3,
        }
    };
    let add = |a: TrajectoryNode, b: TrajectoryNode, scale: f64| TrajectoryNode {
        clearance: a.clearance + b.clearance * scale,
        clearance_rate: a.clearance_rate + b.clearance_rate * scale,
        phase: a.phase + b.phase * scale,
        phase_rate: a.phase_rate + b.phase_rate * scale,
        meridian: a.meridian + b.meridian * scale,
        meridian_rate: a.meridian_rate + b.meridian_rate * scale,
        impact: a.impact + b.impact * scale,
        impact_rate: a.impact_rate + b.impact_rate * scale,
    };
    let first = derivative(time, state);
    let second = derivative(time + step * 0.5, add(state, first, step * 0.5));
    let third = derivative(time + step * 0.5, add(state, second, step * 0.5));
    let fourth = derivative(time + step, add(state, third, step));
    TrajectoryNode {
        clearance: state.clearance
            + step
                * (first.clearance
                    + 2.0 * second.clearance
                    + 2.0 * third.clearance
                    + fourth.clearance)
                / 6.0,
        clearance_rate: state.clearance_rate
            + step
                * (first.clearance_rate
                    + 2.0 * second.clearance_rate
                    + 2.0 * third.clearance_rate
                    + fourth.clearance_rate)
                / 6.0,
        phase: state.phase
            + step * (first.phase + 2.0 * second.phase + 2.0 * third.phase + fourth.phase)
                / 6.0,
        phase_rate: state.phase_rate
            + step
                * (first.phase_rate
                    + 2.0 * second.phase_rate
                    + 2.0 * third.phase_rate
                    + fourth.phase_rate)
                / 6.0,
        meridian: state.meridian
            + step
                * (first.meridian
                    + 2.0 * second.meridian
                    + 2.0 * third.meridian
                    + fourth.meridian)
                / 6.0,
        meridian_rate: state.meridian_rate
            + step
                * (first.meridian_rate
                    + 2.0 * second.meridian_rate
                    + 2.0 * third.meridian_rate
                    + fourth.meridian_rate)
                / 6.0,
        impact: state.impact
            + step * (first.impact + 2.0 * second.impact + 2.0 * third.impact + fourth.impact)
                / 6.0,
        impact_rate: state.impact_rate
            + step
                * (first.impact_rate
                    + 2.0 * second.impact_rate
                    + 2.0 * third.impact_rate
                    + fourth.impact_rate)
                / 6.0,
    }
}

pub(super) fn generate_trajectory() {
    let approach = approach_camera_reference(0.0);
    let mut state = TrajectoryNode {
        clearance: approach.0,
        clearance_rate: approach.1,
        phase: 0.0,
        phase_rate: 0.0,
        meridian: TRAJECTORY_ENTRY_MERIDIAN,
        meridian_rate: 0.0,
        impact: -GRID_APPROACH_AXIS_OFFSET,
        impact_rate: 0.0,
    };
    unsafe { core::ptr::addr_of_mut!(TRAJECTORY[0]).write(state) };
    let mut index = 1usize;
    let approved_opening_end =
        (WORLD_CURVE_START * TRAJECTORY_HZ as f64) as usize + 2;
    while index <= approved_opening_end {
        let time = (index - 1) as f64 * TRAJECTORY_STEP;
        state = trajectory_rk4(time, state, TRAJECTORY_STEP);
        unsafe { core::ptr::addr_of_mut!(TRAJECTORY[index]).write(state) };
        index += 1;
    }
    generate_world_curve();
}

pub(super) fn decelerating_approach_component(
    time: f64,
    start_time: f64,
    end_time: f64,
    end_position: f64,
    start_speed: f64,
    end_speed: f64,
) -> (f64, f64, f64) {
    let duration = end_time - start_time;
    let start_position =
        end_position + duration * (start_speed + end_speed) * 0.5;
    if time <= start_time {
        return (
            start_position + (start_time - time) * start_speed,
            -start_speed,
            0.0,
        );
    }
    if time >= end_time {
        return (end_position, -end_speed, 0.0);
    }
    let x = (time - start_time) / duration;
    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x3 * x;
    let x5 = x4 * x;
    let smooth = x3 * (x * (x * 6.0 - 15.0) + 10.0);
    let smooth_integral = x4 * 2.5 - x5 * 3.0 + x5 * x;
    let smooth_derivative = 30.0 * x2 * (x - 1.0) * (x - 1.0);
    let speed_change = end_speed - start_speed;
    let forward_distance = duration
        * (start_speed * x + speed_change * smooth_integral);
    (
        start_position - forward_distance,
        -(start_speed + speed_change * smooth),
        -speed_change * smooth_derivative / duration,
    )
}

pub(super) fn trajectory_scalar_state(time: f64) -> (TrajectoryNode, (f64, f64, f64, f64)) {
    let scaled = time.clamp(0.0, TRAJECTORY_TABLE_END as f64) * TRAJECTORY_HZ as f64;
    let mut index = scaled as usize;
    if index + 1 >= TRAJECTORY_SAMPLES {
        index = TRAJECTORY_SAMPLES - 2;
    }
    let progress = scaled - index as f64;
    let first = unsafe { core::ptr::addr_of!(TRAJECTORY[index]).read() };
    let second = unsafe { core::ptr::addr_of!(TRAJECTORY[index + 1]).read() };
    let first_acceleration = trajectory_acceleration(index as f64 * TRAJECTORY_STEP, first);
    let second_acceleration =
        trajectory_acceleration((index + 1) as f64 * TRAJECTORY_STEP, second);
    let clearance = quintic_component(
        progress,
        TRAJECTORY_STEP,
        first.clearance,
        second.clearance,
        first.clearance_rate,
        second.clearance_rate,
        first_acceleration.0,
        second_acceleration.0,
    );
    let phase = quintic_component(
        progress,
        TRAJECTORY_STEP,
        first.phase,
        second.phase,
        first.phase_rate,
        second.phase_rate,
        first_acceleration.1,
        second_acceleration.1,
    );
    let meridian = quintic_component(
        progress,
        TRAJECTORY_STEP,
        first.meridian,
        second.meridian,
        first.meridian_rate,
        second.meridian_rate,
        first_acceleration.2,
        second_acceleration.2,
    );
    let impact = quintic_component(
        progress,
        TRAJECTORY_STEP,
        first.impact,
        second.impact,
        first.impact_rate,
        second.impact_rate,
        first_acceleration.3,
        second_acceleration.3,
    );
    (
        TrajectoryNode {
            clearance: clearance.0,
            clearance_rate: clearance.1,
            phase: phase.0,
            phase_rate: phase.1,
            meridian: meridian.0,
            meridian_rate: meridian.1,
            impact: impact.0,
            impact_rate: impact.1,
        },
        (clearance.2, phase.2, meridian.2, impact.2),
    )
}

pub(super) fn trajectory_direction(phase: f64, meridian: f64) -> DVec3 {
    let phase_sine = sine_sample_c2_state(phase).0;
    let phase_cosine = sine_sample_c2_state(phase + 256.0).0;
    let meridian_sine = sine_sample_c2_state(meridian).0;
    let meridian_cosine = sine_sample_c2_state(meridian + 256.0).0;
    dvec_normalize(DVec3 {
        x: meridian_cosine * phase_sine,
        y: meridian_sine,
        z: -meridian_cosine * phase_cosine,
    })
}

// Exact radial datum of the approved opening. It is part of the initial
// camera state; later terrain refinement must not move the starting flight.
pub(super) const APPROVED_APPROACH_RELIEF_WORLD: f64 = f64::from_bits(0x3edc_d567_e7fb_afa6);

pub(super) fn trajectory_relief_state(_time: f64) -> (f64, f64, f64) {
    (APPROVED_APPROACH_RELIEF_WORLD, 0.0, 0.0)
}

pub(super) fn approach_maneuver_state(time: f64) -> ((f64, f64, f64), (f64, f64, f64)) {
    let first_lateral = smootherstep_state(time, 20.5, 24.25);
    let second_lateral = smootherstep_state(time, 24.25, ORBIT_ENTRY as f64);
    let lateral = (
        first_lateral.0 - second_lateral.0,
        first_lateral.1 - second_lateral.1,
        first_lateral.2 - second_lateral.2,
    );
    let first_vertical = smootherstep_state(time, 21.5, 24.75);
    let second_vertical = smootherstep_state(time, 24.75, ORBIT_ENTRY as f64);
    let vertical = (
        (first_vertical.0 - second_vertical.0) * 0.45,
        (first_vertical.1 - second_vertical.1) * 0.45,
        (first_vertical.2 - second_vertical.2) * 0.45,
    );
    (lateral, vertical)
}

pub(super) fn legacy_trajectory_state(time: f64) -> TrajectoryState {
    let (state, acceleration) = trajectory_scalar_state(time);
    let phase_sine = sine_sample_c2_state(state.phase);
    let phase_cosine = sine_sample_c2_state(state.phase + 256.0);
    let meridian_sine = sine_sample_c2_state(state.meridian);
    let meridian_cosine = sine_sample_c2_state(state.meridian + 256.0);
    let compose = |sample: (f64, f64, f64), rate: f64, change: f64| {
        (
            sample.0,
            sample.1 * rate,
            sample.2 * rate * rate + sample.1 * change,
        )
    };
    let phase_sine = compose(phase_sine, state.phase_rate, acceleration.1);
    let phase_cosine = compose(phase_cosine, state.phase_rate, acceleration.1);
    let meridian_sine = compose(meridian_sine, state.meridian_rate, acceleration.2);
    let meridian_cosine = compose(meridian_cosine, state.meridian_rate, acceleration.2);
    let relief = trajectory_relief_state(time);
    let starflight = starflight_displacement(time);
    let radius = FLOW_PLANET_RADIUS_WORLD + state.clearance + relief.0 + starflight.0;
    let product = |a: (f64, f64, f64), b: (f64, f64, f64)| {
        (a.0 * b.0, a.1 * b.0 + a.0 * b.1, a.2 * b.0 + 2.0 * a.1 * b.1 + a.0 * b.2)
    };
    let raw_x = product(meridian_cosine, phase_sine);
    let raw_y = meridian_sine;
    let phase_z = product(meridian_cosine, phase_cosine);
    let raw_z = (-phase_z.0, -phase_z.1, -phase_z.2);
    let raw_length = sqrt_f64(
        raw_x.0 * raw_x.0 + raw_y.0 * raw_y.0 + raw_z.0 * raw_z.0,
    )
    .max(1.0e-12);
    let raw_length_rate =
        (raw_x.0 * raw_x.1 + raw_y.0 * raw_y.1 + raw_z.0 * raw_z.1) / raw_length;
    let raw_length_acceleration = (
        raw_x.1 * raw_x.1
            + raw_y.1 * raw_y.1
            + raw_z.1 * raw_z.1
            + raw_x.0 * raw_x.2
            + raw_y.0 * raw_y.2
            + raw_z.0 * raw_z.2
            - raw_length_rate * raw_length_rate
    ) / raw_length;
    let inverse_length = (
        1.0 / raw_length,
        -raw_length_rate / (raw_length * raw_length),
        2.0 * raw_length_rate * raw_length_rate
            / (raw_length * raw_length * raw_length)
            - raw_length_acceleration / (raw_length * raw_length),
    );
    let unit_x = product(raw_x, inverse_length);
    let unit_y = product(raw_y, inverse_length);
    let unit_z = product(raw_z, inverse_length);
    let radial = (
        radius,
        state.clearance_rate + relief.1 + starflight.1,
        acceleration.0 + relief.2 + starflight.2,
    );
    let x = product(radial, unit_x);
    let y = product(radial, unit_y);
    let z = product(radial, unit_z);
    let (_, field_right, field_down) = flow_field_basis();
    let mut position = dvec_add(
        DVec3 { x: x.0, y: y.0, z: z.0 },
        dvec_scale(field_down, state.impact),
    );
    let mut velocity = dvec_add(
        DVec3 { x: x.1, y: y.1, z: z.1 },
        dvec_scale(field_down, state.impact_rate),
    );
    let mut acceleration_world = dvec_add(
        DVec3 { x: x.2, y: y.2, z: z.2 },
        dvec_scale(field_down, acceleration.3),
    );
    let maneuver = approach_maneuver_state(time);
    position = dvec_add(
        position,
        dvec_add(
            dvec_scale(field_right, maneuver.0.0),
            dvec_scale(field_down, maneuver.1.0),
        ),
    );
    velocity = dvec_add(
        velocity,
        dvec_add(
            dvec_scale(field_right, maneuver.0.1),
            dvec_scale(field_down, maneuver.1.1),
        ),
    );
    acceleration_world = dvec_add(
        acceleration_world,
        dvec_add(
            dvec_scale(field_right, maneuver.0.2),
            dvec_scale(field_down, maneuver.1.2),
        ),
    );
    let radial_down = dvec_normalize(dvec_scale(position, -1.0));
    let forward_world = if dvec_length(velocity) > 1.0e-12 {
        dvec_normalize(velocity)
    } else {
        dvec_normalize(dvec_scale(position, -1.0))
    };
    let mut down_world = dvec_sub(
        radial_down,
        dvec_scale(forward_world, dvec_dot(radial_down, forward_world)),
    );
    if dvec_length(down_world) <= 1.0e-9 {
        down_world = DVec3 { x: 0.0, y: 1.0, z: 0.0 };
    }
    TrajectoryState {
        position,
        velocity,
        acceleration: acceleration_world,
        forward: vec3_from_dvec(forward_world),
        down: vec3_from_dvec(dvec_normalize(down_world)),
        roll: 0.0,
        #[cfg(feature = "phase0-audit")]
        clearance_miles: world_to_miles(state.clearance),
        phase: state.phase,
        phase_rate: state.phase_rate,
    }
}

pub(super) fn approach_camera_reference(time: f64) -> (f64, f64, f64) {
    decelerating_approach_component(
        time,
        STARFIELD_HOLD as f64,
        ORBIT_ENTRY as f64,
        TRAJECTORY_ENTRY_CLEARANCE,
        18.0,
        -TRAJECTORY_ENTRY_RATE,
    )
}

pub(super) fn starflight_displacement(time: f64) -> (f64, f64, f64) {
    if time >= STARFLIGHT_JOIN_TIME {
        return (0.0, 0.0, 0.0);
    }
    quintic_component(
        (time / STARFLIGHT_JOIN_TIME).clamp(0.0, 1.0),
        STARFLIGHT_JOIN_TIME,
        STARFLIGHT_START_DISTANCE_WORLD,
        0.0,
        -STARFLIGHT_START_DISTANCE_WORLD * 0.6 / STARFLIGHT_JOIN_TIME,
        0.0,
        0.0,
        0.0,
    )
}


#[cfg(feature = "phase0-audit")]
use super::world_to_miles;
