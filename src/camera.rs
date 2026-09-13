//! One projection kernel for the fixed world and the physical camera.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct FlowCamera {
    pub(super) position: DVec3,
    pub(super) forward: Vec3,
    pub(super) right: Vec3,
    pub(super) down: Vec3,
    pub(super) altitude_miles: f64,
    pub(super) speed_world_per_second: f64,
    pub(super) near_plane: f64,
}

pub(super) fn field_star_exposure_camera(time:f32,camera:&FlowCamera)->FlowCamera {
    let exposure=if time<=4.0 {field_star_exposure(STAR_CATALOGUE,camera.position)} else {
        let z=local_point(camera.position).z;
        FIELD_MOTION_BLUR_EXPOSURE*(1.0-smootherstep_f64(
            (z+757357884.7961885)/(757357884.7961885-687918663.7148501))) as f32
    };
    if exposure==0.0 {*camera} else {flow_camera((time-exposure).max(0.0))}
}

pub(super) fn star_cell_may_project(
    layout: FieldLayout,
    cell: StarCellIdentity,
    camera: &FlowCamera,
) -> bool {
    let (far_s, near_s) = star_cell_bounds(layout, cell);
    let far_s = far_s.max(layout.field_start_s);
    let near_s = near_s.min(layout.star_volume_end_s);
    if near_s <= far_s {
        return false;
    }
    let center_s = (far_s + near_s) * 0.5;
    let half_axial = (near_s - far_s) * 0.5;
    let transverse_radius = if star_cell_ordinal(cell) < STAR_ORIGINAL_CELL_COUNT {
        layout
            .radius
            .max((FIELD_STAR_CATALOGUE_JOIN_S - far_s).max(0.0) * 1.08)
    } else {
        layout.radius
    };
    let center = dvec_add(layout.axis_origin, dvec_scale(layout.forward, center_s));
    let relative = dvec_sub(center, camera.position);
    let maximum_depth = dvec_dot_vec3(relative, camera.forward)
        + dvec_dot_vec3(layout.forward, camera.forward).abs() * half_axial
        + dvec_dot_vec3(layout.right, camera.forward).abs() * transverse_radius
        + dvec_dot_vec3(layout.down, camera.forward).abs() * transverse_radius;
    maximum_depth > camera.near_plane
}

pub(super) fn clip_exposure_half_space(
    first: f64,
    second: f64,
    enter: &mut f64,
    leave: &mut f64,
) -> bool {
    let change = second - first;
    if change.abs() <= 1.0e-18 {
        return first >= 0.0;
    }
    let crossing = -first / change;
    if change > 0.0 {
        *enter = enter.max(crossing);
    } else {
        *leave = leave.min(crossing);
    }
    *enter <= *leave
}

pub(super) fn project_field_star_exposure(
    layout: FieldLayout,
    identity: StarObjectIdentity,
    exposure_start: &FlowCamera,
    exposure_end: &FlowCamera,
    x_scale: f32,
) -> Option<((f32, f32, f32), (f32, f32, f32))> {
    let point = star_object_position(layout, identity);
    let first_projection = project_flow_depth(exposure_start, point);
    let second_projection = project_flow_depth(exposure_end, point);
    if let (Some(first), Some(second)) = (first_projection, second_projection) {
        return Some((
            (
                160.0 + (first.0 - 160.0) * x_scale,
                first.1,
                first.2,
            ),
            (
                160.0 + (second.0 - 160.0) * x_scale,
                second.1,
                second.2,
            ),
        ));
    }
    if first_projection.is_none() && second_projection.is_none() {
        return None;
    }
    let camera_coordinates = |camera: &FlowCamera| {
        let relative = dvec_sub(point, camera.position);
        (
            dvec_dot_vec3(relative, camera.right) * FLOW_FOCAL as f64 * x_scale as f64,
            dvec_dot_vec3(relative, camera.down) * FLOW_FOCAL as f64,
            dvec_dot_vec3(relative, camera.forward),
            camera.near_plane,
        )
    };
    let first = camera_coordinates(exposure_start);
    let second = camera_coordinates(exposure_end);
    let margin = 6.0f64;
    let mut enter = 0.0f64;
    let mut leave = 1.0f64;
    let planes = [
        (first.2 - first.3, second.2 - second.3),
        (first.0 + (160.0 + margin) * first.2, second.0 + (160.0 + margin) * second.2),
        ((159.0 + margin) * first.2 - first.0, (159.0 + margin) * second.2 - second.0),
        (first.1 + (100.0 + margin) * first.2, second.1 + (100.0 + margin) * second.2),
        ((99.0 + margin) * first.2 - first.1, (99.0 + margin) * second.2 - second.1),
    ];
    let mut plane = 0usize;
    while plane < planes.len() {
        if !clip_exposure_half_space(
            planes[plane].0,
            planes[plane].1,
            &mut enter,
            &mut leave,
        ) {
            return None;
        }
        plane += 1;
    }
    let interpolate = |amount: f64| {
        let x = first.0 + (second.0 - first.0) * amount;
        let y = first.1 + (second.1 - first.1) * amount;
        let depth = first.2 + (second.2 - first.2) * amount;
        (
            (160.0 + x / depth) as f32,
            (100.0 + y / depth) as f32,
            depth as f32,
        )
    };
    Some((interpolate(enter), interpolate(leave)))
}

pub(super) fn canonical_projection_ray(camera: &FlowCamera, point: (f32, f32)) -> DVec3 {
    let forward = dvec_from_vec3(camera.forward);
    let right = dvec_from_vec3(camera.right);
    let down = dvec_from_vec3(camera.down);
    let determinant = dvec_dot(forward, cross64(right, down));
    let dual_forward = dvec_scale(cross64(right, down), 1.0 / determinant);
    let dual_right = dvec_scale(cross64(down, forward), 1.0 / determinant);
    let dual_down = dvec_scale(cross64(forward, right), 1.0 / determinant);
    dvec_add(
        dual_forward,
        dvec_add(
            dvec_scale(
                dual_right,
                (point.0 - 160.0) as f64 / FLOW_FOCAL as f64,
            ),
            dvec_scale(
                dual_down,
                (point.1 - 100.0) as f64 / FLOW_FOCAL as f64,
            ),
        ),
    )
}

pub(super) fn flow_camera(time:f32)->FlowCamera {
    let state=trajectory_state(time as f64);
    let (forward,right,down)=if time<=4.0 {
        let (f,r,d)=flow_field_basis();(vec3_from_dvec(f),vec3_from_dvec(r),vec3_from_dvec(d))
    } else {flight_camera_axes(&state)};
    let relative=dvec_sub(state.position,planet_center());
    let normal=flow_normal_to_planet(dvec_normalize(relative));
    let altitude=dvec_length(relative)-FLOW_PLANET_RADIUS_WORLD-flow_surface_relief_world(normal);
    FlowCamera {position:state.position,forward,right,down,
        altitude_miles:world_to_miles(altitude),
        speed_world_per_second:dvec_length(state.velocity),
        near_plane:0.001f64.min(altitude.max(FLOW_FINAL_ALTITUDE_WORLD)*0.01)}
}

pub(super) fn project_flow_depth(camera: &FlowCamera, point: DVec3) -> Option<(f32, f32, f32)> {
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

pub(super) fn field_mesh_visibility(layout: FieldLayout, s: f64, camera: &FlowCamera) -> f32 {
    let ring_center = dvec_add(layout.axis_origin, dvec_scale(layout.forward, s));
    let distance = dvec_length(dvec_sub(ring_center, camera.position));
    (1.0
        - smootherstep_f64(
            (distance - FIELD_MESH_FADE_START_WORLD)
                / (FIELD_MESH_MAX_DISTANCE_WORLD - FIELD_MESH_FADE_START_WORLD),
        )) as f32
}

pub(super) fn project_field_segment(
    a: DVec3,
    b: DVec3,
    camera: &FlowCamera,
    x_scale: f32,
) -> Option<((f32, f32, f32), (f32, f32, f32))> {
    let near = camera.near_plane * 1.1;
    let mut relative_a = dvec_sub(a, camera.position);
    let mut relative_b = dvec_sub(b, camera.position);
    let mut depth_a = dvec_dot_vec3(relative_a, camera.forward);
    let mut depth_b = dvec_dot_vec3(relative_b, camera.forward);
    if depth_a <= near && depth_b <= near {
        return None;
    }
    if depth_a <= near {
        let amount = (near - depth_a) / (depth_b - depth_a).max(1.0e-12);
        relative_a = dvec_add(
            relative_a,
            dvec_scale(dvec_sub(relative_b, relative_a), amount),
        );
        depth_a = near;
    } else if depth_b <= near {
        let amount = (near - depth_b) / (depth_a - depth_b).max(1.0e-12);
        relative_b = dvec_add(
            relative_b,
            dvec_scale(dvec_sub(relative_a, relative_b), amount),
        );
        depth_b = near;
    }
    // Clip in homogeneous f64 coordinates BEFORE division and conversion to
    // f32. Near-plane-only clipping can project the hidden end of a spoke to
    // millions of pixels; subsequent f32 raster arithmetic then flickers as
    // that ring passes behind the camera. The four-pixel margin retains the
    // entire three-pixel bloom footprint, with caps outside the visible frame.
    let coordinates=|relative:DVec3,depth:f64| (
        dvec_dot_vec3(relative,camera.right)*FLOW_FOCAL as f64*x_scale as f64,
        dvec_dot_vec3(relative,camera.down)*FLOW_FOCAL as f64,depth);
    let a=coordinates(relative_a,depth_a);
    let b=coordinates(relative_b,depth_b);
    let mut enter=0.0f64;
    let mut leave=1.0f64;
    for (first,second) in [
        (a.0+164.0*a.2,b.0+164.0*b.2),
        (164.0*a.2-a.0,164.0*b.2-b.0),
        (a.1+104.0*a.2,b.1+104.0*b.2),
        (104.0*a.2-a.1,104.0*b.2-b.1),
    ] {
        if !clip_exposure_half_space(first,second,&mut enter,&mut leave) {return None;}
    }
    let original_a=relative_a;
    let delta=dvec_sub(relative_b,relative_a);
    let original_depth=depth_a;
    let depth_delta=depth_b-depth_a;
    if enter>0.0 {
        relative_a=dvec_add(original_a,dvec_scale(delta,enter));
        depth_a=original_depth+depth_delta*enter;
    }
    if leave<1.0 {
        relative_b=dvec_add(original_a,dvec_scale(delta,leave));
        depth_b=original_depth+depth_delta*leave;
    }
    let project = |relative: DVec3, depth: f64| {
        (
            160.0
                + (dvec_dot_vec3(relative, camera.right) / depth * FLOW_FOCAL as f64)
                    as f32
                    * x_scale,
            100.0
                + (dvec_dot_vec3(relative, camera.down) / depth * FLOW_FOCAL as f64)
                    as f32,
            depth as f32,
        )
    };
    Some((project(relative_a, depth_a), project(relative_b, depth_b)))
}
