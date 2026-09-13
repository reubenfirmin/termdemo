//! ARCHIVED phase-1 implementation. Not compiled or used by the active flight.
use super::{
    DVec3, FINAL_ALTITUDE_METERS, FLOW_PLANET_RADIUS_WORLD, TRAJECTORY_HZ, TRAJECTORY_STEP,
    TRAJECTORY_TABLE_END, Vec3, WORLD_BRAKE_CAPTURE_DISTANCE, WORLD_BRAKE_DISTANCE,
    WORLD_BRAKE_END, WORLD_BRAKE_END_TICK, WORLD_BRAKE_START_TICK, WORLD_BRAKE_STEP,
    WORLD_BRAKE_SUBSTEPS, WORLD_ORBIT_END_PHASE, approach_camera_reference,
    configure_world_brake, cubic_component, dvec_add, dvec_dot, dvec_length, dvec_normalize,
    dvec_scale, dvec_sub, flow_field_basis, flow_normal_to_planet, flow_surface_relief_world,
    generate_world_orbit_arc, legacy_trajectory_state, quintic_component, sine_sample_c2_state,
    smootherstep_f64, vec3_from_dvec, world_curve_capture_advance, world_curve_capture_frame,
    world_curve_capture_geometry, world_curve_controlled_speed, world_curve_late_clearance,
    world_curve_orbit_length, world_curve_orbit_phase, world_curve_regional_frame,
    world_curve_regional_phase, world_curve_spiral_metric, world_curve_spiral_position,
};

pub(super) static mut WORLD_ORBIT_COMPLETION_TIME: f64 = 60.0;
pub(super) fn flow_aircraft_arrival() -> f64 { unsafe { core::ptr::addr_of!(WORLD_ORBIT_COMPLETION_TIME).read() } }
pub(super) fn flow_high_pass_end() -> f64 { flow_aircraft_arrival() + 2.0 }
pub(super) fn flow_descent_end() -> f64 { flow_aircraft_arrival() + 18.0 }
pub const FLOW_PATH_END: f64 = 137.0;
pub(super) fn flow_valley_overhead() -> f64 { world_curve_altitude_time(1_500.0) }
pub(super) fn flow_valley_corridor() -> f64 { world_curve_altitude_time(200.0) }
// Resolve the fixed orbit's tightest curvature without changing the
// canonical opening table or doing integration in the render loop.
pub(super) const WORLD_CURVE_HZ: usize = 960;
pub(super) const WORLD_CURVE_STEP: f64 = 1.0 / WORLD_CURVE_HZ as f64;
pub(super) const WORLD_CURVE_SAMPLES: usize = WORLD_BRAKE_END_TICK
    + (TRAJECTORY_TABLE_END - 40) * TRAJECTORY_HZ + 1;

#[derive(Clone, Copy)]
pub(super) struct TrajectoryState {
    pub(super) position: DVec3,
    pub(super) velocity: DVec3,
    pub(super) acceleration: DVec3,
    pub(super) forward: Vec3,
    pub(super) down: Vec3,
    pub(super) roll: f64,
    #[cfg(feature = "phase0-audit")]
    pub(super) clearance_miles: f64,
    pub(super) phase: f64,
    pub(super) phase_rate: f64,
}

#[derive(Clone, Copy)]
pub(super) struct WorldCurveNode {
    pub(super) position: DVec3,
    pub(super) velocity: DVec3,
    pub(super) acceleration: DVec3,
    pub(super) phase: f64,
    pub(super) phase_rate: f64,
}

pub(super) const EMPTY_WORLD_CURVE_NODE: WorldCurveNode = WorldCurveNode {
    position: DVec3 { x: 0.0, y: 0.0, z: 0.0 },
    velocity: DVec3 { x: 0.0, y: 0.0, z: 0.0 },
    acceleration: DVec3 { x: 0.0, y: 0.0, z: 0.0 },
    phase: 0.0,
    phase_rate: 0.0,
};

pub(super) static mut WORLD_CURVE: [WorldCurveNode; WORLD_CURVE_SAMPLES] =
    [EMPTY_WORLD_CURVE_NODE; WORLD_CURVE_SAMPLES];

pub(super) const WORLD_CURVE_START: f64 = 20.5;
pub(super) fn world_curve_spiral_motion(phase: f64, time: f64) -> (DVec3, DVec3) {
    let tangent_at = |p: f64| dvec_normalize(dvec_sub(
        world_curve_spiral_position(p + 0.01), world_curve_spiral_position(p - 0.01)));
    let tangent = tangent_at(phase);
    let remaining = unsafe { core::ptr::addr_of!(WORLD_BRAKE_CAPTURE_DISTANCE).read() };
    let distance = remaining + world_curve_orbit_length(phase);
    let speed = world_curve_controlled_speed(time, distance);
    let dt = 0.000_1;
    let acceleration = (world_curve_controlled_speed(time + dt, distance + speed * dt)
        - world_curve_controlled_speed(time - dt, distance - speed * dt)) / (2.0 * dt);
    let curvature = dvec_scale(dvec_sub(tangent_at(phase + 0.01), tangent_at(phase - 0.01)),
        1.0 / (0.02 * world_curve_spiral_metric(phase)));
    (dvec_scale(tangent, speed),
        dvec_add(dvec_scale(tangent, acceleration), dvec_scale(curvature, speed * speed)))
}

pub(super) fn world_curve_late_normal(time: f64) -> DVec3 {
    let (normal, tangent) = world_curve_regional_frame();
    let phase = world_curve_regional_phase(time - flow_aircraft_arrival());
    dvec_normalize(dvec_add(
        dvec_scale(normal, sine_sample_c2_state(phase + 256.0).0),
        dvec_scale(tangent, sine_sample_c2_state(phase).0),
    ))
}

pub(super) fn world_curve_altitude_time(metres: f64) -> f64 {
    let mut lower = 0.0;
    let mut upper = 1.0;
    let amount = (9_144.0 - metres) / (9_144.0 - FINAL_ALTITUDE_METERS);
    let mut i = 0;
    while i < 40 {
        let middle = (lower + upper) * 0.5;
        if smootherstep_f64(middle) < amount { lower = middle; } else { upper = middle; }
        i += 1;
    }
    flow_high_pass_end() + 16.0 * (lower + upper) * 0.5
}

pub(super) fn generate_world_curve() {
    generate_world_orbit_arc();
    let capture_start = legacy_trajectory_state(WORLD_CURVE_START);
    let (orbit_normal, orbit_tangent) = world_curve_capture_frame();
    let mut parameter = 0.0f64;
    let mut brake_parameter = 0.0f64;
    let substep = WORLD_BRAKE_STEP;
    let mut index = 0usize;
    while index < WORLD_CURVE_SAMPLES {
        let time = world_curve_sample_time(index);
        if index == WORLD_BRAKE_START_TICK {
            brake_parameter = parameter;
            configure_world_brake(parameter, capture_start, orbit_normal, orbit_tangent);
        }
        let (position, phase) = if time < WORLD_CURVE_START {
            (legacy_trajectory_state(time).position, 0.0)
        } else if index < WORLD_BRAKE_START_TICK {
            (world_curve_capture_geometry(parameter, capture_start, orbit_normal, orbit_tangent).0, 0.0)
        } else if index < WORLD_BRAKE_END_TICK {
            let sample = (index - WORLD_BRAKE_START_TICK) * WORLD_BRAKE_SUBSTEPS;
            let distance = unsafe { core::ptr::addr_of!(WORLD_BRAKE_DISTANCE[sample]).read() };
            let remaining = unsafe { core::ptr::addr_of!(WORLD_BRAKE_CAPTURE_DISTANCE).read() };
            if distance <= remaining {
                let u = if distance == 0.0 { brake_parameter } else {
                    world_curve_capture_advance(brake_parameter, distance,
                        capture_start, orbit_normal, orbit_tangent).0
                };
                (world_curve_capture_geometry(u, capture_start, orbit_normal, orbit_tangent).0, 0.0)
            } else {
                let phase = world_curve_orbit_phase(distance - remaining);
                (world_curve_spiral_position(phase), phase)
            }
        } else {
            let normal = world_curve_late_normal(time);
            let relief = flow_surface_relief_world(flow_normal_to_planet(normal));
            let radius = FLOW_PLANET_RADIUS_WORLD + relief + world_curve_late_clearance(time);
            (dvec_scale(normal, radius),
                WORLD_ORBIT_END_PHASE + world_curve_regional_phase(time - flow_aircraft_arrival()))
        };
        unsafe {
            core::ptr::addr_of_mut!(WORLD_CURVE[index]).write(WorldCurveNode {
                position,
                velocity: DVec3 { x: 0.0, y: 0.0, z: 0.0 },
                acceleration: DVec3 { x: 0.0, y: 0.0, z: 0.0 },
                phase, phase_rate: 0.0,
            })
        };
        if time >= WORLD_CURVE_START && index < WORLD_BRAKE_START_TICK {
            let mut part = 0;
            while part < WORLD_BRAKE_SUBSTEPS {
                let sample_time = time + part as f64 * substep;
                let speed = -approach_camera_reference(sample_time).1;
                parameter = world_curve_capture_advance(parameter, speed * substep,
                    capture_start, orbit_normal, orbit_tangent).0;
                part += 1;
            }
        }
        index += 1;
    }
    index = 0;
    while index < WORLD_CURVE_SAMPLES {
        let before = if index == 0 { 0 } else { index - 1 };
        let after = if index + 1 >= WORLD_CURVE_SAMPLES {
            WORLD_CURVE_SAMPLES - 1
        } else {
            index + 1
        };
        let duration = world_curve_sample_time(after) - world_curve_sample_time(before);
        let first = unsafe { core::ptr::addr_of!(WORLD_CURVE[before]).read() };
        let middle = unsafe { core::ptr::addr_of!(WORLD_CURVE[index]).read() };
        let last = unsafe { core::ptr::addr_of!(WORLD_CURVE[after]).read() };
        let velocity = dvec_scale(dvec_sub(last.position, first.position), 1.0 / duration);
        let phase_rate = (last.phase - first.phase) / duration;
        unsafe {
            core::ptr::addr_of_mut!(WORLD_CURVE[index]).write(WorldCurveNode {
                velocity,
                phase_rate,
                ..middle
            })
        };
        index += 1;
    }
    index = 1;
    while index + 1 < WORLD_CURVE_SAMPLES {
        let first = unsafe { core::ptr::addr_of!(WORLD_CURVE[index - 1]).read() };
        let middle = unsafe { core::ptr::addr_of!(WORLD_CURVE[index]).read() };
        let last = unsafe { core::ptr::addr_of!(WORLD_CURVE[index + 1]).read() };
        let h1 = world_curve_sample_time(index) - world_curve_sample_time(index - 1);
        let h2 = world_curve_sample_time(index + 1) - world_curve_sample_time(index);
        let left = dvec_scale(dvec_sub(middle.position, first.position), 1.0 / h1);
        let right = dvec_scale(dvec_sub(last.position, middle.position), 1.0 / h2);
        let velocity = dvec_scale(dvec_add(dvec_scale(left, h2), dvec_scale(right, h1)), 1.0 / (h1 + h2));
        let acceleration = dvec_scale(dvec_sub(right, left), 2.0 / (h1 + h2));
        unsafe {
            core::ptr::addr_of_mut!(WORLD_CURVE[index]).write(WorldCurveNode {
                velocity,
                acceleration,
                ..middle
            })
        };
        index += 1;
    }
    index = 2;
    while index + 2 < WORLD_CURVE_SAMPLES {
        if index.abs_diff(WORLD_BRAKE_END_TICK) < 2 {
            index += 1;
            continue;
        }
        let step = world_curve_sample_time(index + 1) - world_curve_sample_time(index);
        let minus_two = unsafe { core::ptr::addr_of!(WORLD_CURVE[index - 2]).read() };
        let minus_one = unsafe { core::ptr::addr_of!(WORLD_CURVE[index - 1]).read() };
        let middle = unsafe { core::ptr::addr_of!(WORLD_CURVE[index]).read() };
        let plus_one = unsafe { core::ptr::addr_of!(WORLD_CURVE[index + 1]).read() };
        let plus_two = unsafe { core::ptr::addr_of!(WORLD_CURVE[index + 2]).read() };
        let velocity = dvec_scale(
            dvec_add(
                dvec_scale(plus_two.position, -1.0),
                dvec_add(
                    dvec_scale(plus_one.position, 8.0),
                    dvec_add(
                        dvec_scale(minus_one.position, -8.0),
                        minus_two.position,
                    ),
                ),
            ),
            1.0 / (12.0 * step),
        );
        let acceleration = dvec_scale(
            dvec_add(
                dvec_scale(plus_two.position, -1.0),
                dvec_add(
                    dvec_scale(plus_one.position, 16.0),
                    dvec_add(
                        dvec_scale(middle.position, -30.0),
                        dvec_add(
                            dvec_scale(minus_one.position, 16.0),
                            dvec_scale(minus_two.position, -1.0),
                        ),
                    ),
                ),
            ),
            1.0 / (12.0 * step * step),
        );
        let phase_rate = (-plus_two.phase + 8.0 * plus_one.phase
            - 8.0 * minus_one.phase
            + minus_two.phase)
            / (12.0 * step);
        unsafe {
            core::ptr::addr_of_mut!(WORLD_CURVE[index]).write(WorldCurveNode {
                velocity,
                acceleration,
                phase_rate,
                ..middle
            })
        };
        index += 1;
    }

    // Phase is the monotone spatial coordinate along the orbit. Give its
    // samples monotone cubic slopes so interpolation cannot invent a reverse
    // at the exact capture-to-orbit join.
    // Derivatives follow the same spatial curve and distance law. A wide
    // temporal stencil aliases the tightest bend and invents speed ripples.
    index = 1;
    while index + 1 < WORLD_CURVE_SAMPLES {
        let node = unsafe { core::ptr::addr_of!(WORLD_CURVE[index]).read() };
        if node.phase > 0.0 && node.phase < WORLD_ORBIT_END_PHASE {
            let (velocity, acceleration) = world_curve_spiral_motion(node.phase, world_curve_sample_time(index));
            unsafe { core::ptr::addr_of_mut!(WORLD_CURVE[index]).write(WorldCurveNode {
                velocity, acceleration, ..node
            }); }
        }
        index += 1;
    }
    let first = unsafe { core::ptr::addr_of!(WORLD_CURVE[0]).read() };
    let second = unsafe { core::ptr::addr_of!(WORLD_CURVE[1]).read() };
    unsafe {
        core::ptr::addr_of_mut!(WORLD_CURVE[0]).write(WorldCurveNode {
            phase_rate: (second.phase - first.phase) / WORLD_CURVE_STEP,
            ..first
        })
    };
    index = 1;
    while index + 1 < WORLD_CURVE_SAMPLES {
        let before = unsafe { core::ptr::addr_of!(WORLD_CURVE[index - 1]).read() };
        let middle = unsafe { core::ptr::addr_of!(WORLD_CURVE[index]).read() };
        let after = unsafe { core::ptr::addr_of!(WORLD_CURVE[index + 1]).read() };
        let left = (middle.phase - before.phase)
            / (world_curve_sample_time(index) - world_curve_sample_time(index - 1));
        let right = (after.phase - middle.phase)
            / (world_curve_sample_time(index + 1) - world_curve_sample_time(index));
        let phase_rate = if left > 0.0 && right > 0.0 {
            2.0 * left * right / (left + right)
        } else {
            0.0
        };
        unsafe {
            core::ptr::addr_of_mut!(WORLD_CURVE[index]).write(WorldCurveNode {
                phase_rate,
                ..middle
            })
        };
        index += 1;
    }
    let last_index = WORLD_CURVE_SAMPLES - 1;
    let before = unsafe { core::ptr::addr_of!(WORLD_CURVE[last_index - 1]).read() };
    let last = unsafe { core::ptr::addr_of!(WORLD_CURVE[last_index]).read() };
    unsafe {
        core::ptr::addr_of_mut!(WORLD_CURVE[last_index]).write(WorldCurveNode {
            phase_rate: (last.phase - before.phase) / TRAJECTORY_STEP,
            ..last
        })
    };
}

pub(super) fn world_curve_sample_time(index: usize) -> f64 {
    if index <= WORLD_BRAKE_END_TICK { index as f64 * WORLD_CURVE_STEP }
    else { WORLD_BRAKE_END + (index - WORLD_BRAKE_END_TICK) as f64 * TRAJECTORY_STEP }
}

pub(super) fn trajectory_state(time: f64) -> TrajectoryState {
    // Preserve the approved opening and approach exactly. The capture curve
    // starts from this function's complete world-space state at 20.5 seconds.
    if time < WORLD_CURVE_START {
        return legacy_trajectory_state(time);
    }
    let time = time.clamp(0.0, TRAJECTORY_TABLE_END as f64);
    let (scaled, step) = if time < WORLD_BRAKE_END {
        (time * WORLD_CURVE_HZ as f64, WORLD_CURVE_STEP)
    } else {
        (WORLD_BRAKE_END_TICK as f64 + (time - WORLD_BRAKE_END) * TRAJECTORY_HZ as f64, TRAJECTORY_STEP)
    };
    let mut index = scaled as usize;
    if index + 1 >= WORLD_CURVE_SAMPLES {
        index = WORLD_CURVE_SAMPLES - 2;
    }
    let progress = scaled - index as f64;
    let first = unsafe { core::ptr::addr_of!(WORLD_CURVE[index]).read() };
    let second = unsafe { core::ptr::addr_of!(WORLD_CURVE[index + 1]).read() };
    let component = |a: f64, b: f64, va: f64, vb: f64, aa: f64, ab: f64| {
        quintic_component(progress, step, a, b, va, vb, aa, ab)
    };
    let x = component(
        first.position.x,
        second.position.x,
        first.velocity.x,
        second.velocity.x,
        first.acceleration.x,
        second.acceleration.x,
    );
    let y = component(
        first.position.y,
        second.position.y,
        first.velocity.y,
        second.velocity.y,
        first.acceleration.y,
        second.acceleration.y,
    );
    let z = component(
        first.position.z,
        second.position.z,
        first.velocity.z,
        second.velocity.z,
        first.acceleration.z,
        second.acceleration.z,
    );
    let phase = cubic_component(
        progress,
        step,
        first.phase,
        second.phase,
        first.phase_rate,
        second.phase_rate,
    );
    let position = DVec3 { x: x.0, y: y.0, z: z.0 };
    let velocity = DVec3 { x: x.1, y: y.1, z: z.1 };
    let acceleration = DVec3 { x: x.2, y: y.2, z: z.2 };
    let forward = dvec_normalize(velocity);
    let toward_planet = dvec_normalize(dvec_scale(position, -1.0));
    let mut down = dvec_sub(
        toward_planet,
        dvec_scale(forward, dvec_dot(toward_planet, forward)),
    );
    if dvec_length(down) <= 1.0e-9 {
        down = flow_field_basis().2;
    }
    #[cfg(feature = "phase0-audit")]
    let surface_normal = flow_normal_to_planet(dvec_normalize(position));
    #[cfg(feature = "phase0-audit")]
    let clearance = dvec_length(position)
        - FLOW_PLANET_RADIUS_WORLD
        - flow_surface_relief_world(surface_normal);
    TrajectoryState {
        position,
        velocity,
        acceleration,
        forward: vec3_from_dvec(forward),
        down: vec3_from_dvec(dvec_normalize(down)),
        roll: 0.0,
        #[cfg(feature = "phase0-audit")]
        clearance_miles: world_to_miles(clearance),
        phase: phase.0,
        phase_rate: phase.1,
    }
}


#[cfg(feature = "phase0-audit")]
use super::world_to_miles;
