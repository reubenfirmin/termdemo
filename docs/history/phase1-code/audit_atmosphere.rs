//! ARCHIVED phase-1 implementation. Not compiled or used by the active flight.
#[cfg(feature = "phase0-audit")]
use super::{
    DEPTH, FRAME, HEIGHT, METERS_PER_MILE, ORBIT_ENTRY, PIXELS, PLANET_CENTER,
    SURFACE_SAMPLE_CACHE, WIDTH, append, append_number, apply_atmospheric_shell,
    atmosphere_exp, atmosphere_optical_depth, atmosphere_ray_segment, atmosphere_scatter_color,
    atmospheric_alpha, atmospheric_alpha_to_depth, blend_depth_pixel, brake_exp,
    clear_depth_buffer, cloud_density, draw_sun, dvec_from_vec3, dvec_normalize, dvec_scale,
    dvec_sub, exit, field_star_exposure_camera, flow_aircraft_arrival, flow_atmosphere_amount,
    flow_camera, flow_camera_ray, flow_entry_heat, flow_high_pass_end, flow_sun_direction,
    flow_sun_projection, flow_surface_color, flow_surface_sample, flow_valley_corridor,
    flow_valley_overhead, landing_forward, landing_up, landscape_flight_fingerprint,
    miles_to_world, process_cpu_ns, put_pixel, record_depth_line, render_demo,
    render_flow_surface, render_universe_field, surface_lod, universe_space_visibility,
    vec_dot, vec_scale, write_all,
};

#[cfg(feature = "phase0-audit")]
pub(super) fn phase6_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase6 atmosphere audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
pub(super) fn phase6_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase6_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
pub(super) fn landscape_depth_fingerprint() -> u64 {
    let mut result = 14_695_981_039_346_656_037u64;
    for pixel in 0..PIXELS {
        result ^= unsafe { core::ptr::addr_of!(DEPTH[pixel]).read() }.to_bits() as u64;
        result = result.wrapping_mul(1_099_511_628_211);
    }
    result
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_phase6_atmosphere_audit() -> ! {
    phase6_require(landscape_flight_fingerprint() == ((1_120_209_875u64 << 32) | 1_356_210_471),
        241, b"atmosphere work changed the frozen flight");
    run_phase6_atmosphere_checks()
}

// The independent aggregate keeps the same fingerprint failure visible while
// still exercising atmosphere checks and the performance tail.
#[cfg(feature = "phase0-audit")]
pub(super) fn run_phase6_atmosphere_checks() -> ! {
    for index in 0..=1_024 {
        let value = -24.0 * index as f64 / 1_024.0;
        phase6_require((atmosphere_exp(value) / brake_exp(value) - 1.0).abs() < 1.0e-8,
            242, b"radiative exponential exceeds its numerical error bound");
    }
    for time in [40.0, 42.0, 58.0, 122.0] {
        let camera = flow_camera(time);
        let direction = dvec_normalize(dvec_from_vec3(camera.forward));
        let near = atmosphere_optical_depth(&camera, direction, 0.0, miles_to_world(10.0 / METERS_PER_MILE));
        phase6_require(atmospheric_alpha(near) < 0.005,
            243, b"nearby terrain inherits distant atmospheric haze");
        let end = miles_to_world(500.0 / METERS_PER_MILE);
        let total = atmosphere_optical_depth(&camera, direction, 0.0, end);
        let split = atmosphere_optical_depth(&camera, direction, 0.0, end * 0.5)
            + atmosphere_optical_depth(&camera, direction, end * 0.5, end);
        phase6_require(total > near && (total - split).abs() < total * 0.001 + 1.0e-6,
            244, b"optical density integral is not additive on the visible interval");
        let upward = dvec_normalize(camera.position);
        let (_, shell_exit, column) = atmosphere_ray_segment(&camera, upward).unwrap();
        let half_column = atmosphere_optical_depth(&camera, upward, 0.0, shell_exit * 0.5)
            + atmosphere_optical_depth(&camera, upward, shell_exit * 0.5, shell_exit);
        phase6_require((column - half_column).abs() < column * 0.01 + 1.0e-5,
            244, b"full atmospheric column does not converge under interval subdivision");
        let mut rotated = camera;
        rotated.down = vec_scale(camera.down, -1.0);
        rotated.right = vec_scale(camera.right, -1.0);
        phase6_require(atmosphere_scatter_color(&camera, upward) == atmosphere_scatter_color(&rotated, upward),
            245, b"sky gradient is attached to screen orientation rather than the world ray");
    }
    for normal in [landing_up(), vec_scale(landing_up(), -1.0), landing_forward()] {
        let cloud = cloud_density(normal, 0.1);
        phase6_require((0.0..=0.12).contains(&cloud) && cloud == cloud_density(normal, 0.1)
            && cloud_density(normal, 5_000.0) == 0.0,
            246, b"cloud field is unbounded, unstable or unfiltered");
    }
    let above = flow_camera(ORBIT_ENTRY);
    let mut entry_time = ORBIT_ENTRY;
    let mut entry = above;
    let mut found_entry = false;
    while entry_time < flow_aircraft_arrival() as f32 {
        let candidate = flow_camera(entry_time);
        let density = flow_atmosphere_amount(&candidate);
        if density > 0.05 && density < 0.95 && flow_entry_heat(&candidate) > 0.0 {
            entry = candidate;
            found_entry = true;
            break;
        }
        entry_time += 0.05;
    }
    phase6_require(
        found_entry
            && flow_atmosphere_amount(&above) == 0.0
            && flow_atmosphere_amount(&entry) > 0.0
            && flow_atmosphere_amount(&entry) < 1.0,
        220,
        b"atmospheric density does not follow physical altitude",
    );
    let toward_surface = dvec_normalize(dvec_sub(PLANET_CENTER, entry.position));
    let away_from_surface = dvec_scale(toward_surface, -1.0);
    let downward = atmosphere_ray_segment(&entry, toward_surface)
        .unwrap_or_else(|| phase6_fail(221, b"surface ray misses the atmosphere shell"));
    let upward = atmosphere_ray_segment(&entry, away_from_surface)
        .unwrap_or_else(|| phase6_fail(222, b"sky ray misses the atmosphere shell"));
    phase6_require(
        downward.0 < downward.1
            && upward.0 < upward.1
            && downward.2 > upward.2
            && atmospheric_alpha(downward.2) > atmospheric_alpha(upward.2),
        223,
        b"optical depth does not increase along the denser surface ray",
    );

    let mut stalled = entry;
    stalled.speed_world_per_second = 0.0;
    phase6_require(
        flow_entry_heat(&entry) > 0.0 && flow_entry_heat(&stalled) == 0.0,
        224,
        b"plasma is not coupled to measured flight speed and density",
    );

    let normal = landing_up();
    let lod = surface_lod(20.0);
    let sample = flow_surface_sample(normal, lod);
    phase6_require(
        flow_surface_color(normal, 70.0, sample, lod)
            == flow_surface_color(normal, 100.0, sample, lod),
        225,
        b"cloud field moves independently of its spherical coordinates",
    );

    clear_depth_buffer();
    record_depth_line((0.0, 0.0), (12.0, 0.0), 5.0, 5.0);
    let recorded_depth = unsafe { core::ptr::addr_of!(DEPTH).cast::<f32>().read() };
    phase6_require(
        (recorded_depth - 5.0).abs() < 0.000_1,
        226,
        b"grid geometry does not expose depth to atmospheric attenuation",
    );
    clear_depth_buffer();
    let entry_exposure_start = field_star_exposure_camera(entry_time, &entry);
    render_universe_field(&entry_exposure_start, &entry, 1.0);
    let grid_depth = core::ptr::addr_of!(DEPTH).cast::<f32>();
    let mut grid_samples = 0usize;
    let mut optically_attenuated = 0usize;
    let mut y = 0usize;
    while y < HEIGHT {
        let mut x = 0usize;
        while x < WIDTH {
            let depth = unsafe { grid_depth.add(y * WIDTH + x).read() };
            if depth < f32::MAX {
                let direction = flow_camera_ray(&entry, 1.0, x as f32 + 0.5, y as f32 + 0.5);
                if let Some(_) =
                    atmosphere_ray_segment(&entry, direction)
                {
                    let grid_alpha = atmospheric_alpha_to_depth(
                        &entry,
                        direction,
                        depth,
                    );
                    let full_alpha = atmospheric_alpha_to_depth(
                        &entry,
                        direction,
                        f32::MAX,
                    );
                    phase6_require(
                        grid_alpha >= 0.0
                            && grid_alpha <= full_alpha + 1.0e-6
                            && full_alpha <= 1.0,
                        228,
                        b"rendered grid attenuation exceeds its physical ray segment",
                    );
                    grid_samples += 1;
                    if grid_alpha > 0.0 {
                        optically_attenuated += 1;
                    }
                }
            }
            x += 1;
        }
        y += 1;
    }
    phase6_require(
        grid_samples > 0 && optically_attenuated > 0,
        229,
        b"rendered atmospheric-entry grid does not exercise optical attenuation",
    );
    let geometric_depth = landscape_depth_fingerprint();
    apply_atmospheric_shell(&entry, 1.0);
    phase6_require(geometric_depth == landscape_depth_fingerprint(),
        247, b"atmospheric shading changes geometric depth ownership");
    let airplane_time = flow_aircraft_arrival() as f32 + 1.0;
    let airplane_pass = flow_camera(airplane_time);
    phase6_require(
        universe_space_visibility(&airplane_pass) == 0.0,
        229,
        b"space field remains materially visible at the 30000-foot airplane pass",
    );
    clear_depth_buffer();
    let airplane_exposure_start = field_star_exposure_camera(airplane_time, &airplane_pass);
    render_universe_field(&airplane_exposure_start, &airplane_pass, 1.0);
    let extinguished_depth = core::ptr::addr_of!(DEPTH).cast::<f32>();
    let mut extinguished_pixel = 0usize;
    while extinguished_pixel < PIXELS {
        phase6_require(
            unsafe { extinguished_depth.add(extinguished_pixel).read() } == f32::MAX,
            229,
            b"extinguished extraterrestrial field still writes depth at 30000 feet",
        );
        extinguished_pixel += 1;
    }
    clear_depth_buffer();
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    put_pixel(frame, 40, 40, 20, 30, 40);
    blend_depth_pixel(frame, 40, 40, 1.0, 20, 30, 40, 255, 0.0);
    draw_sun(40.0, 40.0, 1.0);
    let sun_pixel = unsafe {
        let offset = (40 * WIDTH + 40) * 3;
        (frame.add(offset).read(), frame.add(offset + 1).read(), frame.add(offset + 2).read())
    };
    phase6_require(
        sun_pixel == (20, 30, 40),
        230,
        b"infinite-distance sun overwrites foreground world geometry",
    );

    let airplane_sun_times = [40.0, airplane_time, flow_high_pass_end() as f32,
        flow_valley_overhead() as f32, flow_valley_corridor() as f32, 90.0, 122.0, 137.0];
    let mut sun_index = 0usize;
    let mut exposed_sun_views = 0usize;
    while sun_index < airplane_sun_times.len() {
        let time = airplane_sun_times[sun_index];
        let camera = flow_camera(time);
        if let Some(projection) = flow_sun_projection(&camera, 1.0) {
            if projection.0 >= -14.0
                && projection.0 < WIDTH as f32 + 14.0
                && projection.1 >= -14.0
                && projection.1 < HEIGHT as f32 + 14.0
            {
                clear_depth_buffer();
                let exposure_start = field_star_exposure_camera(time, &camera);
                render_universe_field(&exposure_start, &camera, 1.0);
                render_flow_surface(&camera, time, 1.0);
                let depth = core::ptr::addr_of!(DEPTH).cast::<f32>();
                let center_x = projection.0 as i32;
                let center_y = projection.1 as i32;
                let mut exposed_pixels = 0usize;
                let mut offset_y = -14i32;
                while offset_y <= 14 {
                    let mut offset_x = -14i32;
                    while offset_x <= 14 {
                        let x = center_x + offset_x;
                        let y = center_y + offset_y;
                        if x >= 0
                            && y >= 0
                            && x < WIDTH as i32
                            && y < HEIGHT as i32
                            && offset_x * offset_x + offset_y * offset_y <= 14 * 14
                            && unsafe { depth.add(y as usize * WIDTH + x as usize).read() }
                                == f32::MAX
                        {
                            exposed_pixels += 1;
                        }
                        offset_x += 1;
                    }
                    offset_y += 1;
                }
                if exposed_pixels >= 12 {
                    exposed_sun_views += 1;
                }
                let mut report = [0u8; 192];
                let p = report.as_mut_ptr();
                let mut n = 0;
                append(p, &mut n, b"phase6 sun t/exposed-pixels/top-sky-columns=");
                append_number(p, &mut n, time as u32);
                append(p, &mut n, b"/");
                append_number(p, &mut n, exposed_pixels as u32);
                for x in [32, 96, 160, 224, 288] {
                    let mut empty = 0;
                    for y in 0..48 {
                        if unsafe { depth.add(y * WIDTH + x).read() } == f32::MAX { empty += 1; }
                    }
                    append(p, &mut n, b"/");
                    append_number(p, &mut n, empty);
                }
                append(p, &mut n, b"\n");
                write_all(&report[..n]);
            }
        }
        sun_index += 1;
    }
    if exposed_sun_views != airplane_sun_times.len() {
        let mut report = [0u8; 512];
        let report_pointer = report.as_mut_ptr();
        let mut report_length = 0usize;
        let sun = flow_sun_direction();
        sun_index = 0;
        append(report_pointer, &mut report_length, b"phase6 atmosphere audit failed: airplane-pass sun exposure ");
        append_number(report_pointer, &mut report_length, exposed_sun_views as u32);
        append(report_pointer, &mut report_length, b"/");
        append_number(report_pointer, &mut report_length, airplane_sun_times.len() as u32);
        append(report_pointer, &mut report_length, b" camera coordinates");
        while sun_index < airplane_sun_times.len() {
            let camera = flow_camera(airplane_sun_times[sun_index]);
            append(report_pointer, &mut report_length, b" t=");
            append_number(
                report_pointer,
                &mut report_length,
                airplane_sun_times[sun_index] as u32,
            );
            append(report_pointer, &mut report_length, b" r/d/f+2000=");
            append_number(
                report_pointer,
                &mut report_length,
                ((vec_dot(sun, camera.right) + 2.0) * 1_000.0) as u32,
            );
            append(report_pointer, &mut report_length, b"/");
            append_number(
                report_pointer,
                &mut report_length,
                ((vec_dot(sun, camera.down) + 2.0) * 1_000.0) as u32,
            );
            append(report_pointer, &mut report_length, b"/");
            append_number(
                report_pointer,
                &mut report_length,
                ((vec_dot(sun, camera.forward) + 2.0) * 1_000.0) as u32,
            );
            sun_index += 1;
        }
        append(report_pointer, &mut report_length, b"\n");
        write_all(&report[..report_length]);
        exit(240)
    }

    let checkpoints = [entry_time, 32.0, 34.0, 38.0, 40.0, 41.0, 42.0, 53.0,
        58.0, 65.0, 78.0, 90.0, 106.0, 122.0, 137.0];
    let mut checkpoint = 0usize;
    let mut maximum_frame_ns = 0u64;
    while checkpoint < checkpoints.len() {
        // Measure a cold exact-sample cache, not a warmed repeat of the sun
        // or depth tests above. Cache clearing is outside the frame timer.
        unsafe { core::ptr::addr_of_mut!(SURFACE_SAMPLE_CACHE).write_bytes(0, 1); }
        let started = process_cpu_ns();
        render_demo(checkpoints[checkpoint], 1.0);
        let duration = process_cpu_ns().saturating_sub(started);
        maximum_frame_ns = maximum_frame_ns.max(duration);
        let mut timing = [0u8; 96];
        let p = timing.as_mut_ptr();
        let mut n = 0;
        append(p, &mut n, b"phase6 frame t/microseconds=");
        append_number(p, &mut n, checkpoints[checkpoint] as u32);
        append(p, &mut n, b"/");
        append_number(p, &mut n, (duration / 1_000) as u32);
        append(p, &mut n, b"\n");
        write_all(&timing[..n]);
        checkpoint += 1;
    }
    let mut performance = [0u8; 96];
    let p = performance.as_mut_ptr();
    let mut n = 0;
    append(p, &mut n, b"phase6 maximum frame microseconds=");
    append_number(p, &mut n, (maximum_frame_ns / 1_000) as u32);
    append(p, &mut n, b"\n");
    write_all(&performance[..n]);
    phase6_require(
        maximum_frame_ns <= 33_333_334,
        227,
        b"combined surface and atmosphere frame exceeds 33.3 ms",
    );
    write_all(
        b"phase6 atmosphere audit ok: shell=shared optical-depth=ray/grid field=extinguished-by-30000ft plasma=density*speed clouds=anchored sun=fixed/exposed-valley frame<=33.3ms\n",
    );
    exit(0)
}
