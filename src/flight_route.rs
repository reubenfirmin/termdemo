//! One analytic spatial curve in a fixed planet-local metre frame. The scalar
//! arc cache is an inverse metric, not a sequence of animated camera poses.
use super::{DVec3,dvec_length,flight_math::Jet};
pub(super) const THETA_START:f64=0.001313443189103021;
pub(super) const THETA_END:f64=15.333530429250654;
pub(super) const ENTRY_POLAR:f64=0.7946946488758294;
pub(super) const AXIS_METRES:f64=7_801_802.502068698;
pub(super) const RADIUS_METRES:f64=6_371_000.0;
const SIGMA:f64=0.0026683618430697518;

pub(super) fn route_geometry(theta:f64)->[DVec3;4] {
    let t=Jet::variable(theta);
    let c=Jet::constant;
    let d=c(THETA_END)-t;
    let phi=t-c(0.8);
    let h=(phi.scale(-0.14910784101520966)-phi.square().scale(0.005309956078885768)).exp().scale(300_000.0);
    let taper=d.scale(1.0/SIGMA).square().scale(-1.0).exp();
    let envelope=d.square().square().scale(-1.0).exp();
    let b=c(RADIUS_METRES+20_000.0+30.0)+h*(c(1.0)-taper)
        +(c(45.984012228012084-20_000.0)-d.scale(777.886748302592))*envelope;
    let incoming=t.scale(-4.0).exp()*t.reciprocal().scale(1_000_000.0);
    let sn=t.sin(); let cs=t.cos();
    let a=incoming*sn;
    // Solve |P|²=B²+incoming². r=B would introduce an unwanted perigee/climb.
    let r=a+(b.square()+a.square()).sqrt();
    let q=t-c(THETA_START);
    let correction=(q.square().scale(-3510251014.5343633)
        +(q.square()*q).scale(-833003533673.272))*q.scale(-1000.0).exp();
    let y=r*cs+correction.scale(-0.9999999999682577);
    let z=r*sn-incoming+correction.scale(-0.000007967753208563013);
    let u=(z+c(45_000_000.0)).scale(1.0/20_000_000.0);
    let raw=u.square().square().scale(-1.0).exp()*u.sin().scale(400_000.0);
    let v=(z+c(25_000_000.0)).scale(1.0/15_000_000.0);
    let cutoff=if v.0[0]<=0.0 {c(1.0)} else if v.0[0]>=1.0 {c(0.0)} else {
        c(1.0)-v.square().square()*(c(35.0)+v*(c(-84.0)+v*(c(70.0)-v.scale(20.0))))
    };
    let x=raw*cutoff;
    core::array::from_fn(|i|DVec3{x:x.0[i],y:y.0[i],z:z.0[i]})
}
#[derive(Clone,Copy)]
struct ArcNode { theta:f64, s:f64, metric:f64 }
const EMPTY:ArcNode=ArcNode{theta:0.0,s:0.0,metric:0.0};
const CAPACITY:usize=45_000;
static mut ARC:[ArcNode;CAPACITY]=[EMPTY;CAPACITY];
static mut COUNT:usize=0;
pub(super) fn generate_route_arc() {
    let mut a=ArcNode{theta:THETA_START,s:0.0,metric:dvec_length(route_geometry(THETA_START)[1])};
    unsafe { core::ptr::addr_of_mut!(ARC[0]).write(a); }
    let mut count=1;
    while a.theta<THETA_END {
        let step=0.002f64.min(1e-7f64.max(a.theta.min(THETA_END-a.theta+0.0005)*0.004))/2.0;
        let theta=THETA_END.min(a.theta+step);
        let metric=dvec_length(route_geometry(theta)[1]);
        let s=a.s+(theta-a.theta)/6.0*(a.metric+4.0*dvec_length(route_geometry((theta+a.theta)*0.5)[1])+metric);
        a=ArcNode{theta,s,metric};
        assert!(count<CAPACITY);
        unsafe {core::ptr::addr_of_mut!(ARC[count]).write(a);}
        count+=1;
    }
    unsafe {core::ptr::addr_of_mut!(COUNT).write(count);}
}
pub(super) fn route_theta(s:f64)->f64 {
    let node=|i:usize|unsafe {core::ptr::addr_of!(ARC[i]).read()};
    let mut lo=0; let mut hi=unsafe {core::ptr::addr_of!(COUNT).read()}-1;
    let s=s.clamp(0.0,node(hi).s);
    while hi-lo>1 {let m=(lo+hi)/2; if node(m).s<s {lo=m;} else {hi=m;}}
    let a=node(lo);let b=node(hi);let h=b.theta-a.theta;
    let mut u=((s-a.s)/(b.s-a.s)).clamp(0.0,1.0);
    for _ in 0..6 {
        let value=a.s+(b.s-a.s)*u*u*(3.0-2.0*u)
            +h*(a.metric*u*(1.0-u)*(1.0-u)+b.metric*u*u*(u-1.0));
        let derivative=(b.s-a.s)*6.0*u*(1.0-u)
            +h*(a.metric*(1.0-4.0*u+3.0*u*u)+b.metric*(3.0*u*u-2.0*u));
        u=(u-(value-s)/derivative).clamp(0.0,1.0);
    }
    a.theta+h*u
}
