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
const GRID_SQUARE_START: f32 = 21.0;
const GRID_HORIZONTAL: f32 = 29.0;
const GRID_COLOR_FULL: f32 = GRID_HORIZONTAL - 2.0;
const ORBIT_SEEK: f32 = 44.0;
const CONTINENT_SEEK: f32 = 90.0;
const DESCENT: f32 = 113.0;
const PLANET_INTRO: f32 = 27.0;
const APPROACH_SPEED: f32 = 0.28;
const ORBIT_ENTRY: f32 = 28.0;
const FLOW_FOCAL: f32 = 154.0;
pub const METERS_PER_MILE: f64 = 1_609.344;
const PLANET_RADIUS_MILES: f64 = 13_500.0;
const CLOSE_ORBIT_ALTITUDE_MILES: f64 = 100.0;
pub const FINAL_ALTITUDE_METERS: f64 = 20.0;
const FLOW_PLANET_RADIUS_WORLD: f64 = 0.115;
const FLOW_MAX_RELIEF_MILES: f64 = 12.0;
const GRID_BOTTOM_HEIGHT_WORLD: f64 = FLOW_PLANET_RADIUS_WORLD * 22.0;
const GRID_WELL_RADIUS_WORLD: f64 = FLOW_PLANET_RADIUS_WORLD * 5.5;
const GRID_WELL_DEPTH_WORLD: f64 = FLOW_PLANET_RADIUS_WORLD * 2.0;
const GRID_APPROACH_AXIS_OFFSET: f64 = GRID_BOTTOM_HEIGHT_WORLD + GRID_WELL_DEPTH_WORLD;
const FLOW_ATMOSPHERE_HEIGHT_MILES: f64 = 100_000.0 / METERS_PER_MILE;
const FLOW_ATMOSPHERE_RADIUS_WORLD: f64 =
    FLOW_PLANET_RADIUS_WORLD + FLOW_ATMOSPHERE_HEIGHT_MILES * WORLD_UNITS_PER_MILE;
pub const WORLD_UNITS_PER_MILE: f64 = FLOW_PLANET_RADIUS_WORLD / PLANET_RADIUS_MILES;
pub const FLOW_CLOSE_ORBIT_RADIUS_WORLD: f64 =
    FLOW_PLANET_RADIUS_WORLD + CLOSE_ORBIT_ALTITUDE_MILES * WORLD_UNITS_PER_MILE;
pub const FLOW_FINAL_ALTITUDE_WORLD: f64 =
    FINAL_ALTITUDE_METERS / METERS_PER_MILE * WORLD_UNITS_PER_MILE;
pub const FLOW_DESCENT_START: f64 = 35.0;
const FLOW_BRAKE_CONTROL: f64 = 38.0;
pub const FLOW_ORBIT_TWO_END: f64 = 42.0;
const FLOW_ORBIT_THREE_END: f64 = 60.0;
const FLOW_HIGH_PASS_END: f64 = 90.0;
const FLOW_DESCENT_END: f64 = 116.0;
pub const FLOW_PATH_END: f64 = 137.0;
const FLOW_VALLEY_OVERHEAD: f64 = 98.0;
const FLOW_VALLEY_CORRIDOR: f64 = 108.0;
const FLOW_AIRPLANE_ALTITUDE_MILES: f64 = 30_000.0 / METERS_PER_MILE;
const TRAJECTORY_HZ: usize = 120;
const TRAJECTORY_STEP: f64 = 1.0 / TRAJECTORY_HZ as f64;
const TRAJECTORY_TABLE_END: usize = 140;
const TRAJECTORY_SAMPLES: usize = TRAJECTORY_TABLE_END * TRAJECTORY_HZ + 1;

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
static mut LIGHT: [u8; MAP_BYTES] = [0; MAP_BYTES];
static mut TX: [u8; TX_BYTES] = [0; TX_BYTES];
static mut SINES: [f32; 1_024] = [0.0; 1_024];
static mut LANDING_SLOPE_VALUE: f32 = 0.0;

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

#[derive(Clone, Copy, PartialEq)]
struct DVec3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Clone, Copy)]
struct TrajectoryNode {
    clearance: f64,
    clearance_rate: f64,
    phase: f64,
    phase_rate: f64,
    meridian: f64,
    meridian_rate: f64,
    impact: f64,
    impact_rate: f64,
}

const EMPTY_TRAJECTORY_NODE: TrajectoryNode = TrajectoryNode {
    clearance: 0.0,
    clearance_rate: 0.0,
    phase: 0.0,
    phase_rate: 0.0,
    meridian: 0.0,
    meridian_rate: 0.0,
    impact: 0.0,
    impact_rate: 0.0,
};

static mut TRAJECTORY: [TrajectoryNode; TRAJECTORY_SAMPLES] =
    [EMPTY_TRAJECTORY_NODE; TRAJECTORY_SAMPLES];

#[derive(Clone, Copy)]
struct TrajectoryState {
    position: DVec3,
    velocity: DVec3,
    acceleration: DVec3,
    forward: Vec3,
    down: Vec3,
    roll: f64,
    #[cfg(feature = "phase0-audit")]
    clearance_miles: f64,
    phase: f64,
    phase_rate: f64,
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
struct FlowCamera {
    position: DVec3,
    forward: Vec3,
    right: Vec3,
    down: Vec3,
    altitude_miles: f64,
    speed_world_per_second: f64,
    near_plane: f64,
}

#[derive(Clone, Copy)]
struct SurfaceLod {
    continent: f32,
    ranges: f32,
    mountains: f32,
    valley: f32,
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
const SUN_DIRECTION: Vec3 = Vec3 { x: 0.550, y: -0.250, z: -0.797 };
const PLANET_CENTER: DVec3 = DVec3 { x: 0.0, y: 0.0, z: 0.0 };

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    exit(101)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    generate_sines();
    generate_terrain();
    generate_trajectory();

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
    #[cfg(feature = "phase0-audit")]
    if !interactive && audit_input[0] == b'!' {
        run_phase0_motion_audit();
    }
    #[cfg(feature = "phase0-audit")]
    if !interactive && audit_input[0] == b'@' {
        run_phase1_precision_audit();
    }
    #[cfg(feature = "phase0-audit")]
    if !interactive && audit_input[0] == b'#' {
        run_phase2_motion_audit();
    }
    #[cfg(feature = "phase0-audit")]
    if !interactive && audit_input[0] == b'^' {
        run_phase3_unified_camera_audit();
    }
    #[cfg(feature = "phase0-audit")]
    if !interactive && audit_input[0] == b'%' {
        run_phase4_surface_audit();
    }
    #[cfg(feature = "phase0-audit")]
    if !interactive && audit_input[0] == b'&' {
        run_phase5_refinement_audit();
    }
    #[cfg(feature = "phase0-audit")]
    if !interactive && audit_input[0] == b'*' {
        run_phase6_atmosphere_audit();
    }
    #[cfg(feature = "phase0-audit")]
    if !interactive && audit_input[0] == b'+' {
        run_phase7_hardening_audit();
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
        render_demo(demo_elapsed, projection_x_scale(window));

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
    unsafe { core::ptr::addr_of!(LANDING_SLOPE_VALUE).read() }
}

const LOCAL_VALLEY_START_MILES: f32 = 18.0;
const LOCAL_VALLEY_END_MILES: f32 = 24.5;
const LOCAL_VALLEY_SLOPE: f32 = -2.913;
const LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES: f32 = 0.25;
const LOCAL_VALLEY_WALL_HALF_WIDTH_MILES: f32 = 0.75;
const LOCAL_VALLEY_BLEND_HALF_WIDTH_MILES: f32 = 1.0;
const LOCAL_VALLEY_WALL_HEIGHT_UNITS: f32 = 42.0;

fn local_valley_y(x: f32) -> f32 {
    valley_y(LOCAL_VALLEY_START_MILES)
        + (x - LOCAL_VALLEY_START_MILES) * LOCAL_VALLEY_SLOPE
}

fn local_valley_longitudinal_weight(x: f32) -> f32 {
    smootherstep(x - (LOCAL_VALLEY_START_MILES - 1.0))
        * smootherstep((LOCAL_VALLEY_END_MILES + 1.0) - x)
}

fn local_valley_distance(x: f32, y: f32) -> f32 {
    wrapped_delta(y, local_valley_y(x)).abs()
}

fn local_valley_width_scale(x: f32) -> f32 {
    // The regional approach is a broad funnel which narrows into the
    // 1.5-mile local valley. This keeps the continuously curving approach on
    // the valley floor before the 94-second overhead control without making
    // the final low-flight corridor dozens of miles wide.
    1.0 + smootherstep((x - 23.5) / 0.6) * 8.0
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
    unsafe {
        core::ptr::addr_of_mut!(LANDING_SLOPE_VALUE)
            .write(wrapped_delta(valley_y(18.05), valley_y(17.95)) / 0.10);
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

fn render_demo(elapsed: f32, x_scale: f32) {
    let flow = flow_camera(elapsed);
    clear_depth_buffer();
    render_continuous_grid(elapsed, &flow, x_scale);
    render_world_journey(elapsed, x_scale, &flow);
    draw_second_counter(elapsed);
}

// zero and continuous force controls; no later milestone installs a position,
// velocity, camera frame, or alternate path equation.
const GRID_ROW_WORLD_SPACING: f64 = 0.235;
const TRAJECTORY_ENTRY_CONTROL_CLEARANCE: f64 = 6.832_947_425_471_734;
pub const TRAJECTORY_ENTRY_CLEARANCE: f64 = 6.832_841_017_962_964;
const TRAJECTORY_ENTRY_RATE: f64 = -13.258_229_140_114_77;
const TRAJECTORY_ENTRY_MERIDIAN: f64 = -19.695_327_202_909_073;
const TRAJECTORY_CAPTURE_ASYMPTOTE: f64 = 0.140_199_667_262_364_5;
const TRAJECTORY_CAPTURE_CLEARANCE: f64 = 0.3375;
const TRAJECTORY_CAPTURE_RATE: f64 = -0.042_701_732_517_378_695;
const TRAJECTORY_CAPTURE_ACCELERATION: f64 = 0.011_677_464_087_222_941;

fn smootherstep_state(time: f64, start: f64, end: f64) -> (f64, f64, f64) {
    let duration = end - start;
    let progress = (time - start) / duration;
    if progress <= 0.0 {
        return (0.0, 0.0, 0.0);
    }
    if progress >= 1.0 {
        return (1.0, 0.0, 0.0);
    }
    let x2 = progress * progress;
    let one_less = progress - 1.0;
    (
        x2 * progress * (progress * (progress * 6.0 - 15.0) + 10.0),
        30.0 * x2 * one_less * one_less / duration,
        60.0 * progress * (2.0 * x2 - 3.0 * progress + 1.0)
            / (duration * duration),
    )
}

fn hermite_acceleration(
    time: f64,
    start_time: f64,
    end_time: f64,
    start_position: f64,
    end_position: f64,
    start_velocity: f64,
    end_velocity: f64,
    start_acceleration: f64,
    end_acceleration: f64,
) -> f64 {
    if time < start_time || time > end_time {
        return 0.0;
    }
    let duration = end_time - start_time;
    let x = (time - start_time) / duration;
    let a0 = start_position;
    let a1 = start_velocity * duration;
    let a2 = start_acceleration * duration * duration * 0.5;
    let position_residual = end_position - a0 - a1 - a2;
    let velocity_residual = end_velocity * duration - a1 - 2.0 * a2;
    let acceleration_residual = end_acceleration * duration * duration - 2.0 * a2;
    let a3 = 10.0 * position_residual - 4.0 * velocity_residual
        + 0.5 * acceleration_residual;
    let a4 = -15.0 * position_residual + 7.0 * velocity_residual
        - acceleration_residual;
    let a5 = 6.0 * position_residual - 3.0 * velocity_residual
        + 0.5 * acceleration_residual;
    (2.0 * a2 + x * (6.0 * a3 + x * (12.0 * a4 + x * 20.0 * a5)))
        / (duration * duration)
}

fn trajectory_radial_reference(time: f64) -> (f64, f64, f64) {
    let airplane = miles_to_world(FLOW_AIRPLANE_ALTITUDE_MILES);
    let overhead = miles_to_world(1_500.0 / METERS_PER_MILE);
    let corridor = miles_to_world(200.0 / METERS_PER_MILE);
    let endpoint = FLOW_FINAL_ALTITUDE_WORLD;
    let orbit_descent = quintic_component(
        ((time - FLOW_DESCENT_START) / (FLOW_ORBIT_THREE_END - FLOW_DESCENT_START))
            .clamp(0.0, 1.0),
        FLOW_ORBIT_THREE_END - FLOW_DESCENT_START,
        TRAJECTORY_CAPTURE_CLEARANCE,
        airplane,
        TRAJECTORY_CAPTURE_RATE,
        0.0,
        TRAJECTORY_CAPTURE_ACCELERATION,
        0.0,
    );
    let overhead_descent = smootherstep_state(time, FLOW_HIGH_PASS_END, FLOW_VALLEY_OVERHEAD);
    let corridor_descent = smootherstep_state(time, FLOW_VALLEY_OVERHEAD, FLOW_VALLEY_CORRIDOR);
    let final_descent = smootherstep_state(time, FLOW_VALLEY_CORRIDOR, FLOW_DESCENT_END);
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

fn trajectory_acceleration(time: f64, state: TrajectoryNode) -> (f64, f64, f64, f64) {
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

    let capture_spin = smootherstep_state(time, ORBIT_ENTRY as f64, 30.2).0;
    let first_brake = smootherstep_state(time, 34.5, 42.0).0;
    let second_brake = smootherstep_state(time, 42.0, 60.0).0;
    let regional_brake = smootherstep_state(time, 55.0, 65.0).0;
    let full_rate = 196.402;
    let second_rate = 88.650;
    let third_rate = 22.774;
    let regional_rate = 1.363;
    let phase_target = capture_spin
        * (full_rate
            + (second_rate - full_rate) * first_brake
            + (third_rate - second_rate) * second_brake
            + (regional_rate - third_rate) * regional_brake);
    let phase_acceleration = 2.0 * capture_spin * (phase_target - state.phase_rate);

    let meridian_acceleration = hermite_acceleration(
        time,
        FLOW_ORBIT_THREE_END,
        FLOW_DESCENT_END,
        TRAJECTORY_ENTRY_MERIDIAN,
        -6.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );
    let impact_acceleration = hermite_acceleration(
        time,
        ORBIT_ENTRY as f64,
        31.0,
        -GRID_APPROACH_AXIS_OFFSET,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );
    (
        radial_acceleration,
        phase_acceleration,
        meridian_acceleration,
        impact_acceleration,
    )
}

fn trajectory_rk4(time: f64, state: TrajectoryNode, step: f64) -> TrajectoryNode {
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

fn generate_trajectory() {
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
    while index < TRAJECTORY_SAMPLES {
        let time = (index - 1) as f64 * TRAJECTORY_STEP;
        state = trajectory_rk4(time, state, TRAJECTORY_STEP);
        unsafe { core::ptr::addr_of_mut!(TRAJECTORY[index]).write(state) };
        index += 1;
    }
}

fn quintic_component(
    progress: f64,
    duration: f64,
    start: f64,
    end: f64,
    start_velocity: f64,
    end_velocity: f64,
    start_acceleration: f64,
    end_acceleration: f64,
) -> (f64, f64, f64) {
    let a0 = start;
    let a1 = start_velocity * duration;
    let a2 = start_acceleration * duration * duration * 0.5;
    let position_residual = end - a0 - a1 - a2;
    let velocity_residual = end_velocity * duration - a1 - 2.0 * a2;
    let acceleration_residual = end_acceleration * duration * duration - 2.0 * a2;
    let a3 = 10.0 * position_residual - 4.0 * velocity_residual
        + 0.5 * acceleration_residual;
    let a4 = -15.0 * position_residual + 7.0 * velocity_residual
        - acceleration_residual;
    let a5 = 6.0 * position_residual - 3.0 * velocity_residual
        + 0.5 * acceleration_residual;
    let x = progress.clamp(0.0, 1.0);
    (
        a0 + x * (a1 + x * (a2 + x * (a3 + x * (a4 + x * a5)))),
        (a1 + x * (2.0 * a2 + x * (3.0 * a3 + x * (4.0 * a4 + x * 5.0 * a5))))
            / duration,
        (2.0 * a2 + x * (6.0 * a3 + x * (12.0 * a4 + x * 20.0 * a5)))
            / (duration * duration),
    )
}

fn trajectory_scalar_state(time: f64) -> (TrajectoryNode, (f64, f64, f64, f64)) {
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

fn sine_sample_c2_state(index: f64) -> (f64, f64, f64) {
    let mut base = index as i32;
    if index < base as f64 {
        base -= 1;
    }
    let x = index - base as f64;
    let value = |sample: i32| sine(sample) as f64;
    let start = value(base);
    let end = value(base + 1);
    let start_velocity = (value(base + 1) - value(base - 1)) * 0.5;
    let end_velocity = (value(base + 2) - value(base)) * 0.5;
    let start_acceleration = value(base + 1) - 2.0 * start + value(base - 1);
    let end_acceleration = value(base + 2) - 2.0 * end + value(base);
    quintic_component(
        x,
        1.0,
        start,
        end,
        start_velocity,
        end_velocity,
        start_acceleration,
        end_acceleration,
    )
}

fn trajectory_direction(phase: f64, meridian: f64) -> DVec3 {
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

fn flow_field_basis() -> (DVec3, DVec3, DVec3) {
    let outward = trajectory_direction(0.0, TRAJECTORY_ENTRY_MERIDIAN);
    let forward = dvec_scale(outward, -1.0);
    let world_down = DVec3 { x: 0.0, y: 1.0, z: 0.0 };
    let down = dvec_normalize(dvec_sub(
        world_down,
        dvec_scale(forward, dvec_dot(world_down, forward)),
    ));
    let right = dvec_from_vec3(vec_normalize(vec_cross(
        vec3_from_dvec(down),
        vec3_from_dvec(forward),
    )));
    (forward, right, down)
}

fn flow_field_dual_basis() -> (DVec3, DVec3, DVec3) {
    let (forward, right, down) = flow_field_basis();
    let forward = dvec_from_vec3(vec3_from_dvec(forward));
    let right = dvec_from_vec3(vec3_from_dvec(right));
    let down = dvec_from_vec3(vec3_from_dvec(down));
    let cross = |a: DVec3, b: DVec3| DVec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    };
    let determinant = dvec_dot(forward, cross(right, down));
    (
        dvec_scale(cross(right, down), 1.0 / determinant),
        dvec_scale(cross(down, forward), 1.0 / determinant),
        dvec_scale(cross(forward, right), 1.0 / determinant),
    )
}

fn trajectory_relief_at(time: f64) -> f64 {
    let (state, _) = trajectory_scalar_state(time);
    flow_surface_relief_world(flow_normal_to_planet(trajectory_direction(
        state.phase,
        state.meridian,
    )))
}

fn trajectory_relief_state(time: f64) -> (f64, f64, f64) {
    const SAMPLE_TIME: f64 = 0.002;
    let before = trajectory_relief_at(time - SAMPLE_TIME);
    let at = trajectory_relief_at(time);
    let after = trajectory_relief_at(time + SAMPLE_TIME);
    (
        at,
        (after - before) / (2.0 * SAMPLE_TIME),
        (after - 2.0 * at + before) / (SAMPLE_TIME * SAMPLE_TIME),
    )
}

fn trajectory_state(time: f64) -> TrajectoryState {
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
    let radius = FLOW_PLANET_RADIUS_WORLD + state.clearance + relief.0;
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
        state.clearance_rate + relief.1,
        acceleration.0 + relief.2,
    );
    let x = product(radial, unit_x);
    let y = product(radial, unit_y);
    let z = product(radial, unit_z);
    let field_down = flow_field_basis().2;
    let position = dvec_add(
        DVec3 { x: x.0, y: y.0, z: z.0 },
        dvec_scale(field_down, state.impact),
    );
    let velocity = dvec_add(
        DVec3 { x: x.1, y: y.1, z: z.1 },
        dvec_scale(field_down, state.impact_rate),
    );
    let acceleration_world = dvec_add(
        DVec3 { x: x.2, y: y.2, z: z.2 },
        dvec_scale(field_down, acceleration.3),
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

fn smootherstep_f64(value: f64) -> f64 {
    let x = value.clamp(0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

fn flow_ground_framing(time: f32) -> f32 {
    1.54
        + 1.96 * smootherstep((time - 35.0) / 3.0)
        - 0.50 * smootherstep((time - 38.0) / 4.0)
        - 2.25 * smootherstep((time - 42.0) / 18.0)
        - 0.40 * smootherstep((time - 90.0) / 8.0)
        - 0.17 * smootherstep((time - 98.0) / 10.0)
}

fn flow_camera(time: f32) -> FlowCamera {
    let trajectory = trajectory_state(time as f64);
    let (field_forward, field_right, field_down) = flow_field_basis();
    let dynamics_are_finite = trajectory.acceleration.x.is_finite()
        && trajectory.acceleration.y.is_finite()
        && trajectory.acceleration.z.is_finite()
        && trajectory.phase.is_finite()
        && trajectory.phase_rate.is_finite()
        && trajectory.roll.is_finite();
    let position = trajectory.position;
    let toward_planet =
        vec3_from_dvec(dvec_normalize(dvec_scale(trajectory.position, -1.0)));
    let orbit_window = smootherstep_f64((time as f64 - ORBIT_ENTRY as f64) / 1.0)
        * (1.0 - smootherstep_f64((time as f64 - 34.0) / 2.0));
    let ground_window = smootherstep_f64((time as f64 - 34.0) / 2.0);
    let look_amount = 2.4 * orbit_window
        + flow_ground_framing(time) as f64 * ground_window;
    let orbital_forward = if dynamics_are_finite {
        vec_normalize(vec_add(
            trajectory.forward,
            vec_scale(toward_planet, look_amount as f32),
        ))
    } else {
        Vec3 { x: 0.0, y: 0.0, z: 1.0 }
    };
    let forward = if time <= ORBIT_ENTRY {
        vec3_from_dvec(field_forward)
    } else {
        orbital_forward
    };
    let radial_down = vec_sub(
        toward_planet,
        vec_scale(forward, vec_dot(toward_planet, forward)),
    );
    let world_down = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    let grid_down = vec_sub(world_down, vec_scale(forward, vec_dot(world_down, forward)));
    let down_blend = smootherstep((time - ORBIT_ENTRY) / 5.0);
    let down_seed = vec_add(
        vec_scale(grid_down, 1.0 - down_blend),
        vec_scale(radial_down, down_blend),
    );
    let orbital_down = if vec_length(down_seed) > 0.001 {
        vec_normalize(vec_sub(
            down_seed,
            vec_scale(forward, vec_dot(down_seed, forward)),
        ))
    } else {
        trajectory.down
    };
    let down = if time <= ORBIT_ENTRY {
        vec3_from_dvec(field_down)
    } else {
        orbital_down
    };
    let right = if time <= ORBIT_ENTRY {
        vec3_from_dvec(field_right)
    } else {
        vec_normalize(vec_cross(down, forward))
    };
    let camera_normal = flow_normal_to_planet(dvec_normalize(dvec_sub(position, PLANET_CENTER)));
    let altitude_world = dvec_length(dvec_sub(position, PLANET_CENTER))
        - FLOW_PLANET_RADIUS_WORLD
        - flow_surface_relief_world(camera_normal);
    let altitude_miles = world_to_miles(altitude_world);
    let near_plane =
        0.001f64.min(altitude_world.max(FLOW_FINAL_ALTITUDE_WORLD) * 0.01);
    FlowCamera {
        position,
        forward,
        right,
        down,
        altitude_miles,
        speed_world_per_second: dvec_length(trajectory.velocity),
        near_plane,
    }
}

#[cfg(feature = "phase0-audit")]
fn project_flow_depth(camera: &FlowCamera, point: DVec3) -> Option<(f32, f32, f32)> {
    let relative = dvec_sub(point, camera.position);
    let depth = dvec_dot(relative, dvec_from_vec3(camera.forward));
    if depth <= camera.near_plane {
        return None;
    }
    Some((
        160.0
            + (dvec_dot_vec3(relative, camera.right) / depth * FLOW_FOCAL as f64) as f32,
        100.0
            + (dvec_dot_vec3(relative, camera.down) / depth * FLOW_FOCAL as f64) as f32,
        depth as f32,
    ))
}

#[cfg(feature = "phase0-audit")]
fn project_flow(camera: &FlowCamera, point: DVec3) -> Option<(f32, f32)> {
    let projected = project_flow_depth(camera, point)?;
    Some((projected.0, projected.1))
}

fn flow_grid_world_point(time: f32, section: (f32, f32), depth: f32) -> DVec3 {
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
    let transport = flow_field_transport(time);
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

fn flow_field_transport(time: f32) -> DVec3 {
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

fn flow_star_world_point(
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

fn project_flow_star_segment(
    transport: DVec3,
    local_a: (f32, f32),
    local_b: (f32, f32),
    moving_row: f32,
    camera: &FlowCamera,
    x_scale: f32,
) -> Option<((f32, f32, f32), (f32, f32, f32))> {
    let a = flow_star_world_point(transport, local_a, moving_row);
    let b = flow_star_world_point(transport, local_b, moving_row);
    let project = |point: DVec3| {
        let relative = dvec_sub(point, camera.position);
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

fn project_flow_grid_segment(
    time: f32,
    section_a: (f32, f32),
    depth_a: f32,
    section_b: (f32, f32),
    depth_b: f32,
    camera: &FlowCamera,
    x_scale: f32,
) -> Option<((f32, f32, f32), (f32, f32, f32))> {
    let mut a = flow_grid_world_point(time, section_a, depth_a);
    let mut b = flow_grid_world_point(time, section_b, depth_b);
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
fn phase0_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase0 motion audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
fn phase0_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase0_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
fn phase0_vec_close(a: Vec3, b: Vec3, tolerance: f32) -> bool {
    vec_length(vec_sub(a, b)) <= tolerance
}

#[cfg(feature = "phase0-audit")]
fn phase0_dvec_close(a: DVec3, b: DVec3, tolerance_squared: f64) -> bool {
    dvec_length_squared(dvec_sub(a, b)) <= tolerance_squared
}

#[cfg(feature = "phase0-audit")]
fn phase0_basis_valid(camera: FlowCamera) -> bool {
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
fn run_phase0_motion_audit() -> ! {
    let entry = trajectory_state(ORBIT_ENTRY as f64);
    if !((entry.clearance_miles - world_to_miles(TRAJECTORY_ENTRY_CLEARANCE)).abs() < 1.0
        && entry.phase.abs() < 1.0e-6)
    {
        let mut report = [0u8; 128];
        let pointer = report.as_mut_ptr();
        let mut length = 0usize;
        append(pointer, &mut length, b"phase0 motion audit failed: entry milli-miles=");
        append_number(pointer, &mut length, (entry.clearance_miles.abs() * 1_000.0) as u32);
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
        phase0_require(
            first.position.x.is_finite()
                && first.position.y.is_finite()
                && first.position.z.is_finite()
                && phase0_dvec_close(first.position, second.position, 1.0e-24)
                && phase0_vec_close(first.forward, second.forward, 1.0e-7)
                && first.phase + 1.0e-9 >= previous_phase
                && phase0_basis_valid(flow_camera(time as f32)),
            111,
            b"approach/capture state is invalid or history-dependent",
        );
        previous_phase = first.phase;
        time += 0.01;
    }
    write_all(
        b"phase0 motion audit ok: one-state approach=27-28 capture=28-35 deterministic-seek=yes\n",
    );
    exit(0)
}

#[cfg(feature = "phase0-audit")]
fn run_phase1_precision_audit() -> ! {
    phase0_require(
        (world_to_miles(FLOW_PLANET_RADIUS_WORLD) - PLANET_RADIUS_MILES).abs() < 1.0e-9
            && (world_to_meters(FLOW_FINAL_ALTITUDE_WORLD) - FINAL_ALTITUDE_METERS).abs()
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
fn phase2_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase2 motion audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
fn phase2_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase2_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
fn trajectory_derivatives_agree(time: f64) -> bool {
    let step = 0.002;
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
    dvec_length(dvec_sub(numerical_velocity, at.velocity)) / velocity_scale < 2.0e-4
        && dvec_length(dvec_sub(numerical_acceleration, at.acceleration))
            / acceleration_scale
            < 2.0e-2
}

#[cfg(feature = "phase0-audit")]
fn run_phase2_motion_audit() -> ! {
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
            append(pointer, &mut length, b"\n");
            write_all(&report[..length]);
            exit(140)
        }
        index += 1;
    }

    let orbit_one = trajectory_state(35.0);
    let orbit_two = trajectory_state(FLOW_ORBIT_TWO_END);
    let orbit_three = trajectory_state(FLOW_ORBIT_THREE_END);
    phase2_require(
        (orbit_one.phase - 1_024.0).abs() < 2.0
            && (orbit_two.phase - 2_048.0).abs() < 2.0
            && (orbit_three.phase - 3_072.0).abs() < 2.0,
        141,
        b"integrated orbit crossings missed their timing windows",
    );
    phase2_require(
        dvec_length(orbit_three.velocity) > 0.0
            && dvec_length(orbit_three.velocity) <= dvec_length(orbit_one.velocity) * 0.25
            && orbit_three.phase_rate > 0.0,
        142,
        b"continuous braking is not decisive enough by orbit three",
    );

    let mut time = 35.0f64;
    let mut previous_altitude = trajectory_state(time).clearance_miles + 1.0e-6;
    let mut previous_phase = trajectory_state(time).phase - 1.0e-6;
    let mut previous_acceleration = trajectory_state(time).acceleration;
    while time <= FLOW_DESCENT_END {
        let state = trajectory_state(time);
        if time <= FLOW_ORBIT_THREE_END {
            if state.clearance_miles > previous_altitude + 1.0e-5 {
                let mut report = [0u8; 96];
                let pointer = report.as_mut_ptr();
                let mut length = 0usize;
                append(pointer, &mut length, b"phase2 motion audit failed: altitude rises at ms=");
                append_number(pointer, &mut length, (time * 1_000.0) as u32);
                append(pointer, &mut length, b"\n");
                write_all(&report[..length]);
                exit(143)
            }
        }
        phase2_require(
            state.phase > previous_phase
                && state.phase_rate > 0.0
                && dvec_length(dvec_sub(state.acceleration, previous_acceleration)).is_finite(),
            144,
            b"ground-track direction stopped or trajectory jerk is unbounded",
        );
        previous_altitude = state.clearance_miles;
        previous_phase = state.phase;
        previous_acceleration = state.acceleration;
        time += 1.0 / 120.0;
    }

    let controls = [
        (FLOW_ORBIT_THREE_END, 30_000.0),
        (FLOW_HIGH_PASS_END, 30_000.0),
        (FLOW_VALLEY_OVERHEAD, 1_500.0),
        (FLOW_VALLEY_CORRIDOR, 200.0),
        (FLOW_DESCENT_END, FINAL_ALTITUDE_METERS),
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
            (state.clearance_miles * METERS_PER_MILE - controls[index].1).abs() < 0.5,
            145,
            b"integrated radial control missed a physical-altitude target",
        );
        phase2_require(
            (actual_agl * METERS_PER_MILE - controls[index].1).abs() < 0.5,
            148,
            b"trajectory clearance is not displaced-surface AGL",
        );
        index += 1;
    }

    let pass_start = trajectory_state(FLOW_ORBIT_THREE_END);
    let pass_end = trajectory_state(FLOW_HIGH_PASS_END);
    phase2_require(
        (pass_start.clearance_miles - pass_end.clearance_miles).abs() < 1.0e-6
            && pass_end.phase > pass_start.phase
            && pass_end.phase - pass_start.phase < 128.0,
        146,
        b"airplane-height pass is not level with slow perceptible motion",
    );
    let endpoint = trajectory_state(FLOW_PATH_END);
    let endpoint_after = trajectory_state(FLOW_PATH_END + 0.01);
    phase2_require(
        dvec_length(dvec_sub(endpoint_after.position, endpoint.position)) > 1.0e-10
            && (endpoint.phase - 3_200.0).abs() < 0.1,
        147,
        b"ground speed stops or regional phase misses the destination",
    );

    write_all(
        b"phase2 motion audit ok: one-state=0-137s orbit1=35s orbit2=42s orbit3=60s/30000ft pass=60-90s endpoint=116s/20m\n",
    );
    exit(0)
}

#[cfg(feature = "phase0-audit")]
fn phase3_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase3 unified-camera audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
fn phase3_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase3_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
fn run_phase3_unified_camera_audit() -> ! {
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
    let start_transport = flow_field_transport(0.0);
    index = 0;
    while index < STAR_LINES {
        if let Some((a, b, depth)) = flyby_segment(index, 0.0, CLEAN_BIRTH) {
            if let Some((projected_a, projected_b)) =
                project_flow_star_segment(
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
    let mut canonical_active_samples = 0u32;
    while canonical_tick <= 11 * TRAJECTORY_HZ as u32 {
        let time = canonical_tick as f32 / TRAJECTORY_HZ as f32;
        let camera = flow_camera(time);
        let transport = flow_field_transport(time);
        index = 0;
        while index < STAR_LINES {
            let generated = flyby_segment(index, time, CLEAN_BIRTH);
            let canonical = canonical_flyby_segment(index, time, CLEAN_BIRTH);
            match (generated, canonical) {
                (None, None) => {}
                (Some((a, b, moving_row)), Some((canonical_a, canonical_b))) => {
                    let Some((pa, pb)) =
                        project_flow_star_segment(transport, a, b, moving_row, &camera, 1.0)
                    else {
                        phase3_fail(181, b"canonical star was lost during world projection")
                    };
                    canonical_max_error = canonical_max_error
                        .max((pa.0 - canonical_a.0).abs())
                        .max((pa.1 - canonical_a.1).abs())
                        .max((pb.0 - canonical_b.0).abs())
                        .max((pb.1 - canonical_b.1).abs());
                    canonical_active_samples += 1;
                }
                _ => phase3_fail(181, b"canonical star activation identity changed"),
            }
            index += 1;
        }
        canonical_tick += 1;
    }
    phase3_require(
        canonical_active_samples >= 1_000_000 && canonical_max_error <= 0.000_1,
        181,
        b"world projection does not exactly reproduce canonical star motion",
    );
    let mut mesh_tick = (RING_BIRTH * TRAJECTORY_HZ as f32) as u32;
    let mesh_end_tick = (CLEAN_BIRTH * TRAJECTORY_HZ as f32) as u32;
    let mut canonical_mesh_samples = 0u32;
    let mut canonical_mesh_error = 0.0f32;
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
                            let point = flow_grid_world_point(time, section, depth);
                            let projected = project_flow_depth(&camera, point).unwrap_or_else(|| {
                                phase3_fail(181, b"canonical forming mesh left the forward field")
                            });
                            canonical_mesh_error = canonical_mesh_error
                                .max((projected.0 - (160.0 + section.0 * projective_radius)).abs())
                                .max((projected.1 - (100.0 + section.1 * projective_radius)).abs());
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
    phase3_require(
        canonical_mesh_samples >= 100_000 && canonical_mesh_error <= 0.000_1,
        181,
        b"world projection does not exactly reproduce canonical mesh formation",
    );
    let mut star_tick = 0u32;
    let mut previous_star_distance = f64::MAX;
    while star_tick <= 11_000 {
        let tick = star_tick;
        let time = tick as f32 / 1_000.0;
        let camera = flow_camera(time);
        let transport = flow_field_transport(time);
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
                    project_flow_star_segment(transport, a, b, depth, &camera, 1.0)
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
                    project_flow_grid_segment(time, a, depth, b, depth, &camera, 1.0)
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
    while capture_sample <= 7 * TRAJECTORY_HZ {
        let time = ORBIT_ENTRY + capture_sample as f32 / TRAJECTORY_HZ as f32;
        let camera = flow_camera(time);
        let bounds = projected_surface_bounds(&camera, 1.0);
        if !(bounds.visible > 0.0
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
        phase3_require(
            capture_sample == 0
                || (vec_dot(previous_capture_camera.forward, camera.forward) > 0.995
                    && vec_dot(previous_capture_camera.down, camera.down) > 0.995),
            182,
            b"world-space camera jumps during capture",
        );
        previous_capture_camera = camera;
        capture_sample += 1;
    }
    let embed_times = [26.0f32, 27.0, 27.5, 28.0];
    let fixed_birth = travel_row(PLANET_INTRO);
    let fixed_shape = birth_shape(fixed_birth);
    const SECTION_SCALE: f64 = GRID_BOTTOM_HEIGHT_WORLD / 0.61;
    let field_forward = flow_field_basis().0;
    let mut embed_index = 0usize;
    while embed_index < embed_times.len() {
        let time = embed_times[embed_index];
        let transport = flow_particle_transport(time);
        let planet_depth = dvec_dot(dvec_sub(PLANET_CENTER, transport), field_forward);
        let projective_radius = FLOW_FOCAL as f64 * SECTION_SCALE / planet_depth;
        let depth = (20.0 * sqrt_f64(projective_radius / 260.0)) as f32;
        let bottom = grid_section_point(-7.0, fixed_shape);
        let plate_point = flow_grid_world_point(time, bottom, depth);
        phase3_require(
            dvec_length(dvec_sub(plate_point, PLANET_CENTER)) < 1.0e-4,
            179,
            b"flowing bottom plate does not pass through the fixed planet equator",
        );
        embed_index += 1;
    }
    let extent_time = 27.0f32;
    let extent_bottom = grid_section_point(-7.0, fixed_shape);
    let far_point = flow_grid_world_point(extent_time, extent_bottom, GRID_FOG_START);
    let near_point = flow_grid_world_point(extent_time, extent_bottom, 24.0);
    phase3_require(
        dvec_dot(dvec_sub(far_point, PLANET_CENTER), field_forward)
            > GRID_WELL_RADIUS_WORLD
            && dvec_dot(dvec_sub(near_point, PLANET_CENTER), field_forward)
                < -GRID_WELL_RADIUS_WORLD,
        179,
        b"mesh does not extend continuously from the horizon past the planet",
    );
    let transport_step = 1.0f32 / TRAJECTORY_HZ as f32;
    let transport_before = flow_field_transport(ORBIT_ENTRY - transport_step);
    let transport_entry = flow_field_transport(ORBIT_ENTRY);
    let transport_after = flow_field_transport(ORBIT_ENTRY + transport_step);
    let velocity_before = dvec_scale(
        dvec_sub(transport_entry, transport_before),
        1.0 / transport_step as f64,
    );
    let velocity_after = dvec_scale(
        dvec_sub(transport_after, transport_entry),
        1.0 / transport_step as f64,
    );
    let post_capture_transport = flow_field_transport(29.0);
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
        let point = flow_grid_world_point(extent_time, top, depth);
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
        b"phase3 unified-camera audit ok: geometry=one-world grid=horizon-flow/past-planet bottom-plate=planet-equator camera=trajectory projection=output-only seek=stable\n",
    );
    exit(0)
}
#[cfg(feature = "phase0-audit")]
fn phase4_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase4 spherical-surface audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
fn phase4_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase4_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
fn run_phase4_surface_audit() -> ! {
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

    let endpoint = flow_camera(FLOW_DESCENT_END as f32);
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
    let mut previous = -0.001f32;
    let mut index = 0usize;
    while index < radii.len() {
        view.radius_y = radii[index];
        let polygon = surface_polygon_weight(view);
        phase4_require(
            polygon + 0.000_1 >= previous
                && (0.0..=1.0).contains(&polygon)
                && ((1.0 - polygon) + polygon - 1.0).abs() < 0.000_1,
            242,
            b"surface coverage refinement is discontinuous or incomplete",
        );
        previous = polygon;
        index += 1;
    }

    phase4_require(
        surface_mesh_step() == SURFACE_CELL_PIXELS
            && SURFACE_ROW_CAPACITY >= WIDTH / SURFACE_CELL_PIXELS as usize + 2,
        243,
        b"canonical polygon surface exceeds its fixed row budget",
    );
    write_all(
        b"phase4 polygon-surface audit ok: center=canonical bounds=displaced-sphere coverage=one-sampler surface-edges=absent agl=surface-relative final=20m\n",
    );
    exit(0)
}
#[cfg(feature = "phase0-audit")]
fn phase5_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase5 refinement audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
fn phase5_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase5_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
fn run_phase5_refinement_audit() -> ! {
    let footprints = [5_000.0f64, 1_000.0, 300.0, 50.0, 3.0, 0.1];
    let mut previous = SurfaceLod { continent: 0.0, ranges: 0.0, mountains: 0.0, valley: 0.0 };
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
        phase5_require(
            (flow_base_surface_height(normal, full_surface_lod())
                - terrain_smooth(map.0, map.1))
                .abs()
                < 0.002,
            203,
            b"child bands do not reconstruct the source terrain",
        );
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

    let valley_camera = flow_camera(94.0);
    let valley_point = flow_visible_surface_point(&valley_camera, landing_up());
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
    render_flow_surface(&valley_camera, 94.0, 1.0);
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
        94.0,
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
    let valley_alignment = vec_dot(rendered_normal, landing_up());
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
    let mut corridor_time = FLOW_VALLEY_OVERHEAD;
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
    let corridor_controls = [94.0f32, 100.0, 106.0, 137.0];
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
    let left_wall = vec_normalize(vec_add(
        vec_scale(valley_centre, wall_radial),
        Vec3 { x: -wall_offset, y: 0.0, z: 0.0 },
    ));
    let right_wall = vec_normalize(vec_add(
        vec_scale(valley_centre, wall_radial),
        Vec3 { x: wall_offset, y: 0.0, z: 0.0 },
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
    let wall_camera = flow_camera(122.0);
    clear_depth_buffer();
    render_flow_surface(&wall_camera, 122.0, 1.0);
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
    phase5_require(
        left_wall_pixels >= 16 && right_wall_pixels >= 16,
        224,
        b"both valley walls do not own rendered depth around the low-flight frame",
    );

    write_all(
        b"phase5 refinement audit ok: parent-child=attached bands=ordered geomorph<1px valley=rendered/visible-before-entry/track-inside/walls-rendered\n",
    );
    exit(0)
}

#[cfg(feature = "phase0-audit")]
fn phase6_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase6 atmosphere audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
fn phase6_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase6_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
fn run_phase6_atmosphere_audit() -> ! {
    let above = flow_camera(FLOW_DESCENT_START as f32);
    let entry = flow_camera(78.0);
    phase6_require(
        flow_atmosphere_amount(&above) == 0.0 && flow_atmosphere_amount(&entry) > 0.0,
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
    render_continuous_grid(78.0, &entry, 1.0);
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
                if let Some((start, end, full_optical_depth)) =
                    atmosphere_ray_segment(&entry, direction)
                {
                    let forward = dvec_dot_vec3(direction, entry.forward).max(0.001);
                    let grid_alpha = atmospheric_alpha_to_depth(
                        start,
                        end,
                        full_optical_depth,
                        forward,
                        depth,
                    );
                    let full_alpha = atmospheric_alpha_to_depth(
                        start,
                        end,
                        full_optical_depth,
                        forward,
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

    let valley_sun_times = [106.0f32, 113.0, 122.0, 137.0];
    let mut sun_index = 0usize;
    let mut exposed_sun_views = 0usize;
    while sun_index < valley_sun_times.len() {
        let time = valley_sun_times[sun_index];
        let camera = flow_camera(time);
        if let Some(projection) = flow_sun_projection(&camera, 1.0) {
            if projection.0 >= -14.0
                && projection.0 < WIDTH as f32 + 14.0
                && projection.1 >= -14.0
                && projection.1 < HEIGHT as f32 + 14.0
            {
                clear_depth_buffer();
                render_continuous_grid(time, &camera, 1.0);
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
            }
        }
        sun_index += 1;
    }
    if exposed_sun_views != valley_sun_times.len() {
        let mut report = [0u8; 256];
        let report_pointer = report.as_mut_ptr();
        let mut report_length = 0usize;
        let sun = flow_sun_direction();
        sun_index = 0;
        append(report_pointer, &mut report_length, b"phase6 atmosphere audit failed: valley sun exposure ");
        append_number(report_pointer, &mut report_length, exposed_sun_views as u32);
        append(report_pointer, &mut report_length, b"/4 camera coordinates");
        while sun_index < valley_sun_times.len() {
            let camera = flow_camera(valley_sun_times[sun_index]);
            append(report_pointer, &mut report_length, b" t=");
            append_number(
                report_pointer,
                &mut report_length,
                valley_sun_times[sun_index] as u32,
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

    let checkpoints = [65.0f32, 78.0, 86.0, 94.0, 100.0, 106.0];
    let mut checkpoint = 0usize;
    let mut maximum_frame_ns = 0u64;
    while checkpoint < checkpoints.len() {
        let started = process_cpu_ns();
        render_demo(checkpoints[checkpoint], 1.0);
        maximum_frame_ns = maximum_frame_ns.max(process_cpu_ns().saturating_sub(started));
        checkpoint += 1;
    }
    phase6_require(
        maximum_frame_ns <= 33_333_334,
        227,
        b"combined surface and atmosphere frame exceeds 33.3 ms",
    );
    write_all(
        b"phase6 atmosphere audit ok: shell=shared optical-depth=ray/grid plasma=density*speed clouds=anchored sun=fixed/exposed-valley frame<=33.3ms\n",
    );
    exit(0)
}

#[cfg(feature = "phase0-audit")]
fn phase7_fail(code: usize, message: &[u8]) -> ! {
    write_all(b"phase7 hardening audit failed: ");
    write_all(message);
    write_all(b"\n");
    exit(code)
}

#[cfg(feature = "phase0-audit")]
fn phase7_require(condition: bool, code: usize, message: &[u8]) {
    if !condition {
        phase7_fail(code, message);
    }
}

#[cfg(feature = "phase0-audit")]
fn phase7_frame_hash() -> u64 {
    let frame = core::ptr::addr_of!(FRAME).cast::<u8>();
    let mut result = 14_695_981_039_346_656_037u64;
    let mut index = 0usize;
    while index < FRAME_BYTES {
        result ^= unsafe { frame.add(index).read() } as u64;
        result = result.wrapping_mul(1_099_511_628_211);
        index += 1;
    }
    result
}

#[cfg(feature = "phase0-audit")]
fn run_phase7_hardening_audit() -> ! {
    let mut view = projected_surface_bounds(&flow_camera(29.0), 1.0);
    let radii = [0.0f32, 2.5, 4.0, 7.0, 10.0, 20.0];
    let mut previous = -0.001f32;
    let mut index = 0usize;
    while index < radii.len() {
        view.radius_y = radii[index];
        let polygon = surface_polygon_weight(view);
        phase7_require(
            polygon + 0.000_1 >= previous
                && (0.0..=1.0).contains(&polygon)
                && ((1.0 - polygon) + polygon - 1.0).abs() < 0.000_1,
            230,
            b"surface refinement coverage is discontinuous or incomplete",
        );
        previous = polygon;
        index += 1;
    }
    let checkpoints = [0.0f32, 14.75, 21.0, 27.99, 28.0, 51.959_26, 65.0, 94.0, 137.0];
    index = 0;
    while index < checkpoints.len() {
        let first = flow_camera(checkpoints[index]);
        let second = flow_camera(checkpoints[index]);
        phase7_require(
            phase0_basis_valid(first)
                && dvec_length_squared(dvec_sub(first.position, second.position)) == 0.0
                && PLANET_CENTER == DVec3 { x: 0.0, y: 0.0, z: 0.0 },
            231,
            b"hardened sequence is not deterministic under seeking",
        );
        index += 1;
    }
    let frame_seek_checkpoints = [27.99f32, 28.0, 51.959_26, 65.0, 94.0, 137.0];
    index = 0;
    while index < frame_seek_checkpoints.len() {
        let time = frame_seek_checkpoints[index];
        render_demo(time, 1.0);
        let first = phase7_frame_hash();
        render_demo((time + 0.731).min(FLOW_PATH_END as f32), 1.0);
        render_demo(time, 1.0);
        phase7_require(
            first == phase7_frame_hash(),
            233,
            b"seeking away and back does not reproduce the complete frame",
        );
        index += 1;
    }
    let performance_checkpoints = [
        0.0f32,
        STARFIELD_HOLD,
        STARFIELD_FILL,
        STAR_ORGANIZE,
        RING_BIRTH,
        GRID_SQUARE_START,
        PLANET_INTRO,
        ORBIT_ENTRY,
        FLOW_DESCENT_START as f32,
        65.0,
        78.0,
        86.0,
        94.0,
        100.0,
        FLOW_DESCENT_END as f32,
        FLOW_PATH_END as f32,
    ];
    let mut maximum_frame_ns = 0u64;
    index = 0;
    while index < performance_checkpoints.len() {
        let started = process_cpu_ns();
        render_demo(performance_checkpoints[index], 1.0);
        maximum_frame_ns = maximum_frame_ns.max(process_cpu_ns().saturating_sub(started));
        index += 1;
    }
    phase7_require(
        maximum_frame_ns <= 33_333_334,
        232,
        b"a required timeline checkpoint exceeds the 33.3-ms hard maximum",
    );
    write_all(
        b"phase7 hardening audit ok: legacy=absent surface-lod=continuous seek=frame-deterministic controls=phase0 performance=all-checkpoints\n",
    );
    exit(0)
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

fn smootherstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

fn grid_fog_range(time: f32) -> (f32, f32) {
    let distant_detail = smoothstep((time - (PLANET_INTRO - 1.0)) / 2.0);
    (
        GRID_FOG_START + (1.0 - GRID_FOG_START) * distant_detail,
        GRID_FOG_END + (3.0 - GRID_FOG_END) * distant_detail,
    )
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

fn dvec_from_vec3(value: Vec3) -> DVec3 {
    DVec3 { x: value.x as f64, y: value.y as f64, z: value.z as f64 }
}

fn vec3_from_dvec(value: DVec3) -> Vec3 {
    Vec3 { x: value.x as f32, y: value.y as f32, z: value.z as f32 }
}

fn dvec_add(a: DVec3, b: DVec3) -> DVec3 {
    DVec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z }
}

fn dvec_sub(a: DVec3, b: DVec3) -> DVec3 {
    DVec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}

fn dvec_scale(a: DVec3, scale: f64) -> DVec3 {
    DVec3 { x: a.x * scale, y: a.y * scale, z: a.z * scale }
}

fn dvec_dot(a: DVec3, b: DVec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn dvec_dot_vec3(a: DVec3, b: Vec3) -> f64 {
    a.x * b.x as f64 + a.y * b.y as f64 + a.z * b.z as f64
}

fn dvec_length_squared(a: DVec3) -> f64 {
    dvec_dot(a, a)
}

fn dvec_length(a: DVec3) -> f64 {
    sqrt_f64(dvec_length_squared(a))
}

fn dvec_normalize(a: DVec3) -> DVec3 {
    dvec_scale(a, 1.0 / dvec_length(a).max(1.0e-15))
}

pub const fn miles_to_world(miles: f64) -> f64 {
    miles * WORLD_UNITS_PER_MILE
}

pub const fn world_to_miles(world: f64) -> f64 {
    world / WORLD_UNITS_PER_MILE
}

pub const fn world_to_meters(world: f64) -> f64 {
    world_to_miles(world) * METERS_PER_MILE
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

fn landing_forward() -> Vec3 {
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

fn flow_normal_to_planet(normal: DVec3) -> Vec3 {
    Vec3 {
        x: normal.x as f32,
        y: normal.z as f32,
        z: -normal.y as f32,
    }
}

fn planet_normal_to_flow(normal: Vec3) -> DVec3 {
    DVec3 {
        x: normal.x as f64,
        y: -normal.z as f64,
        z: normal.y as f64,
    }
}

fn flow_surface_map_from_normal(normal: Vec3) -> (f32, f32) {
    let forward = vec_dot(normal, landing_forward());
    let miles_per_radian = PLANET_RADIUS_MILES as f32;
    (
        18.0 + forward * miles_per_radian,
        valley_y(18.0)
            + (normal.x + forward * landing_slope()) * miles_per_radian,
    )
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
    smootherstep_f64(
        (wavelength_miles / footprint_miles.max(1.0e-9) - 0.75) / 2.25,
    ) as f32
}

fn surface_lod(footprint_miles: f64) -> SurfaceLod {
    SurfaceLod {
        continent: surface_lod_weight(3_000.0, footprint_miles),
        ranges: surface_lod_weight(700.0, footprint_miles),
        mountains: surface_lod_weight(90.0, footprint_miles),
        valley: surface_lod_weight(8.0, footprint_miles),
    }
}

fn full_surface_lod() -> SurfaceLod {
    SurfaceLod { continent: 1.0, ranges: 1.0, mountains: 1.0, valley: 1.0 }
}

fn flow_base_surface_height_from_map(map: (f32, f32), lod: SurfaceLod) -> f32 {
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

#[cfg(feature = "phase0-audit")]
fn flow_base_surface_height(normal: Vec3, lod: SurfaceLod) -> f32 {
    flow_base_surface_height_from_map(flow_surface_map_from_normal(normal), lod)
}

fn flow_surface_height_from_map(map: (f32, f32), lod: SurfaceLod) -> f32 {
    let base = flow_base_surface_height_from_map(map, lod);
    if lod.valley <= 0.0 {
        return base;
    }
    let longitudinal = local_valley_longitudinal_weight(map.0);
    if longitudinal <= 0.0 {
        return base;
    }
    let width_scale = local_valley_width_scale(map.0);
    let floor_half_width = LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES * width_scale;
    let wall_half_width = LOCAL_VALLEY_WALL_HALF_WIDTH_MILES * width_scale;
    let blend_half_width = LOCAL_VALLEY_BLEND_HALF_WIDTH_MILES * width_scale;
    let distance = local_valley_distance(map.0, map.1);
    if distance >= blend_half_width {
        return base;
    }
    let wall = smootherstep(
        (distance - floor_half_width) / (wall_half_width - floor_half_width),
    );
    // Keep the navigable floor below the mountain-relief threshold. The wall
    // profile then rises out of that floor continuously; allowing the legacy
    // floor noise to cross the threshold made the 20-metre camera chase a
    // derivative cusp instead of following a graceful valley tangent.
    let floor = valley_floor(map.0).min(105.0);
    let target = floor + wall * LOCAL_VALLEY_WALL_HEIGHT_UNITS;
    let lateral = 1.0 - smootherstep(
        (distance - wall_half_width) / (blend_half_width - wall_half_width),
    );
    base + (target - base) * lod.valley * longitudinal * lateral
}

fn flow_surface_relief_lod_world(normal: Vec3, lod: SurfaceLod) -> f64 {
    flow_surface_sample(normal, lod).relief_world
}

fn flow_surface_sample(normal: Vec3, lod: SurfaceLod) -> SurfaceSample {
    let range_map = flow_surface_map_from_normal(normal);
    let height = flow_surface_height_from_map(range_map, lod);
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

#[cfg(feature = "phase0-audit")]
fn flow_surface_point(normal: Vec3) -> DVec3 {
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
fn flow_visible_surface_point(camera: &FlowCamera, normal: Vec3) -> DVec3 {
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

fn flow_sphere_roots(camera: &FlowCamera, direction: DVec3, radius: f64) -> Option<(f64, f64)> {
    let local = dvec_sub(camera.position, PLANET_CENTER);
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

fn flow_surface_color(
    normal: Vec3,
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
    let broad_valley_distance = wrapped_delta(map.1, valley_y(map.0)).abs();
    let local_weight = local_valley_longitudinal_weight(map.0);
    let valley_distance = broad_valley_distance
        + (local_valley_distance(map.0, map.1) - broad_valley_distance) * local_weight;
    let river = smoothstep((3.2 - valley_distance) / 2.4)
        * lod.valley
        * sample.continent;
    let range_chain = smoothstep(
        (sine_sample(map.0 * 0.11 + map.1 * 0.035 + 97.0) - 0.08) / 0.78,
    ) * lod.ranges;
    land.0 += range_chain * 23.0 - river * 45.0;
    land.1 += range_chain * 13.0 - river * 38.0;
    land.2 += range_chain * 8.0 + river * 53.0;
    let base = (
        ocean.0 + (land.0 - ocean.0) * sample.continent + coast_band * 24.0,
        ocean.1 + (land.1 - ocean.1) * sample.continent + coast_band * 21.0,
        ocean.2 + (land.2 - ocean.2) * sample.continent + coast_band * 8.0,
    );
    let flow_normal = vec3_from_dvec(planet_normal_to_flow(normal));
    let terrain_light = (light_smooth(map.0, map.1) - 128.0) / 68.0;
    let relief_light = 1.0
        + terrain_light
            * (lod.ranges * 0.18 + lod.mountains * 0.24 + lod.valley * 0.22);
    let illumination = (0.30
        + vec_dot(flow_normal, flow_sun_direction()).max(0.0) * 0.82)
        * relief_light;
    let cloud_noise = sine_sample(
        normal.x * 390.0 + normal.y * 570.0 + normal.z * 240.0 + 113.0,
    ) * 0.64
        + sine_sample(
            normal.x * 760.0 - normal.y * 310.0 + normal.z * 610.0 + 733.0,
        ) * 0.36;
    // Keep the anchored cloud layer translucent enough that it cannot hide
    // the continent/range hierarchy it is meant to sit above.
    let cloud = smoothstep((cloud_noise - 0.64) / 0.28) * 0.10;
    (
        (base.0 * illumination + (218.0 - base.0 * illumination) * cloud)
            .clamp(0.0, 255.0) as u8,
        (base.1 * illumination + (226.0 - base.1 * illumination) * cloud)
            .clamp(0.0, 255.0) as u8,
        (base.2 * illumination + (236.0 - base.2 * illumination) * cloud)
            .clamp(0.0, 255.0) as u8,
    )
}

fn flow_camera_ray(
    camera: &FlowCamera,
    x_scale: f32,
    screen_x: f32,
    screen_y: f32,
) -> DVec3 {
    let vertical = (screen_y as f64 - 100.0) / FLOW_FOCAL as f64;
    let horizontal = (screen_x as f64 - 160.0)
        / (FLOW_FOCAL as f64 * x_scale.max(0.001) as f64);
    dvec_normalize(dvec_add(
        dvec_from_vec3(camera.forward),
        dvec_add(
            dvec_scale(dvec_from_vec3(camera.right), horizontal),
            dvec_scale(dvec_from_vec3(camera.down), vertical),
        ),
    ))
}

fn sample_surface_vertex(
    camera: &FlowCamera,
    time: f32,
    x_scale: f32,
    screen_x: f32,
    screen_y: f32,
) -> SurfaceVertex {
    let outer_radius = FLOW_PLANET_RADIUS_WORLD + miles_to_world(FLOW_MAX_RELIEF_MILES);
    let camera_radius = dvec_length(dvec_sub(camera.position, PLANET_CENTER));
    let reference_radius = (camera_radius - miles_to_world(camera.altitude_miles))
        .clamp(FLOW_PLANET_RADIUS_WORLD, outer_radius);
    let direction = flow_camera_ray(camera, x_scale, screen_x, screen_y);
    let forward_cosine = dvec_dot_vec3(direction, camera.forward).max(1.0e-12);
    let Some((mut distance, far_distance)) =
        flow_sphere_roots(camera, direction, reference_radius)
    else {
        return SurfaceVertex { x: screen_x, y: screen_y, ..EMPTY_SURFACE_VERTEX };
    };
    // The near plane is perpendicular to the camera, while sphere roots are
    // distances along an oblique ray. At a wide field of view the former can
    // therefore cut through the near surface even though the ray continues
    // through solid terrain. Own that clipped pixel at the plane instead of
    // exposing a serrated strip of background along the lower limb.
    if distance * forward_cosine <= camera.near_plane {
        if far_distance * forward_cosine <= camera.near_plane {
            return SurfaceVertex { x: screen_x, y: screen_y, ..EMPTY_SURFACE_VERTEX };
        }
        distance = camera.near_plane * 1.000_2 / forward_cosine;
    }
    let point = dvec_add(camera.position, dvec_scale(direction, distance));
    let mut sample_normal = flow_normal_to_planet(dvec_normalize(dvec_sub(point, PLANET_CENTER)));
    let footprint = world_to_miles(distance) / FLOW_FOCAL as f64;
    let lod = surface_lod(footprint);
    let mut surface_sample = flow_surface_sample(sample_normal, lod);
    // Broad planetary relief converges in one displaced-radius lookup. Fine
    // valley walls can move the ray's surface normal enough that a single
    // lookup lands on the adjacent cell, so close-detail rays receive a small
    // fixed iteration budget. This is still the same spherical surface
    // sampler and has no time-selected renderer path.
    let sample_map = (surface_sample.map_x, surface_sample.map_y);
    let local_scale = local_valley_width_scale(sample_map.0);
    let local_floor = LOCAL_VALLEY_FLOOR_HALF_WIDTH_MILES * local_scale;
    let local_width = LOCAL_VALLEY_BLEND_HALF_WIDTH_MILES * local_scale;
    let fine_valley_ray = lod.valley > 0.0
        && local_valley_longitudinal_weight(sample_map.0) > 0.0
        && local_valley_distance(sample_map.0, sample_map.1) > local_floor
        && local_valley_distance(sample_map.0, sample_map.1) < local_width;
    let refinement_limit = if fine_valley_ray { 2 } else { 1 };
    let mut refinement = 0usize;
    while refinement < refinement_limit {
        let surface_radius = FLOW_PLANET_RADIUS_WORLD + surface_sample.relief_world;
        let Some((refined, _)) = flow_sphere_roots(camera, direction, surface_radius) else {
            break;
        };
        // `flow_sphere_roots` clamps a negative near root to the camera near
        // plane.  That is useful for shell segments, but it is not a surface
        // hit: accepting it here turns a valid core ray into an invalid
        // near-plane sample when the camera is inside that local radial
        // bound. Keep the canonical hit until an actual forward root exists.
        if refined * forward_cosine <= camera.near_plane * 1.000_1 {
            break;
        }
        let distance_change = (refined - distance).abs();
        distance = refined;
        let refined_point = dvec_add(camera.position, dvec_scale(direction, distance));
        sample_normal = flow_normal_to_planet(dvec_normalize(dvec_sub(
            refined_point,
            PLANET_CENTER,
        )));
        surface_sample = flow_surface_sample(sample_normal, lod);
        refinement += 1;
        if distance_change < 1.0e-10 {
            break;
        }
    }
    let point = dvec_add(camera.position, dvec_scale(direction, distance));
    let normal = flow_normal_to_planet(dvec_normalize(dvec_sub(point, PLANET_CENTER)));
    let depth = dvec_dot_vec3(dvec_sub(point, camera.position), camera.forward) as f32;
    if depth <= camera.near_plane as f32 || !depth.is_finite() {
        return SurfaceVertex { x: screen_x, y: screen_y, ..EMPTY_SURFACE_VERTEX };
    }
    let mut color = flow_surface_color(normal, time, surface_sample, lod);
    if let Some((start, end, full_optical_depth)) = atmosphere_ray_segment(camera, direction) {
        let visible_end = end.min(distance);
        if visible_end > start {
            let fraction = ((visible_end - start) / (end - start)) as f32;
            let haze = atmospheric_alpha(full_optical_depth * fraction) * 0.72;
            color = (
                (color.0 as f32 + (72.0 - color.0 as f32) * haze) as u8,
                (color.1 as f32 + (137.0 - color.1 as f32) * haze) as u8,
                (color.2 as f32 + (211.0 - color.2 as f32) * haze) as u8,
            );
        }
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
    let minimum_x = floor_i32(a.x.min(b.x).min(c.x)).max(0);
    let maximum_x = ceil_i32(a.x.max(b.x).max(c.x)).min(WIDTH as i32);
    let minimum_y = floor_i32(a.y.min(b.y).min(c.y)).max(0);
    let maximum_y = ceil_i32(a.y.max(b.y).max(c.y)).min(HEIGHT as i32);
    let inverse_area = 1.0 / area;
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let mut y = minimum_y;
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
        y += 1;
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
    let mut y = top;
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
        y += 1;
    }
}

fn surface_mesh_step() -> i32 {
    SURFACE_CELL_PIXELS
}

fn render_polygon_surface_lod(
    camera: &FlowCamera,
    time: f32,
    x_scale: f32,
    visibility: f32,
) {
    if visibility <= 0.0 {
        return;
    }
    let step = surface_mesh_step();
    // The lattice covers the same frustum at every resolved scale. Sphere-ray
    // misses cheaply cull cells outside the planet, avoiding a projected-bound
    // mode change as the view becomes tangential during close orbit.
    let (left, right, top, bottom) =
        (-step, WIDTH as i32 + step, -step, HEIGHT as i32 + step);
    let columns = ((right - left + step - 1) / step + 1).max(0) as usize;
    if columns < 2 || columns > SURFACE_ROW_CAPACITY || bottom <= top {
        return;
    }

    let row_a = core::ptr::addr_of_mut!(SURFACE_ROW_A).cast::<SurfaceVertex>();
    let row_b = core::ptr::addr_of_mut!(SURFACE_ROW_B).cast::<SurfaceVertex>();
    let mut column = 0usize;
    while column < columns {
        let x = left + column as i32 * step;
        unsafe {
            row_a.add(column).write(sample_surface_vertex(
                camera,
                time,
                x_scale,
                x as f32,
                top as f32,
            ));
        }
        column += 1;
    }

    let mut y = top + step;
    while y <= bottom + step {
        column = 0;
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
        y += step;
    }
}

fn render_subpixel_surface_lod(
    camera: &FlowCamera,
    time: f32,
    x_scale: f32,
    view: ProjectedPlanetBounds,
    visibility: f32,
) {
    if visibility <= 0.0 || view.radius_x <= 0.0 || view.radius_y <= 0.0 {
        return;
    }
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let left = (view.x - view.radius_x).max(0.0) as i32;
    let right = (view.x + view.radius_x).min(WIDTH as f32 - 1.0) as i32;
    let top = (view.y - view.radius_y).max(0.0) as i32;
    let bottom = (view.y + view.radius_y).min(HEIGHT as f32 - 1.0) as i32;
    // At sub-polygon scale preserve the accepted half-buried pixel footprint,
    // but resolve every covered pixel through the same perspective ray,
    // spherical surface, terrain sample, color, and depth path as the mesh.
    // Raster ownership transfers continuously as polygons become resolvable.
    let unresolved_depth_bias = surface_unresolved_depth_bias(view);
    let mut y = top;
    while y <= bottom {
        let mut x = left;
        while x <= right {
            let sample = sample_surface_vertex(
                camera,
                time,
                x_scale,
                x as f32 + 0.5,
                y as f32 + 0.5,
            );
            if sample.valid && sample.inverse_depth > 0.0 {
                blend_depth_pixel(
                    frame,
                    x,
                    y,
                    1.0 / sample.inverse_depth,
                    sample.red as u8,
                    sample.green as u8,
                    sample.blue as u8,
                    (visibility * 255.0) as u8,
                    unresolved_depth_bias,
                );
            }
            x += 1;
        }
        y += 1;
    }
}

fn projected_surface_bounds(
    camera: &FlowCamera,
    x_scale: f32,
) -> ProjectedPlanetBounds {
    let center = dvec_sub(PLANET_CENTER, camera.position);
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
    let polygon_weight = surface_polygon_weight(view);
    render_subpixel_surface_lod(
        camera,
        time,
        x_scale,
        view,
        view.visible * (1.0 - polygon_weight),
    );
    render_polygon_surface_lod(
        camera,
        time,
        x_scale,
        view.visible * polygon_weight,
    );
}

fn surface_polygon_weight(view: ProjectedPlanetBounds) -> f32 {
    smootherstep((view.radius_y - 2.5) / 7.5)
}

fn surface_unresolved_depth_bias(view: ProjectedPlanetBounds) -> f32 {
    1.0 - surface_polygon_weight(view)
}

fn draw_entry_sheath(camera: &FlowCamera, intensity: f32) {
    if intensity <= 0.0 {
        return;
    }
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let ground_normal = flow_normal_to_planet(dvec_normalize(dvec_sub(
        camera.position,
        PLANET_CENTER,
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
    let legacy_forward = Vec3 {
        x: 0.0,
        y: sine_sample(262.0),
        z: -sine_sample(6.0),
    };
    let legacy_up = Vec3 {
        x: 0.0,
        y: sine_sample(6.0),
        z: sine_sample(262.0),
    };
    vec3_from_dvec(dvec_normalize(dvec_from_vec3(Vec3 {
        x: SUN_DIRECTION.x,
        y: -vec_dot(SUN_DIRECTION, legacy_up),
        z: vec_dot(SUN_DIRECTION, legacy_forward),
    })))
}

fn flow_sun_projection(camera: &FlowCamera, x_scale: f32) -> Option<(f32, f32)> {
    let direction = flow_sun_direction();
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
        draw_sun(x, y, amount);
    }
}

fn flow_atmosphere_amount(camera: &FlowCamera) -> f32 {
    let altitude_km = camera.altitude_miles * METERS_PER_MILE / 1_000.0;
    smootherstep_f64((100.0 - altitude_km) / 82.0) as f32
}

fn flow_entry_heat(camera: &FlowCamera) -> f32 {
    let altitude_km = camera.altitude_miles * METERS_PER_MILE / 1_000.0;
    let density_window = smootherstep_f64((100.0 - altitude_km) / 55.0)
        * smootherstep_f64((altitude_km - 8.0) / 25.0);
    let speed = smootherstep_f64((camera.speed_world_per_second - 0.002) / 0.012);
    (density_window * speed) as f32
}

fn atmosphere_ray_segment(
    camera: &FlowCamera,
    direction: DVec3,
) -> Option<(f64, f64, f32)> {
    let entry = flow_atmosphere_amount(camera);
    if entry <= 0.0 {
        return None;
    }
    // Atmospheric integration begins at the camera, not at the geometry near
    // plane.  During low flight the remaining shell above the camera is much
    // thinner than the near clip distance.
    let local = dvec_sub(camera.position, PLANET_CENTER);
    let b = dvec_dot(local, direction);
    let c = dvec_dot(local, local)
        - FLOW_ATMOSPHERE_RADIUS_WORLD * FLOW_ATMOSPHERE_RADIUS_WORLD;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }
    let root = sqrt_f64(discriminant);
    let start = (-b - root).max(0.0);
    let end = -b + root;
    if end <= start {
        return None;
    }
    let thickness = FLOW_ATMOSPHERE_RADIUS_WORLD - FLOW_PLANET_RADIUS_WORLD;
    let segment = end - start;
    let mut density = 0.0f64;
    let mut sample = 0usize;
    while sample < 3 {
        let amount = (sample as f64 + 0.5) / 3.0;
        let point = dvec_add(camera.position, dvec_scale(direction, start + segment * amount));
        let radius = dvec_length(dvec_sub(point, PLANET_CENTER));
        let normalized_height =
            ((radius - FLOW_PLANET_RADIUS_WORLD) / thickness).clamp(0.0, 1.0);
        let local_density = 1.0 - normalized_height;
        density += local_density * local_density;
        sample += 1;
    }
    let optical_depth =
        (segment / thickness * density / 3.0 * entry as f64 * 0.72).clamp(0.0, 3.0) as f32;
    if optical_depth <= 0.0 {
        None
    } else {
        Some((start, end, optical_depth))
    }
}

fn atmospheric_alpha(optical_depth: f32) -> f32 {
    1.0 - 1.0 / (1.0 + optical_depth.max(0.0) * 1.8)
}

fn atmospheric_alpha_to_depth(
    start: f64,
    end: f64,
    full_optical_depth: f32,
    ray_forward: f64,
    camera_depth: f32,
) -> f32 {
    let visible_end = if camera_depth < f32::MAX {
        end.min(camera_depth as f64 / ray_forward.max(0.001))
    } else {
        end
    };
    if visible_end <= start {
        return 0.0;
    }
    let fraction = ((visible_end - start) / (end - start)) as f32;
    atmospheric_alpha(full_optical_depth * fraction)
}

fn apply_atmospheric_shell(camera: &FlowCamera, x_scale: f32) {
    if flow_atmosphere_amount(camera) <= 0.0 {
        return;
    }
    const TILE: usize = 4;
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let depth_buffer = core::ptr::addr_of!(DEPTH).cast::<f32>();
    let mut y = 0usize;
    while y < HEIGHT {
        let mut x = 0usize;
        while x < WIDTH {
            let center_x = (x + TILE / 2).min(WIDTH - 1) as f32 + 0.5;
            let center_y = (y + TILE / 2).min(HEIGHT - 1) as f32 + 0.5;
            let direction = flow_camera_ray(camera, x_scale, center_x, center_y);
            if let Some((start, end, full_optical_depth)) =
                atmosphere_ray_segment(camera, direction)
            {
                let forward_amount = dvec_dot_vec3(direction, camera.forward).max(0.001);
                let mut tile_y = y;
                while tile_y < (y + TILE).min(HEIGHT) {
                    let sky_height = 1.0 - tile_y as f32 / HEIGHT as f32;
                    let red = 28.0 + sky_height * 25.0;
                    let green = 72.0 + sky_height * 48.0;
                    let blue = 126.0 + sky_height * 76.0;
                    let mut tile_x = x;
                    while tile_x < (x + TILE).min(WIDTH) {
                        let stored_depth = unsafe {
                            depth_buffer.add(tile_y * WIDTH + tile_x).read()
                        };
                        let optical_alpha = atmospheric_alpha_to_depth(
                            start,
                            end,
                            full_optical_depth,
                            forward_amount,
                            stored_depth,
                        );
                        if optical_alpha > 0.0 {
                            let alpha = optical_alpha * (186.0 + sky_height * 34.0);
                            blend_pixel(
                                frame,
                                tile_x as i32,
                                tile_y as i32,
                                red as u8,
                                green as u8,
                                blue as u8,
                                alpha.clamp(0.0, 220.0) as u8,
                            );
                        }
                        tile_x += 1;
                    }
                    tile_y += 1;
                }
            }
            x += TILE;
        }
        y += TILE;
    }
}

fn render_world_journey(
    elapsed: f32,
    x_scale: f32,
    flow: &FlowCamera,
) {
    apply_atmospheric_shell(flow, x_scale);
    draw_flow_sun(flow, 1.0, x_scale);
    render_flow_surface(flow, elapsed, x_scale);
    draw_entry_sheath(flow, flow_entry_heat(flow));
}

fn star_space_color(direction: DVec3) -> (u8, u8, u8) {
    let sample_x = (direction.x * 1_024.0) as i32;
    let sample_y = (direction.y * 1_024.0) as i32;
    let sample_z = (direction.z * 1_024.0) as i32;
    let vertical = direction.y.abs() as f32;
    let horizontal = direction.x.abs() as f32;
    let diagonal = direction.y as f32 + direction.x as f32 * 0.19 + sine(sample_z * 2) * 0.11;
    let band = (1.0 - diagonal.abs() / 0.78).clamp(0.0, 1.0);
    let cloud = sine(sample_x * 3 + sample_y * 2 + 117)
        + sine(sample_x + sample_y * 5 + 631) * 0.52
        + sine(sample_z * 7 - sample_y * 3 + 349) * 0.24;
    let dust = (band * band * (0.64 + cloud * 0.22)).max(0.0);
    let vignette = (1.0 - horizontal * 0.31 - vertical * 0.24).clamp(0.42, 1.0);
    (
        ((2.0 + dust * 15.0) * vignette) as u8,
        ((3.0 + dust * (10.0 + band * 4.0)) * vignette) as u8,
        ((12.0 + dust * 29.0) * vignette) as u8,
    )
}

fn fill_star_space(camera: &FlowCamera, x_scale: f32) {
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let mut y = 0usize;
    while y < HEIGHT {
        let mut x = 0usize;
        while x < WIDTH {
            let direction = flow_camera_ray(
                camera,
                x_scale,
                x as f32 + 0.5,
                y as f32 + 0.5,
            );
            let (r, g, b) = star_space_color(direction);
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
fn canonical_flyby_segment(
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

fn approach_travel_state(time: f64) -> (f64, f64, f64) {
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

fn approach_camera_reference(time: f64) -> (f64, f64, f64) {
    let entry_row = approach_travel_state(ORBIT_ENTRY as f64).0;
    let clean = approach_travel_state(CLEAN_BIRTH as f64);
    let clean_clearance = TRAJECTORY_ENTRY_CONTROL_CLEARANCE
        + (entry_row - clean.0) * GRID_ROW_WORLD_SPACING;
    let clean_rate = -clean.1 * GRID_ROW_WORLD_SPACING;
    let clean_acceleration = -clean.2 * GRID_ROW_WORLD_SPACING;
    if time <= CLEAN_BIRTH as f64 {
        let travel = approach_travel_state(time);
        return (
            TRAJECTORY_ENTRY_CONTROL_CLEARANCE
                + (entry_row - travel.0) * GRID_ROW_WORLD_SPACING,
            -travel.1 * GRID_ROW_WORLD_SPACING,
            -travel.2 * GRID_ROW_WORLD_SPACING,
        );
    }
    let duration = ORBIT_ENTRY as f64 - CLEAN_BIRTH as f64;
    quintic_component(
        (time - CLEAN_BIRTH as f64) / duration,
        duration,
        clean_clearance,
        TRAJECTORY_ENTRY_CONTROL_CLEARANCE,
        clean_rate,
        TRAJECTORY_ENTRY_RATE,
        clean_acceleration,
        0.0,
    )
}

fn approach_travel_row(time: f32) -> f32 {
    approach_travel_state(time as f64).0 as f32
}

fn travel_row(time: f32) -> f32 {
    approach_travel_row(time)
}

fn birth_shape(birth_position: f32) -> f32 {
    smoothstep(
        (birth_position - approach_travel_row(GRID_SQUARE_START))
            / (approach_travel_row(GRID_HORIZONTAL) - approach_travel_row(GRID_SQUARE_START)),
    )
}

fn birth_neon(birth_position: f32) -> f32 {
    smoothstep(
        (birth_position - approach_travel_row(GRID_SQUARE_START))
            / (approach_travel_row(GRID_COLOR_FULL) - approach_travel_row(GRID_SQUARE_START)),
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

fn render_continuous_grid(
    time: f32,
    flow: &FlowCamera,
    x_scale: f32,
) {
    render_continuous_grid_warped(
        time,
        0.0,
        flow,
        1.0,
        x_scale,
    );
}

fn render_continuous_grid_warped(
    time: f32,
    neon: f32,
    flow: &FlowCamera,
    grid_visibility: f32,
    x_scale: f32,
) {
    fill_star_space(flow, x_scale);
    let frame = core::ptr::addr_of_mut!(FRAME).cast::<u8>();
    let clean_birth = CLEAN_BIRTH;
    let (fog_start, fog_end) = grid_fog_range(time);
    let particle_transport = flow_field_transport(time);

    let mut index = 0usize;
    while index < STAR_LINES {
        let Some((local_a, local_b, depth)) = flyby_segment(index, time, clean_birth) else {
            index += 1;
            continue;
        };
        let Some((projected_a, projected_b)) = project_flow_star_segment(
            particle_transport,
            local_a,
            local_b,
            depth,
            flow,
            x_scale,
        ) else {
            index += 1;
            continue;
        };
        let a = (projected_a.0, projected_a.1);
        let b = (projected_b.0, projected_b.1);
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
            record_depth_line(
                a,
                b,
                projected_a.2,
                projected_b.2,
            );
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

    if grid_visibility <= 0.0 {
        return;
    }
    draw_ordered_radials(
        time,
        clean_birth,
        flow,
        grid_visibility,
        x_scale,
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
            if visible_depth <= fog_start {
                index += 1;
                continue;
            }
            let distance_visibility = if threshold_ring {
                1.0
            } else {
                smoothstep(
                    (visible_depth - fog_start) / (fog_end - fog_start),
                )
            };
            let point_a = grid_section_point(lane_a, ring_shape);
            let point_b = grid_section_point(lane_b, ring_shape);
            let Some((a, b)) = project_flow_grid_segment(
                time,
                point_a,
                depth,
                point_b,
                depth,
                flow,
                x_scale,
            ) else {
                index += 1;
                continue;
            };
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
            draw_bloom_line(frame, (a.0, a.1), (b.0, b.1), color, alpha, glow);
            record_depth_line((a.0, a.1), (b.0, b.1), a.2, b.2);
            blend_pixel(frame, b.0 as i32, b.1 as i32, color.0, color.1, color.2, alpha);
        }
        index += 1;
    }
}

fn draw_ordered_radials(
    time: f32,
    clean_birth: f32,
    flow: &FlowCamera,
    grid_visibility: f32,
    x_scale: f32,
) {
    let introduction = STAR_ORGANIZE + 1.5;
    if time <= introduction {
        return;
    }
    let introduction_position = travel_row(introduction);
    let clean_position = travel_row(clean_birth);
    let (fog_start, fog_end) = grid_fog_range(time);
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
            if visible_depth <= fog_start {
                ring += 1;
                continue;
            }
            let mut outer_shape = birth_shape(outer_birth);
            let mut inner_shape = birth_shape(inner_birth);
            if outer > inner && inner < fog_start {
                let span = (outer - inner).max(0.001);
                let clip = ((fog_start - inner) / span).clamp(0.0, 1.0);
                inner += (outer - inner) * clip;
                inner_shape += (outer_shape - inner_shape) * clip;
            } else if inner > outer && outer < fog_start {
                let span = (inner - outer).max(0.001);
                let clip = ((fog_start - outer) / span).clamp(0.0, 1.0);
                outer += (inner - outer) * clip;
                outer_shape += (inner_shape - outer_shape) * clip;
            }
            let visibility = smoothstep(
                (visible_depth - fog_start) / (fog_end - fog_start),
            ) * prominence;
            let point_a = grid_section_point(lane, outer_shape);
            let point_b = grid_section_point(lane, inner_shape);
            let index = lane_index * GRID_RINGS + ring;
            let Some((a, b)) = project_flow_grid_segment(
                time,
                point_a,
                outer,
                point_b,
                inner,
                flow,
                x_scale,
            ) else {
                ring += 1;
                continue;
            };
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
            draw_bloom_line(frame, (a.0, a.1), (b.0, b.1), color, alpha, glow);
            record_depth_line((a.0, a.1), (b.0, b.1), a.2, b.2);
            ring += 1;
        }
        lane_index += 1;
    }
}

fn grid_section_point(lane: f32, reshaping: f32) -> (f32, f32) {
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

fn fast_sqrt(value: f32) -> f32 {
    if value <= 0.0 {
        return 0.0;
    }
    let mut inverse = f32::from_bits(0x5f37_59df - (value.to_bits() >> 1));
    inverse *= 1.5 - value * 0.5 * inverse * inverse;
    value * inverse
}

fn sqrt_f64(value: f64) -> f64 {
    if value <= 0.0 {
        return 0.0;
    }
    let result: f64;
    unsafe {
        asm!(
            "sqrtsd {result}, {value}",
            value = in(xmm_reg) value,
            result = lateout(xmm_reg) result,
            options(pure, nomem, nostack),
        );
    }
    result
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
        y += 1;
    }
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

fn apply_control_byte(byte: u8, elapsed: &mut f32) -> bool {
    if byte == b'q' || byte == b'Q' || byte == 3 {
        return true;
    }
    match byte {
        b'1' => *elapsed = 0.0,
        b'2' => *elapsed = GRID_SQUARE_START,
        b'3' => *elapsed = GRID_HORIZONTAL,
        b'4' => *elapsed = ORBIT_SEEK,
        b'5' => *elapsed = CONTINENT_SEEK,
        b'6' => *elapsed = DESCENT,
        b' ' => *elapsed += 5.0,
        b']' | b'.' | b'>' | b'f' | b'F' => *elapsed += 2.0,
        b'[' | b',' | b'<' | b'b' | b'B' => *elapsed = (*elapsed - 2.0).max(0.0),
        _ => {}
    }
    false
}

fn handle_input(elapsed: &mut f32) -> bool {
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
            apply_arrow_direction(direction, elapsed);
            continue;
        }
        if apply_control_byte(byte, elapsed) {
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
    unsafe {
        asm!(
            "syscall",
            in("rax") SYS_EXIT,
            in("rdi") code,
            options(noreturn)
        )
    }
}
