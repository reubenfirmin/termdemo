//! Proposed single speed function, independent of position, curvature, or orbit.
//! Audit-only until the complete geometric path can be parameterized by its
//! integral. This must not be installed as another cap on the old controller.

pub const START: f64 = 4.0;
pub const EXIT_MPS: f64 = 18.0 * (13_500.0 * 1_609.344 / 0.115);
pub const CRUISE_MPS: f64 = 133.310_401_397_311_04;
pub const SCALE_SECONDS: f64 = 21.970_931_282_356_474;

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub speed: f64,
    pub acceleration: f64,
    pub jerk: f64,
}

/// v(t) = cruise + (exit-cruise) / (1 + ((t-4)/scale)^32).
/// The extension before START is only a boundary value; the approved opening
/// remains owned by the original trajectory. No runtime parameter fitting.
pub fn sample(time: f64) -> Sample {
    let elapsed = time - START;
    if elapsed <= 0.0 {
        return Sample { speed: EXIT_MPS, acceleration: 0.0, jerk: 0.0 };
    }
    let x = elapsed / SCALE_SECONDS;
    let x2 = x * x;
    let x4 = x2 * x2;
    let x8 = x4 * x4;
    let x16 = x8 * x8;
    let z = x16 * x16;
    let z_rate = 32.0 * z / elapsed;
    let z_acceleration = 31.0 * z_rate / elapsed;
    let denominator = 1.0 + z;
    let amplitude = EXIT_MPS - CRUISE_MPS;
    Sample {
        speed: CRUISE_MPS + amplitude / denominator,
        acceleration: -amplitude * z_rate / (denominator * denominator),
        jerk: amplitude * (2.0 * z_rate * z_rate / denominator - z_acceleration)
            / (denominator * denominator),
    }
}

pub fn distance(start: f64, end: f64) -> f64 {
    let intervals = ((end - start).max(0.0) * 960.0) as usize + 1;
    let dt = (end - start).max(0.0) / intervals as f64;
    let mut result = 0.0;
    for i in 0..intervals {
        let t = start + i as f64 * dt;
        result += dt / 6.0 * (sample(t).speed
            + 4.0 * sample(t + dt * 0.5).speed + sample(t + dt).speed);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_function_is_positive_monotone_and_finite() {
        let mut previous = sample(START).speed;
        for tick in 4 * 1_920..=140 * 1_920 {
            let s = sample(tick as f64 / 1_920.0);
            assert!(s.speed.is_finite() && s.acceleration.is_finite() && s.jerk.is_finite());
            assert!(s.speed >= CRUISE_MPS && s.speed <= previous);
            assert!(s.acceleration <= 0.0);
            previous = s.speed;
        }
    }

    #[test]
    fn boundary_and_aircraft_deadline_are_explicit() {
        let s = sample(START);
        assert_eq!(s.speed, EXIT_MPS);
        assert_eq!(s.acceleration, 0.0);
        assert_eq!(s.jerk, 0.0);
        assert!((sample(40.0).speed - 600.0).abs() < 1.0e-8);
    }

    #[test]
    fn late_braking_relaxes_in_absolute_and_fractional_terms() {
        let mut previous = sample(30.0);
        for tick in 30 * 1_920 + 1..=58 * 1_920 {
            let current = sample(tick as f64 / 1_920.0);
            assert!(current.acceleration >= previous.acceleration);
            assert!(-current.acceleration / current.speed
                <= -previous.acceleration / previous.speed + 1.0e-15);
            previous = current;
        }
    }

    #[test]
    fn function_cannot_silently_authorize_the_old_ground_deadline() {
        // Even a vertical path needs at least 9,124 metres after the 40 s pose.
        // The full function cannot cover that distance by 58 s; path shape
        // cannot correct this by secretly increasing actual world velocity.
        assert!(distance(40.0, 58.0) < 9_124.0);
    }
}
