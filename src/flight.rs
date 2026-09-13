//! Production flight: opening, then P(s(t)). The fixed world registration was
//! chosen once to match the approved opening. Changing routes cannot move it.
use super::*;
use super::flight_math::{atan2,sin_cos};
pub const FLOW_PATH_END:f64=77.0;

#[derive(Clone,Copy)]
#[allow(dead_code)] // Full boundary state is exported by audits and reserved for manual flight.
pub(super) struct TrajectoryState {
    pub(super) position:DVec3,
    pub(super) velocity:DVec3,
    pub(super) acceleration:DVec3,
    pub(super) forward:Vec3,
    pub(super) down:Vec3,
    pub(super) roll:f64,
    #[cfg(feature="phase0-audit")]
    pub(super) clearance_miles:f64,
    pub(super) phase:f64,
    pub(super) phase_rate:f64,
}
const ZERO:DVec3=DVec3{x:0.0,y:0.0,z:0.0};
#[derive(Clone,Copy)]
struct FixedFrame {origin:DVec3, x:DVec3,y:DVec3,z:DVec3,terrain:[DVec3;3]}
// World data, not calculated from the current trajectory/camera. Captured from
// the reviewed registration; all objects and alternative cameras use it.
static mut FRAME:FixedFrame=FixedFrame{
    origin:DVec3{x:0.0,y:-23.67113324021222,z:-188.41925278147025},
    x:DVec3{x:-1.0,y:0.0,z:0.0},
    y:DVec3{x:0.0,y:-0.992707663087438,z:0.12054665340637995},
    z:DVec3{x:0.0,y:0.12054665340637995,z:0.9927076630874377},terrain:[ZERO;3]
};
fn frame()->FixedFrame {unsafe {core::ptr::addr_of!(FRAME).read()}}
pub(super) fn cross64(a:DVec3,b:DVec3)->DVec3 {
    DVec3{x:a.y*b.z-a.z*b.y,y:a.z*b.x-a.x*b.z,z:a.x*b.y-a.y*b.x}
}
pub(super) fn local_direction(v:DVec3)->DVec3 {
    let f=frame(); DVec3{x:dvec_dot(v,f.x),y:dvec_dot(v,f.y),z:dvec_dot(v,f.z)}
}
pub(super) fn world_direction(v:DVec3)->DVec3 {
    let f=frame();dvec_add(dvec_scale(f.x,v.x),dvec_add(dvec_scale(f.y,v.y),dvec_scale(f.z,v.z)))
}
pub(super) fn planet_center()->DVec3 {frame().origin}
pub(super) fn world_point(local_metres:DVec3)->DVec3 {
    dvec_add(planet_center(),dvec_scale(world_direction(local_metres),1.0/METRES_PER_WORLD_UNIT))
}
pub(super) fn local_point(world:DVec3)->DVec3 {
    dvec_scale(local_direction(dvec_sub(world,planet_center())),METRES_PER_WORLD_UNIT)
}
pub(super) fn generate_trajectory() {
    generate_approved_opening();
    let mut f=frame();
    let up=dvec_normalize(dvec_from_vec3(landing_up()));
    let fw=dvec_from_vec3(landing_forward());
    let fw=dvec_normalize(dvec_sub(fw,dvec_scale(up,dvec_dot(fw,up))));
    let sn=17.0/13_500.0;let cs=sqrt_f64(1.0-sn*sn);
    let normal=dvec_add(dvec_scale(up,cs),dvec_scale(fw,sn));
    let tangent=dvec_add(dvec_scale(up,-sn),dvec_scale(fw,cs));
    // Fixed planet orientation, NOT a read of the selected route's endpoint.
    let (s,c)=sin_cos(15.333530429250654);
    f.terrain=[cross64(normal,tangent),
        dvec_sub(dvec_scale(normal,c),dvec_scale(tangent,s)),
        dvec_add(dvec_scale(normal,s),dvec_scale(tangent,c))];
    unsafe {core::ptr::addr_of_mut!(FRAME).write(f);}
    generate_route_arc();
    generate_flight_distance();
}
pub(super) fn terrain_direction(world:DVec3)->Vec3 {
    let p=local_direction(world);let t=frame().terrain;
    vec3_from_dvec(dvec_add(dvec_scale(t[0],p.x),dvec_add(dvec_scale(t[1],p.y),dvec_scale(t[2],p.z))))
}
pub(super) fn terrain_to_world(p:Vec3)->DVec3 {
    let p=dvec_from_vec3(p);let t=frame().terrain;
    world_direction(DVec3{x:dvec_dot(p,t[0]),y:dvec_dot(p,t[1]),z:dvec_dot(p,t[2])})
}
pub(super) fn trajectory_state(time:f64)->TrajectoryState {
    if time<=4.0 {
        let mut state=legacy_trajectory_state(time);
        // Diagnostic orbit progress belongs to the actual planet, even before
        // capture. Preserve all native kinematics, not the old phase=0 label.
        let p=local_point(state.position);
        let v=dvec_scale(local_direction(state.velocity),METRES_PER_WORLD_UNIT);
        state.phase=(atan2(p.z,p.y)-ENTRY_POLAR)*1024.0/core::f64::consts::TAU;
        state.phase_rate=(p.y*v.z-p.z*v.y)/(p.y*p.y+p.z*p.z)*1024.0/core::f64::consts::TAU;
        return state;
    }
    // Playback stops at the review endpoint until manual flight (phase 5) exists.
    // There is deliberately no invented post-arrival orbit/controller here.
    let t=time.min(FLIGHT_END);
    let theta=route_theta(flight_distance(t));
    let [p,d,e,_]=route_geometry(theta);
    let metric=dvec_length(d);let tangent=dvec_scale(d,1.0/metric);
    let curvature=dvec_sub(dvec_scale(e,1.0/(metric*metric)),
        dvec_scale(d,dvec_dot(d,e)/(metric*metric*metric*metric)));
    let (v,dv,_)=flight_speed_state(t);
    let velocity=dvec_scale(world_direction(tangent),v/METRES_PER_WORLD_UNIT);
    let acceleration=dvec_scale(world_direction(dvec_add(dvec_scale(tangent,dv),dvec_scale(curvature,v*v))),1.0/METRES_PER_WORLD_UNIT);
    let (sn,cs)=sin_cos(theta);
    let polar=theta+atan2(p.z*cs-p.y*sn,p.y*cs+p.z*sn);
    let angular_rate=(p.y*tangent.z-p.z*tangent.y)*v/(p.y*p.y+p.z*p.z);
    TrajectoryState {position:world_point(p),velocity,acceleration,
        forward:vec3_from_dvec(dvec_normalize(velocity)),
        down:vec3_from_dvec(world_direction(dvec_normalize(dvec_scale(p,-1.0)))),
        phase:(polar-ENTRY_POLAR)*1024.0/core::f64::consts::TAU,
        phase_rate:angular_rate*1024.0/core::f64::consts::TAU,roll:0.0,
        #[cfg(feature="phase0-audit")]
        clearance_miles:(dvec_length(p)-RADIUS_METRES)/METERS_PER_MILE,
    }
}
pub(super) fn flight_camera_axes(state:&TrajectoryState)->(Vec3,Vec3,Vec3) {
    let p=local_point(state.position);let radius=dvec_length(p);
    let inward=dvec_scale(p,-1.0/radius);
    let flight=local_direction(dvec_normalize(state.velocity));
    let tangent=dvec_normalize(dvec_sub(flight,dvec_scale(inward,dvec_dot(flight,inward))));
    let ratio=(RADIUS_METRES/radius).min(1.0);
    let limb=sqrt_f64(1.0-ratio*ratio);
    let aircraft=smootherstep_f64((100_000.0-(radius-RADIUS_METRES))/90_000.0);
    let (sn,cs)=sin_cos((4.0+8.0*aircraft)*core::f64::consts::PI/180.0);
    let horizon=dvec_add(dvec_scale(tangent,ratio*cs-limb*sn),dvec_scale(inward,limb*cs+ratio*sn));
    let look=smootherstep_f64((ratio-0.05)/0.30);
    let forward=dvec_normalize(dvec_add(dvec_scale(flight,1.0-look),dvec_scale(horizon,look)));
    let reference=dvec_add(DVec3{x:-(1.0-look),y:0.0,z:0.0},dvec_scale(dvec_normalize(cross64(inward,tangent)),look));
    let right=dvec_normalize(dvec_sub(reference,dvec_scale(forward,dvec_dot(reference,forward))));
    let down=dvec_normalize(cross64(forward,right));
    // The original horizontal lens calibration is constant, not an animated FOV.
    (vec3_from_dvec(world_direction(forward)),
     vec3_from_dvec(dvec_scale(world_direction(right),1.0016957521438599)),
     vec3_from_dvec(world_direction(down)))
}
