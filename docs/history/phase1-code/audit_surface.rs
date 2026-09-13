//! ARCHIVED phase-1 implementation. Not compiled or used by the active flight.
#[cfg(feature = "phase0-audit")]
use super::{
    DEPTH, FIELD_MOTION_BLUR_EXPOSURE, FIELD_RENDER_STATS, FIELD_RING_BASE_SPACING,
    FIELD_RING_REGULAR_COUNT, FINAL_ALTITUDE_METERS, FLOW_FINAL_ALTITUDE_WORLD, FLOW_FOCAL,
    FLOW_MAX_RELIEF_MILES, FLOW_PATH_END, FLOW_PLANET_RADIUS_WORLD, FRAME, FlowCamera,
    GRID_LANES, HEIGHT, LOCAL_VALLEY_BLEND_HALF_WIDTH_MILES,
    LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES, LOCAL_VALLEY_WALL_HALF_WIDTH_MILES, METERS_PER_MILE,
    ORBIT_ENTRY, PIXELS, PLANET_CENTER, PLANET_RADIUS_MILES, RailObjectIdentity,
    SURFACE_CELL_PIXELS, SURFACE_ROW_CAPACITY, SurfaceLod, Vec3, WIDTH, WORLD_BRAKE_BIAS,
    WORLD_BRAKE_CAPTURE_DISTANCE, WORLD_BRAKE_DISTANCE, WORLD_BRAKE_END, WORLD_BRAKE_END_TICK,
    WORLD_BRAKE_SAMPLES, WORLD_BRAKE_START, WORLD_BRAKE_START_TICK, WORLD_CURVE,
    WORLD_CURVE_SAMPLES, WORLD_CURVE_STEP, WORLD_ORBIT_END_PHASE, append, append_number,
    apply_atmospheric_shell, brake_log, clear_depth_buffer, draw_second_counter, dvec_add,
    dvec_dot, dvec_dot_vec3, dvec_from_vec3, dvec_length, dvec_normalize, dvec_scale, dvec_sub,
    exit, fast_sqrt, field_first_physical_ring_ordinal, field_first_ring_ordinal,
    field_last_ring_ordinal, field_ring_identity, field_ring_lower_bound, field_ring_s,
    field_star_exposure, field_star_exposure_camera, fill_universe_vacuum,
    flow_base_surface_height, flow_camera, flow_camera_ray, flow_descent_end,
    flow_normal_to_planet, flow_sphere_roots, flow_sun_projection, flow_surface_color,
    flow_surface_map_from_normal, flow_surface_point, flow_surface_relief_lod_world,
    flow_surface_relief_world, flow_surface_sample, flow_valley_corridor, flow_valley_overhead,
    flow_visible_surface_point, full_surface_lod, inherited_watershed_floor, landform_height,
    landing_forward, landing_up, local_valley_distance, local_valley_longitudinal_weight,
    local_valley_width_scale, miles_to_world, now_ns, phase3_require, phase7_frame_hash,
    planet_continent_lod, planet_normal_to_flow, process_cpu_ns, project_flow,
    project_flow_depth, projected_surface_bounds, rail_object_exists, render_demo,
    render_flow_surface, render_universe_field, render_world_journey, sample_surface_vertex,
    surface_lod, surface_mesh_bounds, surface_mesh_step, surface_ray_height,
    surface_ray_intersection, trajectory_state, uncached_surface_sample, universe, vec_add,
    vec_cross, vec_dot, vec_normalize, vec_scale, watershed_center, watershed_floor_weight,
    world_curve_late_normal, world_curve_orbit_length, world_to_meters, world_to_miles,
    write_all,
};
use super::flight_speed;

#[cfg(feature = "phase0-audit")]
pub(super) fn phase4_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase4 spherical-surface audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase4_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase4_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_phase4_surface_audit() -> ! {
    let landing = landing_up();
    let relief = flow_surface_relief_world(landing);
    let surface_point = flow_surface_point(landing);
    phase4_require(
        relief >= 0.0
            && relief < miles_to_world(FLOW_MAX_RELIEF_MILES)
            && (dvec_length(surface_point) - FLOW_PLANET_RADIUS_WORLD - relief).abs()
                < 1.0e-8
            && surface_point == flow_surface_point(landing),
        190,
        b"canonical displaced surface is unbounded or non-deterministic",
    );

    let endpoint = flow_camera(flow_descent_end() as f32);
    let endpoint_normal =
        flow_normal_to_planet(dvec_normalize(dvec_sub(endpoint.position, PLANET_CENTER)));
    let endpoint_agl = dvec_length(dvec_sub(endpoint.position, PLANET_CENTER))
        - FLOW_PLANET_RADIUS_WORLD
        - flow_surface_relief_world(endpoint_normal);
    phase4_require(
        (world_to_meters(endpoint_agl) - FINAL_ALTITUDE_METERS).abs() < 0.02,
        191,
        b"final camera is not 20 metres above the canonical displaced surface",
    );

    let checkpoints = [
        29.0f32, 35.0, 42.0, 50.0, 60.0, 78.0, 94.0, 106.0, 116.0, 137.0,
    ];
    let aspect_scales = [0.5f32, 1.0, 2.0];
    let mut checkpoint = 0usize;
    while checkpoint < checkpoints.len() {
        let time = checkpoints[checkpoint];
        let camera = flow_camera(time);
        let radial_normal =
            flow_normal_to_planet(dvec_normalize(dvec_sub(camera.position, PLANET_CENTER)));
        let agl = dvec_length(dvec_sub(camera.position, PLANET_CENTER))
            - FLOW_PLANET_RADIUS_WORLD
            - flow_surface_relief_world(radial_normal);
        phase4_require(
            (world_to_miles(agl) - camera.altitude_miles).abs() < 0.000_01
                && camera.near_plane <= agl.max(FLOW_FINAL_ALTITUDE_WORLD) * 0.02,
            192 + checkpoint,
            b"camera AGL or near plane does not derive from the canonical surface",
        );

        let toward_center = dvec_normalize(dvec_sub(PLANET_CENTER, camera.position));
        phase4_require(
            flow_sphere_roots(&camera, toward_center, FLOW_PLANET_RADIUS_WORLD).is_some(),
            202 + checkpoint,
            b"camera ray cannot intersect the canonical solid core",
        );

        let unscaled = projected_surface_bounds(&camera, 1.0);
        phase4_require(
            unscaled.visible.is_finite()
                && unscaled.x.is_finite()
                && unscaled.y.is_finite()
                && unscaled.radius_x > 0.0
                && unscaled.radius_y > 0.0,
            212 + checkpoint,
            b"canonical surface bounds are invalid",
        );
        let mut aspect = 0usize;
        while aspect < aspect_scales.len() {
            let x_scale = aspect_scales[aspect];
            let bounds = projected_surface_bounds(&camera, x_scale);
            phase4_require(
                (bounds.x - (160.0 + (unscaled.x - 160.0) * x_scale)).abs() < 0.001
                    && (bounds.radius_x - unscaled.radius_x * x_scale).abs() < 0.001
                    && bounds.y == unscaled.y
                    && bounds.radius_y == unscaled.radius_y,
                222 + checkpoint,
                b"canonical surface bounds distort under terminal aspect correction",
            );
            aspect += 1;
        }

        let center_sample =
            sample_surface_vertex(&camera, time, 1.0, unscaled.x, unscaled.y);
        let repeated_sample =
            sample_surface_vertex(&camera, time, 1.0, unscaled.x, unscaled.y);
        phase4_require(
            center_sample.valid
                && center_sample.valid == repeated_sample.valid
                && center_sample.x == repeated_sample.x
                && center_sample.y == repeated_sample.y
                && center_sample.inverse_depth == repeated_sample.inverse_depth
                && center_sample.red == repeated_sample.red
                && center_sample.green == repeated_sample.green
                && center_sample.blue == repeated_sample.blue
                && center_sample.valid == repeated_sample.valid,
            232 + checkpoint,
            b"surface intersection is not deterministic",
        );
        checkpoint += 1;
    }

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
        phase4_require(
            bounds.0 <= previous_bounds.0
                && bounds.1 >= previous_bounds.1
                && bounds.2 <= previous_bounds.2
                && bounds.3 >= previous_bounds.3
                && bounds.0 >= -surface_mesh_step()
                && bounds.1 <= WIDTH as i32 + surface_mesh_step()
                && bounds.2 >= -surface_mesh_step()
                && bounds.3 <= HEIGHT as i32 + surface_mesh_step(),
            242,
            b"single surface lattice bounds are not conservative and nested",
        );
        previous_bounds = bounds;
        index += 1;
    }

    phase4_require(
        surface_mesh_step() == SURFACE_CELL_PIXELS
            && SURFACE_ROW_CAPACITY >= WIDTH / SURFACE_CELL_PIXELS as usize + 2,
        243,
        b"canonical polygon surface exceeds its fixed row budget",
    );
    write_all(
        b"phase4 polygon-surface audit ok: center=canonical bounds=displaced-sphere coverage=single-bounded-lattice/exact-boundary-rays surface-edges=absent agl=surface-relative final=20m\n",
    );
    exit(0)
}
#[cfg(feature = "phase0-audit")]
pub(super) fn phase5_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase5 refinement audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase5_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase5_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn landscape_flight_fingerprint() -> u64 {
    let mut result = 14_695_981_039_346_656_037u64;
    let mut index = 0;
    while index < WORLD_CURVE_SAMPLES {
        let node = unsafe { core::ptr::addr_of!(WORLD_CURVE[index]).read() };
        for value in [node.position.x, node.position.y, node.position.z,
            node.velocity.x, node.velocity.y, node.velocity.z,
            node.acceleration.x, node.acceleration.y, node.acceleration.z,
            node.phase, node.phase_rate] {
            result ^= value.to_bits();
            result = result.wrapping_mul(1_099_511_628_211);
        }
        index += 1;
    }
    result
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_landscape_profile_audit() -> ! {
    for time in [32.0, 40.0, 53.0, 122.0] {
        let camera = flow_camera(time);
        let before = field_star_exposure_camera(time, &camera);
        clear_depth_buffer();
        let start = process_cpu_ns();
        render_universe_field(&before, &camera, 1.0);
        let field_end = process_cpu_ns();
        apply_atmospheric_shell(&camera, 1.0);
        let sky_end = process_cpu_ns();
        unsafe { LANDSCAPE_HEIGHT_EVALUATIONS = 0; }
        render_flow_surface(&camera, time, 1.0);
        let surface_end = process_cpu_ns();
        let mut report = [0u8; 160];
        let p = report.as_mut_ptr();
        let mut n = 0;
        append(p, &mut n, b"landscape profile t/field-us/sky-us/surface-us/height-evals=");
        for value in [time as u32, ((field_end - start) / 1_000) as u32,
            ((sky_end - field_end) / 1_000) as u32, ((surface_end - sky_end) / 1_000) as u32,
            unsafe { LANDSCAPE_HEIGHT_EVALUATIONS }] {
            append_number(p, &mut n, value);
            append(p, &mut n, b"/");
        }
        append(p, &mut n, b"\n");
        write_all(&report[..n]);
    }
    let fingerprint = landscape_flight_fingerprint();
    let mut report = [0u8; 128];
    let p = report.as_mut_ptr();
    let mut n = 0;
    append(p, &mut n, b"landscape flight fingerprint high/low=");
    append_number(p, &mut n, (fingerprint >> 32) as u32);
    append(p, &mut n, b"/");
    append_number(p, &mut n, fingerprint as u32);
    append(p, &mut n, b"\n");
    write_all(&report[..n]);
    for time in [40.0f32, 42.0, 53.0, 58.0, 90.0, 122.0, 137.0] {
        let camera = flow_camera(time);
        let normal = flow_normal_to_planet(dvec_normalize(camera.position));
        let map = flow_surface_map_from_normal(normal);
        let mut report = [0u8; 256];
        let p = report.as_mut_ptr();
        let mut n = 0;
        append(p, &mut n, b"landscape t/map-x/map-y/cross-track/relief-millimiles=");
        for value in [time, map.0, map.1, local_valley_distance(map.0, map.1),
            world_to_miles(flow_surface_relief_world(normal)) as f32] {
            append_number(p, &mut n, (value * 1_000.0) as u32);
            append(p, &mut n, b"/");
        }
        append(p, &mut n, b" basis-f32bits=");
        for v in [camera.forward, camera.right, camera.down] {
            for value in [v.x, v.y, v.z] {
                append_number(p, &mut n, value.to_bits());
                append(p, &mut n, b"/");
            }
        }
        append(p, &mut n, b"\n");
        write_all(&report[..n]);
    }
    let camera = flow_camera(122.0);
    for y in [0.5, 32.5, 64.5, 96.5, 160.5] {
        for x in [0.5, 80.5, 160.5, 240.5, 319.5] {
            let ray = flow_camera_ray(&camera, 1.0, x, y);
            let distance = surface_ray_intersection(&camera, ray).unwrap_or(0.0);
            let (error, _, sample, _) = surface_ray_height(&camera, ray, distance);
            let mut report = [0u8; 160];
            let p = report.as_mut_ptr();
            let mut n = 0;
            append(p, &mut n, b"wall ray x/y/distance-m/cross-m/map-x-milli/height/residual-mm=");
            for value in [x as f64, y as f64, world_to_meters(distance),
                local_valley_distance(sample.map_x, sample.map_y) as f64 * METERS_PER_MILE,
                sample.map_x as f64 * 1_000.0, sample.height as f64,
                world_to_meters(error.abs()) * 1_000.0] {
                append_number(p, &mut n, value as u32);
                append(p, &mut n, b"/");
            }
            append(p, &mut n, b"\n");
            write_all(&report[..n]);
        }
    }
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn orbital_horizon_y(camera: &FlowCamera, x: f32) -> Option<f32> {
    let mut top = -100.0;
    let mut previous = false;
    for y in -100..401 {
        let hit = flow_sphere_roots(camera, flow_camera_ray(camera, 1.0, x, y as f32),
            FLOW_PLANET_RADIUS_WORLD).is_some();
        if hit && !previous {
            let mut bottom = y as f32;
            for _ in 0..20 {
                let mid = (top + bottom) * 0.5;
                if flow_sphere_roots(camera, flow_camera_ray(camera, 1.0, x, mid),
                    FLOW_PLANET_RADIUS_WORLD).is_some() { bottom = mid; } else { top = mid; }
            }
            return Some((top + bottom) * 0.5);
        }
        top = y as f32;
        previous = hit;
    }
    None
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_speed_function_audit() -> ! {
    let mut last = flight_speed::sample(4.0);
    let mut maximum_rate = 0.0f64;
    let mut maximum_rate_time = 0.0;
    for tick in 4 * 1_920..=140 * 1_920 {
        let time = tick as f64 / 1_920.0;
        let sample = flight_speed::sample(time);
        phase3_require(sample.speed.is_finite() && sample.acceleration.is_finite()
            && sample.jerk.is_finite() && sample.speed >= flight_speed::CRUISE_MPS
            && sample.speed <= last.speed && sample.acceleration <= 0.0,
            253, b"explicit speed function is nonfinite, negative, or accelerates");
        let rate = -sample.acceleration / sample.speed;
        if rate > maximum_rate { maximum_rate = rate; maximum_rate_time = time; }
        if time > 30.0 {
            phase3_require(rate <= -last.acceleration / last.speed + 1.0e-12
                && sample.acceleration >= last.acceleration,
                253, b"explicit speed function strengthens its late brake");
        }
        last = sample;
    }
    phase3_require((flight_speed::sample(40.0).speed - 600.0).abs() < 1.0e-8
        && maximum_rate < 1.3 && maximum_rate_time < 30.0,
        253, b"explicit speed function misses its deadline or late-taper bounds");
    for second in 4..=60 {
        let sample = flight_speed::sample(second as f64);
        let mut report = [0u8; 128];
        let p = report.as_mut_ptr();
        let mut n = 0;
        append(p, &mut n, b"PROPOSED speed seconds/mps/deceleration-mps2/fractional-rate-per-million=");
        for value in [second, sample.speed as u32, (-sample.acceleration) as u32,
            (-sample.acceleration / sample.speed * 1_000_000.0) as u32] {
            append_number(p, &mut n, value); append(p, &mut n, b"/");
        }
        append(p, &mut n, b"\n"); write_all(&report[..n]);
    }
    let mut report = [0u8; 192];
    let p = report.as_mut_ptr();
    let mut n = 0;
    append(p, &mut n, b"PROPOSED peak-rate-per-million/time-ms/distance-40-to58-metres=");
    for value in [maximum_rate * 1_000_000.0, maximum_rate_time * 1_000.0,
        flight_speed::distance(40.0, 58.0)] {
        append_number(p, &mut n, value as u32); append(p, &mut n, b"/");
    }
    append(p, &mut n, b"\nexplicit function checks pass; NOT connected to the camera\n");
    write_all(&report[..n]);
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_wormhole_emergence_audit() -> ! {
    let field = universe().field;
    // The approved camera and exact 32-ms shutter remain intact at every
    // opening sample, including the 4-second endpoint.
    for tick in 0..=3_840 {
        let time = tick as f32 / 960.0;
        let camera = flow_camera(time);
        phase3_require(field_star_exposure(field, camera.position).to_bits()
            == FIELD_MOTION_BLUR_EXPOSURE.to_bits(),
            255, b"opening star exposure changed");
    }
    // Compare complete rendered opening frames to the old fixed-shutter
    // invocation in RAM. No image files, stored baseline, or tolerance.
    for tick in 0..=96 {
        let time = tick as f32 / 24.0;
        let camera = flow_camera(time);
        render_demo(time, 1.0);
        let current = phase7_frame_hash();
        let before = flow_camera((time - FIELD_MOTION_BLUR_EXPOSURE).max(0.0));
        clear_depth_buffer();
        render_universe_field(&before, &camera, 1.0);
        render_world_journey(time, 1.0, &camera);
        draw_second_counter(time);
        phase3_require(phase7_frame_hash() == current,
            255, b"opening frame differs from the unchanged fixed-shutter renderer");
    }
    let mut previous_exposure = FIELD_MOTION_BLUR_EXPOSURE;
    for tick in 480..=600 {
        let time = tick as f32 / 120.0;
        let exposure = field_star_exposure(field, flow_camera(time).position);
        phase3_require(exposure >= 0.0 && exposure <= previous_exposure,
            255, b"emergence rebuilds star motion blur");
        previous_exposure = exposure;
    }
    phase3_require(previous_exposure == 0.0,
        255, b"star exposure persists beyond wormhole emergence");
    // Probe the physical boundary independently of the approved path. A
    // different flight cannot restart streaks simply by taking longer.
    for offset in [16.0, 32.0, 64.0, 160.0, 384.0] {
        let point = dvec_add(field.axis_origin,
            dvec_scale(field.forward, field.ordering_start_s + offset));
        phase3_require(field_star_exposure(field, point) == 0.0,
            255, b"post-wormhole shutter depends on a flight timestamp");
    }
    // Inspect actual submitted, screen-clipped primitives across the entire
    // ring/circular interval. Point stars are retained until the square grid;
    // the geometry, not streak length, supplies the sense of forward motion.
    let mut minimum_spokes = u32::MAX;
    let mut minimum_rings = u32::MAX;
    let mut speed = flow_camera(4.0).speed_world_per_second;
    for tick in 120..=840 {
        let time = tick as f32 / 30.0;
        let camera = flow_camera(time);
        phase3_require(camera.speed_world_per_second <= speed + 1.0e-7,
            255, b"post-wormhole camera accelerates to build the grid");
        speed = camera.speed_world_per_second;
        if time >= 5.0 && time <= 21.0 {
            render_demo(time, 1.0);
            let stats = unsafe { FIELD_RENDER_STATS };
            phase3_require(stats.star_streaks == 0,
                255, b"actual post-wormhole draw calls still contain star streaks");
            phase3_require(stats.star_points > 0 && stats.ring_edges >= 8 && stats.spoke_edges >= 6,
                255, b"removing star blur removed point stars, rings, or assembling spokes");
            minimum_spokes = minimum_spokes.min(stats.spoke_edges);
            minimum_rings = minimum_rings.min(stats.ring_edges);
        }
    }
    let first = field_first_physical_ring_ordinal(field);
    let mature = field_ring_lower_bound(field, field.rails_full_s);
    let mut previous_lanes = 0;
    let mut assembly_steps = 0;
    for ordinal in first..=mature {
        let mut lanes = 0;
        for lane in 0..(GRID_LANES - 1) as u16 {
            if rail_object_exists(field, RailObjectIdentity {
                first_ring: field_ring_identity(ordinal), lane,
            }) { lanes += 1; }
        }
        phase3_require(lanes >= previous_lanes,
            255, b"spoke assembly loses fixed connections downstream");
        if lanes > previous_lanes { assembly_steps += 1; }
        previous_lanes = lanes;
    }
    phase3_require(assembly_steps >= 16 && previous_lanes == GRID_LANES - 1,
        255, b"spokes no longer assemble progressively into the complete cylinder");
    for aspect in [0.5, 2.0] {
        for time in [5.0, 11.0, 19.0] {
            render_demo(time, aspect);
            let stats = unsafe { FIELD_RENDER_STATS };
            phase3_require(stats.star_streaks == 0 && stats.star_points > 0
                && stats.ring_edges >= 8 && stats.spoke_edges >= 6,
                255, b"terminal aspect ratio restores streaks or loses the assembling grid");
        }
    }
    // Negative witness: the old full shutter DOES draw post-exit streaks.
    // This prevents the gate passing merely because it missed the stars.
    let camera = flow_camera(11.0);
    let before = flow_camera(11.0 - FIELD_MOTION_BLUR_EXPOSURE);
    clear_depth_buffer();
    render_universe_field(&before, &camera, 1.0);
    phase3_require(unsafe { FIELD_RENDER_STATS.star_streaks } > 0,
        255, b"star-streak regression detector cannot distinguish the old renderer");
    let mut report = [0u8; 192];
    let p = report.as_mut_ptr();
    let mut n = 0;
    append(p, &mut n, b"wormhole emergence: opening frames exact=97; post-exit streaks=0; minimum rings/spokes/assembly-steps=");
    for value in [minimum_rings, minimum_spokes, assembly_steps] {
        append_number(p, &mut n, value); append(p, &mut n, b"/");
    }
    append(p, &mut n, b"; speed nonincreasing; old-shutter negative witness detected\n");
    write_all(&report[..n]);
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_grid_density_audit() -> ! {
    let field = universe().field;
    let first = field_first_physical_ring_ordinal(field);
    let last = field_last_ring_ordinal(field);
    let point = |n| field_ring_s(field, field_ring_identity(n));
    let virtual_first = field_first_ring_ordinal(field);
    phase3_require(point(virtual_first) <= field.field_start_s
        && point(virtual_first + 1) > field.field_start_s
        && point(first) >= field.ring_start_s && point(first - 1) < field.ring_start_s
        && point(last) >= field.field_end_s && point(last - 1) < field.field_end_s,
        254, b"fixed ring catalogue no longer spans its physical field bounds");
    let first_pitch = point(first + 1) - point(first);
    phase3_require((2.0..4.0).contains(&first_pitch),
        254, b"entry grid gaps still inherit the oversized astronomical scale");
    let mut maximum = 0.0f64;
    for ordinal in first..last {
        let a = point(ordinal);
        let b = point(ordinal + 1);
        let pitch = b - a;
        phase3_require(pitch >= FIELD_RING_BASE_SPACING - 1.0e-12
            && pitch <= first_pitch + 1.0e-12,
            254, b"physical ring catalogue is reversed or has an oversized gap");
        maximum = maximum.max(pitch);
        phase3_require(field_ring_lower_bound(field, a) == ordinal
            && field_ring_lower_bound(field, (a + b) * 0.5) == ordinal + 1,
            254, b"ring range lookup disagrees with immutable coordinates");
        if a > -FIELD_RING_BASE_SPACING * FIELD_RING_REGULAR_COUNT
            && b < FIELD_RING_BASE_SPACING * FIELD_RING_REGULAR_COUNT {
            phase3_require((pitch - 1.5).abs() < 1.0e-12,
                254, b"mature grid pitch changes as the ship approaches the planet");
        }
    }
    let mut report = [0u8; 160];
    let p = report.as_mut_ptr();
    let mut n = 0;
    append(p, &mut n, b"grid rings/front-pitch-milli/regular-pitch-milli/entry-crossings-per-second-milli=");
    for value in [(last - first + 1) as u32, (maximum * 1_000.0) as u32,
        (FIELD_RING_BASE_SPACING * 1_000.0) as u32, (18.0 / first_pitch * 1_000.0) as u32] {
        append_number(p, &mut n, value); append(p, &mut n, b"/");
    }
    append(p, &mut n, b"\n"); write_all(&report[..n]);
    for time in [4.0, 8.0, 11.0, 19.0, 21.0, 26.0] {
        let start = now_ns();
        render_demo(time, 1.0);
        let elapsed = now_ns() - start;
        n = 0;
        append(p, &mut n, b"dense-grid seconds/render-us=");
        append_number(p, &mut n, time as u32); append(p, &mut n, b"/");
        append_number(p, &mut n, (elapsed / 1_000) as u32);
        append(p, &mut n, b"\n"); write_all(&report[..n]);
    }
    write_all(b"fixed physical grid density: ok; render timings reported separately\n");
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_orbital_descent_profile() -> ! {
    let mut setup = [0u8; 192];
    let p = setup.as_mut_ptr();
    let mut n = 0;
    append(p, &mut n, b"orbit setup capture/arc/covered/bias-milli=");
    for value in [unsafe { WORLD_BRAKE_CAPTURE_DISTANCE }, world_curve_orbit_length(WORLD_ORBIT_END_PHASE),
        unsafe { WORLD_BRAKE_DISTANCE[WORLD_BRAKE_SAMPLES - 1] }, unsafe { WORLD_BRAKE_BIAS }] {
        append_number(p, &mut n, (value * 1_000.0) as u32); append(p, &mut n, b"/");
    }
    append(p, &mut n, b"\n"); write_all(&setup[..n]);
    let speed_at = |t: f64| dvec_length(trajectory_state(t).velocity);
    let total = brake_log(speed_at(WORLD_BRAKE_START) / speed_at(WORLD_BRAKE_END));
    let mut maximum_short = 0.0f64;
    let mut maximum_long = 0.0f64;
    for tick in WORLD_BRAKE_START_TICK..WORLD_BRAKE_END_TICK {
        let t = tick as f64 * WORLD_CURVE_STEP;
        maximum_short = maximum_short.max(1.0 - speed_at((t + 0.1).min(WORLD_BRAKE_END)) / speed_at(t));
        maximum_long = maximum_long.max(brake_log(speed_at(t) / speed_at((t + 2.0).min(WORLD_BRAKE_END))) / total);
    }
    n = 0;
    append(p, &mut n, b"brake distribution by35/36to38/max100ms/max2s-permille=");
    for value in [brake_log(speed_at(WORLD_BRAKE_START) / speed_at(35.0)) / total,
        brake_log(speed_at(36.0) / speed_at(38.0)) / total, maximum_short, maximum_long] {
        append_number(p, &mut n, (value * 1_000.0) as u32); append(p, &mut n, b"/");
    }
    append(p, &mut n, b"\n"); write_all(&setup[..n]);
    for tick in 280..=400 {
        let time = tick as f32 / 10.0;
        let camera = flow_camera(time);
        let state = trajectory_state(time as f64);
        let sag = match (orbital_horizon_y(&camera, 64.0), orbital_horizon_y(&camera, 160.0),
            orbital_horizon_y(&camera, 256.0)) {
            (Some(a), Some(b), Some(c)) => ((a + c) * 0.5 - b).abs(),
            _ => 999.0,
        };
        let sun = flow_sun_projection(&camera, 1.0);
        let mut exposed = 0;
        if let Some((x, y)) = sun {
            if x >= -14.0 && x < WIDTH as f32 + 14.0 && y >= -14.0 && y < HEIGHT as f32 + 14.0 {
                render_demo(time, 1.0);
                for dy in -14..=14 {
                    for dx in -14..=14 {
                        let px = x as i32 + dx;
                        let py = y as i32 + dy;
                        if dx * dx + dy * dy <= 14 * 14 && px >= 0 && px < WIDTH as i32
                            && py >= 0 && py < HEIGHT as i32 {
                            let pixel = py as usize * WIDTH + px as usize;
                            let depth = unsafe { core::ptr::addr_of!(DEPTH[pixel]).read() };
                            let rgb = unsafe { core::ptr::addr_of!(FRAME).cast::<u8>().add(pixel * 3) };
                            if depth == f32::MAX && unsafe { rgb.read() > 180 && rgb.add(1).read() > 110 } {
                                exposed += 1;
                            }
                        }
                    }
                }
            }
        }
        let mut report = [0u8; 240];
        let p = report.as_mut_ptr();
        let mut n = 0;
        append(p, &mut n, b"orbit profile ms/turn-milli/agl-m/speed-mps/limb-sag-millipx/sun-x+10000/sun-y+10000/exposed=");
        for value in [(time * 1_000.0) as u32, (state.phase * 1_000.0 / 1_024.0) as u32,
            (camera.altitude_miles * METERS_PER_MILE) as u32, world_to_meters(dvec_length(state.velocity)) as u32,
            (sag * 1_000.0) as u32, sun.map_or(0, |s| (s.0 + 10_000.0) as u32),
            sun.map_or(0, |s| (s.1 + 10_000.0) as u32), exposed] {
            append_number(p, &mut n, value); append(p, &mut n, b"/");
        }
        append(p, &mut n, b"\n"); write_all(&report[..n]);
    }
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_orbital_descent_audit() -> ! {
    let mut curved_frames = 0;
    let mut curved_start = f64::MAX;
    let mut curved_end = 0.0f64;
    let mut sun_frames = [0u32; 5];
    let mut sun_peak = [0u32; 5];
    for tick in 28 * 60..34 * 60 {
        let time = tick as f32 / 60.0;
        let camera = flow_camera(time);
        let state = trajectory_state(time as f64);
        if let (Some(a), Some(b), Some(c)) = (orbital_horizon_y(&camera, 64.0),
            orbital_horizon_y(&camera, 160.0), orbital_horizon_y(&camera, 256.0)) {
            let sag = ((a + c) * 0.5 - b).abs();
            if (3.0..50.0).contains(&sag) {
                curved_frames += 1;
                curved_start = curved_start.min(state.phase);
                curved_end = curved_end.max(state.phase);
            }
        }
        let orbit = (state.phase / 1_024.0) as usize;
        if state.phase <= 0.0 || orbit >= sun_frames.len() { continue; }
        if let Some((x, y)) = flow_sun_projection(&camera, 1.0) {
            if !(0.0..WIDTH as f32).contains(&x) || !(0.0..HEIGHT as f32).contains(&y) { continue; }
            render_demo(time, 1.0);
            let mut exposed = 0;
            for dy in -14..=14 {
                for dx in -14..=14 {
                    let px = x as i32 + dx;
                    let py = y as i32 + dy;
                    if dx * dx + dy * dy > 14 * 14 || px < 0 || px >= WIDTH as i32
                        || py < 0 || py >= HEIGHT as i32 { continue; }
                    let pixel = py as usize * WIDTH + px as usize;
                    let depth = unsafe { core::ptr::addr_of!(DEPTH[pixel]).read() };
                    let rgb = unsafe { core::ptr::addr_of!(FRAME).cast::<u8>().add(pixel * 3) };
                    if depth == f32::MAX && unsafe { rgb.read() > 180 && rgb.add(1).read() > 110 } { exposed += 1; }
                }
            }
            if exposed >= 100 { sun_frames[orbit] += 1; }
            sun_peak[orbit] = sun_peak[orbit].max(exposed);
        }
    }
    let mut report = [0u8; 320];
    let p = report.as_mut_ptr();
    let mut n = 0;
    append(p, &mut n, b"descent curved-limb-ms/revolutions-milli/sun-frames-at60hz-per-orbit=");
    append_number(p, &mut n, curved_frames * 1_000 / 60);
    append(p, &mut n, b"/");
    append_number(p, &mut n, ((curved_end - curved_start) * 1_000.0 / 1_024.0) as u32);
    for value in sun_frames { append(p, &mut n, b"/"); append_number(p, &mut n, value); }
    append(p, &mut n, b" sun-peak-pixels=");
    for value in sun_peak { append_number(p, &mut n, value); append(p, &mut n, b"/"); }
    append(p, &mut n, b"\n"); write_all(&report[..n]);
    // This catches the former 0.3-second globe-to-flat collapse; merely
    // appending a nominal low-speed orbit cannot satisfy the angular test.
    phase3_require(curved_frames >= 90 && curved_end - curved_start >= 1_024.0,
        251, b"curved-limb descent lasts less than 1.5 seconds or one actual revolution");
    for time in [31.0, 32.0, 33.0] {
        let before = flow_camera(time).altitude_miles;
        let after = flow_camera(time + 1.0).altitude_miles;
        phase3_require(before > after * 2.0, 251, b"31-34-second descent has a perceptually idle altitude interval");
    }
    phase3_require(sun_frames[..4].iter().all(|frames| *frames >= 6),
        252, b"fixed world sun lacks a rendered 100-ms transit during each early revolution");
    write_all(b"orbital descent and recurring fixed-sun visibility: ok (live review still required)\n");
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn check_legacy_landscape_fingerprint() {
    phase5_require(landscape_flight_fingerprint() == ((1_120_209_875u64 << 32) | 1_356_210_471),
        225, b"landscape work changed the frozen flight geometry or speed schedule");
    write_all(b"legacy landscape flight fingerprint: ok (historical lock, not current flight approval)\n");
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_phase5_refinement_audit() -> ! {
    check_legacy_landscape_fingerprint();
    run_phase5_refinement_checks()
}

// The aggregate reports the historical fingerprint as its OWN failing gate.
// Running these checks independently does not waive or reset that fingerprint.
#[cfg(feature = "phase0-audit")]
pub(super) fn run_phase5_refinement_checks() -> ! {
    for time in [32.0, 40.0, 53.0, 58.0, 90.0, 122.0, 137.0] {
        let camera = flow_camera(time);
        for offset in 0..128 {
            let point = dvec_add(camera.position,
                dvec_scale(dvec_from_vec3(camera.forward), miles_to_world(offset as f64 * 0.013)));
            let normal = flow_normal_to_planet(dvec_normalize(dvec_sub(point, PLANET_CENTER)));
            for footprint in [0.0, 0.025, 0.25 / 3.0, 0.084] {
                let lod = surface_lod(footprint);
                let expected = uncached_surface_sample(normal, lod);
                let actual = flow_surface_sample(normal, lod);
                phase5_require(expected.height.to_bits() == actual.height.to_bits()
                    && expected.continent.to_bits() == actual.continent.to_bits()
                    && expected.relief_world.to_bits() == actual.relief_world.to_bits()
                    && expected.map_x.to_bits() == actual.map_x.to_bits()
                    && expected.map_y.to_bits() == actual.map_y.to_bits(),
                    230, b"terrain memoization changes the canonical sample");
            }
        }
    }
    let mut distinct_tiles = 0;
    for i in 0..32 {
        let map = (37.1 + i as f32 * 3.7, 204.7 + i as f32 * 1.9);
        if (landform_height(map, full_surface_lod())
            - landform_height((map.0 + 256.0, map.1), full_surface_lod())).abs() > 2.0 {
            distinct_tiles += 1;
        }
    }
    phase5_require(distinct_tiles >= 24, 226, b"landscape still repeats at the inherited 256-mile tile period");
    for x in [18.0, 22.0, 24.5, 30.0] {
        for offset in [-256.0, 256.0] {
            phase5_require(local_valley_distance(x, watershed_center(x) + offset) > 250.0,
                226, b"canyon geometry wraps at the former texture period");
        }
    }
    let mut maximum_airplane_relief = 0.0f32;
    let mut minimum_airplane_relief = f32::MAX;
    let mut landmark_motion = 0;
    let first_camera = flow_camera(40.0);
    let second_camera = flow_camera(42.0);
    for y in [96.0, 112.0, 128.0, 144.0, 160.0, 176.0] {
        for x in [32.0, 64.0, 96.0, 128.0, 192.0, 224.0, 256.0, 288.0] {
            let ray = flow_camera_ray(&first_camera, 1.0, x, y);
            if let Some(distance) = surface_ray_intersection(&first_camera, ray) {
                let (error, normal, sample, _) = surface_ray_height(&first_camera, ray, distance);
                phase5_require(world_to_meters(error.abs()) < 1.0,
                    227, b"airplane surface intersection does not lie on the displaced terrain");
                maximum_airplane_relief = maximum_airplane_relief.max(sample.height);
                minimum_airplane_relief = minimum_airplane_relief.min(sample.height);
                let point = flow_surface_point(normal);
                if let (Some(a), Some(b)) = (project_flow(&first_camera, point), project_flow(&second_camera, point)) {
                    let motion = fast_sqrt((a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1));
                    if b.0 > 0.0 && b.0 < WIDTH as f32 && b.1 > 0.0 && b.1 < HEIGHT as f32
                        && motion > 0.25 && motion < 40.0 { landmark_motion += 1; }
                }
            }
        }
    }
    phase5_require(maximum_airplane_relief - minimum_airplane_relief > 12.0 && landmark_motion >= 12,
        228, b"airplane pass lacks resolved relief or persistent perceptibly moving landmarks");
    let mut framing_target_met = true;
    for time in [40.0, 41.0, 42.0] {
        let camera = flow_camera(time);
        clear_depth_buffer();
        fill_universe_vacuum();
        render_flow_surface(&camera, time, 1.0);
        let mut covered = 0;
        let mut colors = [false; 512];
        let mut color_count = 0;
        for pixel in 0..PIXELS {
            if unsafe { core::ptr::addr_of!(DEPTH[pixel]).read() } < f32::MAX {
                covered += 1;
                let rgb = unsafe { core::ptr::addr_of!(FRAME).cast::<u8>().add(pixel * 3) };
                let bin = unsafe { (rgb.read() as usize / 32)
                    + (rgb.add(1).read() as usize / 32) * 8 + (rgb.add(2).read() as usize / 32) * 64 };
                if !colors[bin] { colors[bin] = true; color_count += 1; }
            }
        }
        let mut report = [0u8; 128];
        let p = report.as_mut_ptr();
        let mut n = 0;
        append(p, &mut n, b"phase5 high pass t/coverage-permille/color-bins/landmarks=");
        for value in [time as u32, (covered * 1_000 / PIXELS) as u32, color_count, landmark_motion] {
            append_number(p, &mut n, value);
            append(p, &mut n, b"/");
        }
        append(p, &mut n, b"\n");
        write_all(&report[..n]);
        framing_target_met &= covered * 100 >= PIXELS * 70 && covered * 100 <= PIXELS * 85;
        phase5_require(color_count >= 8,
            229, b"high-pass terrain lacks readable material/relief variation");
    }
    let footprints = [5_000.0f64, 1_000.0, 300.0, 50.0, 3.0, 0.1];
    let mut previous = SurfaceLod { continent: 0.0, ranges: 0.0, mountains: 0.0, valley: 0.0, footprint_miles: f64::MAX };
    let mut index = 0usize;
    while index < footprints.len() {
        let lod = surface_lod(footprints[index]);
        phase5_require(
            lod.continent + 1.0e-6 >= lod.ranges
                && lod.ranges + 1.0e-6 >= lod.mountains
                && lod.mountains + 1.0e-6 >= lod.valley,
            200,
            b"child terrain resolves before its parent",
        );
        phase5_require(
            lod.continent + 1.0e-6 >= previous.continent
                && lod.ranges + 1.0e-6 >= previous.ranges
                && lod.mountains + 1.0e-6 >= previous.mountains
                && lod.valley + 1.0e-6 >= previous.valley,
            201,
            b"terrain refinement reverses while footprint shrinks",
        );
        previous = lod;
        index += 1;
    }
    phase5_require(
        surface_lod(300.0).continent > surface_lod(300.0).mountains
            && surface_lod(50.0).ranges > surface_lod(50.0).valley
            && surface_lod(3.0).valley > 0.8,
        202,
        b"continent/range/mountain/valley hierarchy is not ordered",
    );

    let normals = [
        landing_up(),
        vec_normalize(Vec3 { x: 0.18, y: -0.96, z: 0.21 }),
        vec_normalize(Vec3 { x: -0.31, y: 0.42, z: -0.85 }),
    ];
    index = 0;
    while index < normals.len() {
        let normal = normals[index];
        let map = flow_surface_map_from_normal(normal);
        let floor = watershed_floor_weight(map);
        let expected = if floor == 1.0 { inherited_watershed_floor(map, full_surface_lod()) }
            else { let base = landform_height(map, full_surface_lod());
                base + (inherited_watershed_floor(map, full_surface_lod()) - base) * floor };
        phase5_require((flow_base_surface_height(normal, full_surface_lod()) - expected).abs() < 0.002,
            203, b"child bands do not reconstruct the persistent terrain source");
        let parent = flow_surface_relief_lod_world(normal, surface_lod(300.0));
        let child = flow_surface_relief_lod_world(normal, surface_lod(3.0));
        let parent_direction = dvec_normalize(dvec_scale(
            planet_normal_to_flow(normal),
            FLOW_PLANET_RADIUS_WORLD + parent,
        ));
        let child_direction = dvec_normalize(dvec_scale(
            planet_normal_to_flow(normal),
            FLOW_PLANET_RADIUS_WORLD + child,
        ));
        phase5_require(
            dvec_dot(parent_direction, child_direction) > 0.999_999_999,
            204,
            b"refinement detaches a landmark from its spherical coordinate",
        );
        index += 1;
    }

    let orbital_lod = surface_lod(1_000.0);
    phase5_require(
        orbital_lod.continent > 0.9
            && orbital_lod.ranges < 0.05
            && orbital_lod.mountains < 0.01
            && orbital_lod.valley < 0.01,
        205,
        b"orbital surface does not isolate persistent continent-scale detail",
    );
    let landing_continent = planet_continent_lod(landing_up(), orbital_lod);
    let opposite = vec_scale(landing_up(), -1.0);
    let opposite_continent = planet_continent_lod(opposite, orbital_lod);
    phase5_require(
        landing_continent > 0.8 && opposite_continent < 0.3,
        206,
        b"orbital continent silhouette is not geographically readable",
    );
    let color_a = flow_surface_color(
        normals[0],
        74.0,
        flow_surface_sample(normals[0], orbital_lod),
        orbital_lod,
    );
    let color_b = flow_surface_color(
        opposite,
        74.0,
        flow_surface_sample(opposite, orbital_lod),
        orbital_lod,
    );
    let color_c = flow_surface_color(
        normals[2],
        74.0,
        flow_surface_sample(normals[2], orbital_lod),
        orbital_lod,
    );
    let color_distance = |first: (u8, u8, u8), second: (u8, u8, u8)| {
        (first.0 as i32 - second.0 as i32).abs()
            + (first.1 as i32 - second.1 as i32).abs()
            + (first.2 as i32 - second.2 as i32).abs()
    };
    phase5_require(
        color_distance(color_a, color_b) > 18,
        208,
        b"landing and second orbital biome colors are indistinct",
    );
    phase5_require(
        color_distance(color_a, color_c) > 18,
        211,
        b"landing and third orbital biome colors are indistinct",
    );
    phase5_require(
        color_distance(color_b, color_c) > 18,
        212,
        b"second and third orbital biome colors are indistinct",
    );

    let frame_step = 1.0f32 / 60.0;
    let mut time = ORBIT_ENTRY;
    while time < FLOW_PATH_END as f32 {
        let first = flow_camera(time);
        let second = flow_camera(time + frame_step);
        index = 0;
        while index < normals.len() {
            let normal = normals[index];
            let nominal_first = dvec_add(
                PLANET_CENTER,
                dvec_scale(planet_normal_to_flow(normal), FLOW_PLANET_RADIUS_WORLD),
            );
            let nominal_second = dvec_add(
                PLANET_CENTER,
                dvec_scale(planet_normal_to_flow(normal), FLOW_PLANET_RADIUS_WORLD),
            );
            let distance_first = dvec_length(dvec_sub(nominal_first, first.position));
            let distance_second = dvec_length(dvec_sub(nominal_second, second.position));
            let relief_first = flow_surface_relief_lod_world(
                normal,
                surface_lod(world_to_miles(distance_first) / FLOW_FOCAL as f64),
            );
            let relief_second = flow_surface_relief_lod_world(
                normal,
                surface_lod(world_to_miles(distance_second) / FLOW_FOCAL as f64),
            );
            let pixel_motion = (relief_second - relief_first).abs()
                / distance_first.min(distance_second).max(1.0e-12)
                * FLOW_FOCAL as f64;
            phase5_require(
                pixel_motion < 1.0,
                207,
                b"one refinement frame moves geometry by more than one pixel",
            );
            index += 1;
        }
        time += frame_step;
    }

    let valley_time = flow_valley_overhead() as f32;
    let valley_camera = flow_camera(valley_time);
    // Inspect the canyon ahead, not the point already below/behind the ship
    // on the obsolete 94-second schedule.
    let valley_normal = flow_normal_to_planet(world_curve_late_normal(flow_descent_end() + 30.0));
    let valley_point = flow_visible_surface_point(&valley_camera, valley_normal);
    let valley_distance = dvec_length(dvec_sub(valley_point, valley_camera.position));
    phase5_require(
        surface_lod(world_to_miles(valley_distance) / FLOW_FOCAL as f64).valley > 0.20,
        209,
        b"destination valley detail is unresolved before entry",
    );
    phase5_require(
        project_flow(&valley_camera, valley_point).is_some(),
        210,
        b"destination valley is outside the forward view before entry",
    );
    let valley_projection = project_flow_depth(&valley_camera, valley_point)
        .unwrap_or_else(|| phase5_fail(213, b"destination valley is behind the overhead camera"));
    phase5_require(
        valley_projection.0 >= 0.5
            && valley_projection.0 < WIDTH as f32 - 0.5
            && valley_projection.1 >= 0.5
            && valley_projection.1 < HEIGHT as f32 - 0.5,
        214,
        b"destination valley projects outside the overhead frame",
    );
    clear_depth_buffer();
    render_flow_surface(&valley_camera, valley_time, 1.0);
    let valley_x = valley_projection.0 as usize;
    let valley_pixel_y = valley_projection.1 as usize;
    let rendered_depth = unsafe {
        core::ptr::addr_of!(DEPTH)
            .cast::<f32>()
            .add(valley_pixel_y * WIDTH + valley_x)
            .read()
    };
    phase5_require(
        rendered_depth < f32::MAX,
        215,
        b"overhead valley point has no rendered surface ownership",
    );
    let exact_surface = sample_surface_vertex(
        &valley_camera,
        valley_time,
        1.0,
        valley_projection.0,
        valley_projection.1,
    );
    phase5_require(
        exact_surface.valid && exact_surface.inverse_depth > 0.0,
        216,
        b"exact overhead valley ray has no shared-surface intersection",
    );
    let exact_ray = flow_camera_ray(
        &valley_camera,
        1.0,
        valley_projection.0,
        valley_projection.1,
    );
    let ray_forward = dvec_dot_vec3(exact_ray, valley_camera.forward).max(0.001);
    let exact_depth = 1.0 / exact_surface.inverse_depth;
    let rendered_point = dvec_add(
        valley_camera.position,
        dvec_scale(exact_ray, exact_depth as f64 / ray_forward),
    );
    let rendered_normal = flow_normal_to_planet(dvec_normalize(dvec_sub(
        rendered_point,
        PLANET_CENTER,
    )));
    let valley_alignment = vec_dot(rendered_normal, valley_normal);
    let valley_depth_error =
        (exact_depth - valley_projection.2).abs() / valley_projection.2.max(1.0e-9);
    if valley_alignment <= 0.999_99 || valley_depth_error >= 0.05 {
        let mut report = [0u8; 176];
        let report_pointer = report.as_mut_ptr();
        let mut report_length = 0usize;
        append(report_pointer, &mut report_length, b"phase5 refinement audit failed: overhead valley alignment ppm/x/y=");
        append_number(
            report_pointer,
            &mut report_length,
            ((valley_alignment + 1.0) * 1_000_000.0) as u32,
        );
        append(report_pointer, &mut report_length, b"/");
        append_number(report_pointer, &mut report_length, valley_x as u32);
        append(report_pointer, &mut report_length, b"/");
        append_number(report_pointer, &mut report_length, valley_pixel_y as u32);
        append(report_pointer, &mut report_length, b" depth-error-ppm=");
        append_number(
            report_pointer,
            &mut report_length,
            (valley_depth_error * 1_000_000.0) as u32,
        );
        append(report_pointer, &mut report_length, b"\n");
        write_all(&report[..report_length]);
        exit(216)
    }

    // Visibility from above is not enough: after the overhead pass the
    // physical camera track must enter the same excavated valley and remain
    // inside it through low flight.  Check the production spherical-map
    // coordinates densely, then require the named controls to converge on the
    // valley centreline instead of merely ending at a nearby normal.
    let mut corridor_time = flow_valley_overhead();
    while corridor_time <= FLOW_PATH_END {
        let camera = flow_camera(corridor_time as f32);
        let normal = flow_normal_to_planet(dvec_normalize(dvec_sub(
            camera.position,
            PLANET_CENTER,
        )));
        let map = flow_surface_map_from_normal(normal);
        let cross_track = local_valley_distance(map.0, map.1);
        let floor_half_width = LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES
            * local_valley_width_scale(map.0);
        phase5_require(
            cross_track < floor_half_width,
            217,
            b"low-flight camera leaves the nested valley floor",
        );
        corridor_time += 0.25;
    }
    let corridor_controls = [flow_valley_overhead() as f32, flow_valley_corridor() as f32,
        flow_descent_end() as f32, FLOW_PATH_END as f32];
    let mut previous_cross_track = f32::MAX;
    let mut corridor_index = 0usize;
    while corridor_index < corridor_controls.len() {
        let camera = flow_camera(corridor_controls[corridor_index]);
        let normal = flow_normal_to_planet(dvec_normalize(dvec_sub(
            camera.position,
            PLANET_CENTER,
        )));
        let map = flow_surface_map_from_normal(normal);
        let cross_track = local_valley_distance(map.0, map.1);
        phase5_require(
            cross_track <= previous_cross_track + 0.01,
            218,
            b"camera track moves away from the valley centreline",
        );
        previous_cross_track = cross_track;
        corridor_index += 1;
    }
    phase5_require(
        previous_cross_track < 0.05,
        219,
        b"low-flight endpoint misses the generated valley centreline",
    );

    let wall_offset = LOCAL_VALLEY_WALL_HALF_WIDTH_MILES / PLANET_RADIUS_MILES as f32;
    let wall_radial = fast_sqrt((1.0 - wall_offset * wall_offset).max(0.0));
    let valley_centre = landing_up();
    let valley_right = vec_normalize(vec_cross(landing_forward(), landing_up()));
    let left_wall = vec_normalize(vec_add(
        vec_scale(valley_centre, wall_radial),
        vec_scale(valley_right, -wall_offset),
    ));
    let right_wall = vec_normalize(vec_add(
        vec_scale(valley_centre, wall_radial),
        vec_scale(valley_right, wall_offset),
    ));
    let floor_relief = flow_surface_relief_world(valley_centre);
    let left_relief = flow_surface_relief_world(left_wall);
    let right_relief = flow_surface_relief_world(right_wall);
    phase5_require(
        world_to_miles(left_relief - floor_relief) > 0.10
            && world_to_miles(right_relief - floor_relief) > 0.10,
        220,
        b"nested valley walls do not rise around the low-flight floor",
    );
    for wall_time in [58.0, 90.0, 122.0, 137.0] {
    let wall_camera = flow_camera(wall_time);
    clear_depth_buffer();
    render_flow_surface(&wall_camera, wall_time, 1.0);
    let depth = core::ptr::addr_of!(DEPTH).cast::<f32>();
    let mut left_wall_pixels = 0usize;
    let mut right_wall_pixels = 0usize;
    let mut wall_y = 0usize;
    while wall_y < HEIGHT {
        let mut wall_x = 0usize;
        while wall_x < WIDTH {
            let stored = unsafe { depth.add(wall_y * WIDTH + wall_x).read() };
            if stored < f32::MAX {
                let ray = flow_camera_ray(
                    &wall_camera,
                    1.0,
                    wall_x as f32 + 0.5,
                    wall_y as f32 + 0.5,
                );
                let ray_forward = dvec_dot_vec3(ray, wall_camera.forward).max(0.001);
                let point = dvec_add(
                    wall_camera.position,
                    dvec_scale(ray, stored as f64 / ray_forward),
                );
                let normal = flow_normal_to_planet(dvec_normalize(dvec_sub(
                    point,
                    PLANET_CENTER,
                )));
                let map = flow_surface_map_from_normal(normal);
                let width_scale = local_valley_width_scale(map.0);
                let distance = local_valley_distance(map.0, map.1);
                let floor_edge = LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES * width_scale;
                let wall_edge = LOCAL_VALLEY_BLEND_HALF_WIDTH_MILES * width_scale;
                if local_valley_longitudinal_weight(map.0) > 0.5
                    && distance > floor_edge
                    && distance < wall_edge
                {
                    if wall_x < WIDTH / 2 {
                        left_wall_pixels += 1;
                    } else {
                        right_wall_pixels += 1;
                    }
                }
            }
            wall_x += 1;
        }
        wall_y += 1;
    }
    {
        let mut report = [0u8; 128];
        let pointer = report.as_mut_ptr();
        let mut length = 0;
        append(pointer, &mut length, b"phase5 wall-depth t/left/right pixels=");
        append_number(pointer, &mut length, wall_time as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, left_wall_pixels as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, right_wall_pixels as u32);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        phase5_require(left_wall_pixels >= 16 && right_wall_pixels >= 16, 224,
            b"both valley walls do not own rendered depth around the low-flight frame");
    }
    }

    let camera = flow_camera(122.0);
    let reference = dvec_length(camera.position) - miles_to_world(camera.altitude_miles);
    let mut above_floor_wall_rays = 0;
    for x in [8.0, 32.0, 64.0, 256.0, 288.0, 312.0] {
        for y in [0.0, 16.0, 32.0, 48.0, 64.0] {
            let ray = flow_camera_ray(&camera, 1.0, x, y);
            if flow_sphere_roots(&camera, ray, reference).is_none() {
                if let Some(distance) = surface_ray_intersection(&camera, ray) {
                    let (error, _, sample, _) = surface_ray_height(&camera, ray, distance);
                    let epsilon = miles_to_world(0.25 / METERS_PER_MILE);
                    let crossing = surface_ray_height(&camera, ray, distance - epsilon).0 >= 0.0
                        && surface_ray_height(&camera, ray, distance + epsilon).0 <= 0.0;
                    if world_to_meters(error.abs()) >= 1.0 && !crossing {
                        let mut report = [0u8; 128];
                        let p = report.as_mut_ptr();
                        let mut n = 0;
                        append(p, &mut n, b"phase5 wall residual x/y/distance-m/error-mm=");
                        for value in [x as u32, y as u32, world_to_meters(distance) as u32,
                            (world_to_meters(error.abs()) * 1_000.0) as u32] {
                            append_number(p, &mut n, value);
                            append(p, &mut n, b"/");
                        }
                        append(p, &mut n, b"\n");
                        write_all(&report[..n]);
                    }
                    phase5_require(world_to_meters(error.abs()) < 1.0 || crossing,
                        227, b"canyon wall intersection is not on the canonical relief");
                    if local_valley_distance(sample.map_x, sample.map_y) > LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES {
                        above_floor_wall_rays += 1;
                    }
                }
            }
        }
    }
    phase5_require(above_floor_wall_rays >= 8, 224,
        b"surface still misses walls above the reference floor sphere");

    phase5_require(framing_target_met, 229,
        b"frozen camera remains below the planned 70-85 percent terrain coverage; framing decision pending");

    write_all(
        b"phase5 refinement audit ok: parent-child=attached bands=ordered geomorph<1px valley=rendered/visible-before-entry/track-inside/walls-rendered\n",
    );
    exit(0)
}

#[cfg(feature = "phase0-audit")]
pub(super) static mut LANDSCAPE_HEIGHT_EVALUATIONS: u32 = 0;
