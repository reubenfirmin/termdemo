//! Read-only binary trace of the actual production camera, not a speed proposal.
//! Rendering probes call the production renderer; no alternative motion law.

use super::{METERS_PER_MILE,exit,flow_camera,trajectory_state,world_to_meters,write_all};

pub(super) fn dispatch(selector: u8) {
    match selector {
        b's' => run(),
        b'G' => query_surface(),
        b'H' => query_opening_stars(),
        b'I' => query_all_stars(),
        b'J' => query_world(),
        b'T' => query_frames(),
        b'P' => profile_frames(true),
        b'p' => profile_frames(false),
        b'C' => check_render_workers(),
        b'N' => check_world_and_clipping(),
        b'!'|b'@'|b'#'|b'^'|b'%'|b'&'|b'*'|b'+'|b'?'|b'='|b'~'|b':'|
        b'v'|b'b'|b'd'|b'w'|b'F'|b'R'|b'A' => {
            write_all(b"Obsolete flight audit selector: run make audit-phase3-live. Historical results are not current passes.\n");
            exit(2);
        }
        _ => {}
    }
}

fn check_world_and_clipping()->! {
    use super::*;
    check_projection_boundaries();
    check_surface_lighting();
    check_exact_terrain_caches();
    let layout=grid_layout();
    let edge=FieldEdgeIdentity {ring:field_ring_identity(240),lane:48,kind:FieldEdgeKind::Axial};
    let original=field_edge_points(layout,edge);
    let star=StarObjectIdentity {cell:star_cell_identity(560),object:12};
    let original_star=star_object_position(STAR_CATALOGUE,star);
    for t in [4.0,7.6,23.0,26.0,28.0,33.0,41.0,57.0,77.0] {
        for offset in [DVec3{x:0.0,y:0.0,z:0.0},DVec3{x:0.2,y:-0.4,z:0.1}] {
            let mut camera=flow_camera(t);
            camera.position=dvec_add(camera.position,offset);
            assert!(grid_layout()==layout);
            assert!(field_edge_points(layout,edge)==original);
            assert!(star_object_position(STAR_CATALOGUE,star)==original_star);
            let f=dvec_from_vec3(camera.forward);
            let a=dvec_add(camera.position,dvec_scale(f,-10.0*camera.near_plane));
            let b=dvec_add(camera.position,dvec_scale(f,20.0*camera.near_plane));
            // A spoke crossing the camera must be clipped, not rejected just
            // because its previous ring lies behind the camera.
            let (pa,pb)=project_field_segment(a,b,&camera,1.0).unwrap();
            assert!(pa.0.is_finite() && pa.1.is_finite() && pb.0.is_finite() && pb.1.is_finite());
            assert!(project_field_segment(a,dvec_add(a,dvec_scale(f,-camera.near_plane)),&camera,1.0).is_none());
            let projected=project_flow_depth(&camera,original.0);
            let _=projected; // Projection is permitted to vary, geometry is not.
        }
    }
    for ring in 0..270 {
        for lane in 0..GRID_LANES-1 {
            let ring=field_ring_identity(ring);
            let a=field_edge_points(layout,FieldEdgeIdentity{ring,lane:lane as u16,kind:FieldEdgeKind::Circumferential});
            let b=field_edge_points(layout,FieldEdgeIdentity{ring,lane:(lane+1) as u16,kind:FieldEdgeKind::Axial});
            assert!(a.1==b.0);
        }
    }
    for ordinal in -1..278 {
        for lane in 0..GRID_LANES {
            let ring=field_ring_identity(ordinal);
            assert!(field_mesh_vertex(layout,ring,lane)==uncached_field_mesh_vertex(layout,ring,lane));
            let mut changed=layout;changed.axis_origin.x+=10.0;
            assert!(field_mesh_vertex(changed,ring,lane)==uncached_field_mesh_vertex(changed,ring,lane));
        }
    }
    write_all(b"world/clipping audit ok: 18 alternate cameras; fixed objects; 51840 shared vertices; exact vertex cache including alternate layouts; 12003 near-plane crossings; displaced-surface normals and pre-haze lighting\n");
    exit(0)
}

fn check_exact_terrain_caches() {
    use super::*;
    // Exercise collisions, negative coordinates, and a zero seed (including
    // the initially empty cache key). References use the uncached hash math.
    for seed in [0,431,719,1237,2057,3071,4048,5025,6002,6979,7019] {
        for index in -4096i32..4096 {
            let ix=index;let iy=index.wrapping_mul(997)%719;
            let sample=|x:i32,y:i32|(hash(x.wrapping_add(seed),y.wrapping_sub(seed)) as f32-127.5)/127.5;
            let expected=[sample(ix,iy),sample(ix+1,iy),sample(ix,iy+1),sample(ix+1,iy+1)];
            assert!(landform_corners(ix,iy,seed)==expected);
            assert!(landform_corners(ix,iy,seed)==expected);
        }
    }
    for wavelength in [0.125f64,0.24,0.25,0.4,1.0,6.0,8.0,12.0,64.0,90.0,700.0,3000.0] {
        for ratio in [0.0,0.01,0.2499999999,0.25,0.2500000001,1.0/3.0,
            0.5,1.0,4.0/3.0,1.9999999999,2.0,2.0000000001,1e9] {
            let footprint=wavelength*ratio;
            let expected=smootherstep_f64((wavelength/footprint.max(1e-9)-0.75)/2.25) as f32;
            assert!(surface_lod_weight(wavelength,footprint).to_bits()==expected.to_bits());
        }
    }
}

fn check_projection_boundaries() {
    use super::*;
    // Two FIXED endpoints; only the camera moves. Unlike the former collinear
    // witness, this spoke leaves the viewport as its first ring crosses near.
    let a=DVec3{x:-0.5,y:0.15,z:0.0};
    let b=DVec3{x:0.5,y:-0.2,z:10.0};
    for aspect in [0.75,1.0,1.5] {
        let mut previous_y:Option<f32>=None;
        for tick in 0..=4000 {
            let camera=FlowCamera {position:DVec3{x:0.0,y:0.0,z:-0.1+tick as f64*0.00005},
                forward:Vec3{x:0.0,y:0.0,z:1.0},right:Vec3{x:1.25,y:0.0,z:0.0},
                down:Vec3{x:0.0,y:1.0,z:0.0},near_plane:0.01,
                altitude_miles:1.0,speed_world_per_second:1.0};
            let (first,last)=project_field_segment(a,b,&camera,aspect).unwrap();
            for endpoint in [first,last] {
                assert!(endpoint.0>=-4.002 && endpoint.0<=324.002);
                assert!(endpoint.1>=-4.002 && endpoint.1<=204.002);
                assert!(endpoint.2.is_finite() && endpoint.2>=camera.near_plane as f32);
            }
            let fraction=(160.0-first.0)/(last.0-first.0);
            assert!(fraction>=0.0 && fraction<=1.0);
            let y=first.1+(last.1-first.1)*fraction;
            // Independent pinhole projection of the world-space midpoint.
            let expected_y=100.0+154.0*(-0.025)/(5.0-camera.position.z);
            assert!((y as f64-expected_y).abs()<0.0001);
            let inverse=(1.0-fraction)/first.2+fraction/last.2;
            assert!((1.0/inverse as f64-(5.0-camera.position.z)).abs()<0.0001);
            if let Some(previous)=previous_y {assert!((y-previous).abs()<0.0001);}
            previous_y=Some(y);
            let reversed=project_field_segment(b,a,&camera,aspect).unwrap();
            assert!((reversed.0.0-last.0).abs()<0.002 && (reversed.1.0-first.0).abs()<0.002);
        }
    }
}

fn check_surface_lighting() {
    use super::*;
    let mut ocean_checks=0;
    for ordinal in 0..64 {
        let z=1.0-2.0*(ordinal as f64+0.5)/64.0;
        let (sn,cs)=flight_math::sin_cos(ordinal as f64*2.399963229728653);
        let radius=sqrt_f64(1.0-z*z);
        let normal=Vec3{x:(radius*cs) as f32,y:(radius*sn) as f32,z:z as f32};
        for footprint in [0.0001,0.1,1.0,100.0,4000.0] {
            let lod=surface_lod(footprint);
            let sample=flow_surface_sample(normal,lod);
            let light=surface_relief_normal(normal,sample,lod);
            let radial=dvec_normalize(planet_normal_to_flow(normal));
            assert!(light.x.is_finite() && light.y.is_finite() && light.z.is_finite());
            assert!((dvec_length(light)-1.0).abs()<1e-12);
            assert!(dvec_dot(light,radial)>0.0);
            // Deep ocean has zero displacement throughout these small samples;
            // lighting must reduce to the sphere's actual radial normal.
            if sample.continent==0.0 && footprint<1.0 {
                assert!(dvec_length(dvec_sub(light,radial))<0.000001);
                ocean_checks+=1;
            }
        }
    }
    assert!(ocean_checks>10);
    // A uniform already-hazed color must remain uniform across both triangle
    // orientations. No post-atmosphere face-light multiplier is permitted.
    fill_universe_vacuum();clear_depth_buffer();
    let a=SurfaceVertex{x:10.0,y:10.0,inverse_depth:2.0,red:50.0,green:100.0,blue:150.0,valid:true};
    let b=SurfaceVertex{x:18.0,..a};let c=SurfaceVertex{y:18.0,..a};let d=SurfaceVertex{x:18.0,y:18.0,..a};
    raster_surface_triangle(a,b,d,1.0);raster_surface_triangle(a,d,c,1.0);
    let frame=core::ptr::addr_of!(FRAME).cast::<u8>();
    for y in 10..18 {for x in 10..18 {for channel in 0..3 {
        let value=unsafe {frame.add((y*WIDTH+x)*3+channel).read()} as i32;
        assert!((value-(channel as i32+1)*50).abs()<=1);
    }}}
}

fn doubles(values:&[f64]) {for value in values {write_all(&value.to_le_bytes());}}

// Fixed-world metadata for independent reconstruction, plus real rendering
// probes. These are observations, not replacement camera/speed calculations.
fn query_world()->! {
    use super::*;
    write_all(b"TDWORLD3");
    let c=planet_center();
    for v in [c,world_direction(DVec3{x:1.0,y:0.0,z:0.0}),
        world_direction(DVec3{x:0.0,y:1.0,z:0.0}),world_direction(DVec3{x:0.0,y:0.0,z:1.0})] {
        doubles(&[v.x,v.y,v.z]);
    }
    let l=grid_layout();
    doubles(&[METRES_PER_WORLD_UNIT,RADIUS_METRES,AXIS_METRES,
        l.ring_start_s*METRES_PER_WORLD_UNIT,l.rails_full_s*METRES_PER_WORLD_UNIT,
        l.square_start_s*METRES_PER_WORLD_UNIT,l.square_end_s*METRES_PER_WORLD_UNIT,
        l.field_end_s*METRES_PER_WORLD_UNIT,2_000_000.0,(GRID_LANES-1) as f64]);
    exit(0)
}

fn read_pair()->Option<(f64,f64)> {
    let mut input=[0u8;16];let mut used=0;
    while used<16 {
        let count=super::syscall3(super::SYS_READ,0,input[used..].as_mut_ptr() as usize,16-used);
        if count==0 && used==0 {return None;}
        if count<=0 {exit(2);}
        used+=count as usize;
    }
    let mut a=[0;8];let mut b=[0;8];a.copy_from_slice(&input[..8]);b.copy_from_slice(&input[8..]);
    let pair=(f64::from_le_bytes(a),f64::from_le_bytes(b));
    if !pair.0.is_finite() || !pair.1.is_finite() || pair.0<0.0 || pair.0>77.0
        || pair.1<=0.0 {exit(3);}
    Some(pair)
}

fn query_frames()->! {
    use super::*;
    write_all(b"TDFRAME4");
    while let Some((t,aspect))=read_pair() {
        let t=t as f32;let aspect=aspect as f32;let camera=flow_camera(t);
        let start=now_ns();render_demo(t,aspect);
        let elapsed=(now_ns()-start) as f64/1e6;
        let stats=unsafe {core::ptr::addr_of!(FIELD_RENDER_STATS).read()};
        let surface=projected_surface_bounds(&camera,aspect);
        let sun=flow_sun_projection(&camera,aspect).unwrap_or((f32::NAN,f32::NAN));
        let mut projection_error=0.0f64;
        for point in [(10.0,10.0),(160.0,100.0),(310.0,190.0)] {
            let ray=flow_camera_ray(&camera,aspect,point.0,point.1);
            let world=dvec_add(camera.position,dvec_scale(ray,camera.near_plane*20.0));
            let actual=project_flow_depth(&camera,world).unwrap();
            projection_error=projection_error.max(((160.0+(actual.0-160.0)*aspect-point.0) as f64).abs())
                .max(((actual.1-point.1) as f64).abs());
        }
        doubles(&[t as f64,aspect as f64,elapsed,stats.star_points as f64,
            stats.star_streaks as f64,stats.ring_edges as f64,stats.spoke_edges as f64,
            surface.visible as f64,sun.0 as f64,sun.1 as f64,projection_error,
            camera.altitude_miles*METERS_PER_MILE,
            unsafe {render_workers::FRAME_CPU_NS} as f64/1e6]);
    }
    exit(0)
}

// Wall time includes worker dispatch/assembly; CPU reports sum rendering work
// in ALL processes. Parent CPU alone is not a valid parallel frame-time metric.
fn profile_frames(parallel:bool)->! {
    use super::*;
    write_all(b"TDPROF04");
    while let Some((t,aspect))=read_pair() {
        unsafe {LANDSCAPE_HEIGHT_EVALUATIONS=0;SURFACE_CACHE_MISSES=0;}
        let start=now_ns();
        if parallel {render_demo(t as f32,aspect as f32);}
        else {render_workers::serial(t as f32,aspect as f32);}
        let elapsed=(now_ns()-start) as f64/1e6;
        let stages=unsafe {core::ptr::addr_of!(FRAME_STAGE_NS).read()};
        doubles(&[t,aspect,elapsed,stages[0] as f64/1e6,stages[1] as f64/1e6,
            stages[2] as f64/1e6,stages[3] as f64/1e6,
            unsafe {LANDSCAPE_HEIGHT_EVALUATIONS} as f64,
            unsafe {SURFACE_CACHE_MISSES} as f64,
            unsafe {render_workers::FRAME_CPU_NS} as f64/1e6]);
    }
    exit(0)
}

fn check_render_workers()->! {
    use super::*;
    static mut REFERENCE:[u8;FRAME_BYTES]=[0;FRAME_BYTES];
    static mut REFERENCE_DEPTH:[f32;PIXELS]=[0.0;PIXELS];
    for t in [0.001,3.985,4.0,5.0,7.6,16.0,23.0,26.0,29.0,35.0,41.0,57.0,65.0,70.0,77.0] {
        for aspect in [0.75,1.0,1.5] {
            render_workers::serial(t,aspect);
            unsafe {
                core::ptr::copy_nonoverlapping(core::ptr::addr_of!(FRAME).cast::<u8>(),core::ptr::addr_of_mut!(REFERENCE).cast::<u8>(),FRAME_BYTES);
                core::ptr::copy_nonoverlapping(core::ptr::addr_of!(DEPTH).cast::<f32>(),core::ptr::addr_of_mut!(REFERENCE_DEPTH).cast::<f32>(),PIXELS);
            }
            render_demo(t,aspect);
            for (length,a,b) in [(FRAME_BYTES,core::ptr::addr_of!(FRAME).cast::<u8>(),core::ptr::addr_of!(REFERENCE).cast::<u8>()),
                (PIXELS*4,core::ptr::addr_of!(DEPTH).cast::<u8>(),core::ptr::addr_of!(REFERENCE_DEPTH).cast::<u8>())] {
                for i in 0..length {if unsafe {a.add(i).read()!=b.add(i).read()} {
                    write_all(b"Worker mismatch (time, aspect, length, offset, actual, reference): ");
                    doubles(&[t as f64,aspect as f64,length as f64,i as f64,
                        unsafe {a.add(i).read()} as f64,unsafe {b.add(i).read()} as f64]);exit(2);
                }}
            }
        }
    }
    render_workers::check_ownership_and_shutdown();
    // Last references contain the serial 77s/aspect1.5 frame. Kill a private
    // idle child and make production dispatch recover that exact same frame.
    let failed_pid=render_workers::fail_one_worker();
    render_demo(77.0,1.5);
    assert!(render_workers::is_serial());
    for (length,a,b) in [(FRAME_BYTES,core::ptr::addr_of!(FRAME).cast::<u8>(),core::ptr::addr_of!(REFERENCE).cast::<u8>()),
        (PIXELS*4,core::ptr::addr_of!(DEPTH).cast::<u8>(),core::ptr::addr_of!(REFERENCE_DEPTH).cast::<u8>())] {
        for i in 0..length {assert!(unsafe {a.add(i).read()==b.add(i).read()});}
    }
    render_workers::check_ownership_and_shutdown();
    write_all(b"worker audit ok: 45 serial/parallel RGB AND depth comparisons; exact row/tile ownership; reaped children; serial fallback");
    if failed_pid>0 {write_all(b" after real worker failure");}
    else {write_all(b" (single-CPU/restricted host; worker-failure injection unavailable)");}
    write_all(b"\n");
    exit(0)
}

fn query_all_stars() -> ! {
    write_all(b"TDALLS01");
    let camera = flow_camera(4.0);
    for axis in [camera.position, super::dvec_from_vec3(camera.forward),
        super::dvec_from_vec3(camera.right), super::dvec_from_vec3(camera.down)] {
        for value in [axis.x, axis.y, axis.z] { write_all(&value.to_le_bytes()); }
    }
    let layout = super::STAR_CATALOGUE;
    for ordinal in 0..super::STAR_CELL_COUNT {
        for object in 0..super::FIELD_STAR_OBJECTS_PER_CELL {
            let star = super::StarObjectIdentity { cell: super::star_cell_identity(ordinal), object };
            if !super::star_object_exists(layout, star) { continue; }
            let point = super::star_object_position(layout, star);
            for value in [ordinal as f64, object as f64, point.x, point.y, point.z] {
                write_all(&value.to_le_bytes());
            }
        }
    }
    exit(0)
}

fn query_opening_stars() -> ! {
    let camera = flow_camera(4.0);
    let earlier = flow_camera(3.95);
    let layout = super::STAR_CATALOGUE;
    let (forward, right, down) = super::flow_field_basis();
    // Version 2 reports the current global metre scale; version 1 used the
    // oversized-world conversion. The catalogue's NATIVE coordinates did not move.
    write_all(b"TDSTAR02");
    for ordinal in 0..super::STAR_ORIGINAL_CELL_COUNT {
        for object in 0..super::FIELD_STAR_OBJECTS_PER_CELL {
            let star = super::StarObjectIdentity { cell: super::star_cell_identity(ordinal), object };
            if !super::star_object_exists(layout, star) { continue; }
            let point = super::star_object_position(layout, star);
            let Some(a) = super::project_flow_depth(&earlier, point) else { continue; };
            let Some(b) = super::project_flow_depth(&camera, point) else { continue; };
            if a.0 < 0.0 || a.0 >= 320.0 || a.1 < 0.0 || a.1 >= 200.0
                || b.0 < 0.0 || b.0 >= 320.0 || b.1 < 0.0 || b.1 >= 200.0 { continue; }
            let relative = super::dvec_sub(point, camera.position);
            for value in [ordinal as f64, object as f64,
                world_to_meters(super::dvec_dot(relative, right)),
                world_to_meters(super::dvec_dot(relative, down)),
                world_to_meters(super::dvec_dot(relative, forward)),
                a.0 as f64, a.1 as f64, b.0 as f64, b.1 as f64] {
                write_all(&value.to_le_bytes());
            }
        }
    }
    exit(0)
}

// Phase-2 read-only sampling of the SAME procedural terrain. The caller pads
// the selector to the 16-byte startup read, then supplies planet-local unit
// normals as triples of little-endian f64. Results are relief metres and the
// existing regional-chart coordinates. This never changes world/camera state.
fn query_surface() -> ! {
    write_all(b"TDSURF01");
    let up = super::landing_up();
    let forward = super::landing_forward();
    let right = super::vec_cross(forward, up);
    for axis in [up, forward, right] {
        for value in [axis.x, axis.y, axis.z] { write_all(&(value as f64).to_le_bytes()); }
    }
    loop {
        let mut input = [0u8; 24];
        let mut used = 0;
        while used < input.len() {
            let count = super::syscall3(super::SYS_READ, 0,
                input[used..].as_mut_ptr() as usize, input.len() - used);
            if count == 0 && used == 0 { exit(0); }
            if count <= 0 { exit(2); }
            used += count as usize;
        }
        let value = |i: usize| {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&input[i..i + 8]);
            f64::from_le_bytes(bytes)
        };
        let normal = super::Vec3 { x: value(0) as f32, y: value(8) as f32, z: value(16) as f32 };
        if !normal.x.is_finite() || !normal.y.is_finite() || !normal.z.is_finite() {
            exit(3);
        }
        let sample = super::flow_surface_sample(normal, super::full_surface_lod());
        for value in [world_to_meters(sample.relief_world), sample.map_x as f64,
            sample.map_y as f64, sample.continent as f64] {
            write_all(&value.to_le_bytes());
        }
    }
}

fn run() -> ! {
    const HZ: u32 = 1_920;
    const COLUMNS: usize = 25;
    const BATCH: usize = 64;
    let count = 77u32 * HZ + 1;
    write_all(b"TDMOTION");
    write_all(&HZ.to_le_bytes());
    write_all(&count.to_le_bytes());
    write_all(&(COLUMNS as u32).to_le_bytes());
    write_all(&world_to_meters(1.0).to_le_bytes());
    let mut bytes = [0u8; COLUMNS * 8 * BATCH];
    let mut used = 0;
    for tick in 0..count {
        // flow_camera consumes f32 playback time. Record that actual input,
        // rather than pretending its sampling timestamps are exact f64 ticks.
        let time = tick as f32 / HZ as f32;
        let camera = flow_camera(time);
        let state = trajectory_state(time as f64);
        let values = [
            time as f64,
            camera.position.x, camera.position.y, camera.position.z,
            state.velocity.x, state.velocity.y, state.velocity.z,
            state.acceleration.x, state.acceleration.y, state.acceleration.z,
            camera.forward.x as f64, camera.forward.y as f64, camera.forward.z as f64,
            camera.right.x as f64, camera.right.y as f64, camera.right.z as f64,
            camera.down.x as f64, camera.down.y as f64, camera.down.z as f64,
            camera.altitude_miles * METERS_PER_MILE,
            camera.speed_world_per_second, camera.near_plane,
            state.phase / 1_024.0, state.phase_rate / 1_024.0, state.roll,
        ];
        for value in values {
            bytes[used..used + 8].copy_from_slice(&value.to_le_bytes());
            used += 8;
        }
        if used == bytes.len() {
            write_all(&bytes);
            used = 0;
        }
    }
    write_all(&bytes[..used]);
    exit(0)
}
