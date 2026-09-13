//! ARCHIVED phase-1 implementation. Not compiled or used by the active flight.
#[cfg(feature = "phase0-audit")]
use super::{
    DVec3, FEET_PER_MILE, FINAL_ALTITUDE_METERS, FLOW_AIRPLANE_ALTITUDE_MILES,
    FLOW_FINAL_ALTITUDE_WORLD, FLOW_FOCAL, FLOW_PATH_END, FLOW_PLANET_RADIUS_WORLD, FlowCamera,
    GRID_WELL_DEPTH_WORLD, HEIGHT, LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES, METERS_PER_MILE,
    ORBIT_ENTRY, PLANET_CENTER, PLANET_RADIUS_MILES, TRAJECTORY_ENTRY_RATE, TRAJECTORY_HZ,
    TRAJECTORY_STEP, Vec3, WIDTH, WORLD_BRAKE_CAPTURE_DISTANCE, WORLD_BRAKE_DISTANCE,
    WORLD_BRAKE_SAMPLES, WORLD_BRAKE_START, WORLD_CURVE_HZ, WORLD_CURVE_ORBIT_RADIUS,
    WORLD_CURVE_START, WORLD_CURVE_STEP, WORLD_ORBIT_ALIGNMENT_PHASE, WORLD_ORBIT_END_PHASE,
    append, append_number, approach_camera_reference, brake_log, dvec_add, dvec_dot,
    dvec_dot_vec3, dvec_length, dvec_length_squared, dvec_normalize, dvec_scale, dvec_sub,
    exit, field_exterior_visibility, field_floor_down, field_point, flow_aircraft_arrival,
    flow_atmosphere_amount, flow_camera, flow_camera_ray, flow_descent_end, flow_high_pass_end,
    flow_normal_to_planet, flow_sphere_roots, flow_sun_direction, flow_surface_color,
    flow_surface_map_from_normal, flow_surface_relief_world, flow_surface_sample,
    flow_valley_corridor, flow_valley_overhead, full_surface_lod, hash,
    legacy_trajectory_state, local_valley_distance, phase3_require, planet_normal_to_flow,
    sample_surface_vertex, surface_city_amount, surface_lod, trajectory_state, universe,
    universe_space_visibility, vec3_from_dvec, vec_dot, vec_length, vec_normalize, vec_sub,
    world_curve_approach_speed, world_curve_capture_frame, world_curve_capture_geometry,
    world_curve_final_orbit_length, world_curve_orbit_length, world_curve_spiral_base_radius,
    world_curve_spiral_metric, world_curve_spiral_position, world_to_meters, world_to_miles,
    write_all,
};

// User-approved fast first orbit: continuity is distinct from slow turning.
#[cfg(feature = "phase0-audit")]
pub(super) const FLIGHT_TURN_RATE_LIMIT: f64 = 1.25 * -TRAJECTORY_ENTRY_RATE / WORLD_CURVE_ORBIT_RADIUS;
#[cfg(feature = "phase0-audit")]
pub(super) const FLIGHT_TURN_ACCELERATION_LIMIT: f64 = 2.0 * TRAJECTORY_ENTRY_RATE * TRAJECTORY_ENTRY_RATE
    / (WORLD_CURVE_ORBIT_RADIUS * WORLD_CURVE_ORBIT_RADIUS);

#[cfg(feature = "phase0-audit")]
pub(super) fn phase0_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase0 motion audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase0_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase0_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase0_vec_close(a: Vec3, b: Vec3, tolerance: f32) -> bool {
    vec_length(vec_sub(a, b)) <= tolerance
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase0_dvec_close(a: DVec3, b: DVec3, tolerance_squared: f64) -> bool {
    dvec_length_squared(dvec_sub(a, b)) <= tolerance_squared
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase0_basis_valid(camera: FlowCamera) -> bool {
    let forward_squared = vec_dot(camera.forward, camera.forward);
    let right_squared = vec_dot(camera.right, camera.right);
    let down_squared = vec_dot(camera.down, camera.down);
    forward_squared.is_finite()
        && right_squared.is_finite()
        && down_squared.is_finite()
        && (forward_squared - 1.0).abs() < 0.025
        && (right_squared - 1.0).abs() < 0.025
        && (down_squared - 1.0).abs() < 0.025
        && vec_dot(camera.forward, camera.right).abs() < 0.025
        && vec_dot(camera.forward, camera.down).abs() < 0.025
        && vec_dot(camera.right, camera.down).abs() < 0.025
}

#[cfg(feature = "phase0-audit")]
pub(super) fn flight_volume_contains(point: DVec3) -> bool {
    let field = universe().field;
    let relative = dvec_sub(point, field.axis_origin);
    let s = dvec_dot(relative, field.forward);
    let right = dvec_dot(relative, field.right);
    let down = dvec_dot(relative, field.down);
    // Side and roof limits have no below-floor exception.
    if !s.is_finite() || !right.is_finite() || !down.is_finite()
        || s < field.field_start_s || s > field.field_end_s
        || right.abs() > field.radius || down < -field.radius
    {
        return false;
    }
    // Test the actual cross-section, including its circular/square morph and
    // displaced floor. An axis-aligned bounding box is not containment.
    let mut inside = false;
    let mut previous = dvec_sub(field_point(field, s, 0.0), field.axis_origin);
    let mut lane = 1;
    while lane <= 128 {
        let current = dvec_sub(field_point(field, s, lane as f64 / 128.0), field.axis_origin);
        let ax = dvec_dot(previous, field.right);
        let ay = dvec_dot(previous, field.down);
        let bx = dvec_dot(current, field.right);
        let by = dvec_dot(current, field.down);
        if (ay > down) != (by > down)
            && right < ax + (bx - ax) * (down - ay) / (by - ay)
        {
            inside = !inside;
        }
        previous = current;
        lane += 1;
    }
    if inside {
        return true;
    }
    // The only exterior volume is a planet-centred ball one tunnel
    // half-width in radius, restricted to below the undeformed floor.
    // In particular this cannot grant a side exit, an early departure from
    // the tunnel, or an arbitrarily deep excursion.
    down >= field.radius
        && dvec_length_squared(dvec_sub(point, universe().planet_center))
            <= field.radius * field.radius
}

#[cfg(feature = "phase0-audit")]
pub(super) fn flight_volume_regression_audit() {
    let field = universe().field;
    let at = |s: f64, right: f64, down: f64| dvec_add(field.axis_origin,
        dvec_add(dvec_scale(field.forward, s),
            dvec_add(dvec_scale(field.right, right), dvec_scale(field.down, down))));
    phase3_require(flight_volume_contains(at(-60.0, 0.0, 0.0))
        && flight_volume_contains(dvec_add(universe().planet_center,
            dvec_scale(field.down, FLOW_PLANET_RADIUS_WORLD * 2.0)))
        && !flight_volume_contains(at(-60.0, 0.0, field.radius + 0.1))
        && !flight_volume_contains(at(0.0, field.radius + 0.1, field.radius + 0.5))
        && !flight_volume_contains(at(0.0, 0.0, -field.radius - 0.1))
        && !flight_volume_contains(at(0.0, 0.0, field.radius * 4.0))
        && !flight_volume_contains(at(field.square_start_s - 1.0,
            field.radius * 0.8, -field.radius * 0.8)),
        229, b"flight-volume oracle admits a side/roof/deep/early exit or circular-corner escape");
    let mut tick = (WORLD_CURVE_START * TRAJECTORY_HZ as f64) as u32;
    while tick <= (FLOW_PATH_END * TRAJECTORY_HZ as f64) as u32 {
        let time = tick as f64 / TRAJECTORY_HZ as f64;
        if !flight_volume_contains(trajectory_state(time).position) {
            let mut report = [0u8; 96];
            let pointer = report.as_mut_ptr();
            let mut length = 0;
            append(pointer, &mut length, b"flight-volume violation at ms=");
            append_number(pointer, &mut length, (time * 1_000.0) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(230)
        }
        tick += 1;
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_flight_requirements_audit() -> ! {
    flight_volume_regression_audit();
    // Test the user's wall-clock deadline against actual world velocity,
    // independently of the phase/rate used to schedule the revolutions.
    let deadline_state = trajectory_state(40.0);
    let deadline_speed = world_to_meters(dvec_length(deadline_state.velocity));
    let mut speed_report = [0u8; 96];
    let speed_pointer = speed_report.as_mut_ptr();
    let mut speed_length = 0;
    append(speed_pointer, &mut speed_length, b"flight speed at 40s metres/second=");
    append_number(speed_pointer, &mut speed_length, deadline_speed as u32);
    append(speed_pointer, &mut speed_length, b"\n");
    write_all(&speed_report[..speed_length]);
    phase3_require((550.0..650.0).contains(&deadline_speed)
        && (9_000.0..9_300.0).contains(&(deadline_state.clearance_miles * METERS_PER_MILE)),
        246, b"camera has not reached fast-jet speed at airline height by 40 seconds");
    for time in [27.0, 28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0, 41.0] {
        let state = trajectory_state(time);
        let mut report = [0u8; 128];
        let pointer = report.as_mut_ptr();
        let mut length = 0;
        append(pointer, &mut length, b"flight arrival seconds/speed-mps/agl-metres=");
        for value in [time, world_to_meters(dvec_length(state.velocity)), state.clearance_miles * METERS_PER_MILE] {
            append_number(pointer, &mut length, value as u32);
            append(pointer, &mut length, b"/");
        }
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
    }
    for (time, minimum, maximum) in [
        (38.0, 650.0, 1_500.0), (39.0, 600.0, 700.0),
        (40.0, 550.0, 650.0), (41.0, 550.0, 650.0),
    ] {
        let state = trajectory_state(time);
        let speed = world_to_meters(dvec_length(state.velocity));
        phase3_require((minimum..maximum).contains(&speed)
            && (9_000.0..9_300.0).contains(&(state.clearance_miles * METERS_PER_MILE)),
            249, b"arrival lacks a sustained fast-jet slowdown at airline height");
    }
    let field = universe().field;
    let start = legacy_trajectory_state(WORLD_CURVE_START);
    let (normal, tangent) = world_curve_capture_frame();
    let mut previous_radius = dvec_length(start.position) + 1.0e-8;
    let mut maximum_entry_slope = 0.0f64;
    let mut previous_entry_down = 0.0f64;
    let mut previous_entry_tangent = dvec_normalize(start.velocity);
    let mut previous_entry_point = start.position;
    let mut i = 0u32;
    while i <= 4_096 {
        let (point, derivative) = world_curve_capture_geometry(i as f64 / 4_096.0, start, normal, tangent);
        let radius = dvec_length(point);
        phase3_require(flight_volume_contains(point),
            230, b"capture leaves the actual tunnel outside the localized under-planet volume");
        if radius > previous_radius + 1.0e-8 {
            let mut report = [0u8; 96];
            let pointer = report.as_mut_ptr();
            let mut length = 0;
            append(pointer, &mut length, b"entry radius reversal parameter/radius-micro=");
            append_number(pointer, &mut length, i);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, (radius * 1_000_000.0) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(231)
        }
        previous_radius = radius;
        let direction = dvec_normalize(derivative);
        let entry_down = dvec_dot(direction, field.down);
        phase3_require(entry_down >= previous_entry_down - 1.0e-10,
            244, b"capture contains an upward counter-turn before the downward orbit");
        if i > 0 {
            let curvature = dvec_length(dvec_sub(direction, previous_entry_tangent))
                / dvec_length(dvec_sub(point, previous_entry_point));
            phase3_require(curvature <= 1.0 / WORLD_CURVE_ORBIT_RADIUS + 0.001,
                244, b"capture curvature spikes beyond the joined orbit curvature");
        }
        previous_entry_down = entry_down;
        previous_entry_tangent = direction;
        previous_entry_point = point;
        maximum_entry_slope = maximum_entry_slope.max(entry_down.abs());
        i += 1;
    }
    phase3_require(maximum_entry_slope < 0.5
        && dvec_dot(tangent, field.down).abs() < 0.5,
        232, b"entry inclination reaches 30 degrees");
    let join_before = world_curve_capture_geometry(1.0 - 1.0e-5, start, normal, tangent);
    let join = world_curve_capture_geometry(1.0, start, normal, tangent);
    let join_curvature = dvec_scale(dvec_sub(dvec_normalize(join.1), dvec_normalize(join_before.1)),
        1.0 / dvec_length(dvec_sub(join.0, join_before.0)));
    phase3_require(dvec_length(dvec_sub(join.0, world_curve_spiral_position(0.0))) < 1.0e-10
        && dvec_dot(dvec_normalize(join.1), tangent) > 1.0 - 1.0e-12
        && dvec_length(dvec_add(join_curvature, dvec_scale(normal, 1.0 / WORLD_CURVE_ORBIT_RADIUS))) < 0.001,
        244, b"capture does not meet orbit position, tangent and curvature");
    // Sample both sides of every interpolation knot, plus inter-knot
    // midpoints. Fast continuous motion is permitted; impulses are not.
    let reference_acceleration = TRAJECTORY_ENTRY_RATE * TRAJECTORY_ENTRY_RATE / WORLD_CURVE_ORBIT_RADIUS;
    let reference_jerk = reference_acceleration * -TRAJECTORY_ENTRY_RATE / WORLD_CURVE_ORBIT_RADIUS;
    let mut maximum_acceleration = 0.0f64;
    let mut maximum_jerk = 0.0f64;
    let mut maximum_direction_rate = 0.0f64;
    let mut motion_tick = (WORLD_CURVE_START * WORLD_CURVE_HZ as f64 * 2.0) as u32;
    while motion_tick < (FLOW_PATH_END * WORLD_CURVE_HZ as f64 * 2.0) as u32 {
        let time = motion_tick as f64 * WORLD_CURVE_STEP * 0.5;
        let h = 0.000_1;
        let before = trajectory_state(time - h);
        let after = trajectory_state(time + h);
        maximum_acceleration = maximum_acceleration.max(dvec_length(dvec_sub(after.velocity, before.velocity)) / (2.0 * h));
        maximum_jerk = maximum_jerk.max(dvec_length(dvec_sub(after.acceleration, before.acceleration)) / (2.0 * h));
        maximum_direction_rate = maximum_direction_rate.max(dvec_length(dvec_sub(
            dvec_normalize(after.velocity), dvec_normalize(before.velocity))) / (2.0 * h));
        motion_tick += 1;
    }
    let mut motion_report = [0u8; 160];
    let motion_pointer = motion_report.as_mut_ptr();
    let mut motion_length = 0;
    append(motion_pointer, &mut motion_length, b"flight dynamics acceleration/jerk/direction-rate-milli=");
    for value in [maximum_acceleration, maximum_jerk, maximum_direction_rate] {
        append_number(motion_pointer, &mut motion_length, (value * 1_000.0) as u32);
        append(motion_pointer, &mut motion_length, b"/");
    }
    append(motion_pointer, &mut motion_length, b"\n");
    write_all(&motion_report[..motion_length]);
    phase3_require(maximum_acceleration < reference_acceleration * 2.0
        && maximum_jerk < reference_jerk * 4.0
        && maximum_direction_rate < FLIGHT_TURN_RATE_LIMIT,
        245, b"continuous fast orbit contains an acceleration, jerk or direction-rate spike");
    // Check every part of all three revolutions, not just their approach.
    let mut phase = 0.0;
    let mut minimum_ceiling_clearance = f64::MAX;
    while phase <= WORLD_ORBIT_END_PHASE {
        let point = world_curve_spiral_position(phase);
        phase3_require(flight_volume_contains(point),
            233, b"orbit leaves the actual tunnel outside the localized under-planet volume");
        let relief = flow_surface_relief_world(flow_normal_to_planet(dvec_normalize(point)));
        phase3_require(dvec_length(point) > FLOW_PLANET_RADIUS_WORLD + relief,
            242, b"orbit intersects displaced terrain");
        let down = dvec_dot(dvec_sub(point, field.axis_origin), field.down);
        minimum_ceiling_clearance = minimum_ceiling_clearance.min(down + field.radius);
        if down < -field.radius {
            let mut report = [0u8; 96];
            let pointer = report.as_mut_ptr();
            let mut length = 0;
            append(pointer, &mut length, b"orbit ceiling violation phase/height-milli=");
            append_number(pointer, &mut length, phase as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, (-down * 1_000.0) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(233)
        }
        phase += 0.25;
    }
    let mut last_depth = GRID_WELL_DEPTH_WORLD + 1.0;
    for radius in [0.0, 0.5, 1.0, 2.0, 4.0, 8.0] {
        let depth = field_floor_down(field, radius, 0.0) - field.radius;
        phase3_require(depth > 0.0 && depth < last_depth
            && (field_floor_down(field, -radius, 0.0) - field_floor_down(field, 0.0, radius)).abs() < 1.0e-12,
            234, b"gravity well lacks a continuous symmetric bowl");
        last_depth = depth;
    }
    phase3_require(field_floor_down(field, field.radius, 0.0) - field.radius > GRID_WELL_DEPTH_WORLD * 0.1
        && field_floor_down(field, 0.0, 0.0) - field_floor_down(field, FLOW_PLANET_RADIUS_WORLD, 0.0)
            > FLOW_PLANET_RADIUS_WORLD * 0.2,
        234, b"gravity well lacks either broad flanks or a pronounced planet-scale depression");
    let orbit_start = trajectory_phase_crossing(0.000_001, 28.0, 30.0);
    let orbit_one = trajectory_phase_crossing(1_024.0, orbit_start, 40.0);
    let orbit_two = trajectory_phase_crossing(2_048.0, orbit_one, 55.0);
    let orbit_three = flow_aircraft_arrival();
    let atmosphere_pass = trajectory_phase_crossing(WORLD_ORBIT_ALIGNMENT_PHASE, orbit_two, 40.0);
    let entry = trajectory_state(atmosphere_pass);
    let plane = trajectory_state(orbit_three);
    let ground = trajectory_state(flow_descent_end());
    // The deadline is physical speed AND displaced-surface altitude.
    phase3_require((3.0..=4.0).contains(&(29.158 - WORLD_BRAKE_START)),
        250, b"braking onset was not moved three to four seconds earlier");
    for time in [24.0, 25.0, WORLD_BRAKE_START - 0.1] {
        let reference = -approach_camera_reference(time).1;
        phase3_require(world_curve_approach_speed(time).to_bits() == reference.to_bits(),
            250, b"early brake changes speed before its authorized onset");
    }
    let expected_distance = unsafe { core::ptr::addr_of!(WORLD_BRAKE_CAPTURE_DISTANCE).read() }
        + world_curve_orbit_length(WORLD_ORBIT_END_PHASE);
    let integrated_distance = unsafe { core::ptr::addr_of!(WORLD_BRAKE_DISTANCE[WORLD_BRAKE_SAMPLES - 1]).read() };
    phase3_require((integrated_distance - expected_distance).abs() < 1.0e-9,
        250, b"braking integral does not reach the unchanged geometric endpoint continuously");
    let brake_start = WORLD_BRAKE_START;
    let mut brake_time = brake_start;
    let mut previous_speed = dvec_length(trajectory_state(brake_time).velocity);
    while brake_time < orbit_three {
        brake_time = (brake_time + WORLD_CURVE_STEP * 0.5).min(orbit_three);
        let speed = dvec_length(trajectory_state(brake_time).velocity);
        // Allow 10 ppm discretization error plus 1e-8 world units/s. Compare
        // against the running minimum so this allowance cannot accumulate
        // into a hidden acceleration over many samples.
        if speed > previous_speed + 1.0e-8 + previous_speed * 1.0e-5 {
            let mut report = [0u8; 160];
            let pointer = report.as_mut_ptr();
            let mut length = 0;
            append(pointer, &mut length, b"brake acceleration time-ms/phase/speed-micro/increase-nano=");
            for value in [brake_time * 1_000.0, trajectory_state(brake_time).phase,
                speed * 1_000_000.0, (speed - previous_speed) * 1_000_000_000.0] {
                append_number(pointer, &mut length, value as u32);
                append(pointer, &mut length, b"/");
            }
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            phase3_require(false, 247, b"actual camera speed increases during the orbital brake");
        }
        previous_speed = previous_speed.min(speed);
    }
    phase3_require((1.0..=1.6).contains(&(orbit_one - orbit_start)),
        235, b"earlier braking excessively delays the first orbit");
    phase3_require(entry.clearance_miles * METERS_PER_MILE < 100_000.0
        && entry.clearance_miles * METERS_PER_MILE > 18_000.0
        && flow_atmosphere_amount(&flow_camera(atmosphere_pass as f32)) > 0.0,
        235, b"last descending orbit starts outside the atmosphere");
    phase3_require(orbit_three - atmosphere_pass > 2.0 * (orbit_two - orbit_one)
        && (plane.clearance_miles * FEET_PER_MILE - 30_000.0).abs() < 2.0
        && world_to_meters(dvec_length(plane.velocity)) > 550.0
        && world_to_meters(dvec_length(plane.velocity)) < 650.0,
        236, b"final orbit fails long duration, airplane altitude, or aircraft ground speed");
    // Measure the distribution of fractional slowdown across the WHOLE
    // brake. A nominal early taper cannot conceal another late speed cliff.
    let initial_speed = dvec_length(trajectory_state(brake_start).velocity);
    let final_speed = dvec_length(plane.velocity);
    let total_log_drop = brake_log(initial_speed / final_speed);
    let speed_at = |t: f64| dvec_length(trajectory_state(t).velocity);
    phase3_require(brake_log(initial_speed / speed_at(35.0)) >= total_log_drop * 0.55
        && brake_log(speed_at(36.0) / speed_at(38.0)) <= total_log_drop * 0.25,
        249, b"braking is still concentrated near 36-38 seconds");
    let mut time = brake_start;
    while time <= orbit_three {
        let before = dvec_length(trajectory_state(time).velocity);
        let after = dvec_length(trajectory_state(time + 0.1).velocity);
        phase3_require(after >= before * 0.75,
            249, b"brake loses over 25 percent of speed within 100 ms");
        phase3_require(brake_log(before / speed_at((time + 2.0).min(orbit_three)))
            <= total_log_drop * 0.35,
            249, b"two seconds contain more than 35 percent of the logarithmic slowdown");
        time += TRAJECTORY_STEP;
    }
    // Independent finer quadrature checks the arc-length table, including
    // off-knot values, rather than comparing its inverse with itself alone.
    let mut fine_length = 0.0;
    let mut fine_phase = WORLD_ORBIT_ALIGNMENT_PHASE;
    let mut previous_metric = world_curve_spiral_metric(fine_phase);
    while fine_phase < WORLD_ORBIT_END_PHASE {
        let next = fine_phase + 0.25;
        let metric = world_curve_spiral_metric(next);
        fine_length += (previous_metric
            + 4.0 * world_curve_spiral_metric(fine_phase + 0.125) + metric) / 24.0;
        if (world_curve_final_orbit_length(next) - fine_length).abs() >= 1.0e-8 {
            let mut report = [0u8; 128];
            let pointer = report.as_mut_ptr();
            let mut length = 0;
            append(pointer, &mut length, b"arc phase/table-nano/fine-nano=");
            for value in [next, world_curve_final_orbit_length(next) * 1.0e9, fine_length * 1.0e9] {
                append_number(pointer, &mut length, value as u32);
                append(pointer, &mut length, b"/");
            }
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            phase3_require(false, 248, b"third-orbit arc length disagrees with independent finer quadrature");
        }
        fine_phase = next;
        previous_metric = metric;
    }
    phase3_require((15.0..=20.0).contains(&(flow_descent_end() - orbit_three))
        && (ground.clearance_miles * METERS_PER_MILE - FINAL_ALTITUDE_METERS).abs() < 0.5,
        237, b"low flight is not reached 15-20 seconds after airplane altitude");
    let mut tick = (orbit_start * TRAJECTORY_HZ as f64) as u32;
    let end_tick = (orbit_three * TRAJECTORY_HZ as f64) as u32;
    while tick <= end_tick {
        let camera = flow_camera(tick as f32 / TRAJECTORY_HZ as f32);
        phase3_require(phase0_basis_valid(camera), 238, b"camera frame degenerates during orbit");
        for x in [24.0, 92.0, 160.0, 228.0, 296.0] {
            let ray = flow_camera_ray(&camera, 1.0, x, 40.0);
            phase3_require(flow_sphere_roots(&camera, ray, FLOW_PLANET_RADIUS_WORLD).is_none(),
                239, b"planet hides the upper sky band");
        }
        tick += 4;
    }
    let mut time = flow_descent_end();
    while time <= FLOW_PATH_END {
        let camera = flow_camera(time as f32);
        let map = flow_surface_map_from_normal(flow_normal_to_planet(dvec_normalize(camera.position)));
        phase3_require(local_valley_distance(map.0, map.1) < LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES,
            240, b"low flight misses the canyon floor");
        time += 0.1;
    }
    let mut day_cities = 0u32;
    let mut night_cities = 0u32;
    i = 0;
    while i < 8_192 {
        let n = vec_normalize(Vec3 {
            x: (hash(i as i32, 237) as f32 + 0.5) / 128.0 - 1.0,
            y: (hash(i as i32, 683) as f32 + 0.5) / 128.0 - 1.0,
            z: (hash(i as i32, 991) as f32 + 0.5) / 128.0 - 1.0,
        });
        let sample = flow_surface_sample(n, full_surface_lod());
        if surface_city_amount(n, sample, full_surface_lod()) > 0.3 {
            let sunlight = vec_dot(vec3_from_dvec(planet_normal_to_flow(n)), flow_sun_direction());
            if sunlight > 0.2 { day_cities += 1; }
            if sunlight < -0.2 { night_cities += 1; }
            phase3_require(flow_surface_color(n, 0.0, sample, full_surface_lod())
                == flow_surface_color(n, 100.0, sample, full_surface_lod()),
                241, b"city identity changes with demo time");
        }
        i += 1;
    }
    phase3_require(day_cities > 0 && night_cities > 0, 241, b"day or night city population is missing");
    let aircraft_camera = flow_camera(orbit_three as f32 + 1.0);
    let mut visible_city_samples = 0;
    let mut surface_samples = 0;
    let mut y = 4;
    while y < HEIGHT {
        let mut x = 4;
        while x < WIDTH {
            let hit = sample_surface_vertex(&aircraft_camera, orbit_three as f32 + 1.0, 1.0, x as f32, y as f32);
            if hit.valid && hit.inverse_depth > 0.0 {
                surface_samples += 1;
                let ray = flow_camera_ray(&aircraft_camera, 1.0, x as f32, y as f32);
                let distance = 1.0 / (hit.inverse_depth as f64 * dvec_dot_vec3(ray, aircraft_camera.forward));
                let point = dvec_add(aircraft_camera.position, dvec_scale(ray, distance));
                let n = flow_normal_to_planet(dvec_normalize(point));
                let lod = surface_lod(world_to_miles(distance) / FLOW_FOCAL as f64);
                if surface_city_amount(n, flow_surface_sample(n, lod), lod) > 0.3 {
                    visible_city_samples += 1;
                }
            }
            x += 8;
        }
        y += 8;
    }
    phase3_require(surface_samples >= 600 && visible_city_samples > 0
        && universe_space_visibility(&aircraft_camera) == 0.0,
        243, b"aircraft view lacks terrain, visible settlements, or atmospheric field extinction");
    let mut report = [0u8; 320];
    let pointer = report.as_mut_ptr();
    let mut length = 0;
    append(pointer, &mut length, b"flight requirements ok: orbit1/orbit2/plane/ground-ms=");
    for time in [orbit_one, orbit_two, orbit_three, flow_descent_end()] {
        append_number(pointer, &mut length, (time * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
    }
    append(pointer, &mut length, b" ceiling-clearance-milli=");
    append_number(pointer, &mut length, (minimum_ceiling_clearance * 1_000.0) as u32);
    append(pointer, &mut length, b" entry-slope-permille=");
    append_number(pointer, &mut length, (maximum_entry_slope * 1_000.0) as u32);
    append(pointer, &mut length, b" day/night-cities=");
    append_number(pointer, &mut length, day_cities);
    append(pointer, &mut length, b"/");
    append_number(pointer, &mut length, night_cities);
    append(pointer, &mut length, b" visible-city-samples=");
    append_number(pointer, &mut length, visible_city_samples);
    append(pointer, &mut length, b" first-orbit-ms=");
    append_number(pointer, &mut length, ((orbit_one - orbit_start) * 1_000.0) as u32);
    append(pointer, &mut length, b"\n");
    write_all(&report[..length]);
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_capture_profile_audit() -> ! {
    let times = [
        20.5f64, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 27.5, 27.8, 28.0,
        29.0, 30.0, 31.0, 32.0, 34.0, 35.0, 38.5, 42.0, flow_aircraft_arrival(),
        90.0, 137.0,
    ];
    let mut index = 0usize;
    while index < times.len() {
        let time = times[index];
        let state = trajectory_state(time);
        let field = universe().field;
        let relative_field = dvec_sub(state.position, field.axis_origin);
        let field_s = dvec_dot(relative_field, field.forward);
        let field_right = dvec_dot(relative_field, field.right);
        let field_down = dvec_dot(relative_field, field.down);
        let mut report = [0u8; 256];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"capture-profile ms/speed-milli/clearance-milli/phase-milli/phase-rate-milli/accel-milli=");
        append_number(pointer, &mut length, (time * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (dvec_length(state.velocity) * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (state.clearance_miles * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (state.phase * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (state.phase_rate * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (dvec_length(state.acceleration) * 1_000.0) as u32);
        append(pointer, &mut length, b" field-s/right/down+20000-milli/exterior-milli=");
        append_number(pointer, &mut length, ((field_s + 20_000.0) * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, ((field_right + 20_000.0) * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, ((field_down + 20_000.0) * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(
            pointer,
            &mut length,
            (field_exterior_visibility(field, state.position) * 1_000.0) as u32,
        );
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        index += 1;
    }
    let capture_start = legacy_trajectory_state(WORLD_CURVE_START);
    let (orbit_normal, orbit_tangent) = world_curve_capture_frame();
    let field = universe().field;
    let mut parameter_index = 0u32;
    while parameter_index <= 20 {
        let parameter = parameter_index as f64 / 20.0;
        let point = world_curve_capture_geometry(
            parameter,
            capture_start,
            orbit_normal,
            orbit_tangent,
        )
        .0;
        let relative = dvec_sub(point, field.axis_origin);
        let s = dvec_dot(relative, field.forward);
        let down = dvec_dot(relative, field.down);
        let mut report = [0u8; 128];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"capture-geometry parameter-milli/s/down+20000-milli=");
        append_number(pointer, &mut length, parameter_index * 50);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, ((s + 20_000.0) * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, ((down + 20_000.0) * 1_000.0) as u32);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        parameter_index += 1;
    }
    let targets = [512.0f64, 1_024.0, 2_048.0, 3_072.0, 4_096.0, WORLD_ORBIT_END_PHASE];
    index = 0;
    let mut time = ORBIT_ENTRY as f64;
    while index < targets.len() && time <= flow_aircraft_arrival() {
        if trajectory_state(time).phase >= targets[index] {
            let mut report = [0u8; 96];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"capture-crossing phase/ms=");
            append_number(pointer, &mut length, targets[index] as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, (time * 1_000.0) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            index += 1;
        }
        time += 1.0 / TRAJECTORY_HZ as f64;
    }
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_phase0_motion_audit() -> ! {
    let curve_start = trajectory_state(WORLD_CURVE_START);
    let legacy_start = legacy_trajectory_state(WORLD_CURVE_START);
    let entry = trajectory_state(ORBIT_ENTRY as f64);
    if !(phase0_dvec_close(curve_start.position, legacy_start.position, 1.0e-16)
        && phase0_dvec_close(curve_start.velocity, legacy_start.velocity, 1.0e-8)
        && (dvec_length(entry.velocity) - world_curve_approach_speed(ORBIT_ENTRY as f64)).abs() < 0.01
        && entry.phase >= 0.0
        && entry.phase < 512.0
        && entry.phase_rate >= 0.0)
    {
        let mut report = [0u8; 128];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase0 motion audit failed: entry speed-milli=");
        append_number(pointer, &mut length, (dvec_length(entry.velocity) * 1_000.0) as u32);
        append(pointer, &mut length, b" phase_milli=");
        append_number(pointer, &mut length, (entry.phase.abs() * 1_000.0) as u32);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(110)
    }
    let mut time = 27.0f64;
    let mut previous_phase = -1.0e-9;
    while time <= 35.0 {
        let first = trajectory_state(time);
        let second = trajectory_state(time);
        let valid = first.position.x.is_finite()
                && first.position.y.is_finite()
                && first.position.z.is_finite()
                && phase0_dvec_close(first.position, second.position, 1.0e-24)
                && phase0_vec_close(first.forward, second.forward, 1.0e-7)
                && first.phase + 1.0e-9 >= previous_phase
                && phase0_basis_valid(flow_camera(time as f32));
        if !valid {
            let mut report = [0u8; 128];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase0 motion audit failed at ms/phase-micro/basis=");
            append_number(pointer, &mut length, (time * 1_000.0) as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, (first.phase.abs() * 1_000_000.0) as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, phase0_basis_valid(flow_camera(time as f32)) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(111)
        }
        previous_phase = first.phase;
        time += 0.01;
    }
    write_all(
        b"phase0 motion audit ok: one-world-curve start=20.5 braking-start=25.658 capture=unchanged-curvature-to-orbit deterministic-seek=yes\n",
    );
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_phase1_precision_audit() -> ! {
    phase0_require(
        (world_to_miles(FLOW_PLANET_RADIUS_WORLD) - PLANET_RADIUS_MILES).abs() < 1.0e-9
            && (world_to_meters(FLOW_FINAL_ALTITUDE_WORLD) - FINAL_ALTITUDE_METERS).abs()
                < 1.0e-9
            && (FLOW_AIRPLANE_ALTITUDE_MILES * FEET_PER_MILE - 30_000.0).abs()
                < 1.0e-9,
        120,
        b"physical scale constants disagree",
    );
    let mut time = 27.0f64;
    while time <= FLOW_PATH_END {
        let state = trajectory_state(time);
        let camera = flow_camera(time as f32);
        phase0_require(
            dvec_length(dvec_sub(dvec_sub(camera.position, PLANET_CENTER), state.position))
                < 1.0e-9
                && state.velocity.x.is_finite()
                && state.velocity.y.is_finite()
                && state.velocity.z.is_finite()
                && state.acceleration.x.is_finite()
                && state.acceleration.y.is_finite()
                && state.acceleration.z.is_finite(),
            121,
            b"camera-relative f64 trajectory state is unstable",
        );
        time += 0.125;
    }
    write_all(
        b"phase1 precision audit ok: planet=13500mi endpoint=20m trajectory=f64 deterministic-table=120hz\n",
    );
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase2_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase2 motion audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase2_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase2_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn trajectory_derivative_errors(time: f64) -> (f64, f64) {
    let step = 0.000_2;
    let before = trajectory_state(time - step);
    let at = trajectory_state(time);
    let after = trajectory_state(time + step);
    let numerical_velocity =
        dvec_scale(dvec_sub(after.position, before.position), 0.5 / step);
    let numerical_acceleration = dvec_scale(
        dvec_add(after.position, dvec_add(dvec_scale(at.position, -2.0), before.position)),
        1.0 / (step * step),
    );
    let velocity_scale = dvec_length(at.velocity).max(1.0);
    let acceleration_scale = dvec_length(at.acceleration).max(1.0);
    (
        dvec_length(dvec_sub(numerical_velocity, at.velocity)) / velocity_scale,
        dvec_length(dvec_sub(numerical_acceleration, at.acceleration))
            / acceleration_scale,
    )
}

#[cfg(feature = "phase0-audit")]
pub(super) fn trajectory_derivatives_agree(time: f64) -> bool {
    let errors = trajectory_derivative_errors(time);
    errors.0 < 2.0e-4 && errors.1 < 4.0e-2
}

#[cfg(feature = "phase0-audit")]
pub(super) fn trajectory_phase_crossing(target: f64, start: f64, end: f64) -> f64 {
    let mut tick = (start * TRAJECTORY_HZ as f64) as usize;
    if tick as f64 * TRAJECTORY_STEP < start {
        tick += 1;
    }
    let end_tick = (end * TRAJECTORY_HZ as f64) as usize;
    while tick <= end_tick {
        let time = tick as f64 * TRAJECTORY_STEP;
        if trajectory_state(time).phase >= target {
            return time;
        }
        tick += 1;
    }
    end + TRAJECTORY_STEP
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_phase2_motion_audit() -> ! {
    let checkpoints = [
        27.0f64, 28.0, 30.0, 34.978_35, 35.0, 38.0, 42.0, 50.0, 60.0, 75.0,
        90.0, 98.0, 108.0, 116.0, 137.0,
    ];
    let mut index = 0usize;
    while index < checkpoints.len() {
        let time = checkpoints[index];
        let first = trajectory_state(time);
        let second = trajectory_state(time);
        let valid = phase0_dvec_close(first.position, second.position, 1.0e-24)
                && phase0_dvec_close(first.velocity, second.velocity, 1.0e-24)
                && phase0_dvec_close(first.acceleration, second.acceleration, 1.0e-20)
                && (time <= ORBIT_ENTRY as f64 || trajectory_derivatives_agree(time))
                && phase0_basis_valid(flow_camera(time as f32))
                && first.roll == 0.0;
        if !valid {
            let mut report = [0u8; 112];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase2 motion audit failed: state checkpoint index=");
            append_number(pointer, &mut length, index as u32);
            append(pointer, &mut length, b" mask=");
            let mask = (trajectory_derivatives_agree(time) as u32)
                | ((phase0_basis_valid(flow_camera(time as f32)) as u32) << 1)
                | ((phase0_dvec_close(first.position, second.position, 1.0e-24) as u32) << 2);
            append_number(pointer, &mut length, mask);
            let errors = trajectory_derivative_errors(time);
            append(pointer, &mut length, b" derivative-micro=");
            append_number(pointer, &mut length, (errors.0 * 1_000_000.0) as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, (errors.1 * 1_000_000.0) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(140)
        }
        index += 1;
    }

    let far_side_time = trajectory_phase_crossing(512.0, ORBIT_ENTRY as f64, 35.0);
    let orbit_one_time = trajectory_phase_crossing(1_024.0, far_side_time, 38.0);
    let orbit_two_time = trajectory_phase_crossing(2_048.0, orbit_one_time, 50.0);
    let orbit_three_time = trajectory_phase_crossing(WORLD_ORBIT_END_PHASE, orbit_two_time, flow_aircraft_arrival() + 0.1);
    let orbit_one = trajectory_state(orbit_one_time);
    let orbit_two = trajectory_state(orbit_two_time);
    let orbit_three = trajectory_state(orbit_three_time);
    phase2_require(
        (28.5..=30.0).contains(&far_side_time)
            && (29.0..=31.0).contains(&orbit_one_time)
            && (31.0..=36.0).contains(&orbit_two_time)
            && orbit_three_time - orbit_two_time > 2.0 * (orbit_two_time - orbit_one_time)
            && (550.0..650.0).contains(&world_to_meters(dvec_length(trajectory_state(40.0).velocity)))
            && (9_000.0..9_300.0).contains(&(trajectory_state(40.0).clearance_miles * METERS_PER_MILE))
            && orbit_one.phase >= 1_024.0
            && trajectory_state(orbit_one_time - TRAJECTORY_STEP).phase < 1_024.0
            && orbit_two.phase >= 2_048.0
            && trajectory_state(orbit_two_time - TRAJECTORY_STEP).phase < 2_048.0
            && orbit_three.phase >= WORLD_ORBIT_END_PHASE
            && trajectory_state(orbit_three_time - TRAJECTORY_STEP).phase < WORLD_ORBIT_END_PHASE,
        141,
        b"integrated orbit crossings missed their timing windows",
    );
    if !(dvec_length(orbit_three.velocity) > 0.0
        && dvec_length(orbit_three.velocity) <= dvec_length(orbit_one.velocity) * 0.25
        && orbit_three.phase_rate > 0.0)
    {
        let mut report = [0u8; 160];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase2 braking failed orbit1/orbit3/rate-micro=");
        append_number(pointer, &mut length, (dvec_length(orbit_one.velocity) * 1_000_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (dvec_length(orbit_three.velocity) * 1_000_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (orbit_three.phase_rate * 1_000_000.0) as u32);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(142)
    }

    let mut time = far_side_time;
    let mut previous_altitude = world_to_miles(
        world_curve_spiral_base_radius(trajectory_state(time).phase)
            - FLOW_PLANET_RADIUS_WORLD,
    ) + 1.0e-6;
    let mut previous_phase = trajectory_state(time).phase - 1.0e-6;
    let mut previous_acceleration = trajectory_state(time).acceleration;
    while time <= flow_descent_end() {
        let state = trajectory_state(time);
        if time <= flow_aircraft_arrival() {
            let reference_altitude = world_to_miles(
                world_curve_spiral_base_radius(state.phase) - FLOW_PLANET_RADIUS_WORLD,
            );
            if reference_altitude > previous_altitude + 1.0e-5 {
                let mut report = [0u8; 96];
                let pointer = report.as_mut_ptr();
                let mut length = 0usize;
                append(pointer, &mut length, b"phase2 motion audit failed: altitude rises at ms=");
                append_number(pointer, &mut length, (time * 1_000.0) as u32);
                append(pointer, &mut length, b" delta-micromiles=");
                append_number(
                    pointer,
                    &mut length,
                    ((reference_altitude - previous_altitude) * 1_000_000.0) as u32,
                );
                append(pointer, &mut length, b"\n");
                write_all(&report[..length]);
                exit(143)
            }
        }
        if !(state.phase > previous_phase
            && state.phase_rate > 0.0
            && dvec_length(dvec_sub(state.acceleration, previous_acceleration)).is_finite())
        {
            let mut report = [0u8; 128];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase2 ground-track failed ms/phase/rate-micro=");
            append_number(pointer, &mut length, (time * 1_000.0) as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, state.phase as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, (state.phase_rate * 1_000_000.0) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(144)
        }
        previous_altitude = if time <= flow_aircraft_arrival() {
            world_to_miles(
                world_curve_spiral_base_radius(state.phase) - FLOW_PLANET_RADIUS_WORLD,
            )
        } else {
            state.clearance_miles
        };
        previous_phase = state.phase;
        previous_acceleration = state.acceleration;
        time += 1.0 / 120.0;
    }

    let controls = [
        (flow_aircraft_arrival(), FLOW_AIRPLANE_ALTITUDE_MILES),
        (flow_high_pass_end(), FLOW_AIRPLANE_ALTITUDE_MILES),
        (flow_valley_overhead(), 1_500.0 / METERS_PER_MILE),
        (flow_valley_corridor(), 200.0 / METERS_PER_MILE),
        (flow_descent_end(), FINAL_ALTITUDE_METERS / METERS_PER_MILE),
    ];
    index = 0;
    while index < controls.len() {
        let state = trajectory_state(controls[index].0);
        let surface_normal = flow_normal_to_planet(dvec_normalize(state.position));
        let actual_agl = world_to_miles(
            dvec_length(state.position)
                - FLOW_PLANET_RADIUS_WORLD
                - flow_surface_relief_world(surface_normal),
        );
        phase2_require(
            ((state.clearance_miles - controls[index].1) * METERS_PER_MILE).abs() < 0.5,
            145,
            b"integrated radial control missed a physical-altitude target",
        );
        phase2_require(
            ((actual_agl - controls[index].1) * METERS_PER_MILE).abs() < 0.5,
            148,
            b"trajectory clearance is not displaced-surface AGL",
        );
        index += 1;
    }

    let pass_start = trajectory_state(flow_aircraft_arrival());
    let pass_end = trajectory_state(flow_high_pass_end());
    if !((pass_start.clearance_miles - pass_end.clearance_miles).abs() < 1.0e-4
        && pass_end.phase > pass_start.phase
        && pass_end.phase - pass_start.phase < 128.0)
    {
        let mut report = [0u8; 128];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase2 pass failed clearance-start/end-micro/phase-start/end-milli=");
        append_number(pointer, &mut length, (pass_start.clearance_miles * 1_000_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (pass_end.clearance_miles * 1_000_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (pass_start.phase * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (pass_end.phase * 1_000.0) as u32);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(146)
    }
    let endpoint = trajectory_state(FLOW_PATH_END);
    let endpoint_after = trajectory_state(FLOW_PATH_END + 0.01);
    phase2_require(
        dvec_length(dvec_sub(endpoint_after.position, endpoint.position)) > 1.0e-10
            && endpoint.phase > WORLD_ORBIT_END_PHASE
            && endpoint.phase < WORLD_ORBIT_END_PHASE + 1.0,
        147,
        b"ground speed stops or regional phase misses the destination",
    );

    write_all(
        b"phase2 motion audit ok: braking=25.658s/unchanged-capture candidate-extended-spiral aircraft=30000ft/600mps-by-40s ground=18s-later/20m no-stopped-phase\n",
    );
    exit(0)
}
