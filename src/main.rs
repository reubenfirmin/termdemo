#![no_std]
#![no_main]
#![allow(linker_messages)]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

global_asm!(
    ".global _start",
    ".type _start,@function",
    "_start:",
    "and rsp, -16",
    "call rust_main",
);

const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const PIXELS: usize = WIDTH * HEIGHT;
const FRAME_BYTES: usize = PIXELS * 3;
const MAP_SIDE: usize = 256;
const MAP_BYTES: usize = MAP_SIDE * MAP_SIDE;
const GRID_RINGS: usize = 25;
const GRID_LANES: usize = 73;
const SPOKE_LINES: usize = GRID_LANES * GRID_RINGS;
const RING_LINES: usize = GRID_RINGS * (GRID_LANES - 1);
const STAR_LINES: usize = SPOKE_LINES + RING_LINES;
const RING_INTERVAL: f32 = 0.05;
const RING_SPEED: f32 = 1.0 / RING_INTERVAL;
const FORMATION_EXCESS: f32 = 0.658_436_4;
const RING_BIRTH: f32 = 14.75;
const CLEAN_BIRTH: f32 = ring_birth(GRID_RINGS);
const TX_BYTES: usize = 264_000;
const RAW_CHUNK: usize = 3_072;
const WATER: u8 = 76;
const GRID_FOG_START: f32 = 4.0;
const GRID_FOG_END: f32 = 6.0;
const STARFIELD_HOLD: f32 = 4.0;
const STARFIELD_FILL: f32 = 7.0;
const STAR_ORGANIZE: f32 = 11.0;
const SCENE_2: f32 = 21.0;
const SCENE_3: f32 = 29.0;
const GRID_COLOR_FULL: f32 = SCENE_3 - 2.0;
const SCENE_4: f32 = 44.0;
const SCENE_5: f32 = 90.0;
const DESCENT: f32 = 113.0;
const SCENE_6: f32 = 137.0;
const PLANET_REVEAL: f32 = SCENE_3 + 3.6;
const APPROACH_SPEED: f32 = 0.08;

const SYS_READ: usize = 0;
const SYS_WRITE: usize = 1;
const SYS_IOCTL: usize = 16;
const SYS_NANOSLEEP: usize = 35;
const SYS_EXIT: usize = 60;
const SYS_CLOCK_GETTIME: usize = 228;

const TCGETS: usize = 0x5401;
const TCSETS: usize = 0x5402;
const TIOCGWINSZ: usize = 0x5413;

const ISIG: u32 = 0x0001;
const ICANON: u32 = 0x0002;
const ECHO: u32 = 0x0008;
const IEXTEN: u32 = 0x8000;
const ICRNL: u32 = 0x0100;
const IXON: u32 = 0x0400;
const OPOST: u32 = 0x0001;
const VTIME: usize = 5;
const VMIN: usize = 6;

static mut FRAME: [u8; FRAME_BYTES] = [0; FRAME_BYTES];
static mut TERRAIN: [u8; MAP_BYTES] = [0; MAP_BYTES];
static mut LIGHT: [u8; MAP_BYTES] = [0; MAP_BYTES];
static mut TX: [u8; TX_BYTES] = [0; TX_BYTES];
static mut SINES: [f32; 1_024] = [0.0; 1_024];

#[repr(C)]
#[derive(Clone, Copy)]
struct KernelTermios {
    input: u32,
    output: u32,
    control: u32,
    local: u32,
    line: u8,
    chars: [u8; 19],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct WinSize {
    rows: u16,
    columns: u16,
    pixel_width: u16,
    pixel_height: u16,
}

#[repr(C)]
struct TimeSpec {
    seconds: i64,
    nanos: i64,
}

#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy)]
struct CameraPose {
    position: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    altitude: f32,
}

const PLANET_RADIUS: f32 = 1_000.0;
const GRID_FLOOR_Z: f32 = -1_000.0;
const PLANET_CENTER: Vec3 = Vec3 { x: 0.0, y: 9_000.0, z: -1_250.0 };
const APPROACH_CENTER: Vec3 = Vec3 { x: 0.0, y: 9_000.0, z: -1_000.0 };
const SUN_DIRECTION: Vec3 = Vec3 { x: -0.550, y: 0.250, z: 0.797 };
const ORBIT_END: f32 = 105.0;
const SURFACE_SCALE: f32 = 0.30;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    exit(101)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    generate_sines();
    generate_terrain();

    let mut saved = KernelTermios {
        input: 0,
        output: 0,
        control: 0,
        local: 0,
        line: 0,
        chars: [0; 19],
    };
    let interactive = syscall3(SYS_IOCTL, 0, TCGETS, addr_mut(&mut saved)) >= 0;
    let mut audit_input = [0u8; 16];
    let mut audit_length = 0usize;
    if !interactive {
        let read = syscall3(SYS_READ, 0, audit_input.as_mut_ptr() as usize, audit_input.len());
        if read > 0 {
            audit_length = read as usize;
        }
    }
    if interactive {
        let mut raw = saved;
        raw.input &= !(ICRNL | IXON);
        raw.output &= !OPOST;
        raw.local &= !(ISIG | ICANON | ECHO | IEXTEN);
        raw.chars[VTIME] = 0;
        raw.chars[VMIN] = 0;
        syscall3(SYS_IOCTL, 0, TCSETS, addr(&raw));
        write_all(b"\x1b[?1049h\x1b[?25l\x1b[2J\x1b[H");
    }

    let mut first = true;
    let audit_time = match audit_input[0] {
        b'a' => 29.0,
        b'b' => 34.0,
        b'c' => 42.0,
        b'd' => 50.0,
        b'e' => 58.0,
        b'f' => 66.0,
        b'g' => 74.0,
        b'h' => 81.0,
        b'i' => 85.0,
        b'j' => 89.0,
        b'k' => 93.0,
        b'l' => 97.0,
        b'm' => 28.9,
        _ => parse_audit_time(&audit_input[..audit_length]),
    };
    let mut demo_elapsed = if audit_time > 0.0 { audit_time - 1.0 / 60.0 } else { 0.0 };
    let mut previous = now_ns();
    let mut deadline = previous;

    loop {
        let current = now_ns();
        let mut dt = (current.saturating_sub(previous) as f32) * 0.000_000_001;
        if first || dt <= 0.0 || dt > 0.1 {
            dt = 1.0 / 60.0;
        }
        previous = current;
        demo_elapsed += dt;

        let window = window_size();
        render_demo(demo_elapsed, planet_x_scale(window));

        if audit_time > 0.0 {
            let frame = unsafe {
                core::slice::from_raw_parts(core::ptr::addr_of!(FRAME).cast::<u8>(), FRAME_BYTES)
            };
            write_all(frame);
            exit(0);
        }

        let tx_len = encode_frame(first, window.columns, window.rows);
        write_tx(tx_len);
        first = false;

        if !interactive || handle_input(&mut demo_elapsed) {
            break;
        }

        deadline = deadline.saturating_add(16_666_667);
        let after = now_ns();
        if deadline > after {
            sleep_ns(deadline - after);
        } else if after - deadline > 100_000_000 {
            deadline = after;
        }
    }
    if interactive {
        write_all(b"\x1b_Ga=d,d=I,i=1,q=2\x1b\\\x1b[?25h\x1b[?1049l");
        syscall3(SYS_IOCTL, 0, TCSETS, addr(&saved));
    }
    exit(0)
}

fn generate_sines() {
    let table = core::ptr::addr_of_mut!(SINES).cast::<f32>();
    let mut sine = 0.0f32;
    let mut cosine = 1.0f32;
    let step_sine = 0.006_135_884_7f32;
    let step_cosine = 0.999_981_16f32;
    let mut i = 0;
    while i < 1_024 {
        unsafe { table.add(i).write(sine) };
        let next_sine = sine * step_cosine + cosine * step_sine;
        cosine = cosine * step_cosine - sine * step_sine;
        sine = next_sine;
        i += 1;
    }
}

fn hash(x: i32, y: i32) -> i32 {
    let mut value = (x as u32).wrapping_mul(0x45d9_f3b)
        ^ (y as u32).wrapping_mul(0x27d4_eb2d)
        ^ 0xa53a_9d17;
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    (value & 255) as i32
}

fn value_noise(x: i32, y: i32, shift: u32) -> i32 {
    let scale = 1i32 << shift;
    let mask = (MAP_SIDE as i32 >> shift) - 1;
    let cell_x = x >> shift;
    let cell_y = y >> shift;
    let fx = x & (scale - 1);
    let fy = y & (scale - 1);
    let a = hash(cell_x & mask, cell_y & mask);
    let b = hash((cell_x + 1) & mask, cell_y & mask);
    let c = hash(cell_x & mask, (cell_y + 1) & mask);
    let d = hash((cell_x + 1) & mask, (cell_y + 1) & mask);
    let upper = a + ((b - a) * fx >> shift);
    let lower = c + ((d - c) * fx >> shift);
    upper + ((lower - upper) * fy >> shift)
}

fn valley_y(x: f32) -> f32 {
    128.0 + sine_sample(x * 4.0) * 38.0 + sine_sample(x * 12.0 + 173.0) * 7.0
}

fn valley_floor(x: f32) -> f32 {
    92.0 + sine_sample(x * 8.0 + 340.0) * 16.0 + sine_sample(x * 20.0 + 80.0) * 6.0
}

fn landing_slope() -> f32 {
    wrapped_delta(valley_y(18.05), valley_y(17.95)) / 0.10
}

fn wrapped_delta(target: f32, current: f32) -> f32 {
    let mut delta = target - current;
    if delta > 128.0 {
        delta -= 256.0;
    } else if delta < -128.0 {
        delta += 256.0;
    }
    delta
}

fn normalize(x: f32, y: f32) -> (f32, f32) {
    let high = x.abs().max(y.abs());
    let low = x.abs().min(y.abs());
    let inverse = 1.0 / (high + low * 0.375).max(0.001);
    (x * inverse, y * inverse)
}

fn generate_terrain() {
    let map = core::ptr::addr_of_mut!(TERRAIN).cast::<u8>();
    let mut y = 0;
    while y < MAP_SIDE as i32 {
        let mut x = 0;
        while x < MAP_SIDE as i32 {
            let broad = value_noise(x, y, 6) * 5;
            let medium = value_noise(x, y, 5) * 2;
            let detail = value_noise(x, y, 3);
            let ridge = (value_noise(x, y, 4) - 128).abs();
            let mut height = (broad + medium + detail + ridge) >> 3;
            height = ((height - 105) * 3) / 2 + 105;
            let mut valley_distance = (y as f32 - valley_y(x as f32)).abs();
            valley_distance = valley_distance.min(256.0 - valley_distance);
            if valley_distance < 44.0 {
                let valley_height = valley_floor(x as f32) + (valley_distance - 8.0).max(0.0) * 2.65;
                height = height.min(valley_height as i32);
            }
            height = height.clamp(28, 225);
            unsafe { map.add((y as usize) * MAP_SIDE + x as usize).write(height as u8) };
            x += 1;
        }
        y += 1;
    }

    let light = core::ptr::addr_of_mut!(LIGHT).cast::<u8>();
    y = 0;
    while y < MAP_SIDE as i32 {
        let mut x = 0;
        while x < MAP_SIDE as i32 {
            let sample = |px: i32, py: i32| unsafe {
                map.add(((py & 255) as usize) * MAP_SIDE + (px & 255) as usize).read() as i32
            };
            let east = sample(x + 3, y) - sample(x - 3, y);
            let south = sample(x, y + 3) - sample(x, y - 3);
            let value = (128 - east / 3 - south / 5).clamp(94, 162) as u8;
            unsafe { light.add((y as usize) * MAP_SIDE + x as usize).write(value) };
            x += 1;
        }
        y += 1;
    }
}

fn sine(index: i32) -> f32 {
    unsafe { core::ptr::addr_of!(SINES).cast::<f32>().add((index & 1_023) as usize).read() }
}

fn sine_sample(index: f32) -> f32 {
    let mut base = index as i32;
    if index < base as f32 {
        base -= 1;
    }
    let fraction = index - base as f32;
    let a = sine(base);
    a + (sine(base + 1) - a) * fraction
}

fn terrain_smooth(x: f32, y: f32) -> f32 {
    smooth_map(core::ptr::addr_of!(TERRAIN).cast::<u8>(), x, y)
}

fn light_smooth(x: f32, y: f32) -> f32 {
    smooth_map(core::ptr::addr_of!(LIGHT).cast::<u8>(), x, y)
}

fn smooth_map(map: *const u8, x: f32, y: f32) -> f32 {
    let x0 = x as i32;
    let y0 = y as i32;
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let fx = fx * fx * (3.0 - 2.0 * fx);
    let fy = fy * fy * (3.0 - 2.0 * fy);
    let index = |px: i32, py: i32| ((py & 255) as usize) * MAP_SIDE + (px & 255) as usize;
    let a = unsafe { map.add(index(x0, y0)).read() } as f32;
    let b = unsafe { map.add(index(x0 + 1, y0)).read() } as f32;
    let c = unsafe { map.add(index(x0, y0 + 1)).read() } as f32;
    let d = unsafe { map.add(index(x0 + 1, y0 + 1)).read() } as f32;
    let upper = a + (b - a) * fx;
    let lower = c + (d - c) * fx;
    upper + (lower - upper) * fy
}

fn render_demo(elapsed: f32, planet_aspect: f32) {
    render_continuous_grid(elapsed);
    render_world_journey(elapsed, planet_aspect);
    draw_second_counter(elapsed);
}

fn draw_second_counter(elapsed: f32) {
    const DIGITS: [u16; 10] = [
        0b111_101_101_101_111,
        0b010_110_010_010_111,
        0b111_001_111_100_111,
        0b111_001_111_001_111,
        0b101_101_111_001_001,
        0b111_100_111_001_111,
        0b111_100_111_101_111,
        0b111_001_010_010_010,
        0b111_101_111_101_111,
        0b111_101_111_001_111,
    ];
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let mut y = 4i32;
    while y < 19 {
        let mut x = 4i32;
        while x < 43 {
            blend_pixel(frame, x, y, 0, 1, 7, 192);
            x += 1;
        }
        y += 1;
    }

    let whole = elapsed as i32 % 1_000;
    let tenths = (elapsed * 10.0) as i32 % 10;
    let values = [whole / 100, whole / 10 % 10, whole % 10, tenths];
    let positions = [7i32, 15, 23, 34];
    let mut digit = 0usize;
    while digit < values.len() {
        let bits = DIGITS[values[digit] as usize];
        let mut row = 0i32;
        while row < 5 {
            let mut column = 0i32;
            while column < 3 {
                let bit = 14 - (row * 3 + column);
                if bits & (1 << bit) != 0 {
                    let mut py = 0i32;
                    while py < 2 {
                        let mut px = 0i32;
                        while px < 2 {
                            blend_pixel(
                                frame,
                                positions[digit] + column * 2 + px,
                                7 + row * 2 + py,
                                246,
                                222,
                                158,
                                245,
                            );
                            px += 1;
                        }
                        py += 1;
                    }
                }
                column += 1;
            }
            row += 1;
        }
        digit += 1;
    }
    blend_pixel(frame, 31, 15, 246, 222, 158, 245);
    blend_pixel(frame, 32, 15, 246, 222, 158, 245);
    blend_pixel(frame, 31, 16, 246, 222, 158, 245);
    blend_pixel(frame, 32, 16, 246, 222, 158, 245);
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn parse_audit_time(bytes: &[u8]) -> f32 {
    let mut value = 0.0f32;
    let mut decimal = 0.0f32;
    let mut i = 0usize;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte == b'.' {
            decimal = 0.1;
        } else if byte >= b'0' && byte <= b'9' {
            let digit = (byte - b'0') as f32;
            if decimal > 0.0 {
                value += digit * decimal;
                decimal *= 0.1;
            } else {
                value = value * 10.0 + digit;
            }
        }
        i += 1;
    }
    value
}

fn vec_add(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z }
}

fn vec_sub(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}

fn vec_scale(a: Vec3, scale: f32) -> Vec3 {
    Vec3 { x: a.x * scale, y: a.y * scale, z: a.z * scale }
}

fn vec_dot(a: Vec3, b: Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn vec_cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

fn vec_length(a: Vec3) -> f32 {
    fast_sqrt(vec_dot(a, a))
}

fn vec_normalize(a: Vec3) -> Vec3 {
    vec_scale(a, 1.0 / vec_length(a).max(0.000_1))
}

fn landing_up() -> Vec3 {
    Vec3 {
        x: 0.0,
        y: -sine_sample(262.0),
        z: sine_sample(6.0),
    }
}

fn landing_forward() -> Vec3 {
    Vec3 {
        x: 0.0,
        y: sine_sample(6.0),
        z: sine_sample(262.0),
    }
}

fn planet_map_from_normal(normal: Vec3) -> (f32, f32) {
    let landing_slope = landing_slope();
    let forward = vec_dot(normal, landing_forward());
    (
        18.0 + forward * PLANET_RADIUS * SURFACE_SCALE,
        valley_y(18.0)
            + (normal.x + forward * landing_slope) * PLANET_RADIUS * SURFACE_SCALE,
    )
}

fn planet_relief(normal: Vec3) -> f32 {
    let map = planet_map_from_normal(normal);
    let height = terrain_smooth(map.0, map.1);
    let base = (height - WATER as f32).max(0.0) * 0.12;
    let valley_distance = wrapped_delta(map.1, valley_y(map.0)).abs();
    let wall = smoothstep((valley_distance - 5.0) / 27.0);
    let ridge_chain = smoothstep(
        (sine_sample(map.0 * 16.0 + map.1 * 2.0 + 97.0) - 0.02) / 0.82,
    );
    let ridge = 0.18
        + sine_sample(map.0 * 12.0 + map.1 * 3.0 + 311.0).abs() * 0.46
        + sine_sample(map.0 * 28.0 - map.1 * 7.0 + 719.0).abs() * 0.24
        + ridge_chain * 0.76;
    base + (height - 110.0).max(0.0) * 0.12 + wall * ridge * 88.0
}

fn relief_visibility(time: f32, altitude: f32) -> f32 {
    smoothstep((180.0 - altitude) / 140.0) * smoothstep((time - 100.0) / 13.0)
}

fn visible_planet_relief(normal: Vec3, altitude: f32, time: f32) -> f32 {
    // At orbital distances these features are far below a pixel and must not
    // deform the globe silhouette.  Resolve them continuously as we descend.
    planet_relief(normal) * relief_visibility(time, altitude)
}

fn orbit_altitude_and_speed(progress: f32) -> (f32, f32) {
    let orbit_start = 0.08;
    let initial_speed = 66_000.0 / (ORBIT_END - SCENE_3);
    if progress <= orbit_start {
        let remaining = 1.0 - progress;
        return (
            60.0 + 22_000.0 * remaining * remaining * remaining,
            initial_speed * remaining * remaining,
        );
    }
    let orbit_progress = (progress - orbit_start) / (1.0 - orbit_start);
    let remaining = 1.0 - orbit_progress;
    let smooth = orbit_progress * orbit_progress * (3.0 - 2.0 * orbit_progress);
    let compression = 1.0 - 0.95 * smooth;
    let compression_derivative = -0.95 * (6.0 * orbit_progress - 6.0 * orbit_progress * orbit_progress);
    let altitude_span = 22_000.0 * (1.0 - orbit_start) * (1.0 - orbit_start)
        * (1.0 - orbit_start);
    let altitude = 60.0 + altitude_span * remaining * remaining * remaining * compression;
    let derivative = altitude_span
        * (-3.0 * remaining * remaining * compression
            + remaining * remaining * remaining * compression_derivative);
    let duration = (ORBIT_END - SCENE_3) * (1.0 - orbit_start);
    (altitude, -derivative / duration)
}

fn orbit_phase_units(progress: f32) -> f32 {
    // Integrate angular velocity from a monotonically decreasing total-speed
    // profile. Radial velocity is supplied by the altitude curve; the balance
    // becomes tangential velocity, so the ship bends into orbit without first
    // slowing and then accelerating again.
    const STEPS: usize = 128;
    let orbit_start = 0.08;
    if progress <= orbit_start {
        return 6.0;
    }
    let orbit_progress = (progress - orbit_start) / (1.0 - orbit_start);
    let initial_speed = 66_000.0 / (ORBIT_END - SCENE_3);
    let terminal_speed = 3.45 / SURFACE_SCALE;
    let orbit_initial_speed = initial_speed * (1.0 - orbit_start) * (1.0 - orbit_start);
    let speed_shape = 0.070_891_42;
    let mut radians = 0.0;
    let mut step = 0usize;
    let complete_steps = (orbit_progress * STEPS as f32) as usize;
    while step < complete_steps.min(STEPS) {
        let sample_progress = (step as f32 + 0.5) / STEPS as f32;
        let sample = orbit_start + sample_progress * (1.0 - orbit_start);
        let (altitude, radial_speed) = orbit_altitude_and_speed(sample);
        let radius = PLANET_RADIUS + altitude;
        let orbit_remaining = 1.0 - sample_progress;
        let speed_fraction = orbit_remaining
            / (speed_shape + (1.0 - speed_shape) * orbit_remaining);
        let total_speed = terminal_speed
            + (orbit_initial_speed - terminal_speed) * speed_fraction;
        let tangent_speed = fast_sqrt(
            (total_speed * total_speed - radial_speed * radial_speed).max(0.0),
        );
        radians += tangent_speed / radius
            * (ORBIT_END - SCENE_3) * (1.0 - orbit_start)
            / STEPS as f32;
        step += 1;
    }
    if complete_steps < STEPS {
        let completed = complete_steps as f32 / STEPS as f32;
        let partial = orbit_progress - completed;
        if partial > 0.0 {
            let sample_progress = completed + partial * 0.5;
            let sample = orbit_start + sample_progress * (1.0 - orbit_start);
            let (altitude, radial_speed) = orbit_altitude_and_speed(sample);
            let radius = PLANET_RADIUS + altitude;
            let orbit_remaining = 1.0 - sample_progress;
            let speed_fraction = orbit_remaining
                / (speed_shape + (1.0 - speed_shape) * orbit_remaining);
            let total_speed = terminal_speed
                + (orbit_initial_speed - terminal_speed) * speed_fraction;
            let tangent_speed = fast_sqrt(
                (total_speed * total_speed - radial_speed * radial_speed).max(0.0),
            );
            radians += tangent_speed / radius
                * (ORBIT_END - SCENE_3) * (1.0 - orbit_start) * partial;
        }
    }
    6.0 + radians * 271.934_5
}

fn pre_orbit_altitude(elapsed: f32) -> f32 {
    let duration = SCENE_3 - CLEAN_BIRTH;
    let start_speed = 1_250.0;
    let end_speed = 66_000.0 / (ORBIT_END - SCENE_3);
    if elapsed <= CLEAN_BIRTH {
        let transition_distance = duration * (start_speed + end_speed) * 0.5;
        return 22_060.0 + transition_distance + (CLEAN_BIRTH - elapsed) * start_speed;
    }
    let progress = ((elapsed - CLEAN_BIRTH) / duration).clamp(0.0, 1.0);
    let integrated_smooth = progress * progress * progress
        - 0.5 * progress * progress * progress * progress;
    let integrated_speed = start_speed * progress
        + (end_speed - start_speed) * integrated_smooth;
    let total_distance = duration * (start_speed + end_speed) * 0.5;
    22_060.0 + total_distance - duration * integrated_speed
}

fn journey_center(elapsed: f32) -> Vec3 {
    if elapsed < SCENE_3 {
        return APPROACH_CENTER;
    }
    let capture = smoothstep((elapsed - SCENE_3) / ((ORBIT_END - SCENE_3) * 0.08));
    vec_add(
        APPROACH_CENTER,
        vec_scale(vec_sub(PLANET_CENTER, APPROACH_CENTER), capture),
    )
}

fn journey_position(elapsed: f32) -> (Vec3, f32, f32) {
    if elapsed < SCENE_3 {
        let altitude = pre_orbit_altitude(elapsed);
        let phase = 6.0;
        let radial = Vec3 {
            x: 0.0,
            y: -sine_sample(phase + 256.0),
            z: sine_sample(phase),
        };
        let distance = PLANET_RADIUS + altitude;
        return (vec_add(journey_center(elapsed), vec_scale(radial, distance)), altitude, 0.0);
    }
    if elapsed <= ORBIT_END {
        let progress = ((elapsed - SCENE_3) / (ORBIT_END - SCENE_3)).clamp(0.0, 1.0);
        let altitude = orbit_altitude_and_speed(progress).0;
        let phase = orbit_phase_units(progress);
        let radial = Vec3 {
            x: 0.0,
            y: -sine_sample(phase + 256.0),
            z: sine_sample(phase),
        };
        let distance = PLANET_RADIUS + visible_planet_relief(radial, altitude, elapsed) + altitude;
        return (vec_add(journey_center(elapsed), vec_scale(radial, distance)), altitude, progress);
    }

    let local_time = elapsed - ORBIT_END;
    let descent = smoothstep(local_time / (SCENE_6 - ORBIT_END));
    let altitude = 60.0 - descent * 52.0;
    let map_x = 18.0 + local_time * 3.45;
    let map_y = valley_y(map_x);
    let landing_slope = landing_slope();
    let local_x = (map_y - valley_y(18.0) - (map_x - 18.0) * landing_slope)
        / SURFACE_SCALE;
    let local_y = (map_x - 18.0) / SURFACE_SCALE;
    let sphere_z = fast_sqrt((PLANET_RADIUS * PLANET_RADIUS
        - local_x * local_x
        - local_y * local_y)
        .max(1.0)) * 1.001_695_8;
    let normal = vec_scale(vec_add(
        Vec3 { x: local_x, y: 0.0, z: 0.0 },
        vec_add(
            vec_scale(landing_forward(), local_y),
            vec_scale(landing_up(), sphere_z),
        ),
    ), 1.0 / PLANET_RADIUS);
    let distance = PLANET_RADIUS + visible_planet_relief(normal, altitude, elapsed) + altitude;
    (vec_add(PLANET_CENTER, vec_scale(normal, distance)), altitude, 1.0)
}

fn journey_camera(elapsed: f32) -> CameraPose {
    let (position, altitude, progress) = journey_position(elapsed);
    let sample = 0.025;
    let before = journey_position((elapsed - sample).max(CLEAN_BIRTH)).0;
    let after = journey_position(elapsed + sample).0;
    let velocity = vec_normalize(vec_sub(after, before));
    let radial_up = vec_normalize(vec_sub(position, journey_center(elapsed)));
    let down = vec_scale(radial_up, -1.0);
    let look_down = if elapsed <= ORBIT_END {
        0.78 - progress * 0.06
    } else {
        let landscape = smoothstep((elapsed - ORBIT_END) / (SCENE_6 - ORBIT_END));
        0.72 - landscape * 0.64
    };
    let forward = vec_normalize(vec_add(
        vec_scale(velocity, 1.0 - look_down),
        vec_scale(down, look_down),
    ));
    // The orbit lies in Y/Z, so the X axis is a stable roll reference.  Project
    // it onto the view plane to keep the camera basis orthogonal during the
    // later terrain-following section without ever allowing a pole flip.
    let orbit_axis = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    let right = vec_normalize(vec_sub(
        orbit_axis,
        vec_scale(forward, vec_dot(orbit_axis, forward)),
    ));
    let up = vec_normalize(vec_cross(right, forward));
    CameraPose { position, forward, right, up, altitude }
}

fn project_world(camera: &CameraPose, point: Vec3, x_scale: f32) -> Option<(f32, f32, f32)> {
    let relative = vec_sub(point, camera.position);
    let depth = vec_dot(relative, camera.forward);
    if depth <= 8.0 {
        return None;
    }
    let focal = 154.0;
    Some((
        160.0 + vec_dot(relative, camera.right) / depth * focal * x_scale,
        100.0 - vec_dot(relative, camera.up) / depth * focal,
        depth,
    ))
}

fn grid_floor_height(x: f32, y: f32) -> f32 {
    let dx = x - PLANET_CENTER.x;
    let dy = y - PLANET_CENTER.y;
    let distance = fast_sqrt(dx * dx + dy * dy);
    let well = (1.0 - distance / 2_700.0).clamp(0.0, 1.0);
    GRID_FLOOR_Z - well * well * 700.0
}

fn grid_half_width(y: f32) -> f32 {
    let expansion = smoothstep((y + 14_000.0) / 20_500.0);
    3_600.0 + expansion * 1_900.0
}

fn grid_ceiling(y: f32) -> f32 {
    let expansion = smoothstep((y + 14_000.0) / 20_500.0);
    800.0 + expansion * 700.0
}

fn world_grid_shape(time: f32, y: f32) -> f32 {
    let distance_from_emitter = (PLANET_CENTER.y - 2_500.0 - y).max(0.0);
    let emission_time = SCENE_2 + distance_from_emitter / 4_000.0;
    smoothstep((time - emission_time) / 2.0)
}

fn world_grid_point(lane: f32, y: f32, time: f32) -> (Vec3, f32) {
    let angle = ((lane + 14.0) * 1_024.0 / 28.0) as i32;
    let circular_x = sine(angle + 256);
    let circular_y = sine(angle);
    let edge = circular_x.abs().max(circular_y.abs()).max(0.001);
    let square_x = circular_x / edge;
    let square_y = circular_y / edge;
    let reshaping = world_grid_shape(time, y);
    let section_x = circular_x + (square_x - circular_x) * reshaping;
    let section_y = circular_y + (square_y - circular_y) * reshaping;
    let floor = GRID_FLOOR_Z;
    let ceiling = grid_ceiling(y);
    let middle = (floor + ceiling) * 0.5;
    let half_height = (ceiling - floor) * 0.5;
    let circular_width = half_height / 0.61;
    let half_width = circular_width + (grid_half_width(y) - circular_width) * reshaping;
    let x = section_x * half_width;
    let mut z = middle - section_y * half_height;
    let bottom = smoothstep((section_y - 0.96) / 0.04);
    z += (grid_floor_height(x, y) - floor) * bottom;
    (Vec3 { x, y, z }, bottom)
}

fn sphere_roots(origin: Vec3, direction: Vec3, radius: f32) -> Option<(f32, f32)> {
    let local = vec_sub(origin, PLANET_CENTER);
    let b = vec_dot(local, direction);
    let c = vec_dot(local, local) - radius * radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }
    let root = fast_sqrt(discriminant);
    let near = -b - root;
    let far = -b + root;
    if far <= 0.0 {
        None
    } else {
        Some((near.max(0.0), far))
    }
}

fn grid_point_in_front_of_planet(camera: &CameraPose, point: Vec3) -> bool {
    let relative = vec_sub(point, camera.position);
    let distance = vec_length(relative);
    let direction = vec_scale(relative, 1.0 / distance.max(0.001));
    let Some((planet_distance, _)) = sphere_roots(camera.position, direction, PLANET_RADIUS)
    else {
        return true;
    };
    distance <= planet_distance + 3.0
}

fn clip_world_segment_to_planet(
    camera: &CameraPose,
    a: Vec3,
    b: Vec3,
) -> Option<(Vec3, Vec3)> {
    let a_front = grid_point_in_front_of_planet(camera, a);
    let b_front = grid_point_in_front_of_planet(camera, b);
    if a_front && b_front {
        return Some((a, b));
    }
    if !a_front && !b_front {
        return None;
    }
    let (mut front, mut behind) = if a_front { (a, b) } else { (b, a) };
    let mut step = 0usize;
    while step < 10 {
        let middle = vec_scale(vec_add(front, behind), 0.5);
        if grid_point_in_front_of_planet(camera, middle) {
            front = middle;
        } else {
            behind = middle;
        }
        step += 1;
    }
    if a_front {
        Some((a, front))
    } else {
        Some((front, b))
    }
}

fn draw_foreground_grid_segment(
    camera: &CameraPose,
    a: Vec3,
    b: Vec3,
    color: (u8, u8, u8),
    light: f32,
    x_scale: f32,
) {
    let Some((visible_a, visible_b)) = clip_world_segment_to_planet(camera, a, b) else {
        return;
    };
    let Some(screen_a) = project_world(camera, visible_a, x_scale) else {
        return;
    };
    let Some(screen_b) = project_world(camera, visible_b, x_scale) else {
        return;
    };
    if (screen_a.0 < -8.0 && screen_b.0 < -8.0)
        || (screen_a.0 > WIDTH as f32 + 8.0 && screen_b.0 > WIDTH as f32 + 8.0)
        || (screen_a.1 < -8.0 && screen_b.1 < -8.0)
        || (screen_a.1 > HEIGHT as f32 + 8.0 && screen_b.1 > HEIGHT as f32 + 8.0)
    {
        return;
    }
    let distance = ((screen_a.2 + screen_b.2) * 0.5 / 17_000.0).clamp(0.0, 1.0);
    let fog = (1.0 - distance * distance).clamp(0.26, 1.0);
    let brightness = light * fog;
    let lit = (
        (color.0 as f32 * brightness).min(255.0) as u8,
        (color.1 as f32 * brightness).min(255.0) as u8,
        (color.2 as f32 * brightness).min(255.0) as u8,
    );
    let alpha = ((44.0 + fog * 174.0) * light.clamp(0.0, 1.0)) as u8;
    draw_streak_line(
        core::ptr::addr_of_mut!(FRAME).cast::<u8>(),
        (screen_a.0, screen_a.1),
        (screen_b.0, screen_b.1),
        lit,
        alpha,
        (alpha as f32 * 0.28) as u8,
    );
}

fn surface_color(
    _point: Vec3,
    normal: Vec3,
    time: f32,
    distance: f32,
    altitude: f32,
) -> ((u8, u8, u8), f32) {
    let map = planet_map_from_normal(normal);
    let raw = terrain_smooth(map.0, map.1);
    let height = raw.max(WATER as f32);
    let valley_distance = wrapped_delta(map.1, valley_y(map.0)).abs();
    let detail = smoothstep((190.0 - altitude) / 120.0)
        * smoothstep((time - 100.0) / 13.0);
    let detailed_land = terrain_color_smooth(height);
    let broad_elevation = ((raw - 100.0) * 0.11).clamp(-9.0, 14.0);
    let biome = sine_sample(map.0 * 0.73 - map.1 * 0.46 + 590.0) * 0.5 + 0.5;
    let arid = smoothstep(
        (sine_sample(normal.x * 510.0 - normal.y * 370.0 + normal.z * 290.0 + 271.0)
            - 0.02)
            / 0.78,
    );
    let orbital_texture = sine_sample(
        normal.x * 1_170.0 + normal.y * 830.0 - normal.z * 610.0 + 443.0,
    ) * 0.62
        + sine_sample(
            normal.x * 2_410.0 - normal.y * 1_370.0 + normal.z * 1_910.0 + 53.0,
        ) * 0.38;
    let uplands = smoothstep((orbital_texture - 0.18) / 0.66);
    let polar = smoothstep((normal.x.abs() - 0.72) / 0.22);
    let orbital_land = (
        (36.0 + biome * 24.0 + arid * 91.0 + orbital_texture * 22.0 + uplands * 24.0
            + broad_elevation + polar * 42.0) as u8,
        (108.0 + biome * 17.0 - arid * 49.0 + orbital_texture * 28.0 - uplands * 11.0
            + broad_elevation * 1.25 + polar * 40.0) as u8,
        (52.0 - biome * 13.0 - arid * 9.0 + orbital_texture * 12.0 + uplands * 5.0
            + broad_elevation * 0.5 + polar * 58.0) as u8,
    );
    let land = (
        (orbital_land.0 as f32
            + (detailed_land.0 as f32 - orbital_land.0 as f32) * detail) as u8,
        (orbital_land.1 as f32
            + (detailed_land.1 as f32 - orbital_land.1 as f32) * detail) as u8,
        (orbital_land.2 as f32
            + (detailed_land.2 as f32 - orbital_land.2 as f32) * detail) as u8,
    );
    let landing_cap = vec_dot(normal, landing_up()) - 0.46;
    let second_cap = vec_dot(normal, Vec3 { x: 0.72, y: 0.42, z: -0.55 }) - 0.57;
    let third_cap = vec_dot(normal, Vec3 { x: -0.68, y: 0.15, z: -0.72 }) - 0.60;
    let continent_core = landing_cap.max(second_cap).max(third_cap);
    let continent_fractal = sine_sample(
        normal.x * 430.0 + normal.y * 270.0 + normal.z * 190.0 + 50.0,
    ) * 0.12
        + sine_sample(
            normal.x * 910.0 - normal.y * 530.0 + normal.z * 370.0 + 400.0,
        ) * 0.065
        + sine_sample(
            normal.x * 1_570.0 + normal.y * 1_130.0 - normal.z * 790.0 + 811.0,
        ) * 0.035;
    let continent = smoothstep((continent_core + continent_fractal) / 0.13);
    let ocean_depth = ((raw - 42.0) / 90.0).clamp(0.0, 1.0);
    let ocean = (
        (7.0 + ocean_depth * 10.0) as u8,
        (27.0 + ocean_depth * 35.0) as u8,
        (61.0 + ocean_depth * 54.0) as u8,
    );
    let coast = (
        (ocean.0 as f32 + (land.0 as f32 - ocean.0 as f32) * continent) as u8,
        (ocean.1 as f32 + (land.1 as f32 - ocean.1 as f32) * continent) as u8,
        (ocean.2 as f32 + (land.2 as f32 - ocean.2 as f32) * continent) as u8,
    );
    let coast_band = smoothstep((0.045 - (continent - 0.50).abs()) / 0.045);
    let river_presence = detail * smoothstep((continent - 0.18) / 0.35);
    let base = if valley_distance < 1.35 && river_presence > 0.5 {
        (17, 69, 105)
    } else if valley_distance < 2.8 && river_presence > 0.0 {
        color_ramp(
            (
                (coast.0 as f32 + (17.0 - coast.0 as f32) * river_presence) as i32,
                (coast.1 as f32 + (69.0 - coast.1 as f32) * river_presence) as i32,
                (coast.2 as f32 + (105.0 - coast.2 as f32) * river_presence) as i32,
            ),
            (coast.0 as i32, coast.1 as i32, coast.2 as i32),
            ((valley_distance - 1.35) * 10.0) as i32,
            15,
        )
    } else {
        coast
    };
    let terrain_light = light_smooth(map.0, map.1);
    let sun = vec_normalize(SUN_DIRECTION);
    let radial_light = vec_dot(normal, sun).max(0.0);
    let relief_light = 1.0 + (terrain_light - 128.0) / 68.0 * detail;
    let illumination = (0.38 + detail * 0.13 + radial_light * 0.72) * relief_light;
    let cloud_noise = sine_sample(
        normal.x * 390.0 + normal.y * 570.0 + normal.z * 240.0 - time * 3.2 + 113.0,
    ) * 0.58
        + sine_sample(
            normal.x * 760.0 - normal.y * 310.0 + normal.z * 610.0 + time * 1.7 + 733.0,
        ) * 0.31
        + sine_sample(
            normal.x * 1_330.0 + normal.y * 870.0 - normal.z * 460.0 - time * 2.4,
        ) * 0.15;
    let cloud = smoothstep((cloud_noise - 0.58) / 0.36)
        * (0.15 + (1.0 - detail) * 0.07);
    let fog = smoothstep((distance - 4_000.0) / 16_000.0) * 0.18;
    let wall = smoothstep((valley_distance - 5.0) / 27.0) * detail;
    let strata = sine_sample(height * 19.0 + map.0 * 11.0 + map.1 * 2.0 + 317.0)
        * 0.56
        + sine_sample(height * 37.0 - map.0 * 7.0 + map.1 * 5.0 + 733.0) * 0.28;
    let crags = smoothstep((strata - 0.05) / 0.72) * wall;
    let fracture_field = sine_sample(map.0 * 7.0 + map.1 * 3.0 + 211.0) * 0.70
        + sine_sample(map.0 * 15.0 - map.1 * 8.0 + 601.0) * 0.30;
    let fractures = smoothstep((0.17 - fracture_field.abs()) / 0.17) * wall;
    let ridge_light = smoothstep(
        (sine_sample(map.0 * 10.0 + map.1 * 2.0 + 857.0) - 0.06) / 0.76,
    ) * wall;
    let vegetation = (sine_sample(map.0 * 24.0 - map.1 * 17.0 + 193.0) * 0.5 + 0.5)
        * detail
        * (1.0 - wall * 0.72)
        * continent;
    let red = base.0 as f32 * illumination + coast_band * 18.0
        + wall * 20.0 + crags * 28.0
        + ridge_light * 18.0 - fractures * 24.0 - vegetation * 7.0;
    let green = base.1 as f32 * illumination + coast_band * 16.0
        + wall * 8.0 + crags * 14.0
        + ridge_light * 13.0 - fractures * 19.0 + vegetation * 15.0;
    let blue = base.2 as f32 * illumination + coast_band * 10.0
        + wall * 5.0 + crags * 9.0
        + ridge_light * 8.0 - fractures * 14.0 - vegetation * 4.0;
    (
        (
            (red + (212.0 - red) * cloud + (17.0 - red) * fog).clamp(0.0, 255.0) as u8,
            (green + (222.0 - green) * cloud + (20.0 - green) * fog).clamp(0.0, 255.0) as u8,
            (blue + (232.0 - blue) * cloud + (42.0 - blue) * fog).clamp(0.0, 255.0) as u8,
        ),
        (1.0 - continent) * (1.0 - cloud),
    )
}

fn render_world_surface(camera: &CameraPose, time: f32, x_scale: f32) -> f32 {
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let shell_radius = PLANET_RADIUS + 116.0;
    let distance_visibility = smoothstep((22_000.0 - camera.altitude) / 4_000.0);
    if distance_visibility <= 0.0 {
        return 0.0;
    }
    let atmosphere_amount = smoothstep((260.0 - camera.altitude) / 210.0);
    let relief_amount = relief_visibility(time, camera.altitude);
    let aurora_amount = smoothstep((6_000.0 - camera.altitude) / 4_500.0);
    let entry_heat = smoothstep((175.0 - camera.altitude) / 95.0)
        * smoothstep((camera.altitude - 14.0) / 32.0);
    let mut y = 0usize;
    while y < HEIGHT {
        let vertical = (100.0 - (y as f32 + 0.5)) / 154.0;
        let mut x = 0usize;
        while x < WIDTH {
            let horizontal = ((x as f32 + 0.5) - 160.0) / (154.0 * x_scale);
            let direction = vec_normalize(vec_add(
                camera.forward,
                vec_add(vec_scale(camera.right, horizontal), vec_scale(camera.up, vertical)),
            ));
            if let Some((mut distance, far)) = sphere_roots(camera.position, direction, shell_radius) {
                let mut hit = false;
                let mut point = vec_add(camera.position, vec_scale(direction, distance));
                let mut step = 0usize;
                let mut previous_distance = distance;
                let mut previous_signed = 0.0;
                let mut have_previous = false;
                let close_surface = camera.altitude < 120.0;
                let step_limit = if close_surface { 220 } else { 96 };
                while distance <= far && step < step_limit {
                    point = vec_add(camera.position, vec_scale(direction, distance));
                    let relative = vec_sub(point, PLANET_CENTER);
                    let radial_distance = vec_length(relative);
                    let normal = vec_scale(relative, 1.0 / radial_distance.max(0.001));
                    let signed = radial_distance
                        - PLANET_RADIUS
                        - planet_relief(normal) * relief_amount;
                    if signed <= 0.0 {
                        if have_previous {
                            let crossing = previous_signed
                                / (previous_signed - signed).max(0.000_1);
                            distance = previous_distance
                                + (distance - previous_distance) * crossing;
                            point = vec_add(camera.position, vec_scale(direction, distance));
                        }
                        hit = true;
                        break;
                    }
                    previous_distance = distance;
                    previous_signed = signed;
                    have_previous = true;
                    distance += if close_surface {
                        (signed * 0.24).clamp(0.25, 4.0)
                    } else {
                        (signed * 0.35).max(0.35)
                    };
                    step += 1;
                }
                if !hit {
                    if let Some((base_distance, _)) = sphere_roots(camera.position, direction, PLANET_RADIUS) {
                        distance = base_distance;
                        point = vec_add(camera.position, vec_scale(direction, distance));
                        hit = true;
                    }
                }
                if hit {
                    let relative = vec_sub(point, PLANET_CENTER);
                    let normal = vec_normalize(relative);
                    let (color, ocean) = surface_color(
                        point,
                        normal,
                        time,
                        distance,
                        camera.altitude,
                    );
                    let view = vec_scale(direction, -1.0);
                    let half_light = vec_normalize(vec_add(vec_normalize(SUN_DIRECTION), view));
                    let mut specular = vec_dot(normal, half_light).max(0.0);
                    specular *= specular;
                    specular *= specular;
                    specular *= specular;
                    specular *= specular;
                    specular *= specular;
                    specular *= ocean;
                    let grazing = (1.0 + vec_dot(direction, normal)).clamp(0.0, 1.0);
                    let aurora_wave = (sine((normal.x * 760.0 + normal.y * 530.0 + time * 58.0) as i32) * 0.5 + 0.5).clamp(0.0, 1.0);
                    let rim = smoothstep((grazing - 0.55) / 0.45);
                    let aurora = aurora_amount * rim * (0.025 + aurora_wave * 0.075);
                    let heat = entry_heat * rim * (0.20 + aurora_wave * 0.42);
                    blend_pixel(
                        frame,
                        x as i32,
                        y as i32,
                        (color.0 as f32 + specular * 62.0 + aurora * 24.0 + heat * 54.0)
                            .min(255.0) as u8,
                        (color.1 as f32 + specular * 72.0 + aurora * 148.0 + heat * 76.0)
                            .min(255.0) as u8,
                        (color.2 as f32 + specular * 88.0 + aurora * 112.0 + heat * 126.0)
                            .min(255.0) as u8,
                        (distance_visibility * 255.0) as u8,
                    );
                } else if atmosphere_amount > 0.0 {
                    let chord = ((far - distance).max(0.0) / 260.0).clamp(0.0, 1.0);
                    blend_pixel(frame, x as i32, y as i32, 52, 122, 218, (atmosphere_amount * chord * 72.0) as u8);
                }
            }
            x += 1;
        }
        y += 1;
    }
    entry_heat
}

fn draw_entry_sheath(time: f32, intensity: f32) {
    if intensity <= 0.0 {
        return;
    }
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let flow = (time * 310.0) as i32;
    let mut y = 0usize;
    while y < HEIGHT {
        let signed_y = (y as f32 - 100.0) / 100.0;
        let ny = signed_y.abs();
        let mut x = 0usize;
        while x < WIDTH {
            let signed_x = (x as f32 - 160.0) / 160.0;
            let nx = signed_x.abs();
            let edge = nx.max(ny);
            let envelope = smoothstep((edge - 0.43) / 0.57);
            if envelope > 0.0 {
                let warp = (sine(x as i32 * 5 - y as i32 * 7 + flow / 5 + 811)
                    * 118.0) as i32;
                let broad = sine(x as i32 * 4 + y as i32 * 2 - flow + warp) * 0.55
                    + sine(x as i32 * 9 - y as i32 * 3 - flow * 7 / 5 - warp / 2 + 313)
                        * 0.30
                    + sine(x as i32 * 17 + y as i32 * 11 - flow * 2 + 719) * 0.15;
                let wisp = smoothstep((broad - 0.06) / 0.62);
                let second_field = sine(x as i32 * 7 - y as i32 * 4 - flow * 3 / 2 - warp / 3 + 157)
                    * 0.68
                    + sine(x as i32 * 3 + y as i32 * 6 - flow + 557) * 0.32;
                let filament_a = smoothstep((0.15 - broad.abs()) / 0.15);
                let filament_b = smoothstep((0.11 - second_field.abs()) / 0.11);
                let breakup = smoothstep(
                    (sine(x as i32 * 3 + y as i32 * 2 - flow / 4 + 911) + 0.35)
                        / 0.78,
                );
                let filament = filament_a.max(filament_b * 0.76) * breakup;
                let hue = sine(x as i32 * 2 + y as i32 - flow / 7 + 617) * 0.5 + 0.5;
                let alpha = intensity
                    * envelope
                    * envelope
                    * (2.0 + wisp * 20.0 + filament * 72.0);
                blend_pixel(
                    frame,
                    x as i32,
                    y as i32,
                    (58.0 + hue * 112.0 + filament * 42.0) as u8,
                    (116.0 + hue * 92.0 + filament * 24.0) as u8,
                    255,
                    alpha.min(78.0) as u8,
                );
            }
            x += 1;
        }
        y += 1;
    }

}

fn draw_world_sun(camera: &CameraPose, amount: f32, x_scale: f32) {
    let direction = vec_normalize(SUN_DIRECTION);
    let depth = vec_dot(direction, camera.forward);
    if depth <= 0.08 {
        return;
    }
    let x = 160.0 + vec_dot(direction, camera.right) / depth * 154.0 * x_scale;
    let y = 100.0 - vec_dot(direction, camera.up) / depth * 154.0;
    draw_sun(x, y, amount);
}

fn render_foreground_well(camera: &CameraPose, time: f32, x_scale: f32) {
    let atmosphere_transmission = 1.0
        - smoothstep((260.0 - camera.altitude) / 190.0) * 0.94;
    let lane_step = 28.0 / (GRID_LANES - 1) as f32;
    let start = PLANET_CENTER.y - 2_700.0;
    let end = PLANET_CENTER.y + PLANET_RADIUS;
    let mut lane_index = 0usize;
    while lane_index < GRID_LANES {
        let lane = -14.0 + lane_index as f32 * lane_step;
        let (_, bottom) = world_grid_point(lane, PLANET_CENTER.y, time);
        if bottom > 0.5 {
            let c = accent_color(mesh_accent(lane));
            let color = (c.0 as u8, c.1 as u8, c.2 as u8);
            let mut y = start;
            while y < end {
                let next = (y + 260.0).min(end);
                let a = world_grid_point(lane, y, time).0;
                let b = world_grid_point(lane, next, time).0;
                let light = 0.62
                    + (sine((y * 0.08 - time * 48.0) as i32) * 0.5 + 0.5) * 0.38;
                draw_foreground_grid_segment(
                    camera,
                    a,
                    b,
                    color,
                    light * atmosphere_transmission,
                    x_scale,
                );
                y = next;
            }
        }
        lane_index += 1;
    }

    let mut y = start;
    while y <= end {
        let mut lane_index = 0usize;
        while lane_index + 1 < GRID_LANES {
            let lane_a = -14.0 + lane_index as f32 * lane_step;
            let lane_b = lane_a + lane_step;
            let (a, bottom_a) = world_grid_point(lane_a, y, time);
            let (b, bottom_b) = world_grid_point(lane_b, y, time);
            if bottom_a.max(bottom_b) > 0.5 {
                let c = accent_color(mesh_accent((lane_a + lane_b) * 0.5));
                draw_foreground_grid_segment(
                    camera,
                    a,
                    b,
                    (c.0 as u8, c.1 as u8, c.2 as u8),
                    0.78 * atmosphere_transmission,
                    x_scale,
                );
            }
            lane_index += 1;
        }
        y += 260.0;
    }
}

fn fill_atmospheric_sky(camera: &CameraPose) {
    let amount = smoothstep((260.0 - camera.altitude) / 190.0);
    if amount <= 0.0 {
        return;
    }
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let mut y = 0usize;
    while y < HEIGHT {
        let height = 1.0 - y as f32 / HEIGHT as f32;
        let red = 17.0 + height * 24.0;
        let green = 48.0 + height * 55.0;
        let blue = 88.0 + height * 89.0;
        let alpha = amount * (188.0 + height * 42.0);
        let mut x = 0usize;
        while x < WIDTH {
            blend_pixel(frame, x as i32, y as i32, red as u8, green as u8, blue as u8, alpha as u8);
            x += 1;
        }
        y += 1;
    }
}

fn render_world_journey(elapsed: f32, x_scale: f32) {
    let camera = journey_camera(elapsed);
    fill_atmospheric_sky(&camera);
    draw_world_sun(&camera, 1.0, x_scale);
    let entry_heat = render_world_surface(&camera, elapsed, x_scale);
    if camera.altitude < 22_000.0 {
        render_foreground_well(&camera, elapsed, x_scale);
    }
    draw_entry_sheath(elapsed, entry_heat);
}

fn star_space_color(x: usize, y: usize, time: f32) -> (u8, u8, u8) {
    let drift = (time * 1.7) as i32;
    let vertical = (y as f32 - 100.0).abs() / 100.0;
    let horizontal = (x as f32 - 160.0).abs() / 160.0;
    let diagonal = y as f32 - 93.0
        + (x as f32 - 160.0) * 0.19
        + sine(x as i32 * 2 + drift) * 11.0;
    let band = (1.0 - diagonal.abs() / 78.0).clamp(0.0, 1.0);
    let cloud = sine(x as i32 * 3 + y as i32 * 2 + 117 + drift)
        + sine(x as i32 + y as i32 * 5 + 631 - drift) * 0.52
        + sine(x as i32 * 7 - y as i32 * 3 + 349) * 0.24;
    let dust = (band * band * (0.64 + cloud * 0.22)).max(0.0);
    let vignette = (1.0 - horizontal * 0.31 - vertical * 0.24).clamp(0.42, 1.0);
    (
        ((2.0 + dust * 15.0) * vignette) as u8,
        ((3.0 + dust * (10.0 + band * 4.0)) * vignette) as u8,
        ((12.0 + dust * 29.0) * vignette) as u8,
    )
}

fn fill_star_space(time: f32) {
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let mut y = 0usize;
    while y < HEIGHT {
        let mut x = 0usize;
        while x < WIDTH {
            let (r, g, b) = star_space_color(x, y, time);
            put_pixel(frame, x, y, r, g, b);
            x += 1;
        }
        y += 1;
    }
}

fn accent_color(index: i32) -> (f32, f32, f32) {
    match index & 7 {
        0 => (34.0, 142.0, 255.0),
        1 => (34.0, 224.0, 244.0),
        2 => (42.0, 238.0, 154.0),
        3 => (236.0, 232.0, 52.0),
        4 => (255.0, 166.0, 35.0),
        5 => (255.0, 75.0, 76.0),
        6 => (255.0, 65.0, 190.0),
        _ => (166.0, 78.0, 255.0),
    }
}

fn stellar_temperature(index: i32) -> (f32, f32, f32) {
    match hash(index, 997) & 15 {
        0 => (1.0, 0.68, 0.63),
        1 | 2 => (1.0, 0.84, 0.61),
        3 | 4 | 5 | 6 => (1.0, 0.94, 0.82),
        7 | 8 | 9 | 10 | 11 => (0.91, 0.97, 1.0),
        12 | 13 | 14 => (0.67, 0.83, 1.0),
        _ => (0.79, 0.69, 1.0),
    }
}

fn mesh_accent(lane: f32) -> i32 {
    (((lane + 14.0) * 8.0 / 28.0) as i32).clamp(0, 7)
}

fn square_corner_strength(lane: f32) -> f32 {
    let angle = ((lane + 14.0) * 1_024.0 / 28.0) as i32;
    let diagonal = (sine(angle).abs() - sine(angle + 256).abs()).abs();
    smoothstep((0.18 - diagonal) / 0.18)
}

fn line_coordinates(index: usize) -> (f32, f32, f32, f32) {
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

fn cone_direction(lane: f32) -> (f32, f32) {
    let angle = ((lane + 14.0) * 1_024.0 / 28.0) as i32;
    (sine(angle + 256), sine(angle) * 0.61)
}

fn line_ring(index: usize) -> usize {
    if index < SPOKE_LINES {
        index % GRID_RINGS
    } else {
        (index - SPOKE_LINES) / (GRID_LANES - 1)
    }
}

fn flyby_segment(
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

const fn formation_gap(emission: usize) -> f32 {
    let mut excess = FORMATION_EXCESS;
    let mut step = 0usize;
    while step < emission {
        excess *= 0.84;
        step += 1;
    }
    RING_INTERVAL + excess
}

const fn ring_birth(ring: usize) -> f32 {
    let mut birth = RING_BIRTH;
    let mut emission = 0usize;
    while emission < ring {
        birth += formation_gap(emission);
        emission += 1;
    }
    birth
}

fn ring_state(ring: usize, time: f32) -> (f32, f32) {
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

fn regular_spoke_state(ring: usize, time: f32) -> (f32, f32) {
    let current_position = travel_row(time);
    let regular_position = travel_row(CLEAN_BIRTH) + ring as f32;
    let relative_cycle = (current_position - regular_position) / GRID_RINGS as f32;
    let mut cycle = relative_cycle as i32;
    if relative_cycle < cycle as f32 {
        cycle -= 1;
    }
    let birth_position = regular_position + cycle as f32 * GRID_RINGS as f32;
    (current_position - birth_position, birth_position)
}

fn outward_row(time: f32, birth: f32) -> f32 {
    travel_row(time) - travel_row(birth)
}

fn travel_row(time: f32) -> f32 {
    if time <= CLEAN_BIRTH {
        return time * RING_SPEED;
    }
    let duration = PLANET_REVEAL - CLEAN_BIRTH;
    let reduction = 1.0 - APPROACH_SPEED;
    let elapsed = time - CLEAN_BIRTH;
    let before = CLEAN_BIRTH * RING_SPEED;
    if elapsed < duration {
        let progress = elapsed / duration;
        let p2 = progress * progress;
        let p3 = p2 * progress;
        let p4 = p3 * progress;
        let p5 = p4 * progress;
        let pulled_integral = p3 - p4 * 0.5 + 3.0 * (p3 / 3.0 - p4 * 0.5 + p5 * 0.2);
        before + duration * RING_SPEED * (progress - reduction * pulled_integral)
    } else {
        let transition = duration * RING_SPEED * (1.0 - reduction * 0.6);
        before + transition + (elapsed - duration) * RING_SPEED * APPROACH_SPEED
    }
}

fn birth_shape(birth_position: f32) -> f32 {
    smoothstep(
        (birth_position - travel_row(SCENE_2))
            / (travel_row(SCENE_3) - travel_row(SCENE_2)),
    )
}

fn birth_neon(birth_position: f32) -> f32 {
    smoothstep(
        (birth_position - travel_row(SCENE_2))
            / (travel_row(GRID_COLOR_FULL) - travel_row(SCENE_2)),
    )
}

fn line_birth(index: usize) -> f32 {
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

fn lit_mesh_color(index: i32, accent: i32, neon: f32, brightness: f32) -> (u8, u8, u8) {
    let temperature = stellar_temperature(index);
    let target = accent_color(accent);
    let star_r = brightness * temperature.0;
    let star_g = brightness * temperature.1;
    let star_b = brightness * temperature.2;
    let spectral_light = (0.20 + brightness / 255.0 * 0.80).clamp(0.20, 1.0);
    (
        (star_r + (target.0 * spectral_light - star_r) * neon).clamp(0.0, 255.0) as u8,
        (star_g + (target.1 * spectral_light - star_g) * neon).clamp(0.0, 255.0) as u8,
        (star_b + (target.2 * spectral_light - star_b) * neon).clamp(0.0, 255.0) as u8,
    )
}

fn mesh_brightness(
    time: f32,
    lane: f32,
    depth: f32,
    distance: f32,
    luminosity: f32,
    reshaping: f32,
) -> f32 {
    let angle = ((lane + 14.0) * 1_024.0 / 28.0) as i32;
    let key = (sine(angle - (time * 19.0) as i32) * 0.5 + 0.5).clamp(0.0, 1.0);
    let diffuse = 0.48 + key * 0.52;
    let mut specular = key * key;
    specular *= specular;
    specular *= specular;
    let traveling_glint =
        (sine((depth * 57.0 - time * 23.0) as i32 + angle / 3) * 0.5 + 0.5).clamp(0.0, 1.0);
    let source = 42.0 + distance * 136.0 + luminosity * luminosity * 54.0;
    let flat = source * diffuse
        + specular * 108.0
        + traveling_glint * 18.0;
    let mut angular_a =
        (sine(angle - (time * 37.0) as i32) * 0.5 + 0.5).clamp(0.0, 1.0);
    angular_a *= angular_a;
    angular_a *= angular_a;
    let mut depth_a =
        (sine((depth * 43.0 - time * 51.0) as i32 + 211) * 0.5 + 0.5)
            .clamp(0.0, 1.0);
    depth_a *= depth_a;
    let mut angular_b =
        (sine(angle * 2 + (time * 29.0) as i32 + 607) * 0.5 + 0.5)
            .clamp(0.0, 1.0);
    angular_b *= angular_b;
    angular_b *= angular_b;
    let depth_b =
        (sine((depth * 31.0 + time * 39.0) as i32 + 877) * 0.5 + 0.5)
            .clamp(0.0, 1.0);
    let pool_a = angular_a * depth_a;
    let pool_b = angular_b * depth_b * depth_b;
    let dramatic = source * (0.22 + key * 0.18 + pool_a * 0.78 + pool_b * 0.52)
        + specular * 96.0
        + traveling_glint * 14.0;
    (flat + (dramatic - flat) * reshaping).clamp(20.0, 255.0)
}

fn draw_streak_line(
    frame: *mut u8,
    start: (f32, f32),
    end: (f32, f32),
    color: (u8, u8, u8),
    alpha: u8,
    glow: u8,
) {
    let (direction_x, direction_y) = normalize(end.0 - start.0, end.1 - start.1);
    let offset_x = -direction_y * 1.2;
    let offset_y = direction_x * 1.2;
    draw_line(
        frame,
        (start.0 + offset_x) as i32,
        (start.1 + offset_y) as i32,
        (end.0 + offset_x) as i32,
        (end.1 + offset_y) as i32,
        color.0,
        color.1,
        color.2,
        glow,
    );
    draw_line(
        frame,
        (start.0 - offset_x) as i32,
        (start.1 - offset_y) as i32,
        (end.0 - offset_x) as i32,
        (end.1 - offset_y) as i32,
        color.0,
        color.1,
        color.2,
        glow,
    );
    draw_line(
        frame,
        start.0 as i32,
        start.1 as i32,
        end.0 as i32,
        end.1 as i32,
        color.0,
        color.1,
        color.2,
        alpha,
    );
}

fn draw_star_sparkle(
    frame: *mut u8,
    center: (f32, f32),
    color: (u8, u8, u8),
    index: i32,
    time: f32,
) {
    if hash(index, 1_201) < 249 {
        return;
    }
    let settling = 1.0 - smoothstep((time - STARFIELD_HOLD - 1.0) / 5.0) * 0.65;
    let mut pulse = (sine((time * 690.0) as i32 + hash(index, 1_819) * 4) * 0.5 + 0.5)
        .clamp(0.0, 1.0);
    pulse *= pulse;
    pulse *= pulse;
    let intensity = pulse * settling;
    if intensity < 0.16 {
        return;
    }
    let arm = 1 + (intensity * 4.0) as i32;
    let x = center.0 as i32;
    let y = center.1 as i32;
    let halo = (intensity * 92.0) as u8;
    let core = (intensity * 235.0) as u8;
    draw_line(
        frame,
        x - arm,
        y,
        x + arm,
        y,
        color.0,
        color.1,
        color.2,
        halo,
    );
    draw_line(
        frame,
        x,
        y - arm,
        x,
        y + arm,
        color.0,
        color.1,
        color.2,
        halo,
    );
    blend_pixel(frame, x, y, 255, 255, 255, core);
}

fn draw_bloom_line(
    frame: *mut u8,
    start: (f32, f32),
    end: (f32, f32),
    color: (u8, u8, u8),
    alpha: u8,
    glow: u8,
) {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let length_squared = (dx * dx + dy * dy).max(0.001);
    let left = ((start.0.min(end.0) - 3.0) as i32 - 1).max(0);
    let right = ((start.0.max(end.0) + 3.0) as i32 + 1).min(WIDTH as i32 - 1);
    let top = ((start.1.min(end.1) - 3.0) as i32 - 1).max(0);
    let bottom = ((start.1.max(end.1) + 3.0) as i32 + 1).min(HEIGHT as i32 - 1);
    let mut y = top;
    while y <= bottom {
        let sample_y = y as f32 + 0.5;
        let mut x = left;
        while x <= right {
            let sample_x = x as f32 + 0.5;
            let projection = (((sample_x - start.0) * dx + (sample_y - start.1) * dy)
                / length_squared)
                .clamp(0.0, 1.0);
            let nearest_x = start.0 + dx * projection;
            let nearest_y = start.1 + dy * projection;
            let pixel_dx = sample_x - nearest_x;
            let pixel_dy = sample_y - nearest_y;
            let distance_squared = pixel_dx * pixel_dx + pixel_dy * pixel_dy;
            if distance_squared < 9.0 {
                let core = ((2.25 - distance_squared) / 2.0).clamp(0.0, 1.0);
                let halo = ((9.0 - distance_squared) / 9.0).clamp(0.0, 1.0);
                let coverage = (alpha as f32 * core
                    + glow as f32 * halo * halo * (1.0 - core))
                    .min(255.0);
                let hot = core * core * 0.34;
                let red = color.0 as f32 + (255.0 - color.0 as f32) * hot;
                let green = color.1 as f32 + (255.0 - color.1 as f32) * hot;
                let blue = color.2 as f32 + (255.0 - color.2 as f32) * hot;
                blend_pixel(
                    frame,
                    x,
                    y,
                    red as u8,
                    green as u8,
                    blue as u8,
                    coverage as u8,
                );
            }
            x += 1;
        }
        y += 1;
    }
}

fn render_continuous_grid(time: f32) {
    render_continuous_grid_warped(time, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
}

fn render_continuous_grid_warped(
    time: f32,
    neon: f32,
    gravity: f32,
    well_x: f32,
    well_y: f32,
    well_radius: f32,
    view_turn: f32,
    grid_visibility: f32,
) {
    fill_star_space(time);
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let clean_birth = CLEAN_BIRTH;

    let mut index = 0usize;
    while index < STAR_LINES {
        let Some((a, b)) = flyby_segment(index, time, clean_birth) else {
            index += 1;
            continue;
        };
        if !((a.0 < -2.0 && b.0 < -2.0)
            || (a.0 > WIDTH as f32 + 2.0 && b.0 > WIDTH as f32 + 2.0)
            || (a.1 < -2.0 && b.1 < -2.0)
            || (a.1 > HEIGHT as f32 + 2.0 && b.1 > HEIGHT as f32 + 2.0))
        {
            let middle_x = (a.0 + b.0) * 0.5 - 160.0;
            let middle_y = (a.1 + b.1) * 0.5 - 100.0;
            let distance =
                (fast_sqrt(middle_x * middle_x + middle_y * middle_y) / 175.0)
                    .clamp(0.0, 1.0);
            let luminosity = hash(index as i32, 149) as f32 / 255.0;
            let brightness =
                (46.0 + distance * 145.0 + luminosity * luminosity * 64.0).min(255.0);
            let color = lit_mesh_color(index as i32, index as i32, neon, brightness);
            let alpha = (142.0 + luminosity * 92.0).min(255.0) as u8;
            let glow = (12.0 + luminosity * luminosity * 42.0 + neon * 28.0) as u8;
            draw_streak_line(frame, a, b, color, alpha, glow);
            blend_pixel(frame, b.0 as i32, b.1 as i32, color.0, color.1, color.2, alpha);
            draw_star_sparkle(
                frame,
                ((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5),
                color,
                index as i32,
                time,
            );
        }
        index += 1;
    }

    draw_clean_grid_field(time, clean_birth);
    draw_ordered_radials(
        time,
        clean_birth,
        gravity,
        well_x,
        well_y,
        well_radius,
        view_turn,
        grid_visibility,
    );

    index = SPOKE_LINES;
    while index < STAR_LINES {
        let birth = line_birth(index);
        if time >= birth {
            let (lane_a, _, lane_b, _) = line_coordinates(index);
            let ring = (index - SPOKE_LINES) / (GRID_LANES - 1);
            let (depth, current_birth) = ring_state(ring, time);
            let ring_shape = birth_shape(current_birth);
            let visible_depth = depth;
            let threshold_ring = current_birth < travel_row(clean_birth);
            if visible_depth <= GRID_FOG_START {
                index += 1;
                continue;
            }
            let distance_visibility = if threshold_ring {
                1.0
            } else {
                smoothstep(
                    (visible_depth - GRID_FOG_START) / (GRID_FOG_END - GRID_FOG_START),
                )
            };
            let a = rotate_grid_point(
                warp_grid_point(
                    grid_depth_point(lane_a, depth, ring_shape),
                    gravity,
                    well_x,
                    well_y,
                    well_radius,
                ),
                view_turn,
                well_x,
                well_y,
            );
            let b = rotate_grid_point(
                warp_grid_point(
                    grid_depth_point(lane_b, depth, ring_shape),
                    gravity,
                    well_x,
                    well_y,
                    well_radius,
                ),
                view_turn,
                well_x,
                well_y,
            );
            let middle_x = (a.0 + b.0) * 0.5 - 160.0;
            let middle_y = (a.1 + b.1) * 0.5 - 100.0;
            let distance = (fast_sqrt(middle_x * middle_x + middle_y * middle_y) / 175.0)
                .clamp(0.0, 1.0);
            let luminosity = hash(index as i32, 149) as f32 / 255.0;
            let brightness = mesh_brightness(
                time,
                (lane_a + lane_b) * 0.5,
                visible_depth,
                distance,
                luminosity,
                ring_shape,
            );
            let ring_neon = birth_neon(current_birth);
            let lane_middle = (lane_a + lane_b) * 0.5;
            let color = lit_mesh_color(
                index as i32,
                mesh_accent(lane_middle),
                ring_neon,
                brightness,
            );
            let alpha = ((154.0 + luminosity * 78.0).min(238.0)
                * distance_visibility
                * grid_visibility) as u8;
            let glow = ((10.0 + luminosity * luminosity * 30.0 + ring_neon * 12.0)
                * distance_visibility
                * grid_visibility) as u8;
            draw_bloom_line(frame, a, b, color, alpha, glow);
            blend_pixel(frame, b.0 as i32, b.1 as i32, color.0, color.1, color.2, alpha);
        }
        index += 1;
    }
}

fn draw_ordered_radials(
    time: f32,
    clean_birth: f32,
    gravity: f32,
    well_x: f32,
    well_y: f32,
    well_radius: f32,
    view_turn: f32,
    grid_visibility: f32,
) {
    let introduction = STAR_ORGANIZE + 1.5;
    if time <= introduction {
        return;
    }
    let introduction_position = travel_row(introduction);
    let clean_position = travel_row(clean_birth);
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let mut lane_index = 0usize;
    while lane_index < GRID_LANES {
        let lane = -14.0 + lane_index as f32 * (28.0 / (GRID_LANES - 1) as f32);
        let mut ring = 0usize;
        while ring < GRID_RINGS {
            let next_ring = (ring + 1) % GRID_RINGS;
            let (mut outer, outer_birth) = regular_spoke_state(ring, time);
            let (mut inner, inner_birth) = regular_spoke_state(next_ring, time);
            if outer_birth > inner_birth {
                ring += 1;
                continue;
            }
            let prominence = smoothstep(
                (inner_birth - introduction_position)
                    / (clean_position - introduction_position),
            );
            if prominence <= 0.0 {
                ring += 1;
                continue;
            }
            let visible_depth = outer.max(inner);
            if visible_depth <= GRID_FOG_START {
                ring += 1;
                continue;
            }
            let mut outer_shape = birth_shape(outer_birth);
            let mut inner_shape = birth_shape(inner_birth);
            if outer > inner && inner < GRID_FOG_START {
                let span = (outer - inner).max(0.001);
                let clip = ((GRID_FOG_START - inner) / span).clamp(0.0, 1.0);
                inner += (outer - inner) * clip;
                inner_shape += (outer_shape - inner_shape) * clip;
            } else if inner > outer && outer < GRID_FOG_START {
                let span = (inner - outer).max(0.001);
                let clip = ((GRID_FOG_START - outer) / span).clamp(0.0, 1.0);
                outer += (inner - outer) * clip;
                outer_shape += (inner_shape - outer_shape) * clip;
            }
            let visibility = smoothstep(
                (visible_depth - GRID_FOG_START) / (GRID_FOG_END - GRID_FOG_START),
            ) * prominence;
            let a = rotate_grid_point(
                warp_grid_point(
                    grid_depth_point(lane, outer, outer_shape),
                    gravity,
                    well_x,
                    well_y,
                    well_radius,
                ),
                view_turn,
                well_x,
                well_y,
            );
            let b = rotate_grid_point(
                warp_grid_point(
                    grid_depth_point(lane, inner, inner_shape),
                    gravity,
                    well_x,
                    well_y,
                    well_radius,
                ),
                view_turn,
                well_x,
                well_y,
            );
            let index = lane_index * GRID_RINGS + ring;
            let middle_x = (a.0 + b.0) * 0.5 - 160.0;
            let middle_y = (a.1 + b.1) * 0.5 - 100.0;
            let distance = (fast_sqrt(middle_x * middle_x + middle_y * middle_y) / 175.0)
                .clamp(0.0, 1.0);
            let luminosity = hash(index as i32, 149) as f32 / 255.0;
            let brightness = mesh_brightness(
                time,
                lane,
                visible_depth,
                distance,
                luminosity,
                (outer_shape + inner_shape) * 0.5,
            );
            let spoke_neon = birth_neon(inner_birth);
            let seam = square_corner_strength(lane) * spoke_neon;
            let color = lit_mesh_color(
                index as i32,
                mesh_accent(lane),
                spoke_neon,
                brightness,
            );
            let alpha = ((154.0 + luminosity * 78.0 + seam * 17.0).min(250.0)
                * visibility
                * grid_visibility) as u8;
            let glow = ((10.0 + luminosity * luminosity * 30.0 + spoke_neon * 12.0
                + seam * 18.0)
                * visibility
                * grid_visibility) as u8;
            draw_bloom_line(frame, a, b, color, alpha, glow);
            ring += 1;
        }
        lane_index += 1;
    }
}

fn draw_clean_grid_field(time: f32, clean_birth: f32) {
    if time <= clean_birth {
        return;
    }
    let moving_row = outward_row(time, clean_birth);
    let right = grid_depth_point(-14.0, moving_row, 0.0);
    let bottom = grid_depth_point(-7.0, moving_row, 0.0);
    let radius_x = (right.0 - 160.0).max(0.001);
    let radius_y = (bottom.1 - 100.0).max(0.001);
    let feather = (3.0 / radius_x).max(3.0 / radius_y).min(1.0);
    let inner = (1.0 - feather).max(0.0);
    let inner_squared = inner * inner;
    let edge_span = (1.0 - inner_squared).max(0.001);
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let mut y = 0usize;
    while y < HEIGHT {
        let dy = (y as f32 - 100.0) / radius_y;
        let mut x = 0usize;
        while x < WIDTH {
            let dx = (x as f32 - 160.0) / radius_x;
            let distance_squared = dx * dx + dy * dy;
            if distance_squared < 1.0 {
                let alpha = if distance_squared <= inner_squared {
                    255
                } else {
                    ((1.0 - distance_squared) * 255.0 / edge_span) as u8
                };
                blend_pixel(frame, x as i32, y as i32, 0, 0, 0, alpha);
            }
            x += 1;
        }
        y += 1;
    }
}

fn warp_grid_point(
    point: (f32, f32),
    strength: f32,
    well_x: f32,
    well_y: f32,
    radius: f32,
) -> (f32, f32) {
    if strength <= 0.0 {
        return point;
    }
    let dx = well_x - point.0;
    let dy = point.1 - well_y;
    let radius_squared = radius * radius;
    let floor_weight = smoothstep((point.1 - 94.0) / 42.0);
    let influence = strength * floor_weight * radius_squared
        / (dx * dx + dy * dy * 1.6 + radius_squared);
    (
        point.0 + dx * influence * 0.36,
        point.1 + influence * radius * 0.82,
    )
}

fn rotate_grid_point(
    point: (f32, f32),
    turn: f32,
    center_x: f32,
    center_y: f32,
) -> (f32, f32) {
    if turn <= 0.0 {
        return point;
    }
    let angle = (turn * 1_024.0) as i32;
    let cosine = sine(angle + 256);
    let rotation_sine = sine(angle);
    let x = point.0 - center_x;
    let y = (point.1 - center_y) / 0.61;
    (
        center_x + x * cosine - y * rotation_sine,
        center_y + (x * rotation_sine + y * cosine) * 0.61,
    )
}

fn grid_depth_point(lane: f32, moving_row: f32, reshaping: f32) -> (f32, f32) {
    let amount = moving_row / 20.0;
    let angle = ((lane + 14.0) * 1_024.0 / 28.0) as i32;
    let circular_x = sine(angle + 256);
    let circular_y = sine(angle);
    let cone_radius = amount * amount * 260.0;
    let square_edge = circular_x.abs().max(circular_y.abs()).max(0.001);
    let square_x = circular_x / square_edge;
    let square_y = circular_y / square_edge;
    let section_x = circular_x + (square_x - circular_x) * reshaping;
    let section_y = circular_y + (square_y - circular_y) * reshaping;
    let side_push = 1.0 + reshaping * 3.2;
    (
        160.0 + section_x * cone_radius * side_push,
        100.0 + section_y * cone_radius * 0.61,
    )
}

fn fast_sqrt(value: f32) -> f32 {
    if value <= 0.0 {
        return 0.0;
    }
    let mut inverse = f32::from_bits(0x5f37_59df - (value.to_bits() >> 1));
    inverse *= 1.5 - value * 0.5 * inverse * inverse;
    value * inverse
}

fn draw_line(
    frame: *mut u8,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    r: u8,
    g: u8,
    b: u8,
    alpha: u8,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = dx.abs().max(dy.abs()).max(1);
    let mut step = 0i32;
    while step <= steps {
        let x = x0 + dx * step / steps;
        let y = y0 + dy * step / steps;
        blend_pixel(frame, x, y, r, g, b, alpha);
        step += 1;
    }
}

fn blend_pixel(frame: *mut u8, x: i32, y: i32, r: u8, g: u8, b: u8, alpha: u8) {
    if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
        return;
    }
    let offset = (y as usize * WIDTH + x as usize) * 3;
    let inverse = 255 - alpha as i32;
    unsafe {
        let blend = |old: u8, new: u8| ((old as i32 * inverse + new as i32 * alpha as i32) / 255) as u8;
        frame.add(offset).write(blend(frame.add(offset).read(), r));
        frame.add(offset + 1).write(blend(frame.add(offset + 1).read(), g));
        frame.add(offset + 2).write(blend(frame.add(offset + 2).read(), b));
    }
}

fn draw_sun(center_x: f32, center_y: f32, amount: f32) {
    if amount <= 0.0 {
        return;
    }
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let halo_radius = 27.0;
    let left = (center_x - halo_radius).max(0.0) as i32;
    let right = (center_x + halo_radius).min(WIDTH as f32 - 1.0) as i32;
    let top = (center_y - halo_radius).max(0.0) as i32;
    let bottom = (center_y + halo_radius).min(HEIGHT as f32 - 1.0) as i32;
    let mut y = top;
    while y <= bottom {
        let dy = y as f32 - center_y;
        let mut x = left;
        while x <= right {
            let dx = x as f32 - center_x;
            let distance = fast_sqrt(dx * dx + dy * dy);
            if distance < halo_radius {
                let halo = 1.0 - distance / halo_radius;
                let core = smoothstep((14.0 - distance) / 2.5);
                let alpha = amount * (halo * halo * 88.0 + core * 210.0);
                blend_pixel(
                    frame,
                    x,
                    y,
                    255,
                    (139.0 + core * 98.0) as u8,
                    (60.0 + core * 180.0) as u8,
                    alpha.min(255.0) as u8,
                );
            }
            x += 1;
        }
        y += 1;
    }
}

fn terrain_color_smooth(height: f32) -> (u8, u8, u8) {
    let ramp = |from: (f32, f32, f32), to: (f32, f32, f32), amount: f32| {
        (
            (from.0 + (to.0 - from.0) * amount) as u8,
            (from.1 + (to.1 - from.1) * amount) as u8,
            (from.2 + (to.2 - from.2) * amount) as u8,
        )
    };
    if height < 70.0 {
        return (18, 50, 88);
    }
    if height < 90.0 {
        return ramp((18.0, 50.0, 88.0), (130.0, 105.0, 58.0), (height - 70.0) / 20.0);
    }
    if height < 115.0 {
        return ramp((130.0, 105.0, 58.0), (45.0, 88.0, 42.0), (height - 90.0) / 25.0);
    }
    if height < 180.0 {
        return ramp((45.0, 88.0, 42.0), (112.0, 95.0, 88.0), (height - 115.0) / 65.0);
    }
    ramp(
        (112.0, 95.0, 88.0),
        (215.0, 218.0, 225.0),
        ((height - 180.0) / 45.0).clamp(0.0, 1.0),
    )
}

fn color_ramp(from: (i32, i32, i32), to: (i32, i32, i32), amount: i32, range: i32) -> (u8, u8, u8) {
    let blend = |a: i32, b: i32| (a + (b - a) * amount / range).clamp(0, 255) as u8;
    (blend(from.0, to.0), blend(from.1, to.1), blend(from.2, to.2))
}

fn put_pixel(frame: *mut u8, x: usize, y: usize, r: u8, g: u8, b: u8) {
    let offset = (y * WIDTH + x) * 3;
    unsafe {
        frame.add(offset).write(r);
        frame.add(offset + 1).write(g);
        frame.add(offset + 2).write(b);
    }
}

fn encode_frame(first: bool, columns: u16, rows: u16) -> usize {
    let source = core::ptr::addr_of!(FRAME).cast::<u8>();
    let output = core::ptr::addr_of_mut!(TX).cast::<u8>();
    let mut output_at = 0usize;
    let mut source_at = 0usize;
    let mut chunk_index = 0usize;
    append(output, &mut output_at, b"\x1b[?2026h");

    while source_at < FRAME_BYTES {
        let raw_len = (FRAME_BYTES - source_at).min(RAW_CHUNK);
        let more = source_at + raw_len < FRAME_BYTES;
        if chunk_index == 0 {
            if first {
                append(output, &mut output_at, b"\x1b_Ga=T,f=24,s=320,v=200,i=1,p=1,q=2,C=1,c=");
                append_number(output, &mut output_at, columns.max(1) as u32);
                append(output, &mut output_at, b",r=");
                append_number(output, &mut output_at, rows.max(1) as u32);
                append(output, &mut output_at, b",m=");
            } else {
                append(output, &mut output_at, b"\x1b_Ga=f,r=1,i=1,f=24,x=0,y=0,s=320,v=200,q=2,m=");
            }
        } else {
            if first {
                append(output, &mut output_at, b"\x1b_Gm=");
            } else {
                append(output, &mut output_at, b"\x1b_Ga=f,r=1,q=2,m=");
            }
        }
        push(output, &mut output_at, if more { b'1' } else { b'0' });
        push(output, &mut output_at, b';');
        encode_base64(source, source_at, raw_len, output, &mut output_at);
        append(output, &mut output_at, b"\x1b\\");
        source_at += raw_len;
        chunk_index += 1;
    }

    if !first {
        append(output, &mut output_at, b"\x1b_Ga=a,c=1,i=1,q=2\x1b\\");
    }
    append(output, &mut output_at, b"\x1b[?2026l");
    output_at
}

fn encode_base64(source: *const u8, start: usize, length: usize, output: *mut u8, at: &mut usize) {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut i = 0usize;
    while i < length {
        let a = unsafe { source.add(start + i).read() };
        let b = if i + 1 < length { unsafe { source.add(start + i + 1).read() } } else { 0 };
        let c = if i + 2 < length { unsafe { source.add(start + i + 2).read() } } else { 0 };
        push(output, at, ALPHABET[(a >> 2) as usize]);
        push(output, at, ALPHABET[(((a & 3) << 4) | (b >> 4)) as usize]);
        push(output, at, if i + 1 < length { ALPHABET[(((b & 15) << 2) | (c >> 6)) as usize] } else { b'=' });
        push(output, at, if i + 2 < length { ALPHABET[(c & 63) as usize] } else { b'=' });
        i += 3;
    }
}

fn append(output: *mut u8, at: &mut usize, bytes: &[u8]) {
    let mut i = 0;
    while i < bytes.len() {
        push(output, at, bytes[i]);
        i += 1;
    }
}

fn push(output: *mut u8, at: &mut usize, byte: u8) {
    unsafe { output.add(*at).write(byte) };
    *at += 1;
}

fn append_number(output: *mut u8, at: &mut usize, mut number: u32) {
    let mut digits = [0u8; 10];
    let mut count = 0usize;
    loop {
        digits[count] = b'0' + (number % 10) as u8;
        count += 1;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    while count > 0 {
        count -= 1;
        push(output, at, digits[count]);
    }
}

fn window_size() -> WinSize {
    let mut size = WinSize {
        rows: 25,
        columns: 80,
        pixel_width: 0,
        pixel_height: 0,
    };
    syscall3(SYS_IOCTL, 1, TIOCGWINSZ, addr_mut(&mut size));
    if size.rows == 0 {
        size.rows = 25;
    }
    if size.columns == 0 {
        size.columns = 80;
    }
    size
}

fn planet_x_scale(window: WinSize) -> f32 {
    let (physical_width, physical_height) = if window.pixel_width > 0 && window.pixel_height > 0 {
        (window.pixel_width as f32, window.pixel_height as f32)
    } else {
        (window.columns.max(1) as f32, window.rows.max(1) as f32 * 2.0)
    };
    (physical_height * WIDTH as f32 / (physical_width * HEIGHT as f32)).clamp(0.35, 3.0)
}

fn handle_input(elapsed: &mut f32) -> bool {
    loop {
        let mut byte = 0u8;
        if syscall3(SYS_READ, 0, addr_mut(&mut byte), 1) <= 0 {
            return false;
        }
        if byte == 27 {
            let mut prefix = 0u8;
            if syscall3(SYS_READ, 0, addr_mut(&mut prefix), 1) <= 0 {
                sleep_ns(1_000_000);
                if syscall3(SYS_READ, 0, addr_mut(&mut prefix), 1) <= 0 {
                    return true;
                }
            }
            if prefix != b'[' && prefix != b'O' {
                return true;
            }
            let mut direction = 0u8;
            if syscall3(SYS_READ, 0, addr_mut(&mut direction), 1) <= 0 {
                sleep_ns(1_000_000);
                if syscall3(SYS_READ, 0, addr_mut(&mut direction), 1) <= 0 {
                    return true;
                }
            }
            match direction {
                b'C' => *elapsed += 2.0,
                b'D' => *elapsed = (*elapsed - 2.0).max(0.0),
                _ => {}
            }
            continue;
        }
        if byte == b'q' || byte == b'Q' || byte == 3 {
            return true;
        }
        match byte {
            b'1' => *elapsed = 0.0,
            b'2' => *elapsed = SCENE_2,
            b'3' => *elapsed = SCENE_3,
            b'4' => *elapsed = SCENE_4,
            b'5' => *elapsed = SCENE_5,
            b'6' => *elapsed = DESCENT,
            b' ' => *elapsed += 5.0,
            b']' | b'.' | b'>' | b'f' | b'F' => *elapsed += 2.0,
            b'[' | b',' | b'<' | b'b' | b'B' => *elapsed = (*elapsed - 2.0).max(0.0),
            _ => {}
        }
    }
}

fn now_ns() -> u64 {
    let mut time = TimeSpec { seconds: 0, nanos: 0 };
    if syscall2(SYS_CLOCK_GETTIME, 1, addr_mut(&mut time)) < 0 {
        return 0;
    }
    (time.seconds as u64).saturating_mul(1_000_000_000).saturating_add(time.nanos as u64)
}

fn sleep_ns(nanos: u64) {
    let duration = TimeSpec {
        seconds: (nanos / 1_000_000_000) as i64,
        nanos: (nanos % 1_000_000_000) as i64,
    };
    syscall2(SYS_NANOSLEEP, addr(&duration), 0);
}

fn write_tx(length: usize) {
    let bytes = unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(TX).cast::<u8>(), length) };
    write_all(bytes);
}

fn write_all(bytes: &[u8]) {
    let mut written = 0usize;
    while written < bytes.len() {
        let result = syscall3(SYS_WRITE, 1, bytes.as_ptr() as usize + written, bytes.len() - written);
        if result <= 0 {
            break;
        }
        written += result as usize;
    }
}

fn addr<T>(value: &T) -> usize {
    value as *const T as usize
}

fn addr_mut<T>(value: &mut T) -> usize {
    value as *mut T as usize
}

fn syscall2(number: usize, a: usize, b: usize) -> isize {
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") number as isize => result,
            in("rdi") a,
            in("rsi") b,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    result
}

fn syscall3(number: usize, a: usize, b: usize, c: usize) -> isize {
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") number as isize => result,
            in("rdi") a,
            in("rsi") b,
            in("rdx") c,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    result
}

fn exit(code: usize) -> ! {
    unsafe {
        asm!(
            "syscall",
            in("rax") SYS_EXIT,
            in("rdi") code,
            options(noreturn)
        )
    }
}
