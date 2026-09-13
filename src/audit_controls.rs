//! Phase 1 mechanical extraction: existing behavior, not the proposed flight.
#[cfg(feature = "phase0-audit")]
use super::{
    CONTINENT_SEEK, DESCENT, GRID_HORIZONTAL, GRID_SQUARE_START, ORBIT_SEEK, Playback,
    apply_arrow_direction, apply_control_byte, exit, render_demo, write_all,
};

fn phase7_frame_hash()->u64 {
    let mut hash=0xcbf29ce484222325u64;
    for i in 0..super::FRAME_BYTES {
        let value=unsafe {core::ptr::addr_of!(super::FRAME).cast::<u8>().add(i).read()};
        hash=(hash^value as u64).wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(feature = "phase0-audit")]
pub(super) fn run_playback_controls_audit() -> ! {
    let check = |condition: bool, message: &[u8]| {
        if !condition {
            write_all(b"playback controls audit failed: ");
            write_all(message);
            write_all(b"\n");
            exit(180);
        }
    };
    let mut playback = Playback { elapsed: 27.375, paused: false };
    render_demo(playback.elapsed, 1.0);
    let frame = phase7_frame_hash();
    let frozen_time = playback.elapsed.to_bits();
    check(!apply_control_byte(b' ', &mut playback) && playback.paused,
        b"Space does not pause");
    for _ in 0..600 {
        playback.advance(0.1);
    }
    check(playback.elapsed.to_bits() == frozen_time, b"paused playback advances or seeks");
    render_demo(playback.elapsed, 1.0);
    check(frame == phase7_frame_hash(), b"paused frame is not stable");
    check(!apply_control_byte(b' ', &mut playback) && !playback.paused,
        b"second Space does not resume");
    check(playback.elapsed.to_bits() == frozen_time, b"resume changes the held timestamp");
    playback.advance(1.0 / 60.0);
    check(playback.elapsed == 27.375f32 + 1.0 / 60.0,
        b"resume accumulates paused time");

    apply_control_byte(b' ', &mut playback);
    for (key, expected) in [(b'1', 0.0), (b'2', GRID_SQUARE_START),
        (b'3', GRID_HORIZONTAL), (b'4', ORBIT_SEEK), (b'5', CONTINENT_SEEK), (b'6', DESCENT)] {
        apply_control_byte(key, &mut playback);
        playback.advance(1.0);
        check(playback.paused && playback.elapsed == expected,
            b"checkpoint seeking changes pause state or target");
    }
    for key in [b']', b'.', b'>', b'f', b'F'] {
        playback.elapsed = 10.0;
        apply_control_byte(key, &mut playback);
        check(playback.paused && playback.elapsed == 12.0, b"forward seek changed");
    }
    for key in [b'[', b',', b'<', b'b', b'B'] {
        playback.elapsed = 10.0;
        apply_control_byte(key, &mut playback);
        check(playback.paused && playback.elapsed == 8.0, b"backward seek changed");
    }
    playback.elapsed = 0.5;
    apply_arrow_direction(b'D', &mut playback.elapsed);
    check(playback.paused && playback.elapsed == 0.0, b"backward seek passes the start");
    apply_arrow_direction(b'C', &mut playback.elapsed);
    check(playback.paused && playback.elapsed == 2.0, b"Right Arrow does not seek while paused");
    for key in [b'q', b'Q', 3] {
        check(apply_control_byte(key, &mut playback), b"quit is unavailable while paused");
    }
    write_all(b"playback controls audit ok: Space=pause/resume frozen-time/frame no-catch-up seek-while-paused quit\n");
    exit(0)
}
