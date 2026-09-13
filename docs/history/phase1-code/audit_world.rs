//! ARCHIVED phase-1 implementation. Not compiled or used by the active flight.
#[cfg(feature = "phase0-audit")]
use super::{
    BACKGROUND_STAR_COUNT, BackgroundStarIdentity, CLEAN_BIRTH, CanonicalStarParticleIdentity,
    DVec3, FIELD_MOTION_BLUR_EXPOSURE, FIELD_RING_BASE_SPACING, FIELD_RING_CHUNK_SIZE,
    FIELD_RING_DISTANCE_TABLE, FIELD_RING_DISTANCE_TABLE_LEN, FIELD_STAR_CATALOGUE_JOIN_S,
    FIELD_STAR_OBJECTS_PER_CELL, FLIGHT_TURN_ACCELERATION_LIMIT, FLIGHT_TURN_RATE_LIMIT,
    FLOW_ATMOSPHERE_RADIUS_WORLD, FLOW_FOCAL, FLOW_PLANET_RADIUS_WORLD, FieldEdgeIdentity,
    FieldEdgeKind, FieldRegion, FlowCamera, GRID_APPROACH_AXIS_OFFSET,
    GRID_BOTTOM_HEIGHT_WORLD, GRID_LANES, GRID_RINGS, GRID_SQUARE_START,
    GRID_WELL_RADIUS_WORLD, HEIGHT, LIGHT_YEAR_MILES, ORBIT_ENTRY, PLANET_CENTER, PLANET_INTRO,
    RING_BIRTH, RailObjectIdentity, RingObjectIdentity, SPEED_OF_LIGHT_MILES_PER_SECOND,
    SPOKE_LINES, STARFIELD_HOLD, STARFLIGHT_JOIN_TIME, STAR_CELL_COUNT, STAR_LINES,
    STAR_ORGANIZE, STAR_ORIGINAL_CELL_COUNT, SUN_DIRECTION, StarObjectIdentity,
    TRAJECTORY_ENTRY_CLEARANCE, TRAJECTORY_ENTRY_RATE, TRAJECTORY_HZ, TRAJECTORY_STEP, WIDTH,
    WORLD_BRAKE_START, WORLD_CURVE_AIRCRAFT_PHASE_RATE, WORLD_CURVE_START,
    WORLD_ORBIT_END_PHASE, append, append_number, approach_camera_reference,
    background_star_position, birth_shape, canonical_flow_field_transport,
    canonical_flow_grid_world_point, canonical_flyby_segment, canonical_star_exposure_times,
    canonical_star_generation, canonical_star_particle_reference,
    canonical_star_particle_world_position, clip_screen_line, dvec_add, dvec_dot,
    dvec_dot_vec3, dvec_from_vec3, dvec_length, dvec_length_squared, dvec_normalize,
    dvec_scale, dvec_sub, exit, fast_sqrt, field_axis_coordinate, field_edge_points,
    field_exterior_visibility, field_first_physical_ring_ordinal, field_first_ring_ordinal,
    field_last_ring_ordinal, field_mesh_vertex, field_mesh_visibility, field_point,
    field_region, field_ring_distance_value, field_ring_identity, field_ring_ordinal,
    field_ring_s, field_section, field_square_amount, fixed_candidate_from_rays,
    flight_volume_contains, flow_aircraft_arrival, flow_atmosphere_amount, flow_camera,
    flow_entry_heat, flow_field_basis, flow_sun_direction, flyby_segment, grid_fog_range,
    grid_section_point, legacy_trajectory_state, line_birth, line_coordinates,
    local_star_field_visibility, local_star_object_visibility, miles_to_world,
    phase0_basis_valid, project_canonical_flow_grid_segment,
    project_canonical_flow_star_segment, project_field_segment, project_field_star_exposure,
    project_flow_depth, projected_point_error, projected_surface_bounds,
    rail_lane_start_ordinal, rail_object_exists, rail_object_segment, ring_birth,
    ring_object_segment, ring_state, star_cell_bounds, star_cell_identity,
    star_cell_may_project, star_cell_ordinal, star_object_exists, star_object_position,
    starflight_displacement, trajectory_phase_crossing, trajectory_state, travel_row, universe,
    universe_field_layout, universe_space_visibility, vec_dot, vec_length, vec_scale, vec_sub,
    world_curve_approach_speed, world_curve_capture_frame, world_curve_capture_geometry,
    world_curve_spiral_metric, world_curve_spiral_speed, write_all,
};

#[cfg(feature = "phase0-audit")]
pub(super) fn phase3_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase3 unified-camera audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase3_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase3_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_phase3_unified_camera_audit() -> ! {
    let fixed_universe = universe();
    let field = fixed_universe.field;
    let repeated_field = universe_field_layout();
    phase3_require(
        field == repeated_field
            && fixed_universe.planet_center == PLANET_CENTER
            && fixed_universe.atmosphere_radius == FLOW_ATMOSPHERE_RADIUS_WORLD
            && fixed_universe.sun_direction.x.to_bits() == SUN_DIRECTION.x.to_bits()
            && fixed_universe.sun_direction.y.to_bits() == SUN_DIRECTION.y.to_bits()
            && fixed_universe.sun_direction.z.to_bits() == SUN_DIRECTION.z.to_bits()
            && field.field_start_s < field.ordering_start_s
            && field.ordering_start_s < field.ring_start_s
            && field.ring_start_s < field.rings_full_s
            && field.rings_full_s < field.star_volume_end_s
            && field.rings_full_s < field.rails_full_s
            && field.rails_full_s < field.square_start_s
            && field.square_start_s < field.star_volume_end_s
            && field.star_volume_end_s < field.square_end_s
            && field.square_start_s < field.square_end_s
            && field.square_end_s < field.planet_s
            && field.planet_s < field.field_end_s,
        168,
        b"anchored field regions are not deterministic and spatially ordered",
    );
    phase3_require(
        universe_space_visibility(&flow_camera(ORBIT_ENTRY)) == 1.0
            && universe_space_visibility(&flow_camera(flow_aircraft_arrival() as f32)) == 0.0
            && universe_space_visibility(&flow_camera(65.0)) == 0.0,
        168,
        b"physical atmosphere does not continuously extinguish the space field by 30000 feet",
    );
    let camera_s = |time: f64| {
        field_axis_coordinate(field.axis_origin, field.forward, trajectory_state(time).position)
    };
    let boundary_bits = [
        (camera_s(0.0) - GRID_BOTTOM_HEIGHT_WORLD * 8.0).to_bits(),
        camera_s(STARFIELD_HOLD as f64).to_bits(),
        camera_s(STAR_ORGANIZE as f64).to_bits(),
        camera_s(CLEAN_BIRTH as f64).to_bits(),
        camera_s(GRID_SQUARE_START as f64).to_bits(),
        camera_s((ORBIT_ENTRY - 2.0) as f64).to_bits(),
    ];
    let field_boundary_bits = [
        field.field_start_s.to_bits(),
        field.ordering_start_s.to_bits(),
        field.rings_full_s.to_bits(),
        field.rails_full_s.to_bits(),
        field.square_start_s.to_bits(),
        field.square_end_s.to_bits(),
    ];
    let mut boundary_mismatch = false;
    let mut boundary = 0usize;
    while boundary < boundary_bits.len() {
        boundary_mismatch |= if boundary < 4 {
            boundary_bits[boundary] != field_boundary_bits[boundary]
        } else {
            (f64::from_bits(boundary_bits[boundary])
                - f64::from_bits(field_boundary_bits[boundary]))
                .abs()
                > 1.0
        };
        boundary += 1;
    }
    if boundary_mismatch {
        let mut report = [0u8; 384];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 field-boundary bits=");
        boundary = 0;
        while boundary < boundary_bits.len() {
            if boundary > 0 {
                append(pointer, &mut length, b",");
            }
            append_number(pointer, &mut length, (boundary_bits[boundary] >> 32) as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, boundary_bits[boundary] as u32);
            boundary += 1;
        }
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(168)
    }
    phase3_require(
        (field.field_start_s - (camera_s(0.0) - GRID_BOTTOM_HEIGHT_WORLD * 8.0)).abs()
            < 1.0e-5,
        168,
        b"fixed astronomical field start no longer matches the opening camera",
    );
    phase3_require(
        (field.ordering_start_s - camera_s(STARFIELD_HOLD as f64)).abs() < 1.0e-5,
        168,
        b"fixed astronomical streak boundary no longer matches its event",
    );
    if !((field.rings_full_s - camera_s(STAR_ORGANIZE as f64)).abs() < 1.0e-5
        && (field.rails_full_s - camera_s(CLEAN_BIRTH as f64)).abs() < 1.0e-5
        && (field.square_start_s - camera_s(GRID_SQUARE_START as f64)).abs() < 1.0)
    {
        let mut report = [0u8; 160];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(
            pointer,
            &mut length,
            b"phase3 near-field boundary micro-s ring/rail/square=",
        );
        append_number(
            pointer,
            &mut length,
            (-camera_s(STAR_ORGANIZE as f64) * 1_000_000.0) as u32,
        );
        append(pointer, &mut length, b"/");
        append_number(
            pointer,
            &mut length,
            (-camera_s(CLEAN_BIRTH as f64) * 1_000_000.0) as u32,
        );
        append(pointer, &mut length, b"/");
        append_number(
            pointer,
            &mut length,
            (-camera_s(GRID_SQUARE_START as f64) * 1_000_000.0) as u32,
        );
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(168)
    }
    let square_full_s = camera_s((ORBIT_ENTRY - 2.0) as f64);
    if (field.square_end_s - square_full_s).abs() > 1.0 {
        let mut report = [0u8; 128];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 square-full-s bits high/low=");
        append_number(pointer, &mut length, (square_full_s.to_bits() >> 32) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, square_full_s.to_bits() as u32);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(168)
    }
    let light_year_world = miles_to_world(LIGHT_YEAR_MILES);
    let speed_of_light_world = miles_to_world(SPEED_OF_LIGHT_MILES_PER_SECOND);
    let opening_to_grid = field.rings_full_s - camera_s(0.0);
    let opening_grid_radius_pixels =
        FLOW_FOCAL as f64 * field.radius / opening_to_grid.max(1.0e-12);
    let mut starflight_time = 0.0f64;
    let mut previous_starflight_s = camera_s(0.0) - 1.0;
    let mut minimum_starflight_speed = f64::MAX;
    while starflight_time <= STAR_ORGANIZE as f64 {
        let state = trajectory_state(starflight_time);
        let s = field_axis_coordinate(field.axis_origin, field.forward, state.position);
        minimum_starflight_speed = minimum_starflight_speed.min(dvec_length(state.velocity));
        phase3_require(
            s > previous_starflight_s,
            185,
            b"astronomical approach does not move monotonically toward the grid",
        );
        previous_starflight_s = s;
        starflight_time += 0.25;
    }
    phase3_require(
        (opening_to_grid / light_year_world - 4.0).abs() < 0.001
            && opening_grid_radius_pixels < 0.000_1
            && minimum_starflight_speed > speed_of_light_world
            && starflight_displacement(STARFLIGHT_JOIN_TIME) == (0.0, 0.0, 0.0),
        185,
        b"opening field is not astronomical, subpixel, superluminal, and C2-joined",
    );
    let mut approach_tick = (STARFIELD_HOLD * TRAJECTORY_HZ as f32) as u32 + 1;
    let approach_end_tick = (WORLD_CURVE_START * TRAJECTORY_HZ as f64) as u32;
    let mut previous_approach_s = camera_s(STARFIELD_HOLD as f64);
    let mut previous_approach_axial_speed = 18.0f64;
    let mut previous_approach_speed = 18.0f64;
    let mut minimum_approach_axial_speed = f64::MAX;
    while approach_tick <= approach_end_tick {
        let time = approach_tick as f64 / TRAJECTORY_HZ as f64;
        let state = trajectory_state(time);
        let s = field_axis_coordinate(field.axis_origin, field.forward, state.position);
        let axial_speed = dvec_dot(state.velocity, field.forward);
        let speed = dvec_length(state.velocity);
        minimum_approach_axial_speed = minimum_approach_axial_speed.min(axial_speed);
        if s <= previous_approach_s
            || axial_speed <= 0.0
            || axial_speed > previous_approach_axial_speed + 1.0e-8
            || speed > previous_approach_speed + 1.0e-8
        {
            let mut report = [0u8; 144];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase3 approach fails deceleration at ms=");
            append_number(pointer, &mut length, (time * 1_000.0) as u32);
            append(pointer, &mut length, b" speed-micro=");
            append_number(
                pointer,
                &mut length,
                (speed.max(0.0) * 1_000_000.0) as u32,
            );
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(185)
        }
        previous_approach_s = s;
        previous_approach_axial_speed = axial_speed;
        previous_approach_speed = speed;
        approach_tick += 1;
    }
    phase3_require(
        minimum_approach_axial_speed >= -approach_camera_reference(WORLD_CURVE_START).1 - 1.0e-8,
        185,
        b"approved straight approach falls below its 20.5-second target",
    );
    let approach_join = approach_camera_reference(STARFIELD_HOLD as f64);
    let approach_entry = approach_camera_reference(ORBIT_ENTRY as f64);
    let expected_join_clearance = TRAJECTORY_ENTRY_CLEARANCE
        + (ORBIT_ENTRY - STARFIELD_HOLD) as f64
            * (18.0 - TRAJECTORY_ENTRY_RATE)
            * 0.5;
    phase3_require(
        approach_join.0.to_bits() == expected_join_clearance.to_bits()
            && approach_join.1.to_bits() == (-18.0f64).to_bits()
            && approach_join.2.to_bits() == 0.0f64.to_bits()
            && approach_entry.0.to_bits() == TRAJECTORY_ENTRY_CLEARANCE.to_bits()
            && approach_entry.1.to_bits() == TRAJECTORY_ENTRY_RATE.to_bits()
            && approach_entry.2.to_bits() == 0.0f64.to_bits(),
        185,
        b"decelerating approach misses its 18-to-13.258 endpoint states",
    );
    let (capture_orbit_normal, capture_orbit_tangent) = world_curve_capture_frame();
    let capture_start_state = legacy_trajectory_state(WORLD_CURVE_START);
    let mut capture_parameter_sample = 0u32;
    let mut capture_inside_samples = 0u32;
    while capture_parameter_sample <= 1_000 {
        let parameter = capture_parameter_sample as f64 / 1_000.0;
        let position = world_curve_capture_geometry(
            parameter,
            capture_start_state,
            capture_orbit_normal,
            capture_orbit_tangent,
        )
        .0;
        let relative = dvec_sub(position, field.axis_origin);
        let right = dvec_dot(relative, field.right);
        let down = dvec_dot(relative, field.down);
        if !flight_volume_contains(position) {
            let mut report = [0u8; 160];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase3 capture containment failed parameter/right+20000/down+20000-milli=");
            append_number(pointer, &mut length, capture_parameter_sample);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, ((right + 20_000.0) * 1_000.0) as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, ((down + 20_000.0) * 1_000.0) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(185)
        }
        capture_inside_samples += 1;
        capture_parameter_sample += 1;
    }
    phase3_require(
        capture_inside_samples == 1_001,
        185,
        b"capture curve is not wholly contained by the squared tunnel",
    );
    let mut curve_tick = (WORLD_CURVE_START * TRAJECTORY_HZ as f64) as u32 + 1;
    let curve_end_tick = (33.0 * TRAJECTORY_HZ as f64) as u32;
    let mut previous_curve_state = trajectory_state(WORLD_CURVE_START);
    let mut maximum_curve_speed_error = 0.0f64;
    let mut maximum_curve_speed_error_tick = curve_tick;
    while curve_tick <= curve_end_tick {
        let time = curve_tick as f64 / TRAJECTORY_HZ as f64;
        let state = trajectory_state(time);
        if time <= PLANET_INTRO as f64 {
            phase3_require(
                flight_volume_contains(state.position),
                185,
                b"capture camera leaves the squared tunnel before planet introduction",
            );
        }
        let speed = dvec_length(state.velocity);
        let expected_speed = if state.phase > 0.0 {
            world_curve_spiral_speed(state.phase, time)
        } else {
            world_curve_approach_speed(time)
        };
        let speed_error = (speed - expected_speed).abs();
        if speed_error > maximum_curve_speed_error {
            maximum_curve_speed_error = speed_error;
            maximum_curve_speed_error_tick = curve_tick;
        }
        let velocity_dot = dvec_dot(
            dvec_normalize(previous_curve_state.velocity),
            dvec_normalize(state.velocity),
        );
        let tangent_change = dvec_length(dvec_sub(dvec_normalize(previous_curve_state.velocity),
            dvec_normalize(state.velocity)));
        if !(tangent_change <= FLIGHT_TURN_RATE_LIMIT * TRAJECTORY_STEP
            && phase0_basis_valid(flow_camera(time as f32))) {
            let mut report = [0u8; 144];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase3 capture tangent failed ms/dot+1e6/basis=");
            append_number(pointer, &mut length, (time * 1_000.0) as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, ((velocity_dot + 1.0) * 1_000_000.0) as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, phase0_basis_valid(flow_camera(time as f32)) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(185)
        }
        previous_curve_state = state;
        curve_tick += 1;
    }
    if maximum_curve_speed_error >= 0.01 {
        let mut report = [0u8; 96];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 capture speed error micro=");
        append_number(pointer, &mut length, (maximum_curve_speed_error * 1_000_000.0) as u32);
        append(pointer, &mut length, b" at-ms=");
        append_number(pointer, &mut length, maximum_curve_speed_error_tick * 1_000 / TRAJECTORY_HZ as u32);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(185)
    }
    let interior_camera = flow_camera(20.0);
    phase3_require(
        local_star_field_visibility(field, flow_camera(0.0).position) == 1.0
            && local_star_field_visibility(
                field,
                dvec_add(
                    field.axis_origin,
                    dvec_scale(field.forward, field.square_start_s),
                ),
            ) == 1.0
            && local_star_field_visibility(
                field,
                dvec_add(
                    field.axis_origin,
                    dvec_scale(field.forward, field.star_volume_end_s),
                ),
            ) == 0.0
            && local_star_field_visibility(field, flow_camera(ORBIT_ENTRY).position) == 0.0
            && field_exterior_visibility(field, interior_camera.position) == 0.0,
        185,
        b"near stars do not remain full through the circular tunnel and fade inside the square grid",
    );
    let protected_opening_camera = flow_camera(STARFIELD_HOLD);
    let mut opening_extension_stars = 0u32;
    let mut extension_cell_index = STAR_ORIGINAL_CELL_COUNT;
    while extension_cell_index < STAR_CELL_COUNT {
        let cell = star_cell_identity(extension_cell_index);
        let mut object = 0u8;
        while object < FIELD_STAR_OBJECTS_PER_CELL {
            let star = StarObjectIdentity { cell, object };
            if star_object_exists(field, star) {
                let point = star_object_position(field, star);
                phase3_require(
                    local_star_object_visibility(
                        star,
                        protected_opening_camera.position,
                        point,
                    ) == 0.0,
                    185,
                    b"forward star extension alters the protected opening through four seconds",
                );
                opening_extension_stars += 1;
            }
            object += 1;
        }
        extension_cell_index += 1;
    }
    phase3_require(
        opening_extension_stars >= 1_000,
        185,
        b"forward fixed-star extension is underpopulated",
    );
    let mut first_exterior_found = false;
    let mut second_exterior_found = false;
    let mut first_exterior_phase = 0.0f64;
    let mut first_exterior_camera = interior_camera;
    let mut second_exterior_camera = interior_camera;
    let mut exterior_tick = (ORBIT_ENTRY * TRAJECTORY_HZ as f32) as u32;
    let exterior_end_tick = (flow_aircraft_arrival() * TRAJECTORY_HZ as f64) as u32;
    while exterior_tick <= exterior_end_tick && !second_exterior_found {
        let time = exterior_tick as f64 / TRAJECTORY_HZ as f64;
        let camera = flow_camera(time as f32);
        if field_exterior_visibility(field, camera.position) >= 0.99 {
            if first_exterior_found {
                // Compare nearby views, not opposite hemispheres half a
                // second apart during the newly approved fast first orbit.
                if trajectory_state(time).phase - first_exterior_phase >= 64.0 {
                    second_exterior_camera = camera;
                    second_exterior_found = true;
                }
            } else {
                first_exterior_phase = trajectory_state(time).phase;
                first_exterior_camera = camera;
                first_exterior_found = true;
            }
        }
        exterior_tick += 1;
    }
    phase3_require(
        first_exterior_found,
        185,
        b"orbit never leaves the horizontal grid enclosure",
    );
    phase3_require(
        second_exterior_found,
        185,
        b"orbit does not sustain an exterior view for background parallax",
    );
    let mut background_parallax_stars = 0u32;
    let mut maximum_background_parallax = 0.0f32;
    let translated_exterior_camera = FlowCamera {
        position: second_exterior_camera.position,
        ..first_exterior_camera
    };
    let mut background_ordinal = 0u16;
    while background_ordinal < BACKGROUND_STAR_COUNT {
        let identity = BackgroundStarIdentity {
            ordinal: background_ordinal,
        };
        let point = background_star_position(fixed_universe, identity);
        phase3_require(
            point == background_star_position(fixed_universe, identity),
            185,
            b"background star identity does not retain one fixed world point",
        );
        if let (Some(first), Some(second)) = (
            project_flow_depth(&first_exterior_camera, point),
            project_flow_depth(&translated_exterior_camera, point),
        ) {
            if second.0 >= 0.0
                && second.0 < WIDTH as f32
                && second.1 >= 0.0
                && second.1 < HEIGHT as f32
            {
                background_parallax_stars += 1;
                maximum_background_parallax = maximum_background_parallax.max(fast_sqrt(
                    (second.0 - first.0) * (second.0 - first.0)
                        + (second.1 - first.1) * (second.1 - first.1),
                ));
            }
        }
        background_ordinal += 1;
    }
    phase3_require(
        background_parallax_stars >= 24 && maximum_background_parallax >= 0.25,
        185,
        b"fixed exterior background starfield lacks sparse parallax coverage",
    );
    phase3_require(
        (dvec_length(field.forward) - 1.0).abs() < 1.0e-6
            && (dvec_length(field.right) - 1.0).abs() < 1.0e-6
            && (dvec_length(field.down) - 1.0).abs() < 1.0e-6
            && dvec_dot(field.forward, field.right).abs() < 1.0e-6
            && dvec_dot(field.forward, field.down).abs() < 1.0e-6
            && dvec_dot(field.right, field.down).abs() < 1.0e-6
            && dvec_length(dvec_sub(
                field.axis_origin,
                dvec_sub(
                    PLANET_CENTER,
                    dvec_scale(field.down, GRID_APPROACH_AXIS_OFFSET),
                ),
            )) < 1.0e-12,
        168,
        b"anchored field frame is not fixed and orthonormal",
    );
    let midpoint = |a: f64, b: f64| a + (b - a) * 0.5;
    phase3_require(
        field_region(field, field.field_start_s) == FieldRegion::Starfield
            && field_region(
                field,
                midpoint(field.ordering_start_s, field.rings_full_s),
            ) == FieldRegion::RingApproach
            && field_region(
                field,
                midpoint(field.rings_full_s, field.rails_full_s),
            ) == FieldRegion::LogarithmicRings
            && field_region(
                field,
                midpoint(field.rails_full_s, field.square_start_s),
            ) == FieldRegion::CircularTunnel
            && field_region(
                field,
                midpoint(field.square_start_s, field.square_end_s),
            ) == FieldRegion::SquaringTunnel
            && field_region(field, field.field_end_s) == FieldRegion::SquaredTunnel,
        168,
        b"field regions do not cover the complete star-to-square geometry",
    );
    let circular_s = midpoint(field.rails_full_s, field.square_start_s);
    let circular_center = dvec_add(field.axis_origin, dvec_scale(field.forward, circular_s));
    let circular_right = field_point(field, circular_s, 0.0);
    let circular_top = field_point(field, circular_s, 0.75);
    let square_section = field_section(0.125, 1.0);
    let planet_plate = field_point(field, field.planet_s, 0.25);
    phase3_require(
        (dvec_length(dvec_sub(circular_right, circular_center)) - field.radius).abs()
            < 1.0e-5
            && (dvec_length(dvec_sub(circular_top, circular_center)) - field.radius).abs()
                < 1.0e-5,
        168,
        b"field circular region is not a world-space cylinder",
    );
    phase3_require(
        (square_section.0.abs().max(square_section.1.abs()) - 1.0)
            .abs()
            < 1.0e-5
            && square_section.0.abs() > 0.99
            && square_section.1.abs() > 0.99
            && field_square_amount(field, field.rails_full_s) == 0.0
            && field_square_amount(field, field.square_end_s) == 1.0,
        168,
        b"field circular-to-square cross-section is inconsistent",
    );
    phase3_require(
        dvec_length(dvec_sub(planet_plate, PLANET_CENTER)) < 1.0e-9
            && field_point(field, circular_s, 0.0) == circular_right,
        168,
        b"fixed field plate misses the planet or changes identity",
    );
    let mut ordinal = -256i32;
    while ordinal <= 256 {
        let identity = field_ring_identity(ordinal);
        phase3_require(
            identity.ring < FIELD_RING_CHUNK_SIZE as u8
                && field_ring_ordinal(identity) == ordinal
                && field_ring_s(field, identity).to_bits()
                    == field_ring_s(repeated_field, identity).to_bits(),
            183,
            b"stable ring identity changes chunk or world coordinate",
        );
        ordinal += 1;
    }
    let mut distance_step = 0usize;
    while distance_step < FIELD_RING_DISTANCE_TABLE_LEN {
        phase3_require(
            FIELD_RING_DISTANCE_TABLE[distance_step].to_bits()
                == field_ring_distance_value(distance_step as u32).to_bits(),
            183,
            b"ring distance cache changes its fixed coordinate function",
        );
        distance_step += 1;
    }
    let first_ring = field_first_ring_ordinal(field);
    let first_physical_ring = field_first_physical_ring_ordinal(field);
    let last_ring = field_last_ring_ordinal(field);
    let ring_object_count = (last_ring - first_physical_ring + 1) as u32;
    let mut rail_object_count = 0u32;
    let mut rail_start_mask = 0u64;
    let mut lane = 0u16;
    while lane < (GRID_LANES - 1) as u16 {
        let start = rail_lane_start_ordinal(field, lane);
        phase3_require(
            start >= first_physical_ring
                && field_ring_s(field, field_ring_identity(start)) < field.rails_full_s,
            183,
            b"rail lane does not have a permanent start in the formation volume",
        );
        rail_start_mask |= 1u64 << ((start - first_physical_ring) as u32).min(63);
        ordinal = first_physical_ring;
        while ordinal < last_ring {
            if rail_object_exists(
                field,
                RailObjectIdentity {
                    first_ring: field_ring_identity(ordinal),
                    lane,
                },
            ) {
                rail_object_count += 1;
            }
            ordinal += 1;
        }
        lane += 1;
    }
    let mut star_object_count = 0u32;
    let mut overlapping_star_count = 0u32;
    let mut star_cell_index = 0u32;
    while star_cell_index < STAR_CELL_COUNT {
        let cell = star_cell_identity(star_cell_index);
        phase3_require(
            star_cell_ordinal(cell) == star_cell_index,
            183,
            b"independent star-cell identity does not round trip",
        );
        let mut object = 0u8;
        while object < FIELD_STAR_OBJECTS_PER_CELL {
            let star = StarObjectIdentity { cell, object };
            if star_object_exists(field, star) {
                star_object_count += 1;
                let point = star_object_position(field, star);
                let s = field_axis_coordinate(field.axis_origin, field.forward, point);
                if s >= field.rings_full_s {
                    overlapping_star_count += 1;
                }
            }
            object += 1;
        }
        star_cell_index += 1;
    }
    let last_original_star_bounds = star_cell_bounds(
        field,
        star_cell_identity(STAR_ORIGINAL_CELL_COUNT - 1),
    );
    let first_extension_star_bounds = star_cell_bounds(
        field,
        star_cell_identity(STAR_ORIGINAL_CELL_COUNT),
    );
    let last_extension_star_bounds =
        star_cell_bounds(field, star_cell_identity(STAR_CELL_COUNT - 1));
    // Same assertions as before extraction, now named so a failing legacy
    // catalogue invariant does not report only unrelated object counts.
    let catalogue_checks: &[(&[u8], bool)] = &[
        (b"first ring ordinal", first_ring < -(GRID_RINGS as i32)),
        (b"last ring ordinal", last_ring > GRID_RINGS as i32),
        (b"first ring covers field start", field_ring_s(field, field_ring_identity(first_ring)) <= field.field_start_s),
        (b"next ring follows field start", field_ring_s(field, field_ring_identity(first_ring + 1)) > field.field_start_s),
        (b"last ring covers field end", field_ring_s(field, field_ring_identity(last_ring)) >= field.field_end_s),
        (b"previous ring precedes field end", field_ring_s(field, field_ring_identity(last_ring - 1)) < field.field_end_s),
        (b"first physical ring follows ring start", field_ring_s(field, field_ring_identity(first_physical_ring)) >= field.ring_start_s),
        (b"previous physical ring precedes ring start", field_ring_s(field, field_ring_identity(first_physical_ring - 1)) < field.ring_start_s),
        (b"original stars cover field start", last_original_star_bounds.0 <= field.field_start_s),
        (b"original star upper bound", last_original_star_bounds.1 > field.field_start_s),
        (b"extension catalogue join", first_extension_star_bounds.0.to_bits() == FIELD_STAR_CATALOGUE_JOIN_S.to_bits()),
        (b"last extension lower bound", last_extension_star_bounds.0 < field.star_volume_end_s),
        (b"last extension upper bound", last_extension_star_bounds.1 >= field.star_volume_end_s),
        (b"ring object count", ring_object_count >= 90),
        (b"rail object count", rail_object_count >= 6_000),
        (b"rail start bins", rail_start_mask.count_ones() >= 4),
        (b"star object count", star_object_count >= 12_000),
        (b"overlapping star count", overlapping_star_count >= 64),
    ];
    let catalogue_valid = catalogue_checks.iter().all(|check| check.1);
    if !catalogue_valid {
        for (label, valid) in catalogue_checks {
            if !valid {
                write_all(b"phase3 catalogue failed invariant: ");
                write_all(label);
                write_all(b"\n");
            }
        }
        let mut report = [0u8; 192];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 universe catalogue rings/rails/stars=");
        append_number(pointer, &mut length, ring_object_count);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, rail_object_count);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, star_object_count);
        append(pointer, &mut length, b" overlap/start-bins=");
        append_number(pointer, &mut length, overlapping_star_count);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, rail_start_mask.count_ones());
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(183)
    }
    let mut previous_s = field_ring_s(field, field_ring_identity(first_ring));
    let mut region_mask = 0u32;
    ordinal = first_ring;
    while ordinal <= last_ring {
        let identity = field_ring_identity(ordinal);
        let s = field_ring_s(field, identity);
        if ordinal > first_ring {
            phase3_require(
                s > previous_s,
                183,
                b"permanent ring order is not strictly axial",
            );
        }
        region_mask |= match field_region(field, s) {
            FieldRegion::Starfield => 1,
            FieldRegion::RingApproach => 2,
            FieldRegion::LogarithmicRings => 4,
            FieldRegion::CircularTunnel => 8,
            FieldRegion::SquaringTunnel => 16,
            FieldRegion::SquaredTunnel => 32,
        };
        previous_s = s;
        ordinal += 1;
    }
    phase3_require(
        region_mask == 63,
        183,
        b"stable ring identities do not cover every field region",
    );
    let downstream_spacing_1 = field_ring_s(field, field_ring_identity(2))
        - field_ring_s(field, field_ring_identity(1));
    let downstream_spacing_2 = field_ring_s(field, field_ring_identity(3))
        - field_ring_s(field, field_ring_identity(2));
    let upstream_spacing_1 = field_ring_s(field, field_ring_identity(-2))
        - field_ring_s(field, field_ring_identity(-1));
    let upstream_spacing_2 = field_ring_s(field, field_ring_identity(-3))
        - field_ring_s(field, field_ring_identity(-2));
    phase3_require(
        downstream_spacing_1 == FIELD_RING_BASE_SPACING
            && downstream_spacing_2 == FIELD_RING_BASE_SPACING
            && upstream_spacing_1 == -FIELD_RING_BASE_SPACING
            && upstream_spacing_2 == -FIELD_RING_BASE_SPACING,
        183,
        b"mature cylindrical/square grid does not have uniform physical pitch",
    );
    let topology_ring = field_ring_identity(-8);
    let next_ring_edge = FieldEdgeIdentity {
        ring: field_ring_identity(field_ring_ordinal(topology_ring) + 1),
        lane: 17,
        kind: FieldEdgeKind::Circumferential,
    };
    let next_lane_edge = FieldEdgeIdentity {
        ring: topology_ring,
        lane: 18,
        kind: FieldEdgeKind::Circumferential,
    };
    let axial_points = rail_object_segment(
        field,
        RailObjectIdentity {
            first_ring: topology_ring,
            lane: 17,
        },
    );
    let ring_points = ring_object_segment(
        field,
        RingObjectIdentity { ring: topology_ring },
        17,
    );
    let next_ring_points = ring_object_segment(
        field,
        RingObjectIdentity {
            ring: next_ring_edge.ring,
        },
        17,
    );
    let next_lane_points = field_edge_points(field, next_lane_edge);
    phase3_require(
        axial_points.0 == ring_points.0
            && axial_points.1 == next_ring_points.0
            && ring_points.1 == next_lane_points.0,
        183,
        b"physical ring and rail objects do not share one stable mesh topology",
    );
    let exposure_start = flow_camera(3.99);
    let exposure_end = flow_camera(4.0);
    let mut exposure_samples = 0u32;
    let mut exposure_motion = 0.0f32;
    star_cell_index = 0;
    while star_cell_index < STAR_CELL_COUNT {
        let cell = star_cell_identity(star_cell_index);
        let mut object = 0u8;
        while object < FIELD_STAR_OBJECTS_PER_CELL {
            let identity = StarObjectIdentity { cell, object };
            if star_object_exists(field, identity) {
                let fixed_point = star_object_position(field, identity);
                phase3_require(
                    fixed_point == star_object_position(field, identity),
                    183,
                    b"star identity does not retain a fixed world point",
                );
                if let Some((a, b)) = project_field_star_exposure(
                    field,
                    identity,
                    &exposure_start,
                    &exposure_end,
                    1.0,
                ) {
                    exposure_motion = exposure_motion
                        .max((b.0 - a.0).abs())
                        .max((b.1 - a.1).abs());
                    exposure_samples += 1;
                }
            }
            object += 1;
        }
        star_cell_index += 1;
    }
    phase3_require(
        exposure_samples >= 16 && exposure_motion > 0.000_1,
        183,
        b"fixed star exposure does not derive apparent flow from camera motion",
    );
    // Exit clipping belongs to the opening rush, not a second acceleration
    // or streak-building requirement inside the ring structure.
    let streak_exit_times = [0.5f32, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
    let mut partially_visible_exits = 0u32;
    let mut exit_time_index = 0usize;
    while exit_time_index < streak_exit_times.len() && partially_visible_exits < 8 {
        let exit_time = streak_exit_times[exit_time_index];
        let exit_camera = flow_camera(exit_time);
        let exit_exposure_camera =
            flow_camera((exit_time - FIELD_MOTION_BLUR_EXPOSURE).max(0.0));
        star_cell_index = 0;
        while star_cell_index < STAR_CELL_COUNT && partially_visible_exits < 8 {
            let cell = star_cell_identity(star_cell_index);
            if star_cell_may_project(field, cell, &exit_exposure_camera)
                || star_cell_may_project(field, cell, &exit_camera)
            {
                let mut object = 0u8;
                while object < FIELD_STAR_OBJECTS_PER_CELL && partially_visible_exits < 8 {
                    let star = StarObjectIdentity { cell, object };
                    if star_object_exists(field, star) {
                        let point = star_object_position(field, star);
                        let before = project_flow_depth(&exit_exposure_camera, point);
                        let after = project_flow_depth(&exit_camera, point);
                        if before.is_some() != after.is_some() {
                            if let Some((clipped_before, clipped_after)) =
                                project_field_star_exposure(
                                    field,
                                    star,
                                    &exit_exposure_camera,
                                    &exit_camera,
                                    1.0,
                                )
                            {
                                phase3_require(
                                    clipped_before.0.is_finite()
                                        && clipped_before.1.is_finite()
                                        && clipped_before.2.is_finite()
                                        && clipped_after.0.is_finite()
                                        && clipped_after.1.is_finite()
                                        && clipped_after.2.is_finite()
                                        && clip_screen_line(
                                            (clipped_before.0, clipped_before.1),
                                            (clipped_after.0, clipped_after.1),
                                            6.0,
                                        )
                                        .is_some(),
                                    183,
                                    b"partial streak exit is not finitely clipped to the camera",
                                );
                                partially_visible_exits += 1;
                            }
                        }
                    }
                    object += 1;
                }
            }
            star_cell_index += 1;
        }
        exit_time_index += 1;
    }
    phase3_require(
        partially_visible_exits >= 8,
        183,
        b"fixed star shutter paths disappear instead of clipping at camera exit",
    );
    let mut clipped_rail_witnesses = 0u32;
    let mut rail_clip_tick = (CLEAN_BIRTH * TRAJECTORY_HZ as f32) as u32;
    let rail_clip_end_tick = (GRID_SQUARE_START * TRAJECTORY_HZ as f32) as u32;
    while rail_clip_tick <= rail_clip_end_tick {
        let camera = flow_camera(rail_clip_tick as f32 / TRAJECTORY_HZ as f32);
        let near = camera.near_plane * 1.1;
        ordinal = first_physical_ring;
        while ordinal < last_ring {
            let ring = field_ring_identity(ordinal);
            let mut lane = 0u16;
            while lane < (GRID_LANES - 1) as u16 {
                let rail = RailObjectIdentity {
                    first_ring: ring,
                    lane,
                };
                if rail_object_exists(field, rail) {
                    let points = rail_object_segment(field, rail);
                    let depth_a = dvec_dot_vec3(
                        dvec_sub(points.0, camera.position),
                        camera.forward,
                    );
                    let depth_b = dvec_dot_vec3(
                        dvec_sub(points.1, camera.position),
                        camera.forward,
                    );
                    if (depth_a <= near && depth_b > near)
                        || (depth_b <= near && depth_a > near)
                    {
                        let projected = project_field_segment(
                            points.0,
                            points.1,
                            &camera,
                            1.0,
                        );
                        phase3_require(
                            projected.is_some(),
                            183,
                            b"rail joined to a ring behind the camera disappears at the near plane",
                        );
                        let projected = projected.unwrap();
                        let clipped = if depth_a <= near {
                            projected.0
                        } else {
                            projected.1
                        };
                        phase3_require(
                            clipped.0.is_finite()
                                && clipped.1.is_finite()
                                && clipped.2.is_finite()
                                && (clipped.2 - near as f32).abs() <= 1.0e-6,
                            183,
                            b"partially visible rail is not stably closed on the near plane",
                        );
                        clipped_rail_witnesses += 1;
                    }
                }
                lane += 1;
            }
            ordinal += 1;
        }
        rail_clip_tick += 1;
    }
    phase3_require(
        clipped_rail_witnesses >= 8,
        183,
        b"circular tunnel has insufficient physical rail near-plane witnesses",
    );
    let horizon_camera = flow_camera(0.0);
    let mut horizon_ring_count = 0u32;
    let mut previous_horizon_radius = f64::MAX;
    ordinal = first_ring;
    while ordinal <= last_ring {
        let ring = field_ring_identity(ordinal);
        let s = field_ring_s(field, ring);
        if s >= field.rings_full_s && s < field.rails_full_s {
            let point = field_mesh_vertex(field, ring, 0);
            let relative = dvec_sub(point, horizon_camera.position);
            let depth = dvec_dot_vec3(relative, horizon_camera.forward);
            if depth > horizon_camera.near_plane {
                let center = dvec_add(field.axis_origin, dvec_scale(field.forward, s));
                let radius = dvec_length(dvec_sub(point, center)) / depth;
                phase3_require(
                    radius < previous_horizon_radius,
                    184,
                    b"fixed logarithmic rings do not converge toward the projected horizon",
                );
                previous_horizon_radius = radius;
                horizon_ring_count += 1;
            }
        }
        ordinal += 1;
    }
    phase3_require(
        horizon_ring_count >= 4,
        184,
        b"fixed field has insufficient forward rings for horizon convergence",
    );
    let event_times = [
        (0.0f32, FieldRegion::Starfield),
        (STARFIELD_HOLD, FieldRegion::RingApproach),
        (STAR_ORGANIZE, FieldRegion::LogarithmicRings),
        (CLEAN_BIRTH, FieldRegion::CircularTunnel),
        (GRID_SQUARE_START, FieldRegion::SquaringTunnel),
    ];
    let mut event_index = 0usize;
    let mut previous_event_s = f64::NEG_INFINITY;
    while event_index < event_times.len() {
        let event = event_times[event_index];
        let camera = flow_camera(event.0);
        let camera_s = field_axis_coordinate(field.axis_origin, field.forward, camera.position);
        let event_reached = if event_index + 1 == event_times.len() {
            let next = flow_camera(event.0 + TRAJECTORY_STEP as f32);
            camera_s <= field.square_start_s + 0.01
                && field_axis_coordinate(field.axis_origin, field.forward, next.position) >= field.square_start_s
        } else { field_region(field, camera_s) == event.1 };
        phase3_require(
            camera_s > previous_event_s && event_reached,
            185,
            b"approved approach event no longer occurs in its fixed field region",
        );
        previous_event_s = camera_s;
        event_index += 1;
    }
    let initial_camera = flow_camera(0.0);
    let mut anchored_stars = 0u32;
    let mut anchored_star_bins = 0u64;
    star_cell_index = 0;
    while star_cell_index < STAR_CELL_COUNT {
        let cell = star_cell_identity(star_cell_index);
        let mut object = 0u8;
        while object < FIELD_STAR_OBJECTS_PER_CELL {
            let star = StarObjectIdentity { cell, object };
            let point = star_object_position(field, star);
            phase3_require(
                point == star_object_position(field, star),
                185,
                b"star object identity does not resolve to one fixed world position",
            );
            if star_object_exists(field, star) {
                if let Some(projected) = project_flow_depth(&initial_camera, point) {
                    if projected.0 >= 0.0
                        && projected.0 < WIDTH as f32
                        && projected.1 >= 0.0
                        && projected.1 < HEIGHT as f32
                    {
                        anchored_stars += 1;
                        let bin_x = (projected.0 * 8.0 / WIDTH as f32) as u32;
                        let bin_y = (projected.1 * 5.0 / HEIGHT as f32) as u32;
                        anchored_star_bins |= 1u64 << (bin_y * 8 + bin_x);
                    }
                }
            }
            object += 1;
        }
        star_cell_index += 1;
    }
    if anchored_stars < 320 || anchored_star_bins.count_ones() < 24 {
        let mut report = [0u8; 160];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 anchored opening stars/bins=");
        append_number(pointer, &mut length, anchored_stars);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, anchored_star_bins.count_ones());
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(185)
    }
    let mut streak_axial_bins = 0u64;
    let mut streak_axial_particles = 0u32;
    star_cell_index = 0;
    while star_cell_index < STAR_CELL_COUNT {
        let cell = star_cell_identity(star_cell_index);
        let (far_s, near_s) = star_cell_bounds(field, cell);
        let mut object = 0u8;
        while object < FIELD_STAR_OBJECTS_PER_CELL {
            let star = StarObjectIdentity { cell, object };
            if star_object_exists(field, star) {
                let point = star_object_position(field, star);
                let particle_s = field_axis_coordinate(
                    field.axis_origin,
                    field.forward,
                    point,
                );
                if field_region(field, particle_s) == FieldRegion::RingApproach {
                    let axial_amount = (particle_s - far_s) / (near_s - far_s);
                    phase3_require(
                        axial_amount > 0.0 && axial_amount < 1.0,
                        185,
                        b"independent star leaves its permanent axial cell",
                    );
                    let axial_bin = (axial_amount * 64.0) as u32;
                    streak_axial_bins |= 1u64 << axial_bin.min(63);
                    streak_axial_particles += 1;
                }
            }
            object += 1;
        }
        star_cell_index += 1;
    }
    if streak_axial_particles < 240 || streak_axial_bins.count_ones() < 60 {
        let mut report = [0u8; 128];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 independent streak particles/bins=");
        append_number(pointer, &mut length, streak_axial_particles);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, streak_axial_bins.count_ones());
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(185)
    }
    let streak_camera = flow_camera(STARFIELD_HOLD);
    let streak_past_camera = flow_camera(STARFIELD_HOLD - FIELD_MOTION_BLUR_EXPOSURE);
    let mut anchored_streaks = 0u32;
    let mut anchored_streak_length = 0.0f32;
    star_cell_index = 0;
    while star_cell_index < STAR_CELL_COUNT {
        let cell = star_cell_identity(star_cell_index);
        let mut object = 0u8;
        while object < FIELD_STAR_OBJECTS_PER_CELL {
            let star = StarObjectIdentity { cell, object };
            if star_object_exists(field, star) {
                let point = star_object_position(field, star);
                let s = field_axis_coordinate(field.axis_origin, field.forward, point);
                if field_region(field, s) == FieldRegion::RingApproach {
                    if let (Some(before), Some(after)) = (
                        project_flow_depth(&streak_past_camera, point),
                        project_flow_depth(&streak_camera, point),
                    ) {
                        if after.0 >= 0.0
                            && after.0 < WIDTH as f32
                            && after.1 >= 0.0
                            && after.1 < HEIGHT as f32
                        {
                            anchored_streaks += 1;
                            anchored_streak_length = anchored_streak_length.max(
                                fast_sqrt(
                                    (after.0 - before.0) * (after.0 - before.0)
                                        + (after.1 - before.1) * (after.1 - before.1),
                                ),
                            );
                        }
                    }
                }
            }
            object += 1;
        }
        star_cell_index += 1;
    }
    phase3_require(
        anchored_streaks >= 64 && anchored_streak_length >= 0.25,
        185,
        b"anchored streak space loses its event or perceptible camera motion",
    );
    let mut continuity_tick = 0u32;
    while continuity_tick <= 56 {
        let time = continuity_tick as f32 * 0.5;
        let camera = flow_camera(time);
        let exposure_camera = flow_camera((time - FIELD_MOTION_BLUR_EXPOSURE).max(0.0));
        let mut visible_objects = 0u32;
        let mut visible_star_objects = 0u32;
        let mut visible_mesh_objects = 0u32;
        let mut visible_bins = 0u64;
        let mut maximum_motion = 0.0f32;

        star_cell_index = 0;
        while star_cell_index < STAR_CELL_COUNT {
            let cell = star_cell_identity(star_cell_index);
            if star_cell_may_project(field, cell, &exposure_camera)
                && star_cell_may_project(field, cell, &camera)
            {
                let mut object = 0u8;
                while object < FIELD_STAR_OBJECTS_PER_CELL {
                    let star = StarObjectIdentity { cell, object };
                    if star_object_exists(field, star) {
                        let point = star_object_position(field, star);
                        if let (Some(before), Some(after)) = (
                            project_flow_depth(&exposure_camera, point),
                            project_flow_depth(&camera, point),
                        ) {
                            if after.0 >= 0.0
                                && after.0 < WIDTH as f32
                                && after.1 >= 0.0
                                && after.1 < HEIGHT as f32
                            {
                                visible_objects += 1;
                                visible_star_objects += 1;
                                let bin_x = (after.0 * 8.0 / WIDTH as f32) as u32;
                                let bin_y = (after.1 * 5.0 / HEIGHT as f32) as u32;
                                visible_bins |= 1u64 << (bin_y * 8 + bin_x);
                                maximum_motion = maximum_motion.max(fast_sqrt(
                                    (after.0 - before.0) * (after.0 - before.0)
                                        + (after.1 - before.1) * (after.1 - before.1),
                                ));
                            }
                        }
                    }
                    object += 1;
                }
            }
            star_cell_index += 1;
        }

        ordinal = first_physical_ring;
        while ordinal <= last_ring {
            let ring = field_ring_identity(ordinal);
            let s = field_ring_s(field, ring);
            if field_mesh_visibility(field, s, &camera) > 0.01 {
                let mut mesh_lane = 0u16;
                while mesh_lane < (GRID_LANES - 1) as u16 {
                    let points = ring_object_segment(
                        field,
                        RingObjectIdentity { ring },
                        mesh_lane,
                    );
                    if let (Some(before), Some(after)) = (
                        project_field_segment(points.0, points.1, &exposure_camera, 1.0),
                        project_field_segment(points.0, points.1, &camera, 1.0),
                    ) {
                        if !((after.0.0 < 0.0 && after.1.0 < 0.0)
                            || (after.0.0 >= WIDTH as f32 && after.1.0 >= WIDTH as f32)
                            || (after.0.1 < 0.0 && after.1.1 < 0.0)
                            || (after.0.1 >= HEIGHT as f32 && after.1.1 >= HEIGHT as f32))
                        {
                            visible_objects += 1;
                            visible_mesh_objects += 1;
                            let middle_x = ((after.0.0 + after.1.0) * 0.5)
                                .clamp(0.0, WIDTH as f32 - 1.0);
                            let middle_y = ((after.0.1 + after.1.1) * 0.5)
                                .clamp(0.0, HEIGHT as f32 - 1.0);
                            let before_x = (before.0.0 + before.1.0) * 0.5;
                            let before_y = (before.0.1 + before.1.1) * 0.5;
                            let bin_x = (middle_x * 8.0 / WIDTH as f32) as u32;
                            let bin_y = (middle_y * 5.0 / HEIGHT as f32) as u32;
                            visible_bins |= 1u64 << (bin_y * 8 + bin_x);
                            maximum_motion = maximum_motion.max(fast_sqrt(
                                (middle_x - before_x) * (middle_x - before_x)
                                    + (middle_y - before_y) * (middle_y - before_y),
                            ));
                        }
                    }
                    mesh_lane += 1;
                }
            }
            ordinal += 1;
        }

        let minimum_bins = if time >= WORLD_CURVE_START as f32 { 4 } else { 6 };
        if visible_objects < 32
            || visible_bins.count_ones() < minimum_bins
            || (continuity_tick > 0 && maximum_motion < 0.01)
        {
            let mut report = [0u8; 192];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(
                pointer,
                &mut length,
                b"phase3 production field continuity ms/stars/mesh/bins/motion_millipixels=",
            );
            append_number(pointer, &mut length, continuity_tick * 500);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, visible_star_objects);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, visible_mesh_objects);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, visible_bins.count_ones());
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, (maximum_motion * 1_000.0) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(185)
        }
        continuity_tick += 1;
    }
    let hold_camera = flow_camera(STARFIELD_HOLD);
    let pre_exit_camera = flow_camera(STARFIELD_HOLD - 0.1);
    let mut premature_mesh_visibility = 0.0f32;
    ordinal = first_ring;
    while ordinal <= last_ring {
        let ring = field_ring_identity(ordinal);
        let s = field_ring_s(field, ring);
        if field_region(field, s) != FieldRegion::Starfield
            && field_region(field, s) != FieldRegion::RingApproach
        {
            premature_mesh_visibility = premature_mesh_visibility
                .max(field_mesh_visibility(field, s, &initial_camera))
                .max(field_mesh_visibility(field, s, &pre_exit_camera));
        }
        ordinal += 1;
    }
    phase3_require(
        premature_mesh_visibility == 0.0,
        185,
        b"ring field is visible before the camera emerges from the jump",
    );
    let exit_camera_s = field_axis_coordinate(
        field.axis_origin,
        field.forward,
        hold_camera.position,
    );
    let mut exit_ring_identities = 0u32;
    let mut exit_strong_ring_identities = 0u32;
    let mut exit_ring_edges = 0u32;
    let mut exit_rail_edges = 0u32;
    ordinal = first_physical_ring;
    while ordinal <= last_ring {
        let ring = field_ring_identity(ordinal);
        let s = field_ring_s(field, ring);
        let visibility = field_mesh_visibility(field, s, &hold_camera);
        if s > exit_camera_s && visibility > 0.01 {
            let mut visible_ring_edges = 0u32;
            let mut lane = 0u16;
            while lane < (GRID_LANES - 1) as u16 {
                let ring_points = ring_object_segment(
                    field,
                    RingObjectIdentity { ring },
                    lane,
                );
                if let Some((a, b)) =
                    project_field_segment(ring_points.0, ring_points.1, &hold_camera, 1.0)
                {
                    if clip_screen_line((a.0, a.1), (b.0, b.1), 6.0).is_some() {
                        visible_ring_edges += 1;
                        exit_ring_edges += 1;
                    }
                }
                let rail = RailObjectIdentity {
                    first_ring: ring,
                    lane,
                };
                if rail_object_exists(field, rail) {
                    let rail_points = rail_object_segment(field, rail);
                    if let Some((a, b)) =
                        project_field_segment(rail_points.0, rail_points.1, &hold_camera, 1.0)
                    {
                        if clip_screen_line((a.0, a.1), (b.0, b.1), 6.0).is_some() {
                            exit_rail_edges += 1;
                        }
                    }
                }
                lane += 1;
            }
            if visible_ring_edges > 0 {
                exit_ring_identities += 1;
                if visibility >= 0.25 {
                    exit_strong_ring_identities += 1;
                }
            }
        }
        ordinal += 1;
    }
    if exit_ring_identities < 4
        || exit_strong_ring_identities < 3
        || exit_ring_edges < 128
        || exit_rail_edges < 6
    {
        let mut report = [0u8; 176];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(
            pointer,
            &mut length,
            b"phase3 jump-exit ring structure rings/strong/edges/rails=",
        );
        append_number(pointer, &mut length, exit_ring_identities);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, exit_strong_ring_identities);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, exit_ring_edges);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, exit_rail_edges);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(185)
    }
    let rendered_event_spaces = [
        (STAR_ORGANIZE, FieldRegion::LogarithmicRings),
        (CLEAN_BIRTH - 1.0, FieldRegion::CircularTunnel),
        (GRID_SQUARE_START, FieldRegion::SquaringTunnel),
    ];
    event_index = 0;
    while event_index < rendered_event_spaces.len() {
        let event = rendered_event_spaces[event_index];
        let camera = flow_camera(event.0);
        let camera_s = field_axis_coordinate(field.axis_origin, field.forward, camera.position);
        let mut visible_edges = 0u32;
        let mut visible_edge_bins = 0u64;
        ordinal = first_ring;
        while ordinal <= last_ring {
            let ring = field_ring_identity(ordinal);
            let s = field_ring_s(field, ring);
            if s > camera_s
                && field_region(field, s) == event.1
                && field_mesh_visibility(field, s, &camera) > 0.01
            {
                let mut lane = 0usize;
                while lane < GRID_LANES - 1 {
                    let edge = FieldEdgeIdentity {
                        ring,
                        lane: lane as u16,
                        kind: FieldEdgeKind::Circumferential,
                    };
                    let points = field_edge_points(field, edge);
                    if let Some((a, b)) =
                        project_field_segment(points.0, points.1, &camera, 1.0)
                    {
                        if !((a.0 < 0.0 && b.0 < 0.0)
                            || (a.0 >= WIDTH as f32 && b.0 >= WIDTH as f32)
                            || (a.1 < 0.0 && b.1 < 0.0)
                            || (a.1 >= HEIGHT as f32 && b.1 >= HEIGHT as f32))
                        {
                            visible_edges += 1;
                            let middle_x =
                                ((a.0 + b.0) * 0.5).clamp(0.0, WIDTH as f32 - 1.0);
                            let middle_y =
                                ((a.1 + b.1) * 0.5).clamp(0.0, HEIGHT as f32 - 1.0);
                            let bin_x = (middle_x * 8.0 / WIDTH as f32) as u32;
                            let bin_y = (middle_y * 5.0 / HEIGHT as f32) as u32;
                            visible_edge_bins |= 1u64 << (bin_y * 8 + bin_x);
                        }
                    }
                    lane += 1;
                }
            }
            ordinal += 1;
        }
        if visible_edges < 8 || visible_edge_bins.count_ones() < 4 {
            let mut report = [0u8; 128];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase3 ring/tunnel event-index/edges/bins=");
            append_number(pointer, &mut length, event_index as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, visible_edges);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, visible_edge_bins.count_ones());
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(185)
        }
        event_index += 1;
    }
    let planet_intro_bounds = projected_surface_bounds(&flow_camera(PLANET_INTRO), 1.0);
    if !(planet_intro_bounds.visible > 0.0
        && planet_intro_bounds.x >= 0.0
        && planet_intro_bounds.x < WIDTH as f32
        && planet_intro_bounds.y >= 0.0
        && planet_intro_bounds.y < HEIGHT as f32)
    {
        let intro_state = trajectory_state(PLANET_INTRO as f64);
        let toward = dvec_normalize(dvec_scale(intro_state.position, -1.0));
        let mut report = [0u8; 128];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 planet intro x/y/dot+1k/distance-milli=");
        append_number(pointer, &mut length, (planet_intro_bounds.x + 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (planet_intro_bounds.y + 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, ((dvec_dot(toward, intro_state.velocity) / dvec_length(intro_state.velocity) + 1.0) * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (dvec_length(intro_state.position) * 1_000.0) as u32);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(185)
    }
    let mut fixed_star_candidates = 0u32;
    let mut inconsistent_fixed_stars = 0u32;
    let mut fixed_star_max_error = 0.0f32;
    let mut moving_star_identities = 0u32;
    let mut proof_tick = 0u32;
    while proof_tick <= 10_500 {
        let first_time = proof_tick as f32 / 1_000.0;
        let middle_time = first_time + 0.25;
        let last_time = first_time + 0.5;
        let first_camera = flow_camera(first_time);
        let middle_camera = flow_camera(middle_time);
        let last_camera = flow_camera(last_time);
        let mut proof_index = 0usize;
        while proof_index < STAR_LINES {
            let generation = canonical_star_generation(proof_index, first_time, CLEAN_BIRTH);
            if generation == canonical_star_generation(proof_index, middle_time, CLEAN_BIRTH)
                && generation == canonical_star_generation(proof_index, last_time, CLEAN_BIRTH)
            {
                let first = canonical_flyby_segment(proof_index, first_time, CLEAN_BIRTH);
                let middle = canonical_flyby_segment(proof_index, middle_time, CLEAN_BIRTH);
                let last = canonical_flyby_segment(proof_index, last_time, CLEAN_BIRTH);
                if let (
                    Some((first_a, first_b)),
                    Some((middle_a, middle_b)),
                    Some((last_a, last_b)),
                ) = (first, middle, last)
                {
                    let first_center = (
                        (first_a.0 + first_b.0) * 0.5,
                        (first_a.1 + first_b.1) * 0.5,
                    );
                    let middle_center = (
                        (middle_a.0 + middle_b.0) * 0.5,
                        (middle_a.1 + middle_b.1) * 0.5,
                    );
                    let last_center = (
                        (last_a.0 + last_b.0) * 0.5,
                        (last_a.1 + last_b.1) * 0.5,
                    );
                    if let Some(candidate) = fixed_candidate_from_rays(
                        &first_camera,
                        first_center,
                        &last_camera,
                        last_center,
                    ) {
                        let endpoint_error = projected_point_error(
                            &first_camera,
                            candidate,
                            first_center,
                        )
                        .max(projected_point_error(
                            &last_camera,
                            candidate,
                            last_center,
                        ));
                        if endpoint_error <= 0.000_1 {
                            let error =
                                projected_point_error(&middle_camera, candidate, middle_center);
                            fixed_star_max_error = fixed_star_max_error.max(error);
                            if error > 0.000_1 {
                                inconsistent_fixed_stars += 1;
                            }
                            fixed_star_candidates += 1;
                        }
                    }
                    let identity = CanonicalStarParticleIdentity {
                        line: proof_index,
                        generation,
                    };
                    if let (Some(first_world), Some(last_world)) = (
                        canonical_star_particle_world_position(field, identity, first_time),
                        canonical_star_particle_world_position(field, identity, last_time),
                    ) {
                        if dvec_length(dvec_sub(last_world, first_world)) > 1.0e-8 {
                            moving_star_identities += 1;
                        }
                    }
                }
            }
            proof_index += 1;
        }
        proof_tick += 250;
    }
    if !(fixed_star_candidates >= 7_000
        && inconsistent_fixed_stars >= 1_000
        && fixed_star_max_error > 0.01
        && moving_star_identities >= 1_000)
    {
        let mut report = [0u8; 144];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 canonical moving-star proof candidates/inconsistent/error-micro/moving=");
        append_number(pointer, &mut length, fixed_star_candidates);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, inconsistent_fixed_stars);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (fixed_star_max_error * 1_000_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, moving_star_identities);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(184)
    }
    let mut fixed_mesh_candidates = 0u32;
    let mut inconsistent_fixed_mesh = 0u32;
    let mut fixed_mesh_max_error = 0.0f32;
    let mut proof_ring = 0usize;
    while proof_ring < 12 {
        let first_time = ring_birth(proof_ring) + 0.1;
        let middle_time = first_time + 0.35;
        let last_time = first_time + 0.7;
        let first_camera = flow_camera(first_time);
        let middle_camera = flow_camera(middle_time);
        let last_camera = flow_camera(last_time);
        let first_state = ring_state(proof_ring, first_time);
        let middle_state = ring_state(proof_ring, middle_time);
        let last_state = ring_state(proof_ring, last_time);
        if first_state.1.to_bits() == middle_state.1.to_bits()
            && first_state.1.to_bits() == last_state.1.to_bits()
        {
            let shape = birth_shape(first_state.1);
            let mut lane = 0usize;
            while lane < GRID_LANES {
                let lane_coordinate =
                    -14.0 + lane as f32 * (28.0 / (GRID_LANES - 1) as f32);
                let section = grid_section_point(lane_coordinate, shape);
                let screen_point = |depth: f32| {
                    let amount = depth / 20.0;
                    let radius = amount * amount * 260.0;
                    (160.0 + section.0 * radius, 100.0 + section.1 * radius)
                };
                let first_point = screen_point(first_state.0);
                let middle_point = screen_point(middle_state.0);
                let last_point = screen_point(last_state.0);
                if let Some(candidate) = fixed_candidate_from_rays(
                    &first_camera,
                    first_point,
                    &last_camera,
                    last_point,
                ) {
                    let endpoint_error = projected_point_error(
                        &first_camera,
                        candidate,
                        first_point,
                    )
                    .max(projected_point_error(&last_camera, candidate, last_point));
                    if endpoint_error <= 0.000_1 {
                        let error =
                            projected_point_error(&middle_camera, candidate, middle_point);
                        fixed_mesh_max_error = fixed_mesh_max_error.max(error);
                        if error > 0.000_1 {
                            inconsistent_fixed_mesh += 1;
                        }
                        fixed_mesh_candidates += 1;
                    }
                }
                lane += 1;
            }
        }
        proof_ring += 1;
    }
    if !(fixed_mesh_candidates >= 50
        && inconsistent_fixed_mesh >= 50
        && fixed_mesh_max_error > 0.01)
    {
        let mut report = [0u8; 128];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 canonical moving-mesh proof candidates/inconsistent/error-micro=");
        append_number(pointer, &mut length, fixed_mesh_candidates);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, inconsistent_fixed_mesh);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (fixed_mesh_max_error * 1_000_000.0) as u32);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(184)
    }
    let samples = [
        0.5f32, 4.0, 11.0, 20.063, 27.0, 27.99, 28.0, 28.01, 35.0,
        42.0, 51.959_26, 60.0, 78.0, 94.0, 106.0, 122.0, 137.0,
    ];
    let mut index = 0usize;
    while index < samples.len() {
        let time = samples[index];
        let camera = flow_camera(time);
        let repeat = flow_camera(time);
        let trajectory = trajectory_state(time as f64);
        phase3_require(
            phase0_basis_valid(camera)
                && dvec_length_squared(dvec_sub(camera.position, trajectory.position)) == 0.0
                && dvec_length_squared(dvec_sub(camera.position, repeat.position)) == 0.0
                && vec_dot(camera.forward, repeat.forward) > 0.999_999
                && vec_dot(camera.down, repeat.down) > 0.999_999,
            170,
            b"camera is not a deterministic view of the continuous trajectory",
        );

        let bounds = projected_surface_bounds(&camera, 1.0);
        let repeat_bounds = projected_surface_bounds(&repeat, 1.0);
        phase3_require(
            bounds.visible.is_finite()
                && bounds.x.is_finite()
                && bounds.y.is_finite()
                && bounds.radius_x.is_finite()
                && bounds.radius_y.is_finite()
                && bounds == repeat_bounds,
            171,
            b"canonical planet bounds are invalid or seek-dependent",
        );

        let atmosphere = flow_atmosphere_amount(&camera);
        let heat = flow_entry_heat(&camera);
        phase3_require(
            atmosphere.is_finite()
                && heat.is_finite()
                && (0.0..=1.0).contains(&atmosphere)
                && (0.0..=1.0).contains(&heat),
            172,
            b"same-time atmospheric inputs are invalid",
        );
        index += 1;
    }

    let entry_before = flow_camera(ORBIT_ENTRY - 0.001);
    let entry = flow_camera(ORBIT_ENTRY);
    let entry_after = flow_camera(ORBIT_ENTRY + 0.001);
    let incoming = dvec_sub(entry.position, entry_before.position);
    let outgoing = dvec_sub(entry_after.position, entry.position);
    phase3_require(
        dvec_dot(dvec_normalize(incoming), dvec_normalize(outgoing)) > 0.99
            && dvec_length(dvec_sub(outgoing, incoming))
                / dvec_length(incoming).max(1.0e-12)
                < 0.20
            && vec_dot(entry_before.forward, entry.forward) > 0.999_999
            && vec_dot(entry.forward, entry_after.forward) > 0.999,
        173,
        b"single camera is discontinuous at orbital steering",
    );

    let aspect_camera = flow_camera(29.0);
    let unscaled = projected_surface_bounds(&aspect_camera, 1.0);
    let narrow = projected_surface_bounds(&aspect_camera, 0.5);
    let wide = projected_surface_bounds(&aspect_camera, 2.0);
    phase3_require(
        (narrow.x - (160.0 + (unscaled.x - 160.0) * 0.5)).abs() < 0.001
            && (wide.x - (160.0 + (unscaled.x - 160.0) * 2.0)).abs() < 0.001
            && (narrow.radius_x - unscaled.radius_x * 0.5).abs() < 0.001
            && (wide.radius_x - unscaled.radius_x * 2.0).abs() < 0.001
            && narrow.y == unscaled.y
            && wide.y == unscaled.y
            && narrow.radius_y == unscaled.radius_y
            && wide.radius_y == unscaled.radius_y,
        174,
        b"shared camera aspect transform is inconsistent",
    );

    phase3_require(
        PLANET_CENTER == DVec3 { x: 0.0, y: 0.0, z: 0.0 }
            && (vec_dot(flow_sun_direction(), flow_sun_direction()) - 1.0).abs() < 0.001,
        175,
        b"world origin or sun direction is not canonical",
    );
    let start_camera = flow_camera(0.0);
    let late_approach_camera = flow_camera(27.0);
    phase3_require(
        dvec_length(dvec_sub(start_camera.position, PLANET_CENTER))
            > dvec_length(dvec_sub(late_approach_camera.position, PLANET_CENTER)) + 1.0,
        180,
        b"camera does not fly through the world during the approach",
    );
    let mut visible_stars = 0usize;
    let mut occupied_star_bins = 0u64;
    let start_transport = canonical_flow_field_transport(0.0);
    index = 0;
    while index < STAR_LINES {
        if let Some((a, b, depth)) = flyby_segment(index, 0.0, CLEAN_BIRTH) {
            if let Some((projected_a, projected_b)) =
                project_canonical_flow_star_segment(
                    start_transport,
                    a,
                    b,
                    depth,
                    &start_camera,
                    1.0,
                )
            {
                if !((projected_a.0 < 0.0 && projected_b.0 < 0.0)
                    || (projected_a.0 >= WIDTH as f32 && projected_b.0 >= WIDTH as f32)
                    || (projected_a.1 < 0.0 && projected_b.1 < 0.0)
                    || (projected_a.1 >= HEIGHT as f32 && projected_b.1 >= HEIGHT as f32))
                {
                    visible_stars += 1;
                    let middle_x = ((projected_a.0 + projected_b.0) * 0.5)
                        .clamp(0.0, WIDTH as f32 - 1.0);
                    let middle_y = ((projected_a.1 + projected_b.1) * 0.5)
                        .clamp(0.0, HEIGHT as f32 - 1.0);
                    let bin_x = (middle_x * 8.0 / WIDTH as f32) as u32;
                    let bin_y = (middle_y * 5.0 / HEIGHT as f32) as u32;
                    occupied_star_bins |= 1u64 << (bin_y * 8 + bin_x);
                }
            }
        }
        index += 1;
    }
    let occupied_star_bin_count = occupied_star_bins.count_ones();
    if visible_stars < 32 || occupied_star_bin_count < 16 {
        let mut report = [0u8; 256];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 unified-camera audit failed: start visible stars=");
        append_number(pointer, &mut length, visible_stars as u32);
        append(pointer, &mut length, b" bins=");
        append_number(pointer, &mut length, occupied_star_bin_count);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(176)
    }
    let mut canonical_tick = 0u32;
    let mut canonical_max_error = 0.0f32;
    let mut canonical_max_error_tick = 0u32;
    let mut canonical_active_samples = 0u32;
    let mut explicit_particle_max_error = 0.0f32;
    let mut exact_exposure_max_error = 0.0f32;
    let mut explicit_particle_samples = 0u32;
    let mut exact_exposure_samples = 0u32;
    let mut nonphysical_streak_samples = 0u32;
    while canonical_tick <= 11 * TRAJECTORY_HZ as u32 {
        let time = canonical_tick as f32 / TRAJECTORY_HZ as f32;
        let camera = flow_camera(time);
        let transport = canonical_flow_field_transport(time);
        phase3_require(
            dvec_length(dvec_sub(
                canonical_star_particle_reference(field, time),
                camera.position,
            )) < 1.0e-9,
            184,
            b"explicit star trajectory reference diverges from canonical approach",
        );
        index = 0;
        while index < STAR_LINES {
            let generated = flyby_segment(index, time, CLEAN_BIRTH);
            let canonical = canonical_flyby_segment(index, time, CLEAN_BIRTH);
            match (generated, canonical) {
                (None, None) => {}
                (Some((a, b, moving_row)), Some((canonical_a, canonical_b))) => {
                    let Some((pa, pb)) =
                        project_canonical_flow_star_segment(
                            transport,
                            a,
                            b,
                            moving_row,
                            &camera,
                            1.0,
                        )
                    else {
                        phase3_fail(181, b"canonical star was lost during world projection")
                    };
                    let sample_error = 0.0f32
                        .max((pa.0 - canonical_a.0).abs())
                        .max((pa.1 - canonical_a.1).abs())
                        .max((pb.0 - canonical_b.0).abs())
                        .max((pb.1 - canonical_b.1).abs());
                    if sample_error > canonical_max_error {
                        canonical_max_error = sample_error;
                        canonical_max_error_tick = canonical_tick;
                    }
                    canonical_active_samples += 1;
                    let generation = canonical_star_generation(index, time, CLEAN_BIRTH);
                    let identity = CanonicalStarParticleIdentity {
                        line: index,
                        generation,
                    };
                    let center_world = canonical_star_particle_world_position(
                        field,
                        identity,
                        time,
                    )
                    .unwrap_or_else(|| {
                        phase3_fail(184, b"explicit canonical star identity was lost")
                    });
                    let canonical_center = (
                        (canonical_a.0 + canonical_b.0) * 0.5,
                        (canonical_a.1 + canonical_b.1) * 0.5,
                    );
                    explicit_particle_max_error = explicit_particle_max_error
                        .max(projected_point_error(&camera, center_world, canonical_center));
                    explicit_particle_samples += 1;
                    if time >= 7.0 {
                        if let Some((exposure_start, exposure_end)) =
                            canonical_star_exposure_times(identity, time)
                        {
                            let first_world = canonical_star_particle_world_position(
                                field,
                                identity,
                                exposure_start,
                            )
                            .unwrap_or_else(|| {
                                phase3_fail(
                                    184,
                                    b"canonical streak lost its first particle sample",
                                )
                            });
                            let second_world = canonical_star_particle_world_position(
                                field,
                                identity,
                                exposure_end,
                            )
                            .unwrap_or_else(|| {
                                phase3_fail(
                                    184,
                                    b"canonical streak lost its last particle sample",
                                )
                            });
                            let first_camera = flow_camera(exposure_start);
                            let second_camera = flow_camera(exposure_end);
                            let exposure_error = 0.0f32
                                .max(projected_point_error(
                                    &first_camera,
                                    first_world,
                                    canonical_a,
                                ))
                                .max(projected_point_error(
                                    &second_camera,
                                    second_world,
                                    canonical_b,
                                ));
                            if exposure_error <= 0.000_1 {
                                exact_exposure_max_error =
                                    exact_exposure_max_error.max(exposure_error);
                                exact_exposure_samples += 1;
                            } else {
                                nonphysical_streak_samples += 1;
                            }
                        } else {
                            nonphysical_streak_samples += 1;
                        }
                    }
                }
                _ => phase3_fail(181, b"canonical star activation identity changed"),
            }
            index += 1;
        }
        canonical_tick += 1;
    }
    if !(canonical_active_samples >= 1_000_000 && canonical_max_error <= 0.000_1) {
        let mut report = [0u8; 160];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 canonical star samples/error-micropixels=");
        append_number(pointer, &mut length, canonical_active_samples);
        append(pointer, &mut length, b"/");
        append_number(
            pointer,
            &mut length,
            (canonical_max_error * 1_000_000.0) as u32,
        );
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, canonical_max_error_tick);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(181)
    }
    if !(explicit_particle_samples >= 1_000_000
        && exact_exposure_samples >= 100_000
        && nonphysical_streak_samples >= 1_000
        && explicit_particle_max_error > 0.000_1
        && explicit_particle_max_error <= 0.01
        && exact_exposure_max_error <= 0.000_1)
    {
        let mut report = [0u8; 256];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 star proof failed center/exposure/nonphysical/center-error/exposure-error-micropixels=");
        append_number(pointer, &mut length, explicit_particle_samples);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, exact_exposure_samples);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, nonphysical_streak_samples);
        append(pointer, &mut length, b"/");
        append_number(
            pointer,
            &mut length,
            (explicit_particle_max_error * 1_000_000.0) as u32,
        );
        append(pointer, &mut length, b"/");
        append_number(
            pointer,
            &mut length,
            (exact_exposure_max_error * 1_000_000.0) as u32,
        );
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(184)
    }
    let mut mesh_tick = (RING_BIRTH * TRAJECTORY_HZ as f32) as u32;
    let mesh_end_tick = (CLEAN_BIRTH * TRAJECTORY_HZ as f32) as u32;
    let mut canonical_mesh_samples = 0u32;
    let mut canonical_mesh_error = 0.0f64;
    while mesh_tick <= mesh_end_tick {
        let time = mesh_tick as f32 / TRAJECTORY_HZ as f32;
        let camera = flow_camera(time);
        let mut ring = 0usize;
        while ring < GRID_RINGS {
            if time >= ring_birth(ring) {
                let (depth, birth_position) = ring_state(ring, time);
                if depth > grid_fog_range(time).0 {
                    let shape = birth_shape(birth_position);
                    let projective_radius = (depth / 20.0) * (depth / 20.0) * 260.0;
                    let mut lane_index = 0usize;
                    while lane_index < GRID_LANES {
                        let lane = -14.0
                            + lane_index as f32 * (28.0 / (GRID_LANES - 1) as f32);
                        let section = grid_section_point(lane, shape);
                        if section.1 / 0.61 <= 0.25 {
                            let point = canonical_flow_grid_world_point(time, section, depth);
                            project_flow_depth(&camera, point).unwrap_or_else(|| {
                                phase3_fail(181, b"canonical forming mesh left the forward field")
                            });
                            let relative = dvec_sub(point, camera.position);
                            let projection_depth =
                                dvec_dot(relative, dvec_from_vec3(camera.forward));
                            let projected_x = 160.0
                                + dvec_dot_vec3(relative, camera.right) / projection_depth
                                    * FLOW_FOCAL as f64;
                            let projected_y = 100.0
                                + dvec_dot_vec3(relative, camera.down) / projection_depth
                                    * FLOW_FOCAL as f64;
                            let expected_x = 160.0 + section.0 as f64 * projective_radius as f64;
                            let expected_y = 100.0 + section.1 as f64 * projective_radius as f64;
                            canonical_mesh_error = canonical_mesh_error
                                .max((projected_x - expected_x).abs())
                                .max((projected_y - expected_y).abs());
                            canonical_mesh_samples += 1;
                        }
                        lane_index += 1;
                    }
                }
            }
            ring += 1;
        }
        mesh_tick += 1;
    }
    if !(canonical_mesh_samples >= 100_000 && canonical_mesh_error <= 1.0e-9) {
        let mut report = [0u8; 160];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 canonical mesh samples/error-micropixels=");
        append_number(pointer, &mut length, canonical_mesh_samples);
        append(pointer, &mut length, b"/");
        append_number(
            pointer,
            &mut length,
            (canonical_mesh_error * 1_000_000.0) as u32,
        );
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(181)
    }
    let mut star_tick = 0u32;
    let mut previous_star_distance = f64::MAX;
    while star_tick <= 11_000 {
        let tick = star_tick;
        let time = tick as f32 / 1_000.0;
        let camera = flow_camera(time);
        let transport = canonical_flow_field_transport(time);
        let mut projected_count = 0u32;
        let mut visible_count = 0u32;
        let mut visible_bins = 0u64;
        let mut central_count = 0u32;
        let mut visible_x_sum = 0.0f32;
        let mut visible_y_sum = 0.0f32;
        let mut raster_count = 0u32;
        let mut maximum_length = 0.0f32;
        let mut canonical_projection_error = 0.0f32;
        index = 0;
        while index < STAR_LINES {
            if let Some((a, b, depth)) = flyby_segment(index, time, CLEAN_BIRTH) {
                if let Some((pa, pb)) =
                    project_canonical_flow_star_segment(
                        transport,
                        a,
                        b,
                        depth,
                        &camera,
                        1.0,
                    )
                {
                    let point_error = (pa.0 - (160.0 + a.0))
                        .abs()
                        .max((pa.1 - (100.0 + a.1)).abs())
                        .max((pb.0 - (160.0 + b.0)).abs())
                        .max((pb.1 - (100.0 + b.1)).abs());
                    if point_error > 0.000_1 {
                        let mut report = [0u8; 256];
                        let pointer = report.as_mut_ptr();
                        let mut length = 0usize;
                        append(pointer, &mut length, b"phase3 canonical star point failed ms/index=");
                        append_number(pointer, &mut length, tick);
                        append(pointer, &mut length, b"/");
                        append_number(pointer, &mut length, index as u32);
                        append(pointer, &mut length, b" expected/actual x milli-offset=");
                        append_number(pointer, &mut length, ((160.0 + a.0) * 1_000.0) as u32);
                        append(pointer, &mut length, b"/");
                        append_number(pointer, &mut length, (pa.0 * 1_000.0) as u32);
                        append(pointer, &mut length, b" y=");
                        append_number(pointer, &mut length, ((100.0 + a.1) * 1_000.0) as u32);
                        append(pointer, &mut length, b"/");
                        append_number(pointer, &mut length, (pa.1 * 1_000.0) as u32);
                        append(pointer, &mut length, b"\n");
                        write_all(&report[..length]);
                        exit(181)
                    }
                    canonical_projection_error = canonical_projection_error
                        .max((pa.0 - (160.0 + a.0)).abs())
                        .max((pa.1 - (100.0 + a.1)).abs())
                        .max((pb.0 - (160.0 + b.0)).abs())
                        .max((pb.1 - (100.0 + b.1)).abs());
                    projected_count += 1;
                    let dx = pb.0 - pa.0;
                    let dy = pb.1 - pa.1;
                    let length = fast_sqrt(dx * dx + dy * dy);
                    let middle_x = (pa.0 + pb.0) * 0.5;
                    let middle_y = (pa.1 + pb.1) * 0.5;
                    if middle_x >= 0.0
                        && middle_x < WIDTH as f32
                        && middle_y >= 0.0
                        && middle_y < HEIGHT as f32
                    {
                        visible_count += 1;
                        visible_x_sum += middle_x;
                        visible_y_sum += middle_y;
                        let center_x = middle_x - WIDTH as f32 * 0.5;
                        let center_y = middle_y - HEIGHT as f32 * 0.5;
                        if center_x * center_x + center_y * center_y < 35.0 * 35.0 {
                            central_count += 1;
                        }
                        let bin_x = (middle_x * 8.0 / WIDTH as f32) as u32;
                        let bin_y = (middle_y * 5.0 / HEIGHT as f32) as u32;
                        visible_bins |= 1u64 << (bin_y * 8 + bin_x);
                    }
                    if length >= 0.75 {
                        raster_count += 1;
                    }
                    maximum_length = maximum_length.max(length);
                }
            }
            index += 1;
        }
        let camera_distance = dvec_length(dvec_sub(camera.position, PLANET_CENTER));
        let visible_center_x = visible_x_sum / visible_count.max(1) as f32;
        let visible_center_y = visible_y_sum / visible_count.max(1) as f32;
        if canonical_projection_error > 0.000_1 {
            let mut report = [0u8; 192];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase3 canonical star projection failed at ms=");
            append_number(pointer, &mut length, tick);
            append(pointer, &mut length, b" error_micropixels=");
            append_number(
                pointer,
                &mut length,
                (canonical_projection_error * 1_000_000.0) as u32,
            );
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(181)
        }
        phase3_require(
            projected_count >= 250
                && visible_count >= 150
                && visible_bins.count_ones() >= 16
                && (time > STARFIELD_HOLD
                    || (central_count >= 10
                        && (visible_center_x - WIDTH as f32 * 0.5).abs() <= 25.0
                        && (visible_center_y - HEIGHT as f32 * 0.5).abs() <= 20.0))
                && (time < 7.0 || (raster_count >= 500 && maximum_length >= 1.0))
                && camera_distance < previous_star_distance,
            181,
            b"starfield differs from canonical motion or loses world visibility",
        );
        previous_star_distance = camera_distance;
        star_tick += 500;
    }
    let approach_ticks = [26_000u32, 27_000, 27_250, 27_500, 27_750, 28_000];
    let mut tick_index = 0usize;
    while tick_index < approach_ticks.len() {
        let tick = approach_ticks[tick_index];
        let time = tick as f32 / 1_000.0;
        let camera = flow_camera(time);
        let mut count = 0u32;
        let mut radius_sum = 0.0f32;
        let mut bins = 0u64;
        index = SPOKE_LINES;
        while index < STAR_LINES {
            let birth = line_birth(index);
            if time >= birth {
                let (lane_a, _, lane_b, _) = line_coordinates(index);
                let ring = (index - SPOKE_LINES) / (GRID_LANES - 1);
                let (depth, current_birth) = ring_state(ring, time);
                let shape = birth_shape(current_birth);
                let a = grid_section_point(lane_a, shape);
                let b = grid_section_point(lane_b, shape);
                if let Some((pa, pb)) =
                    project_canonical_flow_grid_segment(
                        time,
                        a,
                        depth,
                        b,
                        depth,
                        &camera,
                        1.0,
                    )
                {
                    let x = (pa.0 + pb.0) * 0.5;
                    let y = (pa.1 + pb.1) * 0.5;
                    if x >= 0.0 && x < WIDTH as f32 && y >= 0.0 && y < HEIGHT as f32 {
                        count += 1;
                        let dx = x - 160.0;
                        let dy = y - 100.0;
                        radius_sum += fast_sqrt(dx * dx + dy * dy);
                        let bin_x = (x * 8.0 / WIDTH as f32) as u32;
                        let bin_y = (y * 5.0 / HEIGHT as f32) as u32;
                        bins |= 1u64 << (bin_y * 8 + bin_x);
                    }
                }
            }
            index += 1;
        }
        let mean_radius = if count > 0 { radius_sum / count as f32 } else { 0.0 };
        if !(count >= 100
            && bins.count_ones() >= 6
            && (10.0..=120.0).contains(&mean_radius))
        {
            let mut report = [0u8; 256];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase3 fixed-world field sample t/count/bins/radius=");
            append_number(pointer, &mut length, tick);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, count);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, bins.count_ones());
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, (mean_radius * 10.0) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(177)
        }
        tick_index += 1;
    }
    let mut capture_sample = 0usize;
    let mut previous_capture_camera = flow_camera(ORBIT_ENTRY);
    let settled_orbit_speed = -TRAJECTORY_ENTRY_RATE;
    let mut minimum_pre_brake_speed = settled_orbit_speed;
    let mut maximum_pre_brake_speed = settled_orbit_speed;
    let mut maximum_post_far_speed = 0.0f64;
    let mut far_side_speed = 0.0f64;
    let mut first_orbit_speed = 0.0f64;
    let mut second_orbit_speed = 0.0f64;
    let mut previous_capture_speed = dvec_length(trajectory_state(ORBIT_ENTRY as f64).velocity);
    let mut previous_capture_phase = 0.0f64;
    let mut maximum_fractional_speed_step = 0.0f64;
    let capture_end = trajectory_phase_crossing(2_048.0, ORBIT_ENTRY as f64, 50.0) + 0.5;
    while capture_sample <= ((capture_end - ORBIT_ENTRY as f64) * TRAJECTORY_HZ as f64) as usize {
        let time = ORBIT_ENTRY + capture_sample as f32 / TRAJECTORY_HZ as f32;
        let camera = flow_camera(time);
        let capture_state = trajectory_state(time as f64);
        let capture_speed = dvec_length(capture_state.velocity);
        let bounds = projected_surface_bounds(&camera, 1.0);
        // Once the nearby planet fills the lower view, its centre correctly
        // lies below the frame: the geometric horizon is now the subject.
        if dvec_length(camera.position) > FLOW_PLANET_RADIUS_WORLD * 3.0
            && !(bounds.visible > 0.0
                && bounds.x >= 0.0
                && bounds.x < WIDTH as f32
                && bounds.y >= 0.0
                && bounds.y < HEIGHT as f32)
        {
            let mut report = [0u8; 160];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase3 capture centre failed sample/y+1000=");
            append_number(pointer, &mut length, capture_sample as u32);
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, ((bounds.y + 1_000.0) * 10.0) as u32);
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(178)
        }
        if capture_sample > 0
            && !(vec_dot(previous_capture_camera.forward, camera.forward) > 0.995
                && vec_dot(previous_capture_camera.down, camera.down) > 0.995)
        {
            let mut report = [0u8; 160];
            let pointer = report.as_mut_ptr();
            let mut length = 0usize;
            append(pointer, &mut length, b"phase3 capture camera jump sample/forward/down+1000=");
            append_number(pointer, &mut length, capture_sample as u32);
            append(pointer, &mut length, b"/");
            append_number(
                pointer,
                &mut length,
                ((vec_dot(previous_capture_camera.forward, camera.forward) + 1.0) * 1_000.0)
                    as u32,
            );
            append(pointer, &mut length, b"/");
            append_number(
                pointer,
                &mut length,
                ((vec_dot(previous_capture_camera.down, camera.down) + 1.0) * 1_000.0) as u32,
            );
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(182)
        }
        if capture_state.phase > 0.0 && capture_state.phase <= 512.0 {
            minimum_pre_brake_speed = minimum_pre_brake_speed.min(capture_speed);
            maximum_pre_brake_speed = maximum_pre_brake_speed.max(capture_speed);
        }
        if capture_state.phase >= 512.0 {
            maximum_post_far_speed = maximum_post_far_speed.max(capture_speed);
        }
        if previous_capture_phase < 512.0 && capture_state.phase >= 512.0 {
            far_side_speed = capture_speed;
        }
        if previous_capture_phase < 1_024.0 && capture_state.phase >= 1_024.0 {
            first_orbit_speed = capture_speed;
        }
        if previous_capture_phase < 2_048.0 && capture_state.phase >= 2_048.0 {
            second_orbit_speed = capture_speed;
        }
        if capture_state.phase > 0.0 {
            maximum_fractional_speed_step = maximum_fractional_speed_step.max(
                (capture_speed - previous_capture_speed).abs()
                    / settled_orbit_speed,
            );
        }
        previous_capture_camera = camera;
        previous_capture_speed = capture_speed;
        previous_capture_phase = capture_state.phase;
        capture_sample += 1;
    }
    // The unified law still follows the incoming reference before 28 s;
    // the old phase-only law incorrectly used the 28 s speed at all times.
    let initial_brake_speed = -approach_camera_reference(WORLD_BRAKE_START).1;
    let orbit_speed_valid = (world_curve_spiral_speed(0.0, WORLD_BRAKE_START) - initial_brake_speed).abs() < 1.0e-12
            && (world_curve_spiral_speed(512.0, WORLD_BRAKE_START) - initial_brake_speed).abs() < 1.0e-12
            && world_curve_spiral_speed(2_048.0, capture_end) > 0.0
            && (world_curve_spiral_speed(WORLD_ORBIT_END_PHASE, flow_aircraft_arrival()) / world_curve_spiral_metric(WORLD_ORBIT_END_PHASE) - WORLD_CURVE_AIRCRAFT_PHASE_RATE).abs() < 1.0e-8
            && minimum_pre_brake_speed >= settled_orbit_speed * 0.65
            && maximum_pre_brake_speed <= settled_orbit_speed * 1.10
            && far_side_speed > 0.0
            && maximum_post_far_speed <= far_side_speed * 1.05
            && first_orbit_speed >= far_side_speed * 0.70
            && second_orbit_speed > 0.0
            && second_orbit_speed <= far_side_speed * 0.20
            && maximum_fractional_speed_step <= 0.02;
    if !orbit_speed_valid {
        let mut report = [0u8; 176];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 orbit speeds milli settled/min/max/maxpostfar/far/first/second/maxstep=");
        append_number(pointer, &mut length, (settled_orbit_speed * 1_000.0) as u32);
        for value in [minimum_pre_brake_speed, maximum_pre_brake_speed, maximum_post_far_speed, far_side_speed, first_orbit_speed, second_orbit_speed, maximum_fractional_speed_step] {
            append(pointer, &mut length, b"/");
            append_number(pointer, &mut length, (value * 1_000.0) as u32);
        }
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(182)
    }
    let capture_step = 1.0f32 / TRAJECTORY_HZ as f32;
    let mut bank_tick = (WORLD_CURVE_START * TRAJECTORY_HZ as f64) as u32 + 1;
    let bank_end_tick = (capture_end * TRAJECTORY_HZ as f64) as u32;
    let mut previous_bank_camera = flow_camera(WORLD_CURVE_START as f32);
    let mut maximum_forward_rate = 0.0f32;
    let mut maximum_down_rate = 0.0f32;
    let mut maximum_forward_acceleration = 0.0f32;
    let mut maximum_down_acceleration = 0.0f32;
    let mut maximum_forward_acceleration_tick = bank_tick;
    let mut maximum_down_acceleration_tick = bank_tick;
    while bank_tick <= bank_end_tick {
        let time = bank_tick as f32 / TRAJECTORY_HZ as f32;
        let camera = flow_camera(time);
        let forward_rate = vec_scale(
            vec_sub(camera.forward, previous_bank_camera.forward),
            1.0 / capture_step,
        );
        let down_rate = vec_scale(
            vec_sub(camera.down, previous_bank_camera.down),
            1.0 / capture_step,
        );
        maximum_forward_rate = maximum_forward_rate.max(vec_length(forward_rate));
        maximum_down_rate = maximum_down_rate.max(vec_length(down_rate));
        const COMFORT_WINDOW: f32 = 0.1;
        if time >= WORLD_CURVE_START as f32 + COMFORT_WINDOW * 2.0 {
            let first_window_camera = flow_camera(time - COMFORT_WINDOW * 2.0);
            let middle_window_camera = flow_camera(time - COMFORT_WINDOW);
            let first_forward_rate = vec_scale(
                vec_sub(middle_window_camera.forward, first_window_camera.forward),
                1.0 / COMFORT_WINDOW,
            );
            let second_forward_rate = vec_scale(
                vec_sub(camera.forward, middle_window_camera.forward),
                1.0 / COMFORT_WINDOW,
            );
            let first_down_rate = vec_scale(
                vec_sub(middle_window_camera.down, first_window_camera.down),
                1.0 / COMFORT_WINDOW,
            );
            let second_down_rate = vec_scale(
                vec_sub(camera.down, middle_window_camera.down),
                1.0 / COMFORT_WINDOW,
            );
            let forward_acceleration = vec_scale(
                vec_sub(second_forward_rate, first_forward_rate),
                1.0 / COMFORT_WINDOW,
            );
            let forward_acceleration = vec_length(vec_sub(
                forward_acceleration,
                vec_scale(
                    middle_window_camera.forward,
                    vec_dot(forward_acceleration, middle_window_camera.forward),
                ),
            ));
            let down_acceleration = vec_scale(
                vec_sub(second_down_rate, first_down_rate),
                1.0 / COMFORT_WINDOW,
            );
            let down_acceleration = vec_length(vec_sub(
                down_acceleration,
                vec_scale(
                    middle_window_camera.down,
                    vec_dot(down_acceleration, middle_window_camera.down),
                ),
            ));
            if forward_acceleration > maximum_forward_acceleration {
                maximum_forward_acceleration = forward_acceleration;
                maximum_forward_acceleration_tick = bank_tick;
            }
            if down_acceleration > maximum_down_acceleration {
                maximum_down_acceleration = down_acceleration;
                maximum_down_acceleration_tick = bank_tick;
            }
        }
        previous_bank_camera = camera;
        bank_tick += 1;
    }
    if !(maximum_forward_rate <= FLIGHT_TURN_RATE_LIMIT as f32
        && maximum_down_rate <= FLIGHT_TURN_RATE_LIMIT as f32
        && maximum_forward_acceleration <= FLIGHT_TURN_ACCELERATION_LIMIT as f32
        && maximum_down_acceleration <= FLIGHT_TURN_ACCELERATION_LIMIT as f32)
    {
        let mut report = [0u8; 192];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase3 capture frame rate/down/acceleration milli=");
        append_number(pointer, &mut length, (maximum_forward_rate * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (maximum_down_rate * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (maximum_forward_acceleration * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, (maximum_down_acceleration * 1_000.0) as u32);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, maximum_forward_acceleration_tick);
        append(pointer, &mut length, b"/");
        append_number(pointer, &mut length, maximum_down_acceleration_tick);
        append(pointer, &mut length, b"\n");
        write_all(&report[..length]);
        exit(182)
    }
    let extent_time = 20.0f32;
    let fixed_shape = birth_shape(travel_row(PLANET_INTRO));
    let field_forward = flow_field_basis().0;
    let first_physical_ring = field_ring_s(
        field,
        field_ring_identity(field_first_physical_ring_ordinal(field)),
    );
    let last_physical_ring =
        field_ring_s(field, field_ring_identity(field_last_ring_ordinal(field)));
    phase3_require(
        first_physical_ring < field.planet_s - GRID_WELL_RADIUS_WORLD
            && last_physical_ring > field.planet_s + GRID_WELL_RADIUS_WORLD,
        179,
        b"mesh does not extend continuously from the horizon past the planet",
    );
    let transport_step = 1.0f32 / TRAJECTORY_HZ as f32;
    let transport_before = canonical_flow_field_transport(ORBIT_ENTRY - transport_step);
    let transport_entry = canonical_flow_field_transport(ORBIT_ENTRY);
    let transport_after = canonical_flow_field_transport(ORBIT_ENTRY + transport_step);
    let velocity_before = dvec_scale(
        dvec_sub(transport_entry, transport_before),
        1.0 / transport_step as f64,
    );
    let velocity_after = dvec_scale(
        dvec_sub(transport_after, transport_entry),
        1.0 / transport_step as f64,
    );
    let post_capture_transport = canonical_flow_field_transport(29.0);
    phase3_require(
        dvec_length(dvec_sub(velocity_before, velocity_after)) < 0.1
            && dvec_dot(
                dvec_sub(post_capture_transport, transport_entry),
                field_forward,
            )
                > 1.0
            && dvec_length(dvec_sub(post_capture_transport, flow_camera(29.0).position)) > 0.1,
        179,
        b"mesh transport jumps at capture or follows the orbiting camera",
    );
    let extent_camera = flow_camera(extent_time);
    let top = grid_section_point(7.0, fixed_shape);
    let flow_depths = [0.25f32, 0.5, 1.0, 2.0, 4.0, 8.0, 12.0, 16.0, 20.0, 24.0];
    let mut previous_radius = -1.0f32;
    let mut flow_index = 0usize;
    while flow_index < flow_depths.len() {
        let depth = flow_depths[flow_index];
        let point = canonical_flow_grid_world_point(extent_time, top, depth);
        let projected = project_flow_depth(&extent_camera, point)
            .unwrap_or_else(|| phase3_fail(179, b"mesh flow leaves the forward world field"));
        let dx = projected.0 - 160.0;
        let dy = projected.1 - 100.0;
        let radius = fast_sqrt(dx * dx + dy * dy);
        phase3_require(
            radius > previous_radius,
            179,
            b"mesh radius does not flow monotonically out of the horizon",
        );
        previous_radius = radius;
        flow_index += 1;
    }
    write_all(
        b"phase3 unified-camera audit ok: universe=fixed opening=exact-through-20.5 stars=through-circular-tunnel rails=near-plane-clipped capture=one-arc-length-curve/20-degree-over-under-entry/strict-tunnel/localized-underpass braking=25.658s/continuous-through-orbit frame=geometry-derived/continuous-fast-orbit planet=fixed seek=stable\n",
    );
    exit(0)
}
