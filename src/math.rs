//! Phase 1 mechanical extraction: existing behavior, not the proposed flight.
use core::arch::asm;

use super::{
    sine,
};

#[derive(Clone, Copy)]
pub(super) struct Vec3 {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) z: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct DVec3 {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) z: f64,
}

pub(super) fn smootherstep_state(time: f64, start: f64, end: f64) -> (f64, f64, f64) {
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



pub(super) fn quintic_component(
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



pub(super) fn sine_sample_c2_state(index: f64) -> (f64, f64, f64) {
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

// Freestanding, range-reduced f64 log/exp for positive braking ratios.


pub(super) fn brake_exp(value: f64) -> f64 {
    let exponent = (value / core::f64::consts::LN_2) as i32;
    let remainder = value - exponent as f64 * core::f64::consts::LN_2;
    let mut term = 1.0;
    let mut sum = 1.0;
    let mut index = 1;
    while index <= 18 {
        term *= remainder / index as f64;
        sum += term;
        index += 1;
    }
    sum * f64::from_bits(((exponent + 1_023) as u64) << 52)
}

pub(super) fn smootherstep_f64(value: f64) -> f64 {
    let x = value.clamp(0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

pub(super) fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

pub(super) fn smootherstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}



pub(super) fn dvec_from_vec3(value: Vec3) -> DVec3 {
    DVec3 { x: value.x as f64, y: value.y as f64, z: value.z as f64 }
}

pub(super) fn vec3_from_dvec(value: DVec3) -> Vec3 {
    Vec3 { x: value.x as f32, y: value.y as f32, z: value.z as f32 }
}

pub(super) fn dvec_add(a: DVec3, b: DVec3) -> DVec3 {
    DVec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z }
}

pub(super) fn dvec_sub(a: DVec3, b: DVec3) -> DVec3 {
    DVec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}

pub(super) fn dvec_scale(a: DVec3, scale: f64) -> DVec3 {
    DVec3 { x: a.x * scale, y: a.y * scale, z: a.z * scale }
}

pub(super) fn dvec_dot(a: DVec3, b: DVec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub(super) fn dvec_dot_vec3(a: DVec3, b: Vec3) -> f64 {
    a.x * b.x as f64 + a.y * b.y as f64 + a.z * b.z as f64
}

pub(super) fn dvec_length_squared(a: DVec3) -> f64 {
    dvec_dot(a, a)
}

pub(super) fn dvec_cross(a: DVec3, b: DVec3) -> DVec3 {
    DVec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

pub(super) fn dvec_length(a: DVec3) -> f64 {
    sqrt_f64(dvec_length_squared(a))
}

pub(super) fn dvec_normalize(a: DVec3) -> DVec3 {
    dvec_scale(a, 1.0 / dvec_length(a).max(1.0e-15))
}



pub(super) fn vec_scale(a: Vec3, scale: f32) -> Vec3 {
    Vec3 { x: a.x * scale, y: a.y * scale, z: a.z * scale }
}

pub(super) fn vec_dot(a: Vec3, b: Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub(super) fn vec_cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

pub(super) fn vec_length(a: Vec3) -> f32 {
    fast_sqrt(vec_dot(a, a))
}

pub(super) fn vec_normalize(a: Vec3) -> Vec3 {
    vec_scale(a, 1.0 / vec_length(a).max(0.000_1))
}

pub(super) fn fast_sqrt(value: f32) -> f32 {
    if value <= 0.0 {
        return 0.0;
    }
    let mut inverse = f32::from_bits(0x5f37_59df - (value.to_bits() >> 1));
    inverse *= 1.5 - value * 0.5 * inverse * inverse;
    value * inverse
}

pub(super) fn sqrt_f64(value: f64) -> f64 {
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
