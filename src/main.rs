#![no_std]
#![no_main]
#![allow(linker_messages)]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

pub use units::{
    METERS_PER_MILE, FINAL_ALTITUDE_METERS, WORLD_UNITS_PER_MILE,
    FLOW_CLOSE_ORBIT_RADIUS_WORLD, FLOW_FINAL_ALTITUDE_WORLD, miles_to_world, world_to_miles,
    world_to_meters,
};
pub use flight::FLOW_PATH_END;
mod world; use world::*;
mod units; use units::*;
mod approved_opening; use approved_opening::*;
mod flight; use flight::*;
mod flight_route; use flight_route::*;
mod flight_speedlaw; use flight_speedlaw::*;
mod flight_math;
mod math; use math::*;
mod camera; use camera::*;
mod render_workers;
mod field_snapshot;
#[cfg(feature="phase0-audit")] mod audit_trace;
#[cfg(feature="phase0-audit")] mod audit_controls;
#[cfg(feature="phase0-audit")] use audit_controls::*;
#[cfg(feature="phase0-audit")] static mut LANDSCAPE_HEIGHT_EVALUATIONS:u64=0;
#[cfg(feature="phase0-audit")] static mut SURFACE_CACHE_MISSES:u64=0;
#[cfg(feature="phase0-audit")] static mut FRAME_STAGE_NS:[u64;4]=[0;4];

global_asm!(
    ".global _start",
    ".type _start,@function",
    "_start:",
    "and rsp, -16",
    "call rust_main",
    ".global memcpy",
    ".type memcpy,@function",
    "memcpy:",
    "mov rax, rdi",
    "mov rcx, rdx",
    "rep movsb",
    "ret",
    ".global memset",
    ".type memset,@function",
    "memset:",
    "mov r8, rdi",
    "mov rcx, rdx",
    "mov rax, rsi",
    "rep stosb",
    "mov rax, r8",
    "ret",
);

const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const PIXELS: usize = WIDTH * HEIGHT;
const FRAME_BYTES: usize = PIXELS * 3;
const MAP_SIDE: usize = 256;
const MAP_BYTES: usize = MAP_SIDE * MAP_SIDE;
const TX_BYTES: usize = 264_000;
const RAW_CHUNK: usize = 3_072;
const WATER: u8 = 76;
const GRID_SQUARE_START: f32 = 23.0;
const GRID_HORIZONTAL: f32 = 26.0;
const ORBIT_SEEK: f32 = 33.097328;
const CONTINENT_SEEK: f32 = 57.0;
const DESCENT: f32 = 70.0;
const FLOW_FOCAL: f32 = 154.0;
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
static mut DEPTH: [f32; PIXELS] = [0.0; PIXELS];
static mut TERRAIN: [u8; MAP_BYTES] = [0; MAP_BYTES];
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
#[derive(Clone, Copy, PartialEq, Eq)]
struct WinSize {
    rows: u16,
    columns: u16,
    pixel_width: u16,
    pixel_height: u16,
}

struct Playback {
    elapsed: f32,
    paused: bool,
}

impl Playback {
    fn advance(&mut self, dt: f32) {
        if !self.paused {
            self.elapsed = (self.elapsed + dt).min(FLOW_PATH_END as f32);
            if self.elapsed >= FLOW_PATH_END as f32 { self.paused = true; }
        }
    }
}

#[repr(C)]
struct TimeSpec {
    seconds: i64,
    nanos: i64,
}

#[derive(Clone, Copy, PartialEq)]
struct ProjectedPlanetBounds {
    visible: f32,
    x: f32,
    y: f32,
    radius_x: f32,
    radius_y: f32,
}

#[derive(Clone, Copy)]
struct SurfaceLod {
    continent: f32,
    ranges: f32,
    mountains: f32,
    valley: f32,
    footprint_miles: f64,
}

#[derive(Clone, Copy)]
struct SurfaceSample {
    height: f32,
    continent: f32,
    relief_world: f64,
    map_x: f32,
    map_y: f32,
}

#[derive(Clone, Copy)]
struct SurfaceVertex {
    x: f32,
    y: f32,
    inverse_depth: f32,
    red: f32,
    green: f32,
    blue: f32,
    valid: bool,
}

const EMPTY_SURFACE_VERTEX: SurfaceVertex = SurfaceVertex {
    x: 0.0,
    y: 0.0,
    inverse_depth: 0.0,
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    valid: false,
};

// Two scan rows are enough for the stable screen-space spherical mesh. Its
// topology never changes with time, altitude, or projected radius; exact
// boundary sampling handles the curved limb without increasing the interior
// polygon density.
const SURFACE_CELL_PIXELS: i32 = 4;
const SURFACE_ROW_CAPACITY: usize = WIDTH / SURFACE_CELL_PIXELS as usize + 4;
static mut SURFACE_ROW_A: [SurfaceVertex; SURFACE_ROW_CAPACITY] =
    [EMPTY_SURFACE_VERTEX; SURFACE_ROW_CAPACITY];
static mut SURFACE_ROW_B: [SurfaceVertex; SURFACE_ROW_CAPACITY] =
    [EMPTY_SURFACE_VERTEX; SURFACE_ROW_CAPACITY];
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    exit(101)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    generate_sines();
    generate_terrain();
    generate_trajectory();
    generate_field_vertices();

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
    #[cfg(feature="phase0-audit")]
    if !interactive {
        if audit_input[0]==b'_' {run_playback_controls_audit();}
        audit_trace::dispatch(audit_input[0]);
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
    let mut playback = Playback {
        elapsed: if audit_time > 0.0 { audit_time - 1.0 / 60.0 } else { 0.0 },
        paused: false,
    };
    let mut displayed_time = playback.elapsed;
    let mut displayed_window = window_size();
    let mut previous = now_ns();
    let mut deadline = previous;

    loop {
        let current = now_ns();
        let mut dt = (current.saturating_sub(previous) as f32) * 0.000_000_001;
        if first || dt <= 0.0 || dt > 0.1 {
            dt = 1.0 / 60.0;
        }
        previous = current;
        // Keep sampling the wall clock while paused: paused wall time must
        // never accumulate into a catch-up jump when playback resumes.
        playback.advance(dt);

        let window = window_size();
        if first || !playback.paused || playback.elapsed != displayed_time
            || window != displayed_window {
            render_demo(playback.elapsed, projection_x_scale(window));

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
            displayed_time = playback.elapsed;
            displayed_window = window;
        }

        let was_paused = playback.paused;
        if !interactive || handle_input(&mut playback) {
            break;
        }
        if was_paused && !playback.paused {
            previous = now_ns();
            deadline = previous;
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

const LOCAL_VALLEY_START_MILES: f32 = 18.0;
const LOCAL_VALLEY_END_MILES: f32 = 24.5;
const LOCAL_VALLEY_SLOPE: f32 = -2.913;
const LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES: f32 = 0.25;
const LOCAL_VALLEY_WALL_HALF_WIDTH_MILES: f32 = 0.75;
const LOCAL_VALLEY_BLEND_HALF_WIDTH_MILES: f32 = 1.0;
const LOCAL_VALLEY_WALL_HEIGHT_UNITS: f32 = 42.0;

fn local_valley_y(x: f32) -> f32 {
    (unsafe { LOCAL_VALLEY_ORIGIN_Y })
        + (x - LOCAL_VALLEY_START_MILES) * LOCAL_VALLEY_SLOPE
}

static mut LOCAL_VALLEY_ORIGIN_Y: f32 = 0.0;
static mut LANDING_FRAME: [Vec3; 3] = [Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 3];

fn local_valley_longitudinal_weight(x: f32) -> f32 {
    let inherited = smootherstep(x - (LOCAL_VALLEY_START_MILES - 1.0))
        * smootherstep((LOCAL_VALLEY_END_MILES + 1.0) - x);
    // Continue the watercourse downstream beyond the flown section. The
    // former excavation ended just ahead of the last camera pose, creating
    // a closing wall across the entire sky. This extension is fixed terrain;
    // its support starts beyond the inherited floor, which remains exact.
    let downstream = smootherstep((x - 24.9) / 0.4)
        * (1.0 - smootherstep((x - 40.0) / 2.0));
    inherited + (1.0 - inherited) * downstream
}

fn local_valley_distance(x: f32, y: f32) -> f32 {
    // The regional chart is unwrapped. Wrapping this distance would clone
    // the entire canyon at the former 256-mile texture period.
    (y - watershed_center(x)).abs()
}

fn local_valley_width_scale(x: f32) -> f32 {
    // Upstream is the low-x end of this fixed watershed. The inherited
    // funnel widened AHEAD of the current flight, so the approaching walls
    // continually receded outside the view. Keep the same floor and narrow
    // the river's banks downstream, before the existing local valley.
    1.0 + smootherstep((18.0 - x) / 2.0) * 8.0
}

fn normalize(x: f32, y: f32) -> (f32, f32) {
    let high = x.abs().max(y.abs());
    let low = x.abs().min(y.abs());
    let inverse = 1.0 / (high + low * 0.375).max(0.001);
    (x * inverse, y * inverse)
}

fn generate_terrain() {
    let up = initial_landing_up();
    let forward = initial_landing_forward();
    unsafe {
        LOCAL_VALLEY_ORIGIN_Y = valley_y(LOCAL_VALLEY_START_MILES);
        LANDING_FRAME = [up, forward, vec_normalize(vec_cross(forward, up))];
    }
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

fn render_demo(elapsed: f32, x_scale: f32) {
    render_workers::render(elapsed,x_scale);
}

fn render_frame_region(elapsed: f32, x_scale: f32) {
    let flow = flow_camera(elapsed);
    let exposure_start = field_star_exposure_camera(elapsed, &flow);
    // Parallel tiles receive their exact initial depth from the field snapshot.
    // Clearing the whole framebuffer for each tile would repeat this 25 times.
    if !render_workers::parallel_region() {clear_depth_buffer();}
    #[cfg(feature="phase0-audit")] let start=process_cpu_ns();
    render_universe_field(&exposure_start, &flow, x_scale);
    #[cfg(feature="phase0-audit")] unsafe {FRAME_STAGE_NS[0]=process_cpu_ns()-start;}
    render_world_journey(elapsed, x_scale, &flow);
    draw_second_counter(elapsed);
}

fn rail_object_exists(layout: FieldLayout, identity: RailObjectIdentity) -> bool {
    field_ring_ordinal(identity.first_ring) >= rail_lane_start_ordinal(layout, identity.lane)
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

fn landing_up() -> Vec3 { unsafe { core::ptr::addr_of!(LANDING_FRAME[0]).read() } }
fn landing_forward() -> Vec3 { unsafe { core::ptr::addr_of!(LANDING_FRAME[1]).read() } }

fn initial_landing_up() -> Vec3 {
    let phase_sine = sine_sample(3_200.0);
    let phase_cosine = sine_sample(3_200.0 + 256.0);
    let tilt_sine = sine_sample(6.0);
    let tilt_cosine = sine_sample(262.0);
    Vec3 {
        x: tilt_cosine * phase_sine,
        y: -tilt_cosine * phase_cosine,
        z: tilt_sine,
    }
}

fn initial_landing_forward() -> Vec3 {
    let phase_sine = sine_sample(3_200.0);
    let phase_cosine = sine_sample(3_200.0 + 256.0);
    let tilt_sine = sine_sample(6.0);
    let tilt_cosine = sine_sample(262.0);
    Vec3 {
        x: -tilt_sine * phase_sine,
        y: tilt_sine * phase_cosine,
        z: tilt_cosine,
    }
}

fn planet_continent_lod(normal: Vec3, lod: SurfaceLod) -> f32 {
    let landing_cap = vec_dot(normal, landing_up()) - 0.46;
    let second_cap = vec_dot(normal, Vec3 { x: 0.72, y: 0.42, z: -0.55 }) - 0.57;
    let third_cap = vec_dot(normal, Vec3 { x: -0.68, y: 0.15, z: -0.72 }) - 0.60;
    let core = landing_cap.max(second_cap).max(third_cap);
    // Exact saturation bounds for the three following bounded sine terms.
    let perturbation = 0.12 * lod.continent + 0.065 * lod.ranges + 0.035 * lod.mountains;
    if core - perturbation >= 0.13 { return 1.0; }
    if core + perturbation <= 0.0 { return 0.0; }
    let broad = sine_sample(
        normal.x * 430.0 + normal.y * 270.0 + normal.z * 190.0 + 50.0,
    ) * 0.12 * lod.continent;
    let regional = sine_sample(
            normal.x * 910.0 - normal.y * 530.0 + normal.z * 370.0 + 400.0,
        ) * 0.065 * lod.ranges;
    let local = sine_sample(
            normal.x * 1_570.0 + normal.y * 1_130.0 - normal.z * 790.0 + 811.0,
        ) * 0.035 * lod.mountains;
    smoothstep((core + broad + regional + local) / 0.13)
}

fn flow_normal_to_planet(normal:DVec3)->Vec3 {terrain_direction(normal)}

fn planet_normal_to_flow(normal:Vec3)->DVec3 {terrain_to_world(normal)}

fn flow_surface_map_from_normal(normal: Vec3) -> (f32, f32) {
    let forward = vec_dot(normal, landing_forward());
    let right = unsafe { core::ptr::addr_of!(LANDING_FRAME[2]).read() };
    let miles_per_radian = TERRAIN_CHART_UNITS_PER_RADIAN as f32;
    let map_x = 18.0 + forward * miles_per_radian;
    let broad = valley_y(map_x);
    let chart_alignment = smootherstep((map_x - (LOCAL_VALLEY_START_MILES - 3.0)) / 2.0)
        * smootherstep(((LOCAL_VALLEY_END_MILES + 3.0) - map_x) / 2.0);
    let centre = broad + (local_valley_y(map_x) - broad)
        * chart_alignment;
    (map_x, centre + vec_dot(normal, right) * miles_per_radian)
}

fn terrain_parent_sample(x: f32, y: f32, cell: f32) -> f32 {
    let scaled_x = x / cell;
    let scaled_y = y / cell;
    let mut cell_x = scaled_x as i32;
    let mut cell_y = scaled_y as i32;
    if scaled_x < cell_x as f32 {
        cell_x -= 1;
    }
    if scaled_y < cell_y as f32 {
        cell_y -= 1;
    }
    let origin_x = cell_x as f32 * cell;
    let origin_y = cell_y as f32 * cell;
    let blend_x = smoothstep((x - origin_x) / cell);
    let blend_y = smoothstep((y - origin_y) / cell);
    let map = core::ptr::addr_of!(TERRAIN).cast::<u8>();
    let sample = |sample_x: f32, sample_y: f32| unsafe {
        map.add(
            ((sample_y as i32 & 255) as usize) * MAP_SIDE
                + (sample_x as i32 & 255) as usize,
        )
        .read() as f32
    };
    let a = sample(origin_x, origin_y);
    let b = sample(origin_x + cell, origin_y);
    let c = sample(origin_x, origin_y + cell);
    let d = sample(origin_x + cell, origin_y + cell);
    let upper = a + (b - a) * blend_x;
    let lower = c + (d - c) * blend_x;
    upper + (lower - upper) * blend_y
}

fn surface_lod_weight(wavelength_miles: f64, footprint_miles: f64) -> f32 {
    let footprint=footprint_miles.max(1.0e-9);
    // Exact saturated values, with deliberately generous margins around the
    // original ratio's 0.75/3.0 thresholds. Most terrain queries are far inside
    // these plateaus; no division/polynomial or detail approximation is needed.
    if footprint<wavelength_miles*0.25 {return 1.0;}
    if footprint>wavelength_miles*2.0 {return 0.0;}
    smootherstep_f64(
        (wavelength_miles / footprint - 0.75) / 2.25,
    ) as f32
}

fn surface_lod(footprint_miles: f64) -> SurfaceLod {
    SurfaceLod {
        continent: surface_lod_weight(3_000.0, footprint_miles),
        ranges: surface_lod_weight(700.0, footprint_miles),
        mountains: surface_lod_weight(90.0, footprint_miles),
        valley: surface_lod_weight(8.0, footprint_miles),
        footprint_miles,
    }
}

fn full_surface_lod() -> SurfaceLod {
    SurfaceLod { continent: 1.0, ranges: 1.0, mountains: 1.0, valley: 1.0, footprint_miles: 0.0 }
}

fn inherited_watershed_floor(map: (f32, f32), lod: SurfaceLod) -> f32 {
    let broad = terrain_parent_sample(map.0, map.1, 64.0);
    let range = terrain_parent_sample(map.0, map.1, 16.0);
    let mountain = terrain_parent_sample(map.0, map.1, 4.0);
    let detail = terrain_smooth(map.0, map.1);
    WATER as f32
        + (broad - WATER as f32) * lod.continent
        + (range - broad) * lod.ranges
        + (mountain - range) * lod.mountains
        + (detail - mountain) * lod.valley
}

// Nonperiodic world-space noise. There is deliberately no MAP_SIDE mask:
// moving 256 miles does not reproduce the same mountain or biome.
const NOISE_CORNER_CACHE_SIZE:usize=4096;
#[derive(Clone,Copy)]
struct NoiseCorners {key:[i32;3],values:[f32;4]}
static mut NOISE_CORNERS:[NoiseCorners;NOISE_CORNER_CACHE_SIZE]=
    [NoiseCorners{key:[0;3],values:[0.0;4]};NOISE_CORNER_CACHE_SIZE];

fn landform_corners(ix:i32,iy:i32,seed:i32)->[f32;4] {
    let key=[ix,iy,seed];
    let mixed=(ix as u32).wrapping_mul(0x9e37_79b9) ^ (iy as u32).wrapping_mul(0x85eb_ca6b)
        ^ (seed as u32).wrapping_mul(0xc2b2_ae35);
    let index=((mixed^(mixed>>16)) as usize)&(NOISE_CORNER_CACHE_SIZE-1);
    let pointer=core::ptr::addr_of_mut!(NOISE_CORNERS).cast::<NoiseCorners>();
    let entry=unsafe {pointer.add(index).read()};
    if entry.key==key && seed!=0 {return entry.values;}
    let sample=|a:i32,b:i32|(hash(a.wrapping_add(seed),b.wrapping_sub(seed)) as f32-127.5)/127.5;
    let values=[sample(ix,iy),sample(ix+1,iy),sample(ix,iy+1),sample(ix+1,iy+1)];
    unsafe {pointer.add(index).write(NoiseCorners{key,values});}
    values
}

fn landform_noise(x: f32, y: f32, wavelength: f32, seed: i32) -> f32 {
    let x = x / wavelength;
    let y = y / wavelength;
    let ix = floor_i32(x);
    let iy = floor_i32(y);
    let u = smootherstep(x - ix as f32);
    let v = smootherstep(y - iy as f32);
    let [a,b,c,d]=landform_corners(ix,iy,seed);
    (a + (b - a) * u) * (1.0 - v) + (c + (d - c) * u) * v
}

fn watershed_center(x: f32) -> f32 {
    let broad = valley_y(x);
    let alignment = smootherstep((x - (LOCAL_VALLEY_START_MILES - 3.0)) / 2.0)
        * smootherstep(((LOCAL_VALLEY_END_MILES + 3.0) - x) / 2.0);
    broad + (local_valley_y(x) - broad) * alignment
}

fn watershed_floor_weight(map: (f32, f32)) -> f32 {
    let cross_track = (map.1 - watershed_center(map.0)).abs();
    // This is the persistent river floor, not a camera-following mask. Its
    // inherited height must stay exact while the surrounding relief grows.
    1.0 - smootherstep((cross_track - 0.25) / 0.50)
}

fn drainage_distance(map: (f32, f32)) -> f32 {
    let cross = map.1 - watershed_center(map.0);
    let mut distance = cross.abs();
    // Fixed asymmetric tributaries join the same trunk. Their widths and
    // hierarchy persist from the overhead basin into the local canyon.
    for (junction, bend, side) in [(5.0, 0.31, -1.0), (16.0, -0.44, 1.0),
        (27.0, 0.57, -1.0), (43.0, -0.28, 1.0), (68.0, 0.39, -1.0)] {
        let upstream = (cross * side).max(0.0);
        let branch_x = junction + upstream * bend + upstream * upstream * 0.003;
        let branch = (map.0 - branch_x).abs() / sqrt_f64(1.0 + bend as f64 * bend as f64) as f32;
        if cross * side > 0.0 { distance = distance.min(branch); }
    }
    distance
}

fn landform_height(map: (f32, f32), lod: SurfaceLod) -> f32 {
    let warp_x = landform_noise(map.0, map.1, 160.0, 719) * 24.0;
    let warp_y = landform_noise(map.0, map.1, 220.0, 1_237) * 32.0;
    let mut x = map.0 + warp_x;
    let mut y = map.1 + warp_y;
    let basin = landform_noise(x, y, 320.0, 2_057);
    let continental = landform_noise(x, y, 2_048.0, 431);
    let mut height = WATER as f32 + 30.0 * lod.continent
        + continental * 20.0 * surface_lod_weight(2_048.0, lod.footprint_miles)
        + basin * 28.0 * surface_lod_weight(320.0, lod.footprint_miles);
    let mut wavelength = 64.0;
    let mut amplitude = 45.0;
    let mut octave = 0;
    while octave < 5 {
        let weight = surface_lod_weight(wavelength as f64, lod.footprint_miles);
        if weight > 0.0 {
            let noise = landform_noise(x, y, wavelength, 3_071 + octave * 977);
            let ridge = 1.0 - noise.abs();
            height += (ridge * ridge - 0.38) * amplitude * weight;
        }
        // Rotated octaves cannot line up into a repeating tilted grid.
        let next_x = x * 0.8 - y * 0.6 + 19.0;
        y = x * 0.6 + y * 0.8 - 31.0;
        x = next_x;
        amplitude *= 0.53;
        wavelength *= 0.25;
        octave += 1;
    }
    let drainage = drainage_distance(map);
    let valley = 1.0 - smootherstep(drainage / 3.0);
    height -= valley * 27.0 * surface_lod_weight(6.0, lod.footprint_miles);
    height.clamp(35.0, 225.0)
}

fn flow_base_surface_height_from_map(map: (f32, f32), lod: SurfaceLod) -> f32 {
    let floor = watershed_floor_weight(map);
    if floor == 1.0 { return inherited_watershed_floor(map, lod); }
    let height = landform_height(map, lod);
    if floor == 0.0 { return height; }
    height + (inherited_watershed_floor(map, lod) - height) * floor
}

fn flow_surface_height_from_map(map: (f32, f32), lod: SurfaceLod) -> f32 {
    if lod.valley <= 0.0 {
        return flow_base_surface_height_from_map(map, lod);
    }
    let longitudinal = local_valley_longitudinal_weight(map.0);
    if longitudinal <= 0.0 {
        return flow_base_surface_height_from_map(map, lod);
    }
    let width_scale = local_valley_width_scale(map.0);
    let floor_half_width = LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES * width_scale;
    let wall_half_width = LOCAL_VALLEY_WALL_HALF_WIDTH_MILES * width_scale;
    let blend_half_width = LOCAL_VALLEY_BLEND_HALF_WIDTH_MILES * width_scale;
    let distance = local_valley_distance(map.0, map.1);
    if distance >= blend_half_width {
        return flow_base_surface_height_from_map(map, lod);
    }
    let wall = smootherstep(
        (distance - floor_half_width) / (wall_half_width - floor_half_width),
    );
    // Keep the navigable floor below the mountain-relief threshold. The wall
    // profile then rises out of that floor continuously; allowing the legacy
    // floor noise to cross the threshold made the 20-metre camera chase a
    // derivative cusp instead of following a graceful valley tangent.
    let floor = valley_floor(map.0).min(105.0);
    let cliff = if wall > 0.0 {
        landform_noise(map.0, map.1 * 0.3, 0.4, 7_019) * 7.0
            * surface_lod_weight(0.4, lod.footprint_miles)
    } else { 0.0 };
    let target = floor + wall * (LOCAL_VALLEY_WALL_HEIGHT_UNITS + cliff);
    let lateral = 1.0 - smootherstep(
        (distance - wall_half_width) / (blend_half_width - wall_half_width),
    );
    let weight = lod.valley * longitudinal * lateral;
    if weight == 1.0 { return target; }
    let base = flow_base_surface_height_from_map(map, lod);
    base + (target - base) * weight
}

fn flow_surface_relief_lod_world(normal: Vec3, lod: SurfaceLod) -> f64 {
    flow_surface_sample(normal, lod).relief_world
}

// Exact memoization, not another terrain representation: a cache hit requires
// all three normal bit patterns to match. Below the finest resolved wavelength
// every height weight is exactly one, so footprint is no longer an input to
// the height function. Color/cloud filtering still uses the original LOD.
const SURFACE_SAMPLE_CACHE_SIZE: usize = 16_384;
#[derive(Clone, Copy)]
struct SurfaceSampleCacheEntry {
    normal: [u32; 3],
    sample: SurfaceSample,
}
static mut SURFACE_SAMPLE_CACHE: [SurfaceSampleCacheEntry; SURFACE_SAMPLE_CACHE_SIZE] =
    [SurfaceSampleCacheEntry {
        normal: [0; 3],
        sample: SurfaceSample { height: 0.0, continent: 0.0, relief_world: 0.0, map_x: 0.0, map_y: 0.0 },
    }; SURFACE_SAMPLE_CACHE_SIZE];

fn flow_surface_sample(normal: Vec3, lod: SurfaceLod) -> SurfaceSample {
    if lod.continent != 1.0 || lod.ranges != 1.0 || lod.mountains != 1.0
        || lod.valley != 1.0 || lod.footprint_miles > 0.25 / 3.0 {
        return uncached_surface_sample(normal, lod);
    }
    let key = [normal.x.to_bits(), normal.y.to_bits(), normal.z.to_bits()];
    let mixed = key[0].wrapping_mul(0x9e37_79b9)
        ^ key[1].wrapping_mul(0x85eb_ca6b) ^ key[2].wrapping_mul(0xc2b2_ae35);
    let index = ((mixed ^ (mixed >> 16)) as usize) & (SURFACE_SAMPLE_CACHE_SIZE - 1);
    let pointer = core::ptr::addr_of_mut!(SURFACE_SAMPLE_CACHE).cast::<SurfaceSampleCacheEntry>();
    let entry = unsafe { pointer.add(index).read() };
    if entry.normal == key { return entry.sample; }
    let sample = uncached_surface_sample(normal, lod);
    unsafe { pointer.add(index).write(SurfaceSampleCacheEntry { normal: key, sample }); }
    sample
}

fn uncached_surface_sample(normal: Vec3, lod: SurfaceLod) -> SurfaceSample {
    #[cfg(feature="phase0-audit")] unsafe {SURFACE_CACHE_MISSES+=1;}
    let range_map = flow_surface_map_from_normal(normal);
    let regional = smootherstep((vec_dot(normal, landing_up()) - 0.97) / 0.02);
    let height = if regional == 1.0 {
        flow_surface_height_from_map(range_map, lod)
    } else {
        // Independent spherical projections prevent the local chart from
        // mirroring the landing landscape on the opposite hemisphere.
        let globe_map = (
            (normal.x * 0.81 + normal.y * 0.33 - normal.z * 0.47) * TERRAIN_CHART_UNITS_PER_RADIAN as f32,
            (normal.x * -0.23 + normal.y * 0.91 + normal.z * 0.36) * TERRAIN_CHART_UNITS_PER_RADIAN as f32,
        );
        let globe = landform_height(globe_map, lod);
        if regional == 0.0 { globe }
        else { globe + (flow_surface_height_from_map(range_map, lod) - globe) * regional }
    };
    let continent = planet_continent_lod(normal, lod);
    let chain = smoothstep(
        (sine_sample(range_map.0 * 0.11 + range_map.1 * 0.035 + 97.0) - 0.08)
            / 0.78,
    );
    let longitudinal = local_valley_longitudinal_weight(range_map.0);
    let local_floor = if lod.valley > 0.0 && longitudinal > 0.0 {
        let width_scale = local_valley_width_scale(range_map.0);
        let local_distance = local_valley_distance(range_map.0, range_map.1);
        let local_floor_half_width = LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES * width_scale;
        longitudinal
            * (1.0 - smootherstep(
                (local_distance - local_floor_half_width) / (0.25 * width_scale),
            ))
            * lod.valley
    } else {
        0.0
    };
    let range_relief = chain * lod.ranges * 0.38 * (1.0 - local_floor * 0.90);
    let mountain_relief = (height - 112.0).max(0.0) * 0.0105;
    SurfaceSample {
        height,
        continent,
        relief_world: miles_to_world(
            continent as f64 * (range_relief as f64 + mountain_relief as f64),
        ),
        map_x: range_map.0,
        map_y: range_map.1,
    }
}

fn flow_surface_relief_world(normal: Vec3) -> f64 {
    flow_surface_relief_lod_world(normal, full_surface_lod())
}

fn flow_sphere_roots(camera: &FlowCamera, direction: DVec3, radius: f64) -> Option<(f64, f64)> {
    let local = dvec_sub(camera.position, planet_center());
    let b = dvec_dot(local, direction);
    let c = dvec_dot(local, local) - radius * radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }
    let root = sqrt_f64(discriminant);
    let near = -b - root;
    let far = -b + root;
    if far <= camera.near_plane {
        None
    } else {
        Some((near.max(camera.near_plane), far))
    }
}

fn surface_city_amount(normal: Vec3, sample: SurfaceSample, lod: SurfaceLod) -> f32 {
    // Permanent settlement footprints in the planet's coordinates. Hashing
    // chooses sites, not per-frame pixels; day roofs and night lights share
    // exactly the same footprint.
    let x = normal.x * 96.0;
    let y = normal.y * 96.0;
    let z = normal.z * 96.0;
    let ix = floor_i32(x);
    let iy = floor_i32(y);
    let iz = floor_i32(z);
    let seed = ix.wrapping_mul(73_856_093) ^ iy.wrapping_mul(19_349_663) ^ iz.wrapping_mul(83_492_791);
    let dx = x - ix as f32 - 0.5;
    let dy = y - iy as f32 - 0.5;
    let dz = z - iz as f32 - 0.5;
    let distance = fast_sqrt(dx * dx + dy * dy + dz * dz);
    let city = if hash(seed, 2_479) < 48 {
        smoothstep((0.45 - distance) / 0.17)
    } else { 0.0 };
    let town_x = sample.map_x - 17.0;
    let town_y = sample.map_y - (local_valley_y(sample.map_x) + 1.2);
    let town = smoothstep((1.0 - fast_sqrt(town_x * town_x + town_y * town_y)) / 0.35);
    let regional_x = (sample.map_x - 39.0) / 3.0;
    let regional_y = (sample.map_y - (valley_y(sample.map_x) + 4.0)) / 3.0;
    let regional = smoothstep((1.0 - fast_sqrt(regional_x * regional_x + regional_y * regional_y)) / 0.35);
    // The local chart has a unique spherical hemisphere; do not duplicate
    // its settlement on the opposite side of the planet.
    let local = town.max(regional) * smoothstep((vec_dot(normal, landing_up()) - 0.99) / 0.009);
    city.max(local) * sample.continent * lod.mountains
}

fn flow_surface_color(
    normal: Vec3,
    light_normal: DVec3,
    _time: f32,
    sample: SurfaceSample,
    lod: SurfaceLod,
) -> (u8, u8, u8) {
    let map = (sample.map_x, sample.map_y);
    let moisture = sine_sample(
        normal.x * 371.0 - normal.y * 229.0 + normal.z * 157.0 + 89.0,
    ) * 0.5 + 0.5;
    let warmth = (0.25
        + sine_sample(normal.x * 181.0 + normal.y * 263.0 - normal.z * 97.0 + 431.0)
            * 0.22
        + (1.0 - normal.x.abs()) * 0.53)
        .clamp(0.0, 1.0);
    let arid = smoothstep((warmth - moisture - 0.02) / 0.46);
    let highland = smoothstep((sample.height - 108.0) / 72.0);
    let summit = smoothstep((sample.height - 172.0) / 42.0) * lod.mountains;
    let polar = smoothstep((normal.x.abs() - 0.76) / 0.18);
    let lowland = (
        39.0 + arid * 92.0 + moisture * 18.0,
        116.0 - arid * 48.0 + moisture * 31.0,
        54.0 - arid * 19.0 + moisture * 20.0,
    );
    let upland = (
        103.0 + arid * 61.0,
        91.0 + moisture * 24.0 - arid * 19.0,
        66.0 + moisture * 14.0,
    );
    let snow = (220.0, 224.0, 218.0);
    let mut land = (
        lowland.0 + (upland.0 - lowland.0) * highland,
        lowland.1 + (upland.1 - lowland.1) * highland,
        lowland.2 + (upland.2 - lowland.2) * highland,
    );
    let snow_amount = summit.max(polar * (0.35 + lod.ranges * 0.65));
    land = (
        land.0 + (snow.0 - land.0) * snow_amount,
        land.1 + (snow.1 - land.1) * snow_amount,
        land.2 + (snow.2 - land.2) * snow_amount,
    );

    let ocean_depth = ((WATER as f32 - sample.height + 48.0) / 76.0).clamp(0.0, 1.0);
    let ocean = (
        8.0 + (3.0 - 8.0) * ocean_depth,
        43.0 + (24.0 - 43.0) * ocean_depth,
        104.0 + (67.0 - 104.0) * ocean_depth,
    );
    let coast_band = smoothstep((0.10 - (sample.continent - 0.50).abs()) / 0.10)
        * lod.continent;
    let river = smoothstep((0.12 - drainage_distance(map)) / 0.08)
        * surface_lod_weight(0.24, lod.footprint_miles)
        * sample.continent;
    let range_chain = smoothstep(
        (sine_sample(map.0 * 0.11 + map.1 * 0.035 + 97.0) - 0.08) / 0.78,
    ) * lod.ranges;
    land.0 += range_chain * 23.0 - river * 45.0;
    land.1 += range_chain * 13.0 - river * 38.0;
    land.2 += range_chain * 8.0 + river * 53.0;
    let city = surface_city_amount(normal, sample, lod);
    let block_lod = surface_lod_weight(0.125, lod.footprint_miles);
    let blocks = (sine_sample(map.0 * 8_192.0).abs()
        * sine_sample(map.1 * 8_192.0).abs()).clamp(0.0, 1.0);
    let roof = 0.825 + (blocks - 0.5) * 0.35 * block_lod;
    land.0 += (142.0 * roof - land.0) * city;
    land.1 += (139.0 * roof - land.1) * city;
    land.2 += (128.0 * roof - land.2) * city;
    let base = (
        ocean.0 + (land.0 - ocean.0) * sample.continent + coast_band * 24.0,
        ocean.1 + (land.1 - ocean.1) * sample.continent + coast_band * 21.0,
        ocean.2 + (land.2 - ocean.2) * sample.continent + coast_band * 8.0,
    );
    let flow_normal = vec3_from_dvec(planet_normal_to_flow(normal));
    let sunlight = vec_dot(flow_normal, flow_sun_direction());
    let night_lights = city * smoothstep((0.08 - sunlight) / 0.18)
        * (0.775 + 0.45 * (blocks - 0.5) * block_lod);
    let illumination = 0.30 + dvec_dot_vec3(light_normal,flow_sun_direction()).max(0.0) as f32 * 0.82;
    (
        (base.0 * illumination + night_lights * 190.0)
            .clamp(0.0, 255.0) as u8,
        (base.1 * illumination + night_lights * 136.0)
            .clamp(0.0, 255.0) as u8,
        (base.2 * illumination + night_lights * 58.0)
            .clamp(0.0, 255.0) as u8,
    )
}

fn flow_camera_ray(
    camera: &FlowCamera,
    x_scale: f32,
    screen_x: f32,
    screen_y: f32,
) -> DVec3 {
    // Invert the actual projection basis, including its fixed horizontal
    // calibration. Treating the calibrated basis as orthonormal skews rays.
    dvec_normalize(canonical_projection_ray(camera,
        (160.0 + (screen_x - 160.0) / x_scale.max(0.001), screen_y)))
}

// Conservative bound derived from height<=225 and the 0.38-mile range term.
// It encloses every active displaced sample, including the canyon walls.
const SURFACE_RELIEF_BOUND_MILES: f64 = 2.0;
fn surface_ray_height(
    camera: &FlowCamera, direction: DVec3, distance: f64,
) -> (f64, Vec3, SurfaceSample, SurfaceLod) {
    #[cfg(feature = "phase0-audit")]
    unsafe { LANDSCAPE_HEIGHT_EVALUATIONS += 1; }
    let relative = dvec_sub(dvec_add(camera.position, dvec_scale(direction, distance)), planet_center());
    let radius = dvec_length(relative);
    let normal = flow_normal_to_planet(dvec_scale(relative, 1.0 / radius));
    let lod = surface_lod(world_to_miles(distance) / FLOW_FOCAL as f64);
    let sample = flow_surface_sample(normal, lod);
    (radius - FLOW_PLANET_RADIUS_WORLD - sample.relief_world, normal, sample, lod)
}

fn surface_ray_intersection(camera: &FlowCamera, direction: DVec3) -> Option<f64> {
    let forward = dvec_dot_vec3(direction, camera.forward).max(1.0e-12);
    let near = camera.near_plane * 1.000_2 / forward;
    let bound = FLOW_PLANET_RADIUS_WORLD + miles_to_world(SURFACE_RELIEF_BOUND_MILES);
    let (enter, exit) = flow_sphere_roots(camera, direction, bound)?;
    let start = enter.max(near);
    let end = flow_sphere_roots(camera, direction, FLOW_PLANET_RADIUS_WORLD)
        // Put the bracket endpoint just inside the solid core. A rounded
        // quadratic root can otherwise be microscopically OUTSIDE sea level,
        // falsely declaring an ocean/lowland ray empty and subdividing its
        // whole cell as a silhouette. This is a search bound, not a hit offset.
        .map(|roots| (roots.0 + 1.0e-10).max(start).min(exit)).unwrap_or(exit);
    if end <= start { return None; }
    let mut before = start;
    let mut before_height = surface_ray_height(camera, direction, before).0;
    if before_height <= 0.0 { return Some(before); }
    let outside = dvec_length(dvec_sub(camera.position, planet_center())) >= bound;
    let mut step = if outside { (end - start) / 8.0 }
        else { miles_to_world(camera.altitude_miles.max(0.001) * 0.25) };
    step = step.max(miles_to_world(0.001));
    let mut index = 0;
    while index < 32 {
        let distance = if index == 31 { end } else { (before + step).min(end) };
        let height = surface_ray_height(camera, direction, distance).0;
        if height <= 0.0 {
            let mut lo = before;
            let mut hi = distance;
            let mut lo_height = before_height;
            let mut hi_height = height;
            let mut iteration = 0;
            while iteration < 24 {
                // Safeguarded secant search preserves the bracket, while
                // nearly planar floor rays converge without twelve redundant
                // height evaluations. The residual, not iteration count,
                // decides early completion.
                let fraction = if iteration % 3 == 2 { 0.5 }
                    else { (lo_height / (lo_height - hi_height)).clamp(0.05, 0.95) };
                let middle = lo + (hi - lo) * fraction;
                let middle_height = surface_ray_height(camera, direction, middle).0;
                if world_to_meters(middle_height.abs()) < 0.02 { return Some(middle); }
                if middle_height > 0.0 { lo = middle; lo_height = middle_height; }
                else { hi = middle; hi_height = middle_height; }
                iteration += 1;
            }
            return Some((lo + hi) * 0.5);
        }
        if distance >= end { break; }
        before = distance;
        before_height = height;
        if !outside { step *= 1.5; }
        index += 1;
    }
    None
}

fn surface_relief_normal(normal:Vec3,sample:SurfaceSample,lod:SurfaceLod)->DVec3 {
    let n=dvec_normalize(dvec_from_vec3(normal));
    let reference=if n.x.abs()<0.8 {DVec3{x:1.0,y:0.0,z:0.0}} else {DVec3{x:0.0,y:1.0,z:0.0}};
    let u=dvec_normalize(dvec_cross(reference,n));
    let v=dvec_cross(n,u);
    // Differentiate the SAME displaced sphere at its projected footprint.
    // These are shading samples, not another surface mesh or collision model.
    let step=(miles_to_world(lod.footprint_miles)/FLOW_PLANET_RADIUS_WORLD)
        .clamp(1.0/RADIUS_METRES,0.001);
    let inverse=1.0/sqrt_f64(1.0+step*step);
    let relief=|tangent:DVec3,sign:f64| {
        let adjacent=dvec_scale(dvec_add(n,dvec_scale(tangent,step*sign)),inverse);
        flow_surface_sample(vec3_from_dvec(adjacent),lod).relief_world
    };
    let radius=FLOW_PLANET_RADIUS_WORLD+sample.relief_world;
    let du=(relief(u,1.0)-relief(u,-1.0))/(2.0*step*radius);
    let dv=(relief(v,1.0)-relief(v,-1.0))/(2.0*step*radius);
    let local=dvec_normalize(dvec_sub(n,dvec_add(dvec_scale(u,du),dvec_scale(v,dv))));
    dvec_normalize(planet_normal_to_flow(vec3_from_dvec(local)))
}

fn sample_surface_vertex(
    camera: &FlowCamera,
    time: f32,
    x_scale: f32,
    screen_x: f32,
    screen_y: f32,
) -> SurfaceVertex {
    let direction = flow_camera_ray(camera, x_scale, screen_x, screen_y);
    let Some(distance) = surface_ray_intersection(camera, direction) else {
        return SurfaceVertex { x: screen_x, y: screen_y, ..EMPTY_SURFACE_VERTEX };
    };
    let (_, normal, surface_sample, lod) = surface_ray_height(camera, direction, distance);
    let depth = (distance * dvec_dot_vec3(direction, camera.forward)) as f32;
    if depth <= camera.near_plane as f32 || !depth.is_finite() {
        return SurfaceVertex { x: screen_x, y: screen_y, ..EMPTY_SURFACE_VERTEX };
    }
    let light_normal=surface_relief_normal(normal,surface_sample,lod);
    let mut color = flow_surface_color(normal, light_normal, time, surface_sample, lod);
    if let Some((start, end)) = atmosphere_ray_bounds(camera, direction) {
        let visible_end = end.min(distance);
        if visible_end > start {
            let haze = atmospheric_alpha(atmosphere_optical_depth(camera, direction, start, visible_end));
            let tint = atmosphere_scatter_color(camera, direction);
            color = (
                (color.0 as f32 + (tint.0 - color.0 as f32) * haze) as u8,
                (color.1 as f32 + (tint.1 - color.1 as f32) * haze) as u8,
                (color.2 as f32 + (tint.2 - color.2 as f32) * haze) as u8,
            );
        }
    }
    let cloud = cloud_ray_amount(camera, direction, distance);
    if cloud > 0.0 {
        color = ((color.0 as f32 + (215.0 - color.0 as f32) * cloud) as u8,
            (color.1 as f32 + (223.0 - color.1 as f32) * cloud) as u8,
            (color.2 as f32 + (232.0 - color.2 as f32) * cloud) as u8);
    }
    SurfaceVertex {
        x: screen_x,
        y: screen_y,
        inverse_depth: 1.0 / depth,
        red: color.0 as f32,
        green: color.1 as f32,
        blue: color.2 as f32,
        valid: true,
    }
}

fn surface_edge(a: SurfaceVertex, b: SurfaceVertex, x: f32, y: f32) -> f32 {
    (x - a.x) * (b.y - a.y) - (y - a.y) * (b.x - a.x)
}

fn floor_i32(value: f32) -> i32 {
    let integer = value as i32;
    if value < integer as f32 { integer - 1 } else { integer }
}

fn ceil_i32(value: f32) -> i32 {
    let integer = value as i32;
    if value > integer as f32 { integer + 1 } else { integer }
}

fn raster_surface_triangle(
    a: SurfaceVertex,
    b: SurfaceVertex,
    c: SurfaceVertex,
    visibility: f32,
) {
    if !a.valid || !b.valid || !c.valid {
        return;
    }
    let area = surface_edge(a, b, c.x, c.y);
    if area.abs() < 0.000_1 {
        return;
    }
    // Vertices and exact boundary pixels share the same terrain-normal light,
    // applied BEFORE atmosphere/clouds. Never relight the haze per triangle.
    let minimum_x = floor_i32(a.x.min(b.x).min(c.x)).max(0);
    let maximum_x = ceil_i32(a.x.max(b.x).max(c.x)).min(WIDTH as i32);
    let minimum_y = floor_i32(a.y.min(b.y).min(c.y)).max(0);
    let maximum_y = ceil_i32(a.y.max(b.y).max(c.y)).min(HEIGHT as i32);
    let inverse_area = 1.0 / area;
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let mut y = render_workers::next_row(minimum_y);
    while y < maximum_y {
        let py = y as f32 + 0.5;
        let mut x = minimum_x;
        while x < maximum_x {
            let px = x as f32 + 0.5;
            let wa = surface_edge(b, c, px, py) * inverse_area;
            let wb = surface_edge(c, a, px, py) * inverse_area;
            let wc = 1.0 - wa - wb;
            if wa >= -0.000_1 && wb >= -0.000_1 && wc >= -0.000_1 {
                let inverse_depth = a.inverse_depth * wa
                    + b.inverse_depth * wb
                    + c.inverse_depth * wc;
                if inverse_depth > 0.0 {
                    blend_depth_pixel(
                        frame,
                        x,
                        y,
                        1.0 / inverse_depth,
                        (a.red * wa + b.red * wb + c.red * wc).clamp(0.0, 255.0) as u8,
                        (a.green * wa + b.green * wb + c.green * wc).clamp(0.0, 255.0) as u8,
                        (a.blue * wa + b.blue * wb + c.blue * wc).clamp(0.0, 255.0) as u8,
                        (visibility * 255.0) as u8,
                        0.0,
                    );
                }
            }
            x += 1;
        }
        y = render_workers::next_row(y + 1);
    }
}

fn raster_surface_boundary_cell(
    camera: &FlowCamera,
    time: f32,
    x_scale: f32,
    a: SurfaceVertex,
    b: SurfaceVertex,
    c: SurfaceVertex,
    d: SurfaceVertex,
    visibility: f32,
) {
    let left = floor_i32(a.x.min(c.x)).max(0);
    let right = ceil_i32(b.x.max(d.x)).min(WIDTH as i32);
    let top = floor_i32(a.y.min(b.y)).max(0);
    let bottom = ceil_i32(c.y.max(d.y)).min(HEIGHT as i32);
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let mut y = render_workers::next_row(top);
    while y < bottom {
        let mut x = left;
        while x < right {
            let sample_x = x as f32 + 0.5;
            let sample_y = y as f32 + 0.5;
            let sample = sample_surface_vertex(
                camera,
                time,
                x_scale,
                sample_x,
                sample_y,
            );
            if sample.valid && sample.inverse_depth > 0.0 {
                // Coverage and depth remain exact ray/sphere results. Shade
                // the mixed cell from its surviving mesh vertices so the
                // color field meets adjacent Gouraud triangles continuously
                // instead of tracing a four-pixel sawtooth around the limb.
                let u = ((sample_x - a.x) / (b.x - a.x).max(1.0)).clamp(0.0, 1.0);
                let v = ((sample_y - a.y) / (c.y - a.y).max(1.0)).clamp(0.0, 1.0);
                let vertices = [
                    (a, (1.0 - u) * (1.0 - v)),
                    (b, u * (1.0 - v)),
                    (c, (1.0 - u) * v),
                    (d, u * v),
                ];
                let mut weight = 0.0f32;
                let mut red = 0.0f32;
                let mut green = 0.0f32;
                let mut blue = 0.0f32;
                let mut vertex_index = 0usize;
                while vertex_index < vertices.len() {
                    let (vertex, vertex_weight) = vertices[vertex_index];
                    if vertex.valid {
                        weight += vertex_weight;
                        red += vertex.red * vertex_weight;
                        green += vertex.green * vertex_weight;
                        blue += vertex.blue * vertex_weight;
                    }
                    vertex_index += 1;
                }
                let (red, green, blue) = if weight > 0.000_1 {
                    (red / weight, green / weight, blue / weight)
                } else {
                    (sample.red, sample.green, sample.blue)
                };
                blend_depth_pixel(
                    frame,
                    x,
                    y,
                    1.0 / sample.inverse_depth,
                    red.clamp(0.0, 255.0) as u8,
                    green.clamp(0.0, 255.0) as u8,
                    blue.clamp(0.0, 255.0) as u8,
                    (visibility * 255.0) as u8,
                    0.0,
                );
            }
            x += 1;
        }
        y = render_workers::next_row(y + 1);
    }
}

fn surface_mesh_step() -> i32 {
    SURFACE_CELL_PIXELS
}

fn surface_mesh_bounds(view: ProjectedPlanetBounds, step: i32) -> (i32, i32, i32, i32) {
    (
        (floor_i32(view.x - view.radius_x) - step).max(-step),
        (ceil_i32(view.x + view.radius_x) + step).min(WIDTH as i32 + step),
        (floor_i32(view.y - view.radius_y) - step).max(-step),
        (ceil_i32(view.y + view.radius_y) + step).min(HEIGHT as i32 + step),
    )
}

fn render_polygon_surface_lod(
    camera: &FlowCamera,
    time: f32,
    x_scale: f32,
    view: ProjectedPlanetBounds,
    visibility: f32,
) {
    if visibility <= 0.0 {
        return;
    }
    let step = surface_mesh_step();
    // One stable screen-space lattice owns the surface at every scale. These
    // conservative displaced-sphere bounds only cull cells that cannot hit it;
    // boundary cells still use exact perspective rays, so the bounds do not
    // introduce another coverage representation as the planet grows.
    let (left, right, top, bottom) = surface_mesh_bounds(view, step);
    let columns = ((right - left + step - 1) / step + 1).max(0) as usize;
    if columns < 2 || columns > SURFACE_ROW_CAPACITY || bottom <= top {
        return;
    }

    let row_a = core::ptr::addr_of_mut!(SURFACE_ROW_A).cast::<SurfaceVertex>();
    let row_b = core::ptr::addr_of_mut!(SURFACE_ROW_B).cast::<SurfaceVertex>();
    let mut cached_row=None;
    let mut y = top + step;
    while y <= bottom + step {
        // Partition raster ownership, not geometry. Keep the original lattice
        // coordinates, including the shared vertices on stripe boundaries.
        if render_workers::next_row((y-step).max(0))>=y.min(HEIGHT as i32) {
            y+=step;continue;
        }
        if cached_row!=Some(y-step) {
            for column in 0..columns {
                let x=left+column as i32*step;
                unsafe {row_a.add(column).write(sample_surface_vertex(camera,time,x_scale,x as f32,(y-step) as f32));}
            }
        }
        let mut column = 0;
        while column < columns {
            let x = left + column as i32 * step;
            unsafe {
                row_b.add(column).write(sample_surface_vertex(
                    camera,
                    time,
                    x_scale,
                    x as f32,
                    y as f32,
                ));
            }
            column += 1;
        }
        column = 0;
        while column + 1 < columns {
            let a = unsafe { row_a.add(column).read() };
            let b = unsafe { row_a.add(column + 1).read() };
            let c = unsafe { row_b.add(column).read() };
            let d = unsafe { row_b.add(column + 1).read() };
            if a.valid && b.valid && c.valid && d.valid {
                raster_surface_triangle(a, b, d, visibility);
                raster_surface_triangle(a, d, c, visibility);
            } else {
                // Any non-interior cell can contain a curved surface sliver
                // even when its corners and midpoint miss. Resolve it once at
                // exact pixel centres so the polygon lattice stays watertight.
                raster_surface_boundary_cell(
                    camera, time, x_scale, a, b, c, d, visibility,
                );
            }
            column += 1;
        }
        column = 0;
        while column < columns {
            unsafe { row_a.add(column).write(row_b.add(column).read()) };
            column += 1;
        }
        cached_row=Some(y);
        y += step;
    }
}

fn projected_surface_bounds(
    camera: &FlowCamera,
    x_scale: f32,
) -> ProjectedPlanetBounds {
    let center = dvec_sub(planet_center(), camera.position);
    let camera_x = dvec_dot_vec3(center, camera.right);
    let camera_y = dvec_dot_vec3(center, camera.down);
    let camera_z = dvec_dot_vec3(center, camera.forward);
    let radius = FLOW_PLANET_RADIUS_WORLD + miles_to_world(FLOW_MAX_RELIEF_MILES);
    let distance = dvec_length(center);
    let axis_bounds = |axis: f64| {
        let denominator = camera_z * camera_z - radius * radius;
        let tangent_plane = axis * axis + camera_z * camera_z - radius * radius;
        if denominator <= 1.0e-15 || tangent_plane <= 0.0 {
            return None;
        }
        let root = radius * sqrt_f64(tangent_plane);
        let first = (axis * camera_z - root) / denominator;
        let second = (axis * camera_z + root) / denominator;
        Some((first.min(second), first.max(second)))
    };
    let fallback_bounds = || {
        (
            160.0,
            WIDTH as f32 * 4.0 * x_scale,
            100.0,
            HEIGHT as f32 * 4.0,
        )
    };
    let (x, radius_x, y, radius_y) = if distance <= radius * 1.001 {
        fallback_bounds()
    } else {
        let horizontal = axis_bounds(camera_x);
        let vertical = axis_bounds(camera_y);
        match (horizontal, vertical) {
            (Some(horizontal), Some(vertical)) => {
                let left = 160.0 + horizontal.0 as f32 * FLOW_FOCAL * x_scale;
                let right = 160.0 + horizontal.1 as f32 * FLOW_FOCAL * x_scale;
                let top = 100.0 + vertical.0 as f32 * FLOW_FOCAL;
                let bottom = 100.0 + vertical.1 as f32 * FLOW_FOCAL;
                (
                    (left + right) * 0.5,
                    (right - left).abs() * 0.5,
                    (top + bottom) * 0.5,
                    (bottom - top).abs() * 0.5,
                )
            }
            _ => fallback_bounds(),
        }
    };
    let visible = if camera_z + radius <= camera.near_plane {
        0.0
    } else {
        smootherstep(radius_y / 2.0)
    };
    ProjectedPlanetBounds {
        visible,
        x,
        y,
        radius_x,
        radius_y,
    }
}

fn render_flow_surface(
    camera: &FlowCamera,
    time: f32,
    x_scale: f32,
) {
    let view = projected_surface_bounds(camera, x_scale);
    render_polygon_surface_lod(camera, time, x_scale, view, view.visible);
}

fn draw_entry_sheath(camera: &FlowCamera, intensity: f32) {
    if intensity <= 0.0 {
        return;
    }
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let ground_normal = flow_normal_to_planet(dvec_normalize(dvec_sub(
        camera.position,
        planet_center(),
    )));
    let ground_map = flow_surface_map_from_normal(ground_normal);
    let flow = (ground_map.0 * 31.0
        + ground_map.1 * 17.0
        + camera.speed_world_per_second as f32 * 24_000.0) as i32;
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
                let mut block_y = y;
                while block_y < (y + 2).min(HEIGHT) {
                    let mut block_x = x;
                    while block_x < (x + 2).min(WIDTH) {
                        blend_pixel(
                            frame,
                            block_x as i32,
                            block_y as i32,
                            (58.0 + hue * 112.0 + filament * 42.0) as u8,
                            (116.0 + hue * 92.0 + filament * 24.0) as u8,
                            255,
                            alpha.min(78.0) as u8,
                        );
                        block_x += 1;
                    }
                    block_y += 1;
                }
            }
            x += 2;
        }
        y += 2;
    }

}

fn flow_sun_direction() -> Vec3 {
    universe().sun_direction
}

fn flow_sun_projection(camera: &FlowCamera, x_scale: f32) -> Option<(f32, f32)> {
    let direction = flow_sun_direction();
    if flow_sphere_roots(camera, dvec_from_vec3(direction), FLOW_PLANET_RADIUS_WORLD).is_some() {
        return None;
    }
    let depth = vec_dot(direction, camera.forward);
    if depth <= 0.08 {
        return None;
    }
    Some((
        160.0 + vec_dot(direction, camera.right) / depth * FLOW_FOCAL * x_scale,
        100.0 + vec_dot(direction, camera.down) / depth * FLOW_FOCAL,
    ))
}

fn draw_flow_sun(camera: &FlowCamera, amount: f32, x_scale: f32) {
    if let Some((x, y)) = flow_sun_projection(camera, x_scale) {
        let cloud = cloud_ray_amount(camera, dvec_from_vec3(flow_sun_direction()), f64::MAX);
        draw_sun(x, y, amount * (1.0 - cloud));
    }
}

fn flow_atmosphere_amount(camera: &FlowCamera) -> f32 {
    let altitude_km = camera.altitude_miles * METERS_PER_MILE / 1_000.0;
    smootherstep_f64((100.0 - altitude_km) / 82.0) as f32
}

fn universe_space_visibility(camera: &FlowCamera) -> f32 {
    1.0 - flow_atmosphere_amount(camera)
}

fn flow_entry_heat(camera: &FlowCamera) -> f32 {
    let altitude_km = camera.altitude_miles * METERS_PER_MILE / 1_000.0;
    let density_window = smootherstep_f64((100.0 - altitude_km) / 55.0)
        * smootherstep_f64((altitude_km - 8.0) / 25.0);
    let speed = smootherstep_f64((camera.speed_world_per_second - 0.002) / 0.012);
    (density_window * speed) as f32
}

fn atmosphere_ray_bounds(
    camera: &FlowCamera,
    direction: DVec3,
) -> Option<(f64, f64)> {
    let entry = flow_atmosphere_amount(camera);
    if entry <= 0.0 {
        return None;
    }
    // Atmospheric integration begins at the camera, not at the geometry near
    // plane.  During low flight the remaining shell above the camera is much
    // thinner than the near clip distance.
    let universe = universe();
    let local = dvec_sub(camera.position, universe.planet_center);
    let b = dvec_dot(local, direction);
    let c = dvec_dot(local, local)
        - universe.atmosphere_radius * universe.atmosphere_radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }
    let root = sqrt_f64(discriminant);
    let start = (-b - root).max(0.0);
    let mut end = -b + root;
    // The atmosphere ends at solid ground, not at the shell on the far
    // side of the planet. Surface depth can shorten this interval further.
    if let Some((ground, _)) = flow_sphere_roots(camera, direction, FLOW_PLANET_RADIUS_WORLD) {
        if ground > start { end = end.min(ground); }
    }
    if end <= start {
        return None;
    }
    Some((start, end))
}

fn atmosphere_ray_segment(camera: &FlowCamera, direction: DVec3) -> Option<(f64, f64, f32)> {
    let (start, end) = atmosphere_ray_bounds(camera, direction)?;
    Some((start, end, atmosphere_optical_depth(camera, direction, start, end)))
}

fn atmosphere_optical_depth(camera: &FlowCamera, direction: DVec3, start: f64, end: f64) -> f32 {
    if end <= start { return 0.0; }
    // Eight-point quadrature of the density on THIS visible interval. A
    // fraction of a full-column average is incorrect for short terrain rays.
    let midpoint = (start + end) * 0.5;
    let half = (end - start) * 0.5;
    let mut density = 0.0;
    for (node, weight) in [(0.183_434_642_495_649_8, 0.362_683_783_378_362),
        (0.525_532_409_916_329, 0.313_706_645_877_887_3),
        (0.796_666_477_413_626_7, 0.222_381_034_453_374_5),
        (0.960_289_856_497_536_3, 0.101_228_536_290_376_3)] {
        for sign in [-1.0, 1.0] {
            let point = dvec_add(camera.position, dvec_scale(direction, midpoint + sign * half * node));
            let height_km = world_to_meters((dvec_length(dvec_sub(point, planet_center()))
                - FLOW_PLANET_RADIUS_WORLD).max(0.0)) / 1_000.0;
            density += weight * atmosphere_exp(-height_km / 8.0);
        }
    }
    (world_to_meters(half) / 1_000.0 * density * 0.18) as f32
}

fn atmosphere_exp(value: f64) -> f64 {
    // Range-reduced radiative exponential. Ten terms are sufficient for
    // <1e-8 relative error on the bounded optical range, audited against
    // the longer braking implementation. No geometry/detail is discarded.
    let exponent = (value / core::f64::consts::LN_2) as i32;
    let remainder = value - exponent as f64 * core::f64::consts::LN_2;
    let mut term = 1.0;
    let mut sum = 1.0;
    let mut index = 1;
    while index <= 10 {
        term *= remainder / index as f64;
        sum += term;
        index += 1;
    }
    sum * f64::from_bits(((exponent + 1_023) as u64) << 52)
}

fn atmosphere_scatter_color(camera: &FlowCamera, direction: DVec3) -> (f32, f32, f32) {
    let up = dvec_normalize(dvec_sub(camera.position, planet_center()));
    let horizon = 1.0 - smoothstep(dvec_dot(up, direction).abs() as f32 / 0.5);
    let sunlight = dvec_dot_vec3(up, flow_sun_direction()) as f32;
    let day = smoothstep((sunlight + 0.10) / 0.40);
    let sunward = smoothstep((dvec_dot_vec3(direction, flow_sun_direction()) as f32 - 0.70) / 0.30);
    (8.0 + day * (39.0 + horizon * 44.0 + sunward * 18.0),
     15.0 + day * (104.0 + horizon * 39.0 + sunward * 12.0),
     35.0 + day * (169.0 + horizon * 8.0))
}

fn cloud_density(normal: Vec3, footprint_miles: f64) -> f32 {
    let x = (normal.x * 0.73 + normal.y * 0.37 - normal.z * 0.55) * TERRAIN_CHART_UNITS_PER_RADIAN as f32;
    let y = (normal.x * -0.44 + normal.y * 0.81 + normal.z * 0.29) * TERRAIN_CHART_UNITS_PER_RADIAN as f32;
    let broad = landform_noise(x, y, 160.0, 10_831)
        * surface_lod_weight(160.0, footprint_miles);
    let detail = landform_noise(x, y, 12.0, 12_871)
        * surface_lod_weight(12.0, footprint_miles);
    smoothstep((broad + detail * 0.3 - 0.35) / 0.5) * 0.12
}

fn cloud_ray_amount(camera: &FlowCamera, direction: DVec3, limit_distance: f64) -> f32 {
    let radius = FLOW_PLANET_RADIUS_WORLD + miles_to_world(2_000.0 / METERS_PER_MILE);
    let Some((near, far)) = flow_sphere_roots(camera, direction, radius) else { return 0.0; };
    let distance = if dvec_length(dvec_sub(camera.position, planet_center())) < radius { far } else { near };
    if distance >= limit_distance { return 0.0; }
    let point = dvec_add(camera.position, dvec_scale(direction, distance));
    let normal = flow_normal_to_planet(dvec_normalize(dvec_sub(point, planet_center())));
    cloud_density(normal, world_to_miles(distance) / FLOW_FOCAL as f64)
}

fn atmospheric_alpha(optical_depth: f32) -> f32 {
    1.0 - atmosphere_exp(-(optical_depth.clamp(0.0, 24.0) as f64)) as f32
}

fn atmospheric_alpha_to_depth(
    camera: &FlowCamera,
    direction: DVec3,
    camera_depth: f32,
) -> f32 {
    let Some((start, end)) = atmosphere_ray_bounds(camera, direction) else { return 0.0; };
    let ray_forward = dvec_dot_vec3(direction, camera.forward);
    let visible_end = if camera_depth < f32::MAX {
        end.min(camera_depth as f64 / ray_forward.max(0.001))
    } else {
        end
    };
    if visible_end <= start {
        return 0.0;
    }
    atmospheric_alpha(atmosphere_optical_depth(camera, direction, start, visible_end))
}

fn apply_atmospheric_shell(camera: &FlowCamera, x_scale: f32) {
    if flow_atmosphere_amount(camera) <= 0.0 {
        return;
    }
    const TILE: usize = 4;
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let depth_buffer = core::ptr::addr_of!(DEPTH).cast::<f32>();
    let mut y = render_workers::next_row(0) as usize;
    while y < HEIGHT {
        let mut x = 0usize;
        while x < WIDTH {
            let center_x = (x + TILE / 2).min(WIDTH - 1) as f32 + 0.5;
            let center_y = (y + TILE / 2).min(HEIGHT - 1) as f32 + 0.5;
            let direction = flow_camera_ray(camera, x_scale, center_x, center_y);
            if let Some((_, _, full_optical_depth)) =
                atmosphere_ray_segment(camera, direction)
            {
                let sky_alpha = atmospheric_alpha(full_optical_depth);
                let (red, green, blue) = atmosphere_scatter_color(camera, direction);
                let sky_cloud = cloud_ray_amount(camera, direction, f64::MAX);
                let mut tile_y = y;
                while tile_y < (y + TILE).min(HEIGHT) {
                    let mut tile_x = x;
                    while tile_x < (x + TILE).min(WIDTH) {
                        let stored_depth = unsafe {
                            depth_buffer.add(tile_y * WIDTH + tile_x).read()
                        };
                        let optical_alpha = if stored_depth == f32::MAX { sky_alpha }
                            else { atmospheric_alpha_to_depth(camera, direction, stored_depth) };
                        if optical_alpha > 0.0 {
                            let alpha = optical_alpha * 255.0;
                            blend_pixel(
                                frame,
                                tile_x as i32,
                                tile_y as i32,
                                red as u8,
                                green as u8,
                                blue as u8,
                                alpha.clamp(0.0, 255.0) as u8,
                            );
                        }
                        if stored_depth == f32::MAX && sky_cloud > 0.0 {
                            blend_pixel(frame, tile_x as i32, tile_y as i32,
                                215, 223, 232, (sky_cloud * 255.0) as u8);
                        }
                        tile_x += 1;
                    }
                    tile_y += 1;
                }
            }
            x += TILE;
        }
        y = render_workers::next_row((y+TILE) as i32) as usize;
    }
}

fn render_world_journey(
    elapsed: f32,
    x_scale: f32,
    flow: &FlowCamera,
) {
    #[cfg(feature="phase0-audit")] let start=process_cpu_ns();
    apply_atmospheric_shell(flow, x_scale);
    #[cfg(feature="phase0-audit")] let atmosphere_end=process_cpu_ns();
    draw_flow_sun(flow, 1.0, x_scale);
    #[cfg(feature="phase0-audit")] let sun_end=process_cpu_ns();
    render_flow_surface(flow, elapsed, x_scale);
    #[cfg(feature="phase0-audit")] unsafe {
        FRAME_STAGE_NS[1]=atmosphere_end-start;
        FRAME_STAGE_NS[2]=sun_end-atmosphere_end;
        FRAME_STAGE_NS[3]=process_cpu_ns()-sun_end;
    }
    draw_entry_sheath(flow, flow_entry_heat(flow));
}

fn fill_universe_vacuum() {
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let mut pixel = 0usize;
    while pixel < PIXELS {
        unsafe {
            frame.add(pixel * 3).write(2);
            frame.add(pixel * 3 + 1).write(3);
            frame.add(pixel * 3 + 2).write(12);
        }
        pixel += 1;
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
    visibility: f32,
) {
    if hash(index, 1_201) < 249 {
        return;
    }
    let intensity = (0.65 + hash(index, 1_819) as f32 / 255.0 * 0.35) * visibility;
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

fn draw_background_star(
    frame: *mut u8,
    center: (f32, f32),
    seed: i32,
    visibility: f32,
) {
    let x = center.0 as i32;
    let y = center.1 as i32;
    let luminosity = 0.38 + hash(seed, 7_193) as f32 / 255.0 * 0.62;
    let temperature = stellar_temperature(seed);
    let alpha = (visibility * luminosity * 176.0) as u8;
    blend_pixel(
        frame,
        x,
        y,
        (temperature.0 * 214.0) as u8,
        (temperature.1 * 214.0) as u8,
        (temperature.2 * 214.0) as u8,
        alpha,
    );
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
    let mut y = render_workers::next_row(top);
    while y <= bottom {
        let sample_y = y as f32 + 0.5;
        let mut x = left;
        while x <= right {
            let sample_x = x as f32 + 0.5;
            let projection = (((sample_x - start.0) * dx
                + (sample_y - start.1) * dy)
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
        y = render_workers::next_row(y + 1);
    }
}

#[cfg(feature = "phase0-audit")]
#[derive(Clone, Copy)]
struct FieldRenderStats {
    star_points: u32,
    star_streaks: u32,
    ring_edges: u32,
    spoke_edges: u32,
}

#[cfg(feature = "phase0-audit")]
const EMPTY_FIELD_RENDER_STATS: FieldRenderStats = FieldRenderStats {
    star_points: 0, star_streaks: 0, ring_edges: 0, spoke_edges: 0,
};

#[cfg(feature = "phase0-audit")]
static mut FIELD_RENDER_STATS: FieldRenderStats = EMPTY_FIELD_RENDER_STATS;

fn render_universe_field(
    exposure_start: &FlowCamera,
    flow: &FlowCamera,
    x_scale: f32,
) {
    if field_snapshot::replay() {return;}
    #[cfg(feature = "phase0-audit")]
    unsafe { FIELD_RENDER_STATS = EMPTY_FIELD_RENDER_STATS; }
    fill_universe_vacuum();
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let universe = universe();
    let layout = universe.field;
    let space_visibility = universe_space_visibility(flow);
    if space_visibility <= 0.0 {
        return;
    }
    let neon = 0.0;
    let grid_visibility = space_visibility;
    let local_star_visibility =
        space_visibility * local_star_field_visibility(layout, flow.position);
    let background_star_visibility =
        space_visibility * field_exterior_visibility(layout, flow.position);
    if background_star_visibility > 0.0 {
        let mut background_ordinal = 0u16;
        while background_ordinal < BACKGROUND_STAR_COUNT {
            let identity = BackgroundStarIdentity {
                ordinal: background_ordinal,
            };
            let point = background_star_position(universe, identity);
            if let Some(projected) = project_flow_depth(flow, point) {
                let projected = (
                    160.0 + (projected.0 - 160.0) * x_scale,
                    projected.1,
                );
                if projected.0 >= 0.0
                    && projected.0 < WIDTH as f32
                    && projected.1 >= 0.0
                    && projected.1 < HEIGHT as f32
                {
                    draw_background_star(
                        frame,
                        projected,
                        background_ordinal as i32,
                        background_star_visibility,
                    );
                }
            }
            background_ordinal += 1;
        }
    }
    let star_layout = STAR_CATALOGUE;
    let mut star_cell_ordinal = 0u32;
    while local_star_visibility > 0.0 && star_cell_ordinal < STAR_CELL_COUNT {
        let cell = star_cell_identity(star_cell_ordinal);
        if star_cell_may_project(star_layout, cell, exposure_start)
            || star_cell_may_project(star_layout, cell, flow)
        {
            let mut object = 0u8;
            while object < FIELD_STAR_OBJECTS_PER_CELL {
                let star = StarObjectIdentity { cell, object };
                if star_object_exists(star_layout, star) {
                    let seed = star_object_seed(star);
                    let point = star_object_position(star_layout, star);
                    let object_visibility =
                        local_star_object_visibility(star, flow.position, point);
                    if let Some((previous, current)) = project_field_star_exposure(
                        star_layout,
                        star,
                        exposure_start,
                        flow,
                        x_scale,
                    ) {
                        let previous_point = (previous.0, previous.1);
                        let current_point = (current.0, current.1);
                        let luminosity = hash(seed, 149) as f32 / 255.0;
                        let color = lit_mesh_color(
                            seed,
                            seed,
                            neon,
                            92.0 + luminosity * 150.0,
                        );
                        let alpha =
                            ((138.0 + luminosity * 104.0)
                                * local_star_visibility
                                * object_visibility) as u8;
                        if alpha > 0 {
                            if let Some((visible_previous, visible_current)) =
                                clip_screen_line(previous_point, current_point, 6.0)
                            {
                                #[cfg(feature = "phase0-audit")]
                                unsafe {
                                    if visible_previous == visible_current {
                                        FIELD_RENDER_STATS.star_points += 1;
                                    } else {
                                        FIELD_RENDER_STATS.star_streaks += 1;
                                    }
                                }
                                draw_streak_line(
                                    frame,
                                    visible_previous,
                                    visible_current,
                                    color,
                                    alpha,
                                    (18.0 * space_visibility) as u8,
                                );
                                record_depth_line(
                                    visible_previous,
                                    visible_current,
                                    current.2,
                                    current.2,
                                );
                                if let Some(actual_current) = project_flow_depth(flow, point) {
                                    let actual_current = (
                                        160.0 + (actual_current.0 - 160.0) * x_scale,
                                        actual_current.1,
                                    );
                                    if actual_current.0 >= -6.0
                                        && actual_current.0 < WIDTH as f32 + 6.0
                                        && actual_current.1 >= -6.0
                                        && actual_current.1 < HEIGHT as f32 + 6.0
                                    {
                                        draw_star_sparkle(
                                            frame,
                                            actual_current,
                                            color,
                                            seed,
                                            local_star_visibility * object_visibility,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                object += 1;
            }
        }
        star_cell_ordinal += 1;
    }
    let mut ordinal = field_first_physical_ring_ordinal(layout);
    let last_ordinal = field_last_ring_ordinal(layout);
    while ordinal <= last_ordinal {
        let ring = field_ring_identity(ordinal);
        let s = field_ring_s(layout, ring);
        if s > layout.field_end_s {
            ordinal += 1;
            continue;
        }
        let region_neon = field_square_amount(layout, s) as f32;
        let distance_visibility = field_mesh_visibility(layout, s, flow);
        let mesh_visibility = grid_visibility * distance_visibility;
        let mut lane = 0usize;
        while lane < GRID_LANES - 1 {
            let mesh_seed = ordinal
                .wrapping_mul(1_009)
                .wrapping_add(lane as i32 * 313);
            if mesh_visibility > 0.0 {
                let ring_object = RingObjectIdentity { ring };
                let ring_points = ring_object_segment(layout, ring_object, lane as u16);
                if let Some((a, b)) =
                    project_field_segment(ring_points.0, ring_points.1, flow, x_scale)
                {
                    let luminosity = hash(mesh_seed, 149) as f32 / 255.0;
                    let color = lit_mesh_color(
                        mesh_seed,
                        mesh_accent(-14.0 + lane as f32 * 28.0 / (GRID_LANES - 1) as f32),
                        region_neon,
                        82.0 + luminosity * 142.0,
                    );
                    let alpha = ((142.0 + luminosity * 82.0) * mesh_visibility) as u8;
                    if alpha > 0 {
                        #[cfg(feature = "phase0-audit")]
                        if clip_screen_line((a.0, a.1), (b.0, b.1), 3.0).is_some() {
                            unsafe { FIELD_RENDER_STATS.ring_edges += 1; }
                        }
                        field_snapshot::edge(a,b,color,alpha,12);
                    }
                }
                if ordinal < last_ordinal {
                    let rail = RailObjectIdentity {
                        first_ring: ring,
                        lane: lane as u16,
                    };
                    if rail_object_exists(layout, rail) {
                        let axial_points = rail_object_segment(layout, rail);
                        if let Some((a, b)) =
                            project_field_segment(axial_points.0, axial_points.1, flow, x_scale)
                        {
                            let luminosity = hash(mesh_seed, 503) as f32 / 255.0;
                            let color = lit_mesh_color(
                                mesh_seed,
                                mesh_accent(-14.0 + lane as f32 * 28.0 / (GRID_LANES - 1) as f32),
                                region_neon,
                                72.0 + luminosity * 138.0,
                            );
                            let alpha =
                                ((132.0 + luminosity * 86.0) * mesh_visibility) as u8;
                            if alpha > 0 {
                                #[cfg(feature = "phase0-audit")]
                                if clip_screen_line((a.0, a.1), (b.0, b.1), 3.0).is_some() {
                                    unsafe { FIELD_RENDER_STATS.spoke_edges += 1; }
                                }
                                field_snapshot::edge(a,b,color,alpha,10);
                            }
                        }
                    }
                }
            }
            lane += 1;
        }
        ordinal += 1;
    }
}

fn clip_line_parameter(p: f64, q: f64, enter: &mut f64, leave: &mut f64) -> bool {
    if p == 0.0 {
        return q >= 0.0;
    }
    let crossing = q / p;
    if p < 0.0 {
        if crossing > *leave {
            return false;
        }
        if crossing > *enter {
            *enter = crossing;
        }
    } else {
        if crossing < *enter {
            return false;
        }
        if crossing < *leave {
            *leave = crossing;
        }
    }
    true
}

fn clip_screen_line(
    start: (f32, f32),
    end: (f32, f32),
    margin: f32,
) -> Option<((f32, f32), (f32, f32))> {
    let x0 = start.0 as f64;
    let y0 = start.1 as f64;
    let dx = end.0 as f64 - x0;
    let dy = end.1 as f64 - y0;
    let margin = margin as f64;
    let mut enter = 0.0;
    let mut leave = 1.0;
    if !clip_line_parameter(-dx, x0 + margin, &mut enter, &mut leave)
        || !clip_line_parameter(
            dx,
            WIDTH as f64 - 1.0 + margin - x0,
            &mut enter,
            &mut leave,
        )
        || !clip_line_parameter(-dy, y0 + margin, &mut enter, &mut leave)
        || !clip_line_parameter(
            dy,
            HEIGHT as f64 - 1.0 + margin - y0,
            &mut enter,
            &mut leave,
        )
    {
        return None;
    }
    Some((
        ((x0 + dx * enter) as f32, (y0 + dy * enter) as f32),
        ((x0 + dx * leave) as f32, (y0 + dy * leave) as f32),
    ))
}

fn clear_depth_buffer() {
    let depth = core::ptr::addr_of_mut!(DEPTH).cast::<f32>();
    let mut pixel = 0usize;
    while pixel < PIXELS {
        unsafe { depth.add(pixel).write(f32::MAX) };
        pixel += 1;
    }
}

fn record_depth_line(start: (f32, f32), end: (f32, f32), depth_a: f32, depth_b: f32) {
    if depth_a <= 0.0 || depth_b <= 0.0 || !depth_a.is_finite() || !depth_b.is_finite() {
        return;
    }
    let x0 = start.0 as f64;
    let y0 = start.1 as f64;
    let dx = end.0 as f64 - x0;
    let dy = end.1 as f64 - y0;
    let mut enter = 0.0;
    let mut leave = 1.0;
    if !clip_line_parameter(-dx, x0, &mut enter, &mut leave)
        || !clip_line_parameter(dx, WIDTH as f64 - 1.0 - x0, &mut enter, &mut leave)
        || !clip_line_parameter(-dy, y0, &mut enter, &mut leave)
        || !clip_line_parameter(dy, HEIGHT as f64 - 1.0 - y0, &mut enter, &mut leave)
    {
        return;
    }
    let clipped_start = (x0 + dx * enter, y0 + dy * enter);
    let clipped_end = (x0 + dx * leave, y0 + dy * leave);
    let clipped_dx = clipped_end.0 - clipped_start.0;
    let clipped_dy = clipped_end.1 - clipped_start.1;
    let steps = (clipped_dx.abs().max(clipped_dy.abs()) as i32).max(1);
    let inverse_a = 1.0 / depth_a;
    let inverse_b = 1.0 / depth_b;
    let clipped_inverse_a = inverse_a + (inverse_b - inverse_a) * enter as f32;
    let clipped_inverse_b = inverse_a + (inverse_b - inverse_a) * leave as f32;
    let buffer = core::ptr::addr_of_mut!(DEPTH).cast::<f32>();
    let mut step = 0i32;
    while step <= steps {
        let amount = step as f32 / steps as f32;
        let x = (clipped_start.0 + clipped_dx * amount as f64) as i32;
        let y = (clipped_start.1 + clipped_dy * amount as f64) as i32;
        if x >= 0 && y >= 0 && x < WIDTH as i32 && y < HEIGHT as i32 {
            let inverse_depth =
                clipped_inverse_a + (clipped_inverse_b - clipped_inverse_a) * amount;
            if inverse_depth > 0.0 {
                let depth = 1.0 / inverse_depth;
                let index = y as usize * WIDTH + x as usize;
                let stored = unsafe { buffer.add(index).read() };
                if depth < stored {
                    unsafe { buffer.add(index).write(depth) };
                }
            }
        }
        step += 1;
    }
}

fn blend_depth_pixel(
    frame: *mut u8,
    x: i32,
    y: i32,
    depth: f32,
    r: u8,
    g: u8,
    b: u8,
    alpha: u8,
    depth_bias: f32,
) {
    if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 || !depth.is_finite() {
        return;
    }
    let index = y as usize * WIDTH + x as usize;
    let buffer = core::ptr::addr_of_mut!(DEPTH).cast::<f32>();
    let stored = unsafe { buffer.add(index).read() };
    if depth <= stored + depth_bias {
        if depth < stored {
            unsafe { buffer.add(index).write(depth) };
        }
        blend_pixel(frame, x, y, r, g, b, alpha);
    }
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
    let mut y = render_workers::next_row(top);
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
                blend_depth_pixel(
                    frame,
                    x,
                    y,
                    f32::MAX,
                    255,
                    (139.0 + core * 98.0) as u8,
                    (60.0 + core * 180.0) as u8,
                    alpha.min(255.0) as u8,
                    0.0,
                );
            }
            x += 1;
        }
        y = render_workers::next_row(y + 1);
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

fn projection_x_scale(window: WinSize) -> f32 {
    let (physical_width, physical_height) = if window.pixel_width > 0 && window.pixel_height > 0 {
        (window.pixel_width as f32, window.pixel_height as f32)
    } else {
        (window.columns.max(1) as f32, window.rows.max(1) as f32 * 2.0)
    };
    physical_height * WIDTH as f32 / (physical_width * HEIGHT as f32)
}

fn read_escape_sequence_byte() -> Option<u8> {
    let mut attempts = 0usize;
    while attempts < 20 {
        let mut byte = 0u8;
        if syscall3(SYS_READ, 0, addr_mut(&mut byte), 1) > 0 {
            return Some(byte);
        }
        sleep_ns(1_000_000);
        attempts += 1;
    }
    None
}

fn apply_arrow_direction(direction: u8, elapsed: &mut f32) {
    match direction {
        b'C' => *elapsed += 2.0,
        b'D' => *elapsed = (*elapsed - 2.0).max(0.0),
        _ => {}
    }
}

fn apply_control_byte(byte: u8, playback: &mut Playback) -> bool {
    if byte == b'q' || byte == b'Q' || byte == 3 {
        return true;
    }
    match byte {
        b'1' => playback.elapsed = 0.0,
        b'2' => playback.elapsed = GRID_SQUARE_START,
        b'3' => playback.elapsed = GRID_HORIZONTAL,
        b'4' => playback.elapsed = ORBIT_SEEK,
        b'5' => playback.elapsed = CONTINENT_SEEK,
        b'6' => playback.elapsed = DESCENT,
        b' ' => playback.paused = !playback.paused,
        b']' | b'.' | b'>' | b'f' | b'F' => playback.elapsed += 2.0,
        b'[' | b',' | b'<' | b'b' | b'B' => playback.elapsed = (playback.elapsed - 2.0).max(0.0),
        _ => {}
    }
    false
}

fn handle_input(playback: &mut Playback) -> bool {
    loop {
        let mut byte = 0u8;
        if syscall3(SYS_READ, 0, addr_mut(&mut byte), 1) <= 0 {
            return false;
        }
        if byte == 27 {
            let Some(prefix) = read_escape_sequence_byte() else { return true };
            if prefix != b'[' && prefix != b'O' {
                return true;
            }
            let Some(direction) = read_escape_sequence_byte() else { return true };
            apply_arrow_direction(direction, &mut playback.elapsed);
            continue;
        }
        if apply_control_byte(byte, playback) {
            return true;
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

#[cfg(feature = "phase0-audit")]
fn process_cpu_ns() -> u64 {
    let mut time = TimeSpec { seconds: 0, nanos: 0 };
    if syscall2(SYS_CLOCK_GETTIME, 2, addr_mut(&mut time)) < 0 {
        return 0;
    }
    (time.seconds as u64)
        .saturating_mul(1_000_000_000)
        .saturating_add(time.nanos as u64)
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
    render_workers::shutdown();
    unsafe {
        asm!(
            "syscall",
            in("rax") SYS_EXIT,
            in("rdi") code,
            options(noreturn)
        )
    }
}
