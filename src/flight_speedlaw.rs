//! One total-speed function, fitted offline. No altitude, phase or curvature
//! controller is allowed to change this value. Units are metres and seconds.
use super::flight_math::exp;
pub(super) const FLIGHT_START: f64 = 4.0;
pub(super) const FLIGHT_END: f64 = 77.0;
pub(super) const START_SPEED_METRES: f64 = 70_947_663.29049851;
pub(super) const BRAKE_COEFFICIENTS: [f64;25] = [
    0.283693214667677, 0.06909552551168413, 0.3793492384574534,
    0.500621179259363, 0.0, 0.0, 0.0, 0.7310930544974156,
    0.7099061360059641, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.06455428229223392, 6.189151518944869, 2.7530592215803567,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];
fn basis(n: usize, u: f64) -> [f64;26] {
    let mut b=[0.0;26];
    let base=if u<=0.5 {1.0-u} else {u};
    let mut power=1.0;
    for _ in 0..n { power*=base; }
    if u<=0.5 {
        b[0]=power;
        for i in 1..=n { b[i]=b[i-1]*(n-i+1) as f64/i as f64*u/(1.0-u); }
    } else {
        b[n]=power;
        for i in (1..=n).rev() { b[i-1]=b[i]*i as f64/(n-i+1) as f64*(1.0-u)/u; }
    }
    b
}
/// v, dv/dt, d²v/dt², including all radial and lateral motion.
pub(super) fn flight_speed_state(time: f64)->(f64,f64,f64) {
    let x=time.clamp(FLIGHT_START,FLIGHT_END)-FLIGHT_START;
    let e=exp(-x/0.25);
    let u=(x-0.375+0.5*e-0.125*e*e)/72.625;
    let b=basis(25,u);
    let mut cdf=0.0; let mut exponent=0.0;
    for i in (0..25).rev() { cdf+=b[i+1]; exponent+=BRAKE_COEFFICIENTS[i]*cdf; }
    let v=START_SPEED_METRES*exp(-exponent);
    let b=basis(24,u); let bd=basis(23,u);
    let mut density=0.0; let mut derivative=0.0;
    for i in 0..25 { density+=25.0*BRAKE_COEFFICIENTS[i]*b[i]; }
    for i in 0..24 { derivative+=600.0*(BRAKE_COEFFICIENTS[i+1]-BRAKE_COEFFICIENTS[i])*bd[i]; }
    let g1=(1.0-e)*(1.0-e); let g2=8.0*e*(1.0-e);
    let rate=density*g1/72.625;
    let rate_derivative=derivative*(g1/72.625)*(g1/72.625)+density*g2/72.625;
    (v,-v*rate,v*(rate*rate-rate_derivative))
}

// Only the scalar integral is cached, not camera poses or motion segments.
const HZ: usize=960;
const COUNT: usize=73*HZ+1;
static mut DISTANCE: [f64;COUNT]=[0.0;COUNT];
pub(super) fn generate_flight_distance() {
    let mut total=0.0;
    let dt=1.0/HZ as f64;
    let mut first=flight_speed_state(4.0).0;
    for i in 1..COUNT {
        let t=4.0+i as f64*dt;
        let last=flight_speed_state(t).0;
        total+=dt/6.0*(first+4.0*flight_speed_state(t-dt*0.5).0+last);
        unsafe { core::ptr::addr_of_mut!(DISTANCE[i]).write(total); }
        first=last;
    }
}
pub(super) fn flight_distance(time:f64)->f64 {
    let scaled=(time.clamp(4.0,77.0)-4.0)*HZ as f64;
    let i=(scaled as usize).min(COUNT-2); let u=scaled-i as f64;
    let a=unsafe { core::ptr::addr_of!(DISTANCE[i]).read() };
    let b=unsafe { core::ptr::addr_of!(DISTANCE[i+1]).read() };
    let t=4.0+i as f64/HZ as f64;
    a+(b-a)*u*u*(3.0-2.0*u)+(flight_speed_state(t).0*u*(1.0-u)*(1.0-u)
        +flight_speed_state(t+1.0/HZ as f64).0*u*u*(u-1.0))/HZ as f64
}
