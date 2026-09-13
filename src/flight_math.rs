//! Double-precision elementary functions for the analytic flight, independent
//! of the protected opening's (deliberately unchanged) raster sine table.
use super::sqrt_f64;

pub(super) fn exp(x: f64) -> f64 {
    if x < -745.0 { return 0.0; }
    if x < -700.0 { return exp(x + 700.0) * exp(-700.0); }
    super::brake_exp(x)
}

pub(super) fn sin_cos(x: f64) -> (f64, f64) {
    let half_pi = core::f64::consts::FRAC_PI_2;
    let quadrant = (x / half_pi + if x >= 0.0 { 0.5 } else { -0.5 }) as i64;
    let r = x - quadrant as f64 * half_pi;
    let mut s = r;
    let mut c = 1.0;
    let mut st = r;
    let mut ct = 1.0;
    for k in 1..=10 {
        st *= -r * r / ((2 * k) * (2 * k + 1)) as f64;
        ct *= -r * r / ((2 * k - 1) * (2 * k)) as f64;
        s += st;
        c += ct;
    }
    match quadrant.rem_euclid(4) { 0 => (s,c), 1 => (c,-s), 2 => (-s,-c), _ => (-c,s) }
}

pub(super) fn atan2(y: f64, x: f64) -> f64 {
    let mut z = y.abs() / x.abs().max(1e-300);
    let invert = z > 1.0;
    if invert { z = 1.0 / z; }
    let fold = z > 0.41421356237309503;
    if fold { z = (z - 1.0) / (z + 1.0); }
    let mut term = z;
    let mut a = z;
    for k in 1..=24 { term *= -z * z; a += term / (2*k+1) as f64; }
    if fold { a += core::f64::consts::FRAC_PI_4; }
    if invert { a = core::f64::consts::FRAC_PI_2 - a; }
    if x < 0.0 { a = core::f64::consts::PI - a; }
    if y < 0.0 { -a } else { a }
}

/// Value and its first three derivatives. Product/composition rules keep the
/// spatial curve and its derivatives one authority, including the lateral weave.
#[derive(Clone, Copy)]
pub(super) struct Jet(pub(super) [f64; 4]);
impl Jet {
    pub(super) fn constant(x: f64) -> Self { Self([x,0.0,0.0,0.0]) }
    pub(super) fn variable(x: f64) -> Self { Self([x,1.0,0.0,0.0]) }
    pub(super) fn scale(self, k: f64) -> Self { Self(self.0.map(|x| x*k)) }
    fn compose(self, f: [f64;4]) -> Self {
        let [_,a,b,c] = self.0;
        Self([f[0],f[1]*a,f[2]*a*a+f[1]*b,f[3]*a*a*a+3.0*f[2]*a*b+f[1]*c])
    }
    pub(super) fn exp(self) -> Self {
        let e = exp(self.0[0]);
        if e == 0.0 { Self::constant(0.0) } else { self.compose([e,e,e,e]) }
    }
    pub(super) fn reciprocal(self) -> Self {
        let r = 1.0/self.0[0]; self.compose([r,-r*r,2.0*r*r*r,-6.0*r*r*r*r])
    }
    pub(super) fn sqrt(self) -> Self {
        let r = sqrt_f64(self.0[0]);
        self.compose([r,0.5/r,-0.25/(r*r*r),0.375/(r*r*r*r*r)])
    }
    pub(super) fn sin(self) -> Self { let (s,c)=sin_cos(self.0[0]); self.compose([s,c,-s,-c]) }
    pub(super) fn cos(self) -> Self { let (s,c)=sin_cos(self.0[0]); self.compose([c,-s,-c,s]) }
    pub(super) fn square(self) -> Self { self*self }
}
impl core::ops::Add for Jet {
    type Output=Self;
    fn add(self, b:Self)->Self { Self(core::array::from_fn(|i|self.0[i]+b.0[i])) }
}
impl core::ops::Sub for Jet {
    type Output=Self;
    fn sub(self, b:Self)->Self { Self(core::array::from_fn(|i|self.0[i]-b.0[i])) }
}
impl core::ops::Mul for Jet {
    type Output=Self;
    fn mul(self, b:Self)->Self {
        let a=self.0; let b=b.0;
        Self([a[0]*b[0],a[1]*b[0]+a[0]*b[1],
            a[2]*b[0]+2.0*a[1]*b[1]+a[0]*b[2],
            a[3]*b[0]+3.0*a[2]*b[1]+3.0*a[1]*b[2]+a[0]*b[3]])
    }
}
