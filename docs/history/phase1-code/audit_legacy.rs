//! ARCHIVED phase-1 implementation. Not compiled or used by the active flight.
//! Historical compatibility equations only; excluded from normal builds.
#[cfg(feature = "phase0-audit")]
use super::{
    DVec3, FLOW_FOCAL, FLOW_PLANET_RADIUS_WORLD, FieldLayout, FlowCamera,
    GRID_BOTTOM_HEIGHT_WORLD, GRID_HORIZONTAL, GRID_LANES, GRID_SQUARE_START,
    GRID_WELL_DEPTH_WORLD, GRID_WELL_RADIUS_WORLD, ORBIT_ENTRY, PLANET_CENTER, STARFIELD_HOLD,
    SurfaceLod, Vec3, WIDTH, approach_camera_reference, canonical_projection_ray, dvec_add,
    dvec_dot, dvec_dot_vec3, dvec_length, dvec_normalize, dvec_scale, dvec_sub, fast_sqrt,
    flow_base_surface_height_from_map, flow_field_basis, flow_field_dual_basis,
    flow_surface_map_from_normal, flow_surface_relief_lod_world, flow_surface_relief_world,
    hash, normalize, planet_normal_to_flow, sine, smootherstep_f64, smoothstep, sqrt_f64,
    starflight_displacement, surface_lod, trajectory_state, world_to_miles,
};

#[cfg(feature = "phase0-audit")]
pub(super) const GRID_RINGS: usize = 25;
#[cfg(feature = "phase0-audit")]
pub(super) const SPOKE_LINES: usize = GRID_LANES * GRID_RINGS;
#[cfg(feature = "phase0-audit")]
pub(super) const RING_LINES: usize = GRID_RINGS * (GRID_LANES - 1);
#[cfg(feature = "phase0-audit")]
pub(super) const STAR_LINES: usize = SPOKE_LINES + RING_LINES;
#[cfg(feature = "phase0-audit")]
pub(super) const RING_INTERVAL: f32 = 0.05;
#[cfg(feature = "phase0-audit")]
pub(super) const RING_SPEED: f32 = 1.0 / RING_INTERVAL;
#[cfg(feature = "phase0-audit")]
pub(super) const FORMATION_EXCESS: f32 = 0.658_436_4;
#[cfg(feature = "phase0-audit")]
pub(super) const RING_BIRTH: f32 = 14.75;
#[cfg(feature = "phase0-audit")]
pub(super) const CLEAN_BIRTH: f32 = ring_birth(GRID_RINGS);
#[cfg(feature = "phase0-audit")]
pub(super) const GRID_FOG_START: f32 = 4.0;
#[cfg(feature = "phase0-audit")]
pub(super) const GRID_FOG_END: f32 = 6.0;
#[cfg(feature = "phase0-audit")]
pub(super) const STARFIELD_FILL: f32 = 7.0;
#[cfg(feature = "phase0-audit")]
pub(super) const STAR_ORGANIZE: f32 = 11.0;
#[cfg(feature = "phase0-audit")]
pub(super) const PLANET_INTRO: f32 = 27.0;
#[cfg(feature = "phase0-audit")]
pub(super) const APPROACH_SPEED: f32 = 0.28;
#[cfg(feature = "phase0-audit")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct CanonicalStarParticleIdentity {
    pub(super) line: usize,
    pub(super) generation: i32,
}

#[cfg(feature = "phase0-audit")]
pub(super) fn canonical_star_generation(index: usize, time: f32, clean_birth: f32) -> i32 {
    let period = GRID_RINGS as f32 * RING_INTERVAL;
    let phase = line_ring(index) as f32 * RING_INTERVAL;
    let sample_time = time.min(clean_birth - 0.000_1);
    let relative_cycle = (sample_time - phase) / period;
    let mut generation = relative_cycle as i32;
    if relative_cycle < generation as f32 {
        generation -= 1;
    }
    generation
}

#[cfg(feature = "phase0-audit")]
pub(super) fn canonical_star_particle_reference(layout: FieldLayout, time: f32) -> DVec3 {
    let approach = approach_camera_reference(time as f64);
    let starflight = starflight_displacement(time as f64);
    let outward = dvec_scale(layout.forward, -1.0);
    let relief = f64::from_bits(0x3edc_d567_e7fb_afa6);
    dvec_add(
        layout.axis_origin,
        dvec_scale(
            outward,
            FLOW_PLANET_RADIUS_WORLD + approach.0 + relief + starflight.0,
        ),
    )
}

#[cfg(feature = "phase0-audit")]
pub(super) fn canonical_star_particle_local_position(
    identity: CanonicalStarParticleIdentity,
    time: f32,
) -> Option<((f32, f32), f32)> {
    let id = identity.line as i32;
    let (lane_a, _, lane_b, _) = line_coordinates(identity.line);
    let period = GRID_RINGS as f32 * RING_INTERVAL;
    let phase = line_ring(identity.line) as f32 * RING_INTERVAL;
    let birth_time = phase + identity.generation as f32 * period;
    let activation = -period
        + hash(id, 2_281) as f32 / 255.0 * (STARFIELD_FILL + period);
    if birth_time < activation {
        return None;
    }
    let moving_row = outward_row(time, birth_time);
    let amount = moving_row / 20.0;
    let organization =
        smoothstep((birth_time - STAR_ORGANIZE) / (CLEAN_BIRTH - STAR_ORGANIZE));
    let birth_order = organization * 0.72;
    let random_x = (hash(id, 337 + identity.generation * 17) - 128) as f32 / 112.0;
    let random_y = (hash(id, 811 + identity.generation * 29) - 128) as f32 / 184.0;
    let lane_middle = (lane_a + lane_b) * 0.5;
    let cone = cone_direction(lane_middle);
    let mut motion_x = random_x + (cone.0 - random_x) * birth_order;
    let mut motion_y = random_y + (cone.1 - random_y) * birth_order;
    let motion_length = fast_sqrt(motion_x * motion_x + motion_y * motion_y).max(0.001);
    let exit_scale = (0.65 / motion_length).max(1.0);
    motion_x *= exit_scale;
    motion_y *= exit_scale;
    let radius = amount * amount * 260.0;
    Some(((motion_x * radius, motion_y * radius), moving_row))
}

#[cfg(feature = "phase0-audit")]
pub(super) fn canonical_star_particle_world_position(
    layout: FieldLayout,
    identity: CanonicalStarParticleIdentity,
    time: f32,
) -> Option<DVec3> {
    let (local, moving_row) = canonical_star_particle_local_position(identity, time)?;
    let amount = moving_row / 20.0;
    let projective_radius = (amount * amount * 260.0).max(0.001) as f64;
    let axial_depth = FLOW_FOCAL as f64 * 0.01 / projective_radius;
    let (dual_forward, dual_right, dual_down) = flow_field_dual_basis();
    Some(dvec_add(
        canonical_star_particle_reference(layout, time),
        dvec_add(
            dvec_scale(dual_forward, axial_depth),
            dvec_add(
                dvec_scale(
                    dual_right,
                    local.0 as f64 * axial_depth / FLOW_FOCAL as f64,
                ),
                dvec_scale(
                    dual_down,
                    local.1 as f64 * axial_depth / FLOW_FOCAL as f64,
                ),
            ),
        ),
    ))
}

#[cfg(feature = "phase0-audit")]
pub(super) fn canonical_star_exposure_times(
    identity: CanonicalStarParticleIdentity,
    observation_time: f32,
) -> Option<(f32, f32)> {
    if canonical_star_generation(identity.line, observation_time, CLEAN_BIRTH)
        != identity.generation
    {
        return None;
    }
    let (a, b, moving_row) = flyby_segment(identity.line, observation_time, CLEAN_BIRTH)?;
    let local = canonical_star_particle_local_position(identity, observation_time)?.0;
    let center_radius = (moving_row / 20.0) * (moving_row / 20.0) * 260.0;
    if center_radius <= 0.0 {
        return None;
    }
    let motion_x = local.0 / center_radius;
    let motion_y = local.1 / center_radius;
    let motion_squared = motion_x * motion_x + motion_y * motion_y;
    let first_radius = (a.0 * motion_x + a.1 * motion_y) / motion_squared;
    let second_radius = (b.0 * motion_x + b.1 * motion_y) / motion_squared;
    if first_radius < 0.0 || second_radius < 0.0 {
        return None;
    }
    let phase = line_ring(identity.line) as f32 * RING_INTERVAL;
    let period = GRID_RINGS as f32 * RING_INTERVAL;
    let birth_time = phase + identity.generation as f32 * period;
    let first_row = (20.0 * sqrt_f64(first_radius as f64 / 260.0)) as f32;
    let second_row = (20.0 * sqrt_f64(second_radius as f64 / 260.0)) as f32;
    Some((
        birth_time + first_row / RING_SPEED,
        birth_time + second_row / RING_SPEED,
    ))
}

#[cfg(feature = "phase0-audit")]
pub(super) fn audit_dvec_cross(a: DVec3, b: DVec3) -> DVec3 {
    DVec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn fixed_candidate_from_rays(
    first_camera: &FlowCamera,
    first_point: (f32, f32),
    second_camera: &FlowCamera,
    second_point: (f32, f32),
) -> Option<DVec3> {
    let first_direction = canonical_projection_ray(first_camera, first_point);
    let second_direction = canonical_projection_ray(second_camera, second_point);
    let offset = dvec_sub(first_camera.position, second_camera.position);
    let a = dvec_dot(first_direction, first_direction);
    let b = dvec_dot(first_direction, second_direction);
    let c = dvec_dot(second_direction, second_direction);
    let d = dvec_dot(first_direction, offset);
    let e = dvec_dot(second_direction, offset);
    let denominator = a * c - b * b;
    if denominator.abs() <= 1.0e-18 {
        return None;
    }
    let first_distance = (b * e - c * d) / denominator;
    let second_distance = (a * e - b * d) / denominator;
    if first_distance <= 0.0 || second_distance <= 0.0 {
        return None;
    }
    let first = dvec_add(
        first_camera.position,
        dvec_scale(first_direction, first_distance),
    );
    let second = dvec_add(
        second_camera.position,
        dvec_scale(second_direction, second_distance),
    );
    Some(dvec_scale(dvec_add(first, second), 0.5))
}

#[cfg(feature = "phase0-audit")]
pub(super) fn canonical_flow_grid_world_point(time: f32, section: (f32, f32), depth: f32) -> DVec3 {
    // Ring age is the physical flow coordinate: new mesh is emitted at
    // effectively infinite depth on the field axis and moves toward the
    // approach camera as `depth` grows. The mesh is placed in world space
    // before projection, then deformed by the fixed planetary gravity well.
    const SECTION_SCALE: f64 = GRID_BOTTOM_HEIGHT_WORLD / 0.61;
    let (field_forward, field_right, field_down) = flow_field_basis();
    let (dual_forward, dual_right, dual_down) = flow_field_dual_basis();
    let amount = depth / 20.0;
    let projective_radius = (amount * amount * 260.0).max(0.001) as f64;
    let axial_depth = FLOW_FOCAL as f64 * SECTION_SCALE / projective_radius;
    let transport = canonical_flow_field_transport(time);
    let mut point = dvec_add(
        transport,
        dvec_add(
            dvec_scale(dual_forward, axial_depth),
            dvec_add(
                dvec_scale(dual_right, section.0 as f64 * SECTION_SCALE),
                dvec_scale(dual_down, section.1 as f64 * SECTION_SCALE),
            ),
        ),
    );
    let relative_planet = dvec_sub(point, PLANET_CENTER);
    let lateral = dvec_dot(relative_planet, field_right);
    let axial = dvec_dot(relative_planet, field_forward);
    let floor_weight = smootherstep_f64((section.1 as f64 / 0.61 - 0.25) / 0.75);
    let well_radius_squared = GRID_WELL_RADIUS_WORLD * GRID_WELL_RADIUS_WORLD;
    let well = floor_weight * well_radius_squared
        / (lateral * lateral + axial * axial + well_radius_squared);
    point = dvec_add(point, dvec_scale(field_down, GRID_WELL_DEPTH_WORLD * well));
    point
}

#[cfg(feature = "phase0-audit")]
pub(super) fn canonical_flow_field_transport(time: f32) -> DVec3 {
    // Through the approach, the camera and field share the canonical
    // longitudinal flow. At capture the camera curves toward orbit while the
    // field continues through and past the planet from the same position,
    // velocity, and acceleration. The short acceleration tail goes smoothly
    // to zero, leaving an inertial world-space continuation rather than a mesh
    // that follows the orbiting camera.
    if time <= ORBIT_ENTRY {
        return trajectory_state(time as f64).position;
    }
    const ACCELERATION_TAIL: f64 = 0.5;
    let entry = trajectory_state(ORBIT_ENTRY as f64);
    let elapsed = time as f64 - ORBIT_ENTRY as f64;
    let progress = (elapsed / ACCELERATION_TAIL).min(1.0);
    let p2 = progress * progress;
    let p5 = p2 * p2 * progress;
    let p6 = p5 * progress;
    let p7 = p6 * progress;
    let acceleration_displacement = if progress < 1.0 {
        ACCELERATION_TAIL * ACCELERATION_TAIL
            * (p2 * 0.5 - p5 * 0.5 + p6 * 0.5 - p7 / 7.0)
    } else {
        ACCELERATION_TAIL * ACCELERATION_TAIL * 5.0 / 14.0
            + ACCELERATION_TAIL * 0.5 * (elapsed - ACCELERATION_TAIL)
    };
    dvec_add(
        entry.position,
        dvec_add(
            dvec_scale(entry.velocity, elapsed),
            dvec_scale(entry.acceleration, acceleration_displacement),
        ),
    )
}

#[cfg(feature = "phase0-audit")]
pub(super) fn canonical_flow_star_world_point(
    transport: DVec3,
    local_offset: (f32, f32),
    moving_row: f32,
) -> DVec3 {
    // Use the dual of the exact f32 camera frame. This avoids treating a
    // nearly orthonormal rounded frame as perfectly orthonormal and makes the
    // world-to-camera round trip reproduce the canonical coordinates.
    let (dual_forward, dual_right, dual_down) = flow_field_dual_basis();
    let amount = moving_row / 20.0;
    let projective_radius = (amount * amount * 260.0).max(0.001) as f64;
    let depth = FLOW_FOCAL as f64 * 0.01 / projective_radius;
    dvec_add(
        transport,
        dvec_add(
            dvec_scale(dual_forward, depth),
            dvec_add(
                dvec_scale(
                    dual_right,
                    local_offset.0 as f64 * depth / FLOW_FOCAL as f64,
                ),
                dvec_scale(
                    dual_down,
                    local_offset.1 as f64 * depth / FLOW_FOCAL as f64,
                ),
            ),
        ),
    )
}

#[cfg(feature = "phase0-audit")]
pub(super) fn project_canonical_flow_star_segment(
    _transport: DVec3,
    local_a: (f32, f32),
    local_b: (f32, f32),
    moving_row: f32,
    camera: &FlowCamera,
    x_scale: f32,
) -> Option<((f32, f32, f32), (f32, f32, f32))> {
    // This is the rejected screen-law oracle, evaluated in the camera-relative
    // frame so astronomical world coordinates cannot erase its tiny offsets.
    let origin = DVec3 { x: 0.0, y: 0.0, z: 0.0 };
    let a = canonical_flow_star_world_point(origin, local_a, moving_row);
    let b = canonical_flow_star_world_point(origin, local_b, moving_row);
    let project = |relative: DVec3| {
        let depth = dvec_dot_vec3(relative, camera.forward);
        if depth <= camera.near_plane {
            return None;
        }
        Some((
            160.0
                + (dvec_dot_vec3(relative, camera.right) / depth * FLOW_FOCAL as f64)
                    as f32
                    * x_scale,
            100.0
                + (dvec_dot_vec3(relative, camera.down) / depth * FLOW_FOCAL as f64)
                    as f32,
            depth as f32,
        ))
    };
    Some((project(a)?, project(b)?))
}

#[cfg(feature = "phase0-audit")]
pub(super) fn project_canonical_flow_grid_segment(
    time: f32,
    section_a: (f32, f32),
    depth_a: f32,
    section_b: (f32, f32),
    depth_b: f32,
    camera: &FlowCamera,
    x_scale: f32,
) -> Option<((f32, f32, f32), (f32, f32, f32))> {
    let mut a = canonical_flow_grid_world_point(time, section_a, depth_a);
    let mut b = canonical_flow_grid_world_point(time, section_b, depth_b);
    let near = camera.near_plane * 1.1;
    let mut relative_a = dvec_sub(a, camera.position);
    let mut relative_b = dvec_sub(b, camera.position);
    let depth_a = dvec_dot_vec3(relative_a, camera.forward);
    let depth_b = dvec_dot_vec3(relative_b, camera.forward);
    if depth_a <= near && depth_b <= near {
        return None;
    }
    if depth_a <= near {
        let amount = (near - depth_a) / (depth_b - depth_a).max(1.0e-12);
        a = dvec_add(a, dvec_scale(dvec_sub(b, a), amount));
        relative_a = dvec_sub(a, camera.position);
    } else if depth_b <= near {
        let amount = (near - depth_b) / (depth_a - depth_b).max(1.0e-12);
        b = dvec_add(b, dvec_scale(dvec_sub(a, b), amount));
        relative_b = dvec_sub(b, camera.position);
    }
    let project = |relative: DVec3| {
        let depth = dvec_dot_vec3(relative, camera.forward);
        if depth <= near {
            return None;
        }
        Some((
            160.0
                + (dvec_dot_vec3(relative, camera.right) / depth * FLOW_FOCAL as f64)
                    as f32
                    * x_scale,
            100.0
                + (dvec_dot_vec3(relative, camera.down) / depth * FLOW_FOCAL as f64)
                    as f32,
            depth as f32,
        ))
    };
    Some((project(relative_a)?, project(relative_b)?))
}

#[cfg(feature = "phase0-audit")]
pub(super) fn grid_fog_range(time: f32) -> (f32, f32) {
    let distant_detail = smoothstep((time - (PLANET_INTRO - 1.0)) / 2.0);
    (
        GRID_FOG_START + (1.0 - GRID_FOG_START) * distant_detail,
        GRID_FOG_END + (3.0 - GRID_FOG_END) * distant_detail,
    )
}

#[cfg(feature = "phase0-audit")]
pub(super) fn flow_base_surface_height(normal: Vec3, lod: SurfaceLod) -> f32 {
    flow_base_surface_height_from_map(flow_surface_map_from_normal(normal), lod)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn flow_surface_point(normal: Vec3) -> DVec3 {
    let direction = dvec_normalize(planet_normal_to_flow(normal));
    dvec_add(
        PLANET_CENTER,
        dvec_scale(
            direction,
            FLOW_PLANET_RADIUS_WORLD + flow_surface_relief_world(normal),
        ),
    )
}

#[cfg(feature = "phase0-audit")]
pub(super) fn flow_visible_surface_point(camera: &FlowCamera, normal: Vec3) -> DVec3 {
    let nominal = dvec_add(
        PLANET_CENTER,
        dvec_scale(planet_normal_to_flow(normal), FLOW_PLANET_RADIUS_WORLD),
    );
    let distance = dvec_length(dvec_sub(nominal, camera.position));
    let footprint = world_to_miles(distance) / FLOW_FOCAL as f64;
    dvec_add(
        PLANET_CENTER,
        dvec_scale(
            planet_normal_to_flow(normal),
            FLOW_PLANET_RADIUS_WORLD
                + flow_surface_relief_lod_world(normal, surface_lod(footprint)),
        ),
    )
}

#[cfg(feature = "phase0-audit")]
pub(super) fn put_pixel(frame: *mut u8, x: usize, y: usize, r: u8, g: u8, b: u8) {
    let offset = (y * WIDTH + x) * 3;
    unsafe {
        frame.add(offset).write(r);
        frame.add(offset + 1).write(g);
        frame.add(offset + 2).write(b);
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn line_coordinates(index: usize) -> (f32, f32, f32, f32) {
    let lane_width = 28.0 / (GRID_LANES - 1) as f32;
    if index < SPOKE_LINES {
        let lane_index = index / GRID_RINGS;
        let ring = index % GRID_RINGS;
        let lane = -14.0 + lane_index as f32 * lane_width;
        (lane, ring as f32, lane, ((ring + 1) % GRID_RINGS) as f32)
    } else {
        let ring_index = index - SPOKE_LINES;
        let row_index = ring_index / (GRID_LANES - 1);
        let lane_step = ring_index % (GRID_LANES - 1);
        let row = row_index as f32;
        let lane = -14.0 + lane_step as f32 * lane_width;
        (lane, row, lane + lane_width, row)
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn cone_direction(lane: f32) -> (f32, f32) {
    let angle = ((lane + 14.0) * 1_024.0 / 28.0) as i32;
    (sine(angle + 256), sine(angle) * 0.61)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn line_ring(index: usize) -> usize {
    if index < SPOKE_LINES {
        index % GRID_RINGS
    } else {
        (index - SPOKE_LINES) / (GRID_LANES - 1)
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn flyby_segment(
    index: usize,
    time: f32,
    clean_birth: f32,
) -> Option<((f32, f32), (f32, f32), f32)> {
    let id = index as i32;
    let (lane_a, _, lane_b, _) = line_coordinates(index);
    let period = GRID_RINGS as f32 * RING_INTERVAL;
    let phase = line_ring(index) as f32 * RING_INTERVAL;
    let sample_time = time.min(clean_birth - 0.000_1);
    let relative_cycle = (sample_time - phase) / period;
    let mut generation = relative_cycle as i32;
    if relative_cycle < generation as f32 {
        generation -= 1;
    }
    let birth_time = phase + generation as f32 * period;
    let activation = -period
        + hash(id, 2_281) as f32 / 255.0 * (STARFIELD_FILL + period);
    if birth_time < activation {
        return None;
    }
    let moving_row = outward_row(time, birth_time);
    let amount = moving_row / 20.0;
    let organization =
        smoothstep((birth_time - STAR_ORGANIZE) / (clean_birth - STAR_ORGANIZE));
    let birth_order = organization * 0.72;
    let random_x = (hash(id, 337 + generation * 17) - 128) as f32 / 112.0;
    let random_y = (hash(id, 811 + generation * 29) - 128) as f32 / 184.0;
    let lane_middle = (lane_a + lane_b) * 0.5;
    let cone = cone_direction(lane_middle);
    let mut motion_x = random_x + (cone.0 - random_x) * birth_order;
    let mut motion_y = random_y + (cone.1 - random_y) * birth_order;
    let motion_length = fast_sqrt(motion_x * motion_x + motion_y * motion_y).max(0.001);
    let exit_scale = (0.65 / motion_length).max(1.0);
    motion_x *= exit_scale;
    motion_y *= exit_scale;

    let line_direction = normalize(motion_x, motion_y);
    // This is the signed-off starfield motion law. Keep its arithmetic order
    // and 260-unit transverse scale unchanged; world placement happens only
    // after these canonical local offsets have been evaluated.
    let radius = amount * amount * 260.0;
    let distance = (amount * amount).clamp(0.0, 1.0);
    let luminosity = hash(id, 149) as f32 / 255.0;
    let streak_order = hash(id, 503) as f32 / 255.0;
    let streak_onset = streak_order * 5.4;
    let streaking = smoothstep((time - STARFIELD_HOLD - streak_onset) / 1.5);
    let length = (0.45 + streaking * 36.55)
        * distance
        * (0.66 + luminosity * 0.72);
    let half = length * 0.5;
    Some((
        (
            motion_x * radius - line_direction.0 * half,
            motion_y * radius - line_direction.1 * half,
        ),
        (
            motion_x * radius + line_direction.0 * half,
            motion_y * radius + line_direction.1 * half,
        ),
        moving_row,
    ))
}

#[cfg(feature = "phase0-audit")]
pub(super) fn canonical_flyby_segment(
    index: usize,
    time: f32,
    clean_birth: f32,
) -> Option<((f32, f32), (f32, f32))> {
    let id = index as i32;
    let (lane_a, _, lane_b, _) = line_coordinates(index);
    let period = GRID_RINGS as f32 * RING_INTERVAL;
    let phase = line_ring(index) as f32 * RING_INTERVAL;
    let sample_time = time.min(clean_birth - 0.000_1);
    let relative_cycle = (sample_time - phase) / period;
    let mut generation = relative_cycle as i32;
    if relative_cycle < generation as f32 {
        generation -= 1;
    }
    let birth_time = phase + generation as f32 * period;
    let activation = -period
        + hash(id, 2_281) as f32 / 255.0 * (STARFIELD_FILL + period);
    if birth_time < activation {
        return None;
    }
    let moving_row = outward_row(time, birth_time);
    let amount = moving_row / 20.0;
    let organization =
        smoothstep((birth_time - STAR_ORGANIZE) / (clean_birth - STAR_ORGANIZE));
    let birth_order = organization * 0.72;
    let random_x = (hash(id, 337 + generation * 17) - 128) as f32 / 112.0;
    let random_y = (hash(id, 811 + generation * 29) - 128) as f32 / 184.0;
    let lane_middle = (lane_a + lane_b) * 0.5;
    let cone = cone_direction(lane_middle);
    let mut motion_x = random_x + (cone.0 - random_x) * birth_order;
    let mut motion_y = random_y + (cone.1 - random_y) * birth_order;
    let motion_length = fast_sqrt(motion_x * motion_x + motion_y * motion_y).max(0.001);
    let exit_scale = (0.65 / motion_length).max(1.0);
    motion_x *= exit_scale;
    motion_y *= exit_scale;

    let line_direction = normalize(motion_x, motion_y);
    let radius = amount * amount * 260.0;
    let center = (160.0 + motion_x * radius, 100.0 + motion_y * radius);
    let distance = (amount * amount).clamp(0.0, 1.0);
    let luminosity = hash(id, 149) as f32 / 255.0;
    let streak_order = hash(id, 503) as f32 / 255.0;
    let streak_onset = streak_order * 5.4;
    let streaking = smoothstep((time - STARFIELD_HOLD - streak_onset) / 1.5);
    let length = (0.45 + streaking * 36.55)
        * distance
        * (0.66 + luminosity * 0.72);
    let half = length * 0.5;
    Some((
        (center.0 - line_direction.0 * half, center.1 - line_direction.1 * half),
        (center.0 + line_direction.0 * half, center.1 + line_direction.1 * half),
    ))
}

#[cfg(feature = "phase0-audit")]
pub(super) const fn formation_gap(emission: usize) -> f32 {
    let mut excess = FORMATION_EXCESS;
    let mut step = 0usize;
    while step < emission {
        excess *= 0.84;
        step += 1;
    }
    RING_INTERVAL + excess
}

#[cfg(feature = "phase0-audit")]
pub(super) const fn ring_birth(ring: usize) -> f32 {
    let mut birth = RING_BIRTH;
    let mut emission = 0usize;
    while emission < ring {
        birth += formation_gap(emission);
        emission += 1;
    }
    birth
}

#[cfg(feature = "phase0-audit")]
pub(super) fn ring_state(ring: usize, time: f32) -> (f32, f32) {
    let first_birth = ring_birth(ring);
    let current_position = travel_row(time);
    let regular_position = travel_row(CLEAN_BIRTH) + ring as f32;
    let birth_position = if current_position < regular_position {
        travel_row(first_birth)
    } else {
        let cycle = ((current_position - regular_position) / GRID_RINGS as f32) as i32;
        regular_position + cycle as f32 * GRID_RINGS as f32
    };
    (current_position - birth_position, birth_position)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn outward_row(time: f32, birth: f32) -> f32 {
    travel_row(time) - travel_row(birth)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn approach_travel_state(time: f64) -> (f64, f64, f64) {
    if time <= CLEAN_BIRTH as f64 {
        return (time * RING_SPEED as f64, RING_SPEED as f64, 0.0);
    }
    let duration = (GRID_HORIZONTAL - CLEAN_BIRTH) as f64;
    let reduction = (1.0 - APPROACH_SPEED) as f64;
    let elapsed = time - CLEAN_BIRTH as f64;
    let before = CLEAN_BIRTH as f64 * RING_SPEED as f64;
    if elapsed < duration {
        let progress = elapsed / duration;
        let p2 = progress * progress;
        let p3 = p2 * progress;
        let p4 = p3 * progress;
        let p5 = p4 * progress;
        let pulled_integral = p3 - p4 * 0.5 + 3.0 * (p3 / 3.0 - p4 * 0.5 + p5 * 0.2);
        let pulled_rate = 6.0 * p2 - 8.0 * p3 + 3.0 * p4;
        let pulled_acceleration = 12.0 * progress * (1.0 - progress) * (1.0 - progress);
        (
            before
                + duration
                    * RING_SPEED as f64
                    * (progress - reduction * pulled_integral),
            RING_SPEED as f64 * (1.0 - reduction * pulled_rate),
            -RING_SPEED as f64 * reduction * pulled_acceleration / duration,
        )
    } else {
        let transition = duration * RING_SPEED as f64 * (1.0 - reduction * 0.6);
        (
            before
                + transition
                + (elapsed - duration) * RING_SPEED as f64 * APPROACH_SPEED as f64,
            RING_SPEED as f64 * APPROACH_SPEED as f64,
            0.0,
        )
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn approach_travel_row(time: f32) -> f32 {
    approach_travel_state(time as f64).0 as f32
}

#[cfg(feature = "phase0-audit")]
pub(super) fn travel_row(time: f32) -> f32 {
    approach_travel_row(time)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn birth_shape(birth_position: f32) -> f32 {
    smoothstep(
        (birth_position - approach_travel_row(GRID_SQUARE_START))
            / (approach_travel_row(GRID_HORIZONTAL) - approach_travel_row(GRID_SQUARE_START)),
    )
}

#[cfg(feature = "phase0-audit")]
pub(super) fn line_birth(index: usize) -> f32 {
    if index < SPOKE_LINES {
        let ring = index % GRID_RINGS;
        if ring + 1 < GRID_RINGS {
            ring_birth(ring + 1)
        } else {
            ring_birth(GRID_RINGS)
        }
    } else {
        let ring = (index - SPOKE_LINES) / (GRID_LANES - 1);
        ring_birth(ring)
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn grid_section_point(lane: f32, reshaping: f32) -> (f32, f32) {
    let angle = ((lane + 14.0) * 1_024.0 / 28.0) as i32;
    let circular_x = sine(angle + 256);
    let circular_y = sine(angle);
    let square_edge = circular_x.abs().max(circular_y.abs()).max(0.001);
    let square_x = circular_x / square_edge;
    let square_y = circular_y / square_edge;
    let section_x = circular_x + (square_x - circular_x) * reshaping;
    let section_y = circular_y + (square_y - circular_y) * reshaping;
    let side_push = 1.0 + reshaping * 3.2;
    (
        section_x * side_push,
        section_y * 0.61,
    )
}
