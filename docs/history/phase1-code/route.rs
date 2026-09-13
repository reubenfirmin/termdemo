//! ARCHIVED phase-1 implementation. Not compiled or used by the active flight.
use super::{
    DVec3, FLOW_AIRPLANE_ALTITUDE_MILES, FLOW_MAX_RELIEF_MILES, FLOW_PLANET_RADIUS_WORLD,
    TrajectoryState, WORLD_ORBIT_ARC, brake_exp, brake_log, dvec_add, dvec_cross, dvec_dot,
    dvec_length, dvec_length_squared, dvec_normalize, dvec_scale, dvec_sub,
    flow_normal_to_planet, flow_surface_relief_world, landing_forward, landing_up,
    miles_to_world, planet_normal_to_flow, quintic_component, sine_sample_c2_state,
    smootherstep_f64, sqrt_f64, universe, world_curve_regional_phase,
};

pub(super) const WORLD_CURVE_ORBIT_RADIUS: f64 = 2.4;
pub(super) const WORLD_ORBIT_END_PHASE: f64 = 5_120.0;
pub(super) const WORLD_ORBIT_ALIGNMENT_PHASE: f64 = WORLD_ORBIT_END_PHASE - 1_024.0;
pub(super) fn world_curve_capture_frame() -> (DVec3, DVec3) {
    let layout = universe().field;
    // Enter twenty degrees past the top of a vertical over/under orbit.
    // Its 2.4-unit radius fits the fixed tunnel and the localized underpass.
    const SIN_ENTRY: f64 = 0.342_020_143_325_668_7;
    const COS_ENTRY: f64 = 0.939_692_620_785_908_4;
    (
        dvec_add(dvec_scale(layout.forward, SIN_ENTRY), dvec_scale(layout.down, -COS_ENTRY)),
        dvec_add(dvec_scale(layout.forward, COS_ENTRY), dvec_scale(layout.down, SIN_ENTRY)),
    )
}

pub(super) fn world_curve_capture_geometry(
    parameter: f64,
    start: TrajectoryState,
    orbit_normal: DVec3,
    orbit_tangent: DVec3,
) -> (DVec3, DVec3) {
    let field = universe().field;
    let endpoint = dvec_scale(orbit_normal, WORLD_CURVE_ORBIT_RADIUS);
    let length = dvec_dot(dvec_sub(endpoint, start.position), field.forward);
    let drop = dvec_dot(dvec_sub(endpoint, start.position), field.down);
    let axial_tangent = dvec_dot(orbit_tangent, field.forward);
    let slope = dvec_dot(orbit_tangent, field.down) / axial_tangent;
    let second = 1.0 / (WORLD_CURVE_ORBIT_RADIUS * axial_tangent * axial_tangent * axial_tangent);
    let third = 3.0 * dvec_dot(orbit_normal, field.forward)
        / (WORLD_CURVE_ORBIT_RADIUS * WORLD_CURVE_ORBIT_RADIUS
            * axial_tangent * axial_tangent * axial_tangent * axial_tangent * axial_tangent);
    // One monotone graph over axial distance. These coefficients match the
    // circle's position, tangent and curvature without an overshooting
    // control polygon or an independently animated lateral/vertical move.
    let first_moment = length * slope / drop;
    let second_moment = length * length * second / drop;
    let w4 = (second_moment - 239.0 * first_moment + 12_800.0) / (76.0 * 156.0);
    let w80 = (second_moment - 163.0 * first_moment + 640.0) / (-76.0 * 80.0);
    let w160 = 1.0 - w4 - w80;
    let raw_third = drop * (w4 * 24.0 + w80 * 80.0 * 79.0 * 78.0
        + w160 * 160.0 * 159.0 * 158.0);
    let correction = (raw_third - length * length * length * third) / 6.0;
    let u = parameter;
    let q = 1.0 - u;
    let u2 = u * u;
    let u3 = u2 * u;
    let u4 = u2 * u2;
    let u8 = u4 * u4;
    let u16 = u8 * u8;
    let u32 = u16 * u16;
    let u64 = u32 * u32;
    let u79 = u64 * u8 * u4 * u2 * u;
    let u80 = u79 * u;
    let u159 = u79 * u79 * u;
    let u160 = u159 * u;
    // The last term is zero through second derivative at both ends, and
    // matches third derivative at orbit entry. u^4 gives a C3 straight join.
    let offset = drop * (w4 * u4 + w80 * u80 + w160 * u160)
        + correction * u160 * q * q * q;
    let derivative = drop * (4.0 * w4 * u3 + 80.0 * w80 * u79 + 160.0 * w160 * u159)
        + correction * u159 * q * q * (160.0 * q - 3.0 * u);
    (
        dvec_add(start.position, dvec_add(dvec_scale(field.forward, length * u),
            dvec_scale(field.down, offset))),
        dvec_add(dvec_scale(field.forward, length), dvec_scale(field.down, derivative)),
    )
}
pub(super) fn world_curve_capture_arc_length(
    start_parameter: f64,
    end_parameter: f64,
    start: TrajectoryState,
    orbit_normal: DVec3,
    orbit_tangent: DVec3,
) -> f64 {
    // Long inversions of the same curve must resolve its u^160 end term;
    // short-step quadrature accuracy is insufficient for a full-arc lookup.
    let subdivisions = if end_parameter - start_parameter > 0.01 { 1_024 } else { 128 };
    if end_parameter <= start_parameter {
        return 0.0;
    }
    let step = (end_parameter - start_parameter) / subdivisions as f64;
    let mut sum = 0.0;
    let mut index = 0u32;
    while index <= subdivisions {
        let parameter = start_parameter + index as f64 * step;
        let speed = dvec_length(
            world_curve_capture_geometry(parameter, start, orbit_normal, orbit_tangent).1,
        );
        let weight = if index == 0 || index == subdivisions {
            1.0
        } else if index & 1 == 0 {
            2.0
        } else {
            4.0
        };
        sum += weight * speed;
        index += 1;
    }
    sum * step / 3.0
}

pub(super) fn world_curve_capture_advance(
    parameter: f64,
    distance: f64,
    start: TrajectoryState,
    orbit_normal: DVec3,
    orbit_tangent: DVec3,
) -> (f64, f64) {
    let remaining = world_curve_capture_arc_length(
        parameter,
        1.0,
        start,
        orbit_normal,
        orbit_tangent,
    );
    if distance >= remaining {
        return (1.0, distance - remaining);
    }
    let mut lower = parameter;
    let mut upper = 1.0;
    let mut iteration = 0u32;
    while iteration < 48 {
        let middle = (lower + upper) * 0.5;
        let covered = world_curve_capture_arc_length(
            parameter,
            middle,
            start,
            orbit_normal,
            orbit_tangent,
        );
        if covered < distance {
            lower = middle;
        } else {
            upper = middle;
        }
        iteration += 1;
    }
    ((lower + upper) * 0.5, 0.0)
}

pub(super) fn world_curve_spiral_base_radius(phase: f64) -> f64 {
    let aircraft_radius = FLOW_PLANET_RADIUS_WORLD + miles_to_world(FLOW_AIRPLANE_ALTITUDE_MILES);
    if phase <= 512.0 { return WORLD_CURVE_ORBIT_RADIUS; }
    if phase >= WORLD_ORBIT_END_PHASE { return aircraft_radius; }
    // One shrinking spiral, with two additional revolutions. Interpolating
    // radius linearly spent almost all of the descent far from the planet,
    // then crossed the entire curved-limb view in three tenths of a second.
    // Log clearance distributes the scale change over successive revolutions,
    // including the close-orbit scales (log total radius would flatten here).
    let u = (phase - 512.0) / (WORLD_ORBIT_END_PHASE - 512.0);
    let ramp_integral = |x: f64| {
        let v = (x / 0.1).clamp(0.0, 1.0);
        0.1 * v * v * v * v * (2.5 + v * (-3.0 + v)) + (x - 0.1).max(0.0)
    };
    // A broad, nearly uniform loss of log clearance. Easing across the whole
    // range concentrated most of the visible scale change in its middle.
    // Integrating the edge ramps keeps the same C3 joins without that bulge.
    let progress = (ramp_integral(u) - ramp_integral(u - 0.9)) / 0.9;
    let aircraft_clearance = aircraft_radius - FLOW_PLANET_RADIUS_WORLD;
    FLOW_PLANET_RADIUS_WORLD + aircraft_clearance * brake_exp(
        brake_log((WORLD_CURVE_ORBIT_RADIUS - FLOW_PLANET_RADIUS_WORLD) / aircraft_clearance) * (1.0 - progress))
}

pub(super) fn world_curve_spiral_radius_value(
    phase: f64, orbit_normal: DVec3, orbit_tangent: DVec3,
) -> f64 {
    let normal = world_curve_orbit_normal(WORLD_ORBIT_END_PHASE, orbit_normal, orbit_tangent);
    let relief = flow_surface_relief_world(flow_normal_to_planet(normal));
    let relief_amount = smootherstep_f64((phase - 512.0) / 1_536.0);
    let descent = smootherstep_f64((phase - WORLD_ORBIT_ALIGNMENT_PHASE) / 1_024.0);
    let clearance_reserve = miles_to_world(FLOW_MAX_RELIEF_MILES);
    world_curve_spiral_base_radius(phase) + relief_amount * clearance_reserve
        + descent * (relief - clearance_reserve)
}

pub(super) fn world_curve_spiral_position(phase: f64) -> DVec3 {
    let (normal, tangent) = world_curve_capture_frame();
    dvec_scale(world_curve_orbit_normal(phase, normal, tangent),
        world_curve_spiral_radius_value(phase, normal, tangent))
}

pub(super) fn world_curve_spiral_metric(phase: f64) -> f64 {
    let delta = 0.01;
    dvec_length(dvec_sub(
        world_curve_spiral_position(phase + delta),
        world_curve_spiral_position(phase - delta),
    )) / (2.0 * delta)
}

// One distance coordinate for all orbital geometry. This is a numerical
// integral of the fixed path, not a new path or an independently timed move.
pub(super) const WORLD_ORBIT_INTERVALS: usize = WORLD_ORBIT_END_PHASE as usize;
pub(super) fn world_curve_orbit_length(phase: f64) -> f64 {
    let scaled = phase.clamp(0.0, WORLD_ORBIT_INTERVALS as f64);
    let index = (scaled as usize).min(WORLD_ORBIT_INTERVALS - 1);
    let a = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[index]).read() };
    let b = unsafe { core::ptr::addr_of!(WORLD_ORBIT_ARC[index + 1]).read() };
    quintic_component(scaled - index as f64, 1.0,
        a.distance, b.distance, a.metric, b.metric, a.derivative, b.derivative).0
}

#[cfg(feature = "phase0-audit")]
pub(super) fn world_curve_final_orbit_length(phase: f64) -> f64 {
    world_curve_orbit_length(phase) - world_curve_orbit_length(WORLD_ORBIT_ALIGNMENT_PHASE)
}

pub(super) fn world_curve_orbit_phase(distance: f64) -> f64 {
    let mut lower = 0.0;
    let mut upper = WORLD_ORBIT_INTERVALS as f64;
    let mut iteration = 0;
    while iteration < 48 {
        let middle = (lower + upper) * 0.5;
        if world_curve_orbit_length(middle) < distance { lower = middle; } else { upper = middle; }
        iteration += 1;
    }
    (lower + upper) * 0.5
}

pub(super) fn world_curve_orbit_normal(
    phase: f64, orbit_normal: DVec3, orbit_tangent: DVec3,
) -> DVec3 {
    // Keep the fast over/under revolutions in their entry plane. Reorient
    // toward the regional ground track only over the final descending
    // revolution, after the main gravity-assisted brake has completed.
    let amount = smootherstep_f64((phase - WORLD_ORBIT_ALIGNMENT_PHASE) / 1_024.0);
    let normal = dvec_normalize(orbit_normal);
    let tangent = dvec_normalize(dvec_sub(orbit_tangent,
        dvec_scale(normal, dvec_dot(orbit_tangent, normal))));
    let point = dvec_normalize(dvec_add(
        dvec_scale(normal, sine_sample_c2_state(phase + 256.0).0),
        dvec_scale(tangent, sine_sample_c2_state(phase).0),
    ));
    if amount == 0.0 { return point; }
    let (regional_normal, regional_tangent) = world_curve_regional_frame();
    let regional_tangent = dvec_normalize(dvec_sub(regional_tangent,
        dvec_scale(regional_normal, dvec_dot(regional_tangent, regional_normal))));
    let (scalar, vector) = world_curve_frame_rotation(normal, tangent, regional_normal, regional_tangent);
    // Interpolate ONE rotation of an orthonormal frame. Independently
    // blending its axes made the tangent nearly cancel midway, introducing
    // a high-curvature kink and a hidden slowdown/speedup in the final orbit.
    let w = 1.0 - amount + amount * scalar;
    let v = dvec_scale(vector, amount);
    let inverse_norm = 1.0 / sqrt_f64(w * w + dvec_length_squared(v));
    let w = w * inverse_norm;
    let v = dvec_scale(v, inverse_norm);
    let twice_cross = dvec_scale(dvec_cross(v, point), 2.0);
    dvec_normalize(dvec_add(point,
        dvec_add(dvec_scale(twice_cross, w), dvec_cross(v, twice_cross))))
}

pub(super) fn world_curve_frame_rotation(
    normal: DVec3, tangent: DVec3, target_normal: DVec3, target_tangent: DVec3,
) -> (f64, DVec3) {
    let binormal = dvec_cross(normal, tangent);
    let target_binormal = dvec_cross(target_normal, target_tangent);
    let row = |x: f64, y: f64, z: f64| dvec_add(dvec_scale(normal, x),
        dvec_add(dvec_scale(tangent, y), dvec_scale(binormal, z)));
    let a = row(target_normal.x, target_tangent.x, target_binormal.x);
    let b = row(target_normal.y, target_tangent.y, target_binormal.y);
    let c = row(target_normal.z, target_tangent.z, target_binormal.z);
    let trace = a.x + b.y + c.z;
    let (w, v) = if trace > 0.0 {
        let scale = 2.0 * sqrt_f64(1.0 + trace);
        (scale * 0.25, DVec3 { x: (c.y - b.z) / scale,
            y: (a.z - c.x) / scale, z: (b.x - a.y) / scale })
    } else if a.x > b.y && a.x > c.z {
        let scale = 2.0 * sqrt_f64(1.0 + a.x - b.y - c.z);
        ((c.y - b.z) / scale, DVec3 { x: scale * 0.25,
            y: (a.y + b.x) / scale, z: (a.z + c.x) / scale })
    } else if b.y > c.z {
        let scale = 2.0 * sqrt_f64(1.0 + b.y - a.x - c.z);
        ((a.z - c.x) / scale, DVec3 { x: (a.y + b.x) / scale,
            y: scale * 0.25, z: (b.z + c.y) / scale })
    } else {
        let scale = 2.0 * sqrt_f64(1.0 + c.z - a.x - b.y);
        ((b.x - a.y) / scale, DVec3 { x: (a.z + c.x) / scale,
            y: (b.z + c.y) / scale, z: scale * 0.25 })
    };
    // Select the shorter rotation, including frames more than 90 degrees
    // apart; no assumptions about the current approach's orientation.
    if w < 0.0 { (-w, dvec_scale(v, -1.0)) } else { (w, v) }
}

pub(super) fn world_curve_regional_frame() -> (DVec3, DVec3) {
    let normal = dvec_normalize(planet_normal_to_flow(landing_up()));
    let tangent = dvec_normalize(planet_normal_to_flow(landing_forward()));
    let phase = -world_curve_regional_phase(18.0);
    let cosine = sine_sample_c2_state(phase + 256.0).0;
    let sine = sine_sample_c2_state(phase).0;
    (
        dvec_normalize(dvec_add(dvec_scale(normal, cosine), dvec_scale(tangent, sine))),
        dvec_normalize(dvec_sub(dvec_scale(tangent, cosine), dvec_scale(normal, sine))),
    )
}
