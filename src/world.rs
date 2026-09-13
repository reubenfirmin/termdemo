//! Fixed star, ring and rail objects in the SAME native world coordinates.
//! Catalogue spacing is frozen independently of the revised grid's cell size.
use super::*;

pub(super) const GRID_LANES: usize = 193;
// Permanent universe light, chosen above the destination's regional horizon.
// It never reads the camera or clock; all consumers use this same direction.

pub(super) fn flow_field_basis() -> (DVec3, DVec3, DVec3) {
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



#[derive(Clone, Copy, PartialEq)]
pub(super) struct FieldLayout {
    pub(super) axis_origin: DVec3,
    pub(super) forward: DVec3,
    pub(super) right: DVec3,
    pub(super) down: DVec3,
    pub(super) radius: f64,
    pub(super) field_start_s: f64,
    pub(super) ordering_start_s: f64,
    pub(super) ring_start_s: f64,
    pub(super) rings_full_s: f64,
    pub(super) star_volume_end_s: f64,
    pub(super) rails_full_s: f64,
    pub(super) square_start_s: f64,
    pub(super) square_end_s: f64,
    pub(super) planet_s: f64,
    pub(super) field_end_s: f64,
}



pub(super) const FIELD_ORDERING_START_S: f64 = f64::from_bits(0xc077_e0be_dd18_fa26);
// The opening rush has already crossed the wormhole here. The last 16 world
// units of emergence resolve star motion blur to points, before the ring
// structure is reached. This is a shutter setting, not a speed controller,
// star relocation, visibility fade, or a change to the fixed spoke catalogue.
pub(super) const FIELD_EMERGENCE_LENGTH: f64 = 16.0;
pub(super) const FIELD_RING_START_S: f64 = FIELD_ORDERING_START_S + 64.0;
pub(super) const FIELD_RINGS_FULL_S: f64 = f64::from_bits(0xc070_1747_7e68_a635);
pub(super) const FIELD_STAR_CATALOGUE_JOIN_S: f64 = FIELD_RINGS_FULL_S + 8.0;
pub(super) const FIELD_RAILS_FULL_S: f64 = f64::from_bits(0xc05c_98b0_9c56_f588);
pub(super) const FIELD_SQUARE_START_S: f64 = f64::from_bits(0xc059_4a7b_7908_6c2d);
pub(super) const FIELD_STAR_VOLUME_END_S: f64 = FIELD_SQUARE_START_S + 8.0;
pub(super) const FIELD_SQUARE_FULL_S: f64 = f64::from_bits(0xc041_13a9_d825_fd9f);
pub(super) const FIELD_WORLD_START_S: f64 = f64::from_bits(0xc1a7_e0f3_8d69_915f);
pub(super) const FIELD_WORLD_END_S: f64 = f64::from_bits(0x4034_3d70_a3d7_0a3e);

pub(super) fn field_axis_coordinate(axis_origin: DVec3, forward: DVec3, point: DVec3) -> f64 {
    dvec_dot(dvec_sub(point, axis_origin), forward)
}

pub(super) const STAR_CATALOGUE: FieldLayout = FieldLayout {
    axis_origin: DVec3 {
        x: f64::from_bits(0x0000_0000_0000_0000),
        y: f64::from_bits(0xc005_eb41_3967_09d9),
        z: f64::from_bits(0x3fd5_4b75_42ed_9638),
    },
    forward: DVec3 {
        x: f64::from_bits(0x8000_0000_0000_0000),
        y: f64::from_bits(0x3fbe_dca9_f1b1_5f47),
        z: f64::from_bits(0x3fef_c440_d8c1_da54),
    },
    right: DVec3 { x: 1.0, y: 0.0, z: 0.0 },
    down: DVec3 {
        x: 0.0,
        y: f64::from_bits(0x3fef_c440_d8c1_da53),
        z: f64::from_bits(0xbfbe_dca9_f1b1_5f45),
    },
    radius: 0.115 * 22.0,
    field_start_s: FIELD_WORLD_START_S,
    ordering_start_s: FIELD_ORDERING_START_S,
    ring_start_s: FIELD_RING_START_S,
    rings_full_s: FIELD_RINGS_FULL_S,
    star_volume_end_s: FIELD_STAR_VOLUME_END_S,
    rails_full_s: FIELD_RAILS_FULL_S,
    square_start_s: FIELD_SQUARE_START_S,
    square_end_s: FIELD_SQUARE_FULL_S,
    planet_s: 0.0,
    field_end_s: FIELD_WORLD_END_S,
};

#[derive(Clone, Copy)]
pub(super) struct Universe {
    pub(super) field: FieldLayout,
    pub(super) planet_center: DVec3,
    pub(super) atmosphere_radius: f64,
    pub(super) sun_direction: Vec3,
}

pub(super) fn universe() -> Universe {
    Universe {
        field: grid_layout(),
        planet_center: planet_center(),
        atmosphere_radius: FLOW_ATMOSPHERE_RADIUS_WORLD,
        sun_direction: vec3_from_dvec(world_direction(DVec3{x:0.0,y:-0.6536436208636119,z:-0.7568024953079282})),
    }
}

pub(super) fn grid_layout()->FieldLayout {
    let scale=METRES_PER_WORLD_UNIT;
    FieldLayout {
        axis_origin:world_point(DVec3{x:0.0,y:AXIS_METRES,z:0.0}),
        forward:world_direction(DVec3{x:0.0,y:0.0,z:1.0}),
        right:world_direction(DVec3{x:-1.0,y:0.0,z:0.0}),
        down:world_direction(DVec3{x:0.0,y:-1.0,z:0.0}),
        radius:6_000_000.0/scale,
        field_start_s:-530572987.9786802/scale,
        ordering_start_s:-757357884.7961885/scale,
        ring_start_s:-530572987.9786802/scale,
        rings_full_s:-530572987.9786802/scale,
        star_volume_end_s:-16397233.416589469/scale,
        rails_full_s:-110367254.36538728/scale,
        square_start_s:-57383356.99015985/scale,
        square_end_s:-16397233.416589469/scale,
        planet_s:0.0,
        field_end_s:19_113_000.0/scale,
    }
}





pub(super) fn field_square_amount(layout: FieldLayout, s: f64) -> f64 {
    smootherstep_f64(
        (s - layout.square_start_s) / (layout.square_end_s - layout.square_start_s),
    )
}

pub(super) fn field_section(theta: f64, square_amount: f64) -> (f64, f64) {
    let angle = (theta * 1_024.0) as i32;
    let cardinal = angle & 1_023;
    let (circular_x, circular_y) = match cardinal {
        0 => (1.0, 0.0),
        256 => (0.0, 1.0),
        512 => (-1.0, 0.0),
        768 => (0.0, -1.0),
        _ => {
            let raw_x = sine(angle + 256) as f64;
            let raw_y = sine(angle) as f64;
            let inverse_length =
                1.0 / sqrt_f64(raw_x * raw_x + raw_y * raw_y).max(1.0e-12);
            (raw_x * inverse_length, raw_y * inverse_length)
        }
    };
    let square_edge = circular_x.abs().max(circular_y.abs()).max(0.001);
    let square_x = circular_x / square_edge;
    let square_y = circular_y / square_edge;
    let section_x = circular_x + (square_x - circular_x) * square_amount;
    let section_y = circular_y + (square_y - circular_y) * square_amount;
    (section_x, section_y)
}

pub(super) fn field_floor_down(_layout: FieldLayout, s: f64, right:f64)->f64 {
    let width=1.5*FLOW_PLANET_RADIUS_WORLD;
    GRID_BOTTOM_HEIGHT_WORLD+FLOW_PLANET_RADIUS_WORLD/sqrt_f64(1.0+(s*s+right*right)/(width*width))
}

pub(super) fn field_point(layout:FieldLayout,s:f64,theta:f64)->DVec3 {
    let section=field_section(theta,field_square_amount(layout,s));
    let widen=smootherstep_f64((s-layout.rails_full_s)/(layout.square_end_s-layout.rails_full_s));
    let half_width=layout.radius+(9_000_000.0/METRES_PER_WORLD_UNIT-layout.radius)*widen;
    let half_height=layout.radius+(GRID_BOTTOM_HEIGHT_WORLD-layout.radius)*widen;
    let lateral=section.0*half_width;
    let floor_weight=smootherstep_f64((section.1-0.25)/0.75);
    let well=field_floor_down(layout,s,lateral)-GRID_BOTTOM_HEIGHT_WORLD;
    dvec_add(layout.axis_origin,dvec_add(dvec_scale(layout.forward,s),
        dvec_add(dvec_scale(layout.right,lateral),dvec_scale(layout.down,section.1*half_height+floor_weight*well))))
}

pub(super) const FIELD_RING_CHUNK_SIZE: i32 = 16;
// Grid spacing has its own physical scale. It does not inherit stellar
// catalogue spacing or grow geometrically throughout the cylindrical grid.
pub(super) const STAR_CELL_CHUNK_SIZE: u32 = 32;
pub(super) const STAR_CELL_RATIO: f64 = 33.0 / 32.0;
pub(super) const STAR_CELL_BASE_SPACING: f64 = (0.115 * 22.0) / 16.0;
pub(super) const STAR_ORIGINAL_CELL_COUNT: u32 = 569;
pub(super) const STAR_EXTENSION_CELL_COUNT: u32 = 113;
pub(super) const STAR_CELL_COUNT: u32 = STAR_ORIGINAL_CELL_COUNT + STAR_EXTENSION_CELL_COUNT;
pub(super) const FIELD_STAR_OBJECTS_PER_CELL: u8 = 128;
pub(super) const FIELD_STAR_OBJECT_HASH_LIMIT: i32 = 48;
pub(super) const BACKGROUND_STAR_COUNT: u16 = 4096;
pub(super) const BACKGROUND_STAR_NEAR_WORLD: f64 = 1e13 / METRES_PER_WORLD_UNIT;
pub(super) const BACKGROUND_STAR_DEPTH_WORLD: f64 = 2e13 / METRES_PER_WORLD_UNIT;
pub(super) const EXTENSION_STAR_FADE_START_WORLD: f64 = 120.0;
pub(super) const EXTENSION_STAR_MAX_DISTANCE_WORLD: f64 = 128.0;
pub(super) const FIELD_MESH_FADE_START_WORLD: f64 = 180_000_000.0 / METRES_PER_WORLD_UNIT;
pub(super) const FIELD_MESH_MAX_DISTANCE_WORLD: f64 = 210_000_000.0 / METRES_PER_WORLD_UNIT;
pub(super) const FIELD_MOTION_BLUR_EXPOSURE: f32 = 0.032;

pub(super) fn field_star_exposure(layout: FieldLayout, position: DVec3) -> f32 {
    let s = field_axis_coordinate(layout.axis_origin, layout.forward, position);
    FIELD_MOTION_BLUR_EXPOSURE
        * (1.0 - smootherstep_f64((s - layout.ordering_start_s) / FIELD_EMERGENCE_LENGTH)) as f32
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct FieldRingIdentity {
    pub(super) chunk: i32,
    pub(super) ring: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FieldEdgeKind {
    Axial,
    Circumferential,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct FieldEdgeIdentity {
    pub(super) ring: FieldRingIdentity,
    pub(super) lane: u16,
    pub(super) kind: FieldEdgeKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct StarCellIdentity {
    pub(super) chunk: u16,
    pub(super) cell: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct StarObjectIdentity {
    pub(super) cell: StarCellIdentity,
    pub(super) object: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct RingObjectIdentity {
    pub(super) ring: FieldRingIdentity,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct RailObjectIdentity {
    pub(super) first_ring: FieldRingIdentity,
    pub(super) lane: u16,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct BackgroundStarIdentity {
    pub(super) ordinal: u16,
}

pub(super) fn field_ring_identity(ordinal: i32) -> FieldRingIdentity {
    let chunk = ordinal.div_euclid(FIELD_RING_CHUNK_SIZE);
    FieldRingIdentity {
        chunk,
        ring: ordinal.rem_euclid(FIELD_RING_CHUNK_SIZE) as u8,
    }
}

pub(super) fn field_ring_ordinal(identity: FieldRingIdentity) -> i32 {
    identity.chunk * FIELD_RING_CHUNK_SIZE + identity.ring as i32
}









pub(super) fn field_ring_s(layout:FieldLayout,identity:FieldRingIdentity)->f64 {
    layout.ring_start_s+field_ring_ordinal(identity) as f64*2_000_000.0/METRES_PER_WORLD_UNIT
}



pub(super) fn field_last_ring_ordinal(layout: FieldLayout) -> i32 {
    field_ring_lower_bound(layout, layout.field_end_s)
}

pub(super) fn field_first_physical_ring_ordinal(layout: FieldLayout) -> i32 {
    field_ring_lower_bound(layout, layout.ring_start_s)
}

pub(super) fn field_ring_lower_bound(layout: FieldLayout, axial: f64) -> i32 {
    let mut lower = -1;
    let mut upper = 1;
    while field_ring_s(layout, field_ring_identity(lower)) >= axial { lower *= 2; }
    while field_ring_s(layout, field_ring_identity(upper)) < axial { upper *= 2; }
    while upper - lower > 1 {
        let middle = lower + (upper - lower) / 2;
        if field_ring_s(layout, field_ring_identity(middle)) < axial { lower = middle; }
        else { upper = middle; }
    }
    upper
}

pub(super) fn field_mesh_vertex(
    layout: FieldLayout,
    ring: FieldRingIdentity,
    lane: usize,
) -> DVec3 {
    let ordinal=field_ring_ordinal(ring);
    // Exact fixed-world catalogue, built after the sine table. Non-default
    // layouts and objects beyond cache capacity still use the source geometry.
    if ordinal>=0 && (ordinal as usize)<FIELD_VERTEX_CACHE_RINGS && layout==grid_layout() {
        let index=ordinal as usize*(GRID_LANES-1)+lane%(GRID_LANES-1);
        return unsafe {core::ptr::addr_of!(FIELD_VERTEX_CACHE).cast::<DVec3>().add(index).read()};
    }
    uncached_field_mesh_vertex(layout,ring,lane)
}

const FIELD_VERTEX_CACHE_RINGS:usize=276;
static mut FIELD_VERTEX_CACHE:[DVec3;FIELD_VERTEX_CACHE_RINGS*(GRID_LANES-1)]=
    [DVec3{x:0.0,y:0.0,z:0.0};FIELD_VERTEX_CACHE_RINGS*(GRID_LANES-1)];

pub(super) fn generate_field_vertices() {
    let layout=grid_layout();
    let pointer=core::ptr::addr_of_mut!(FIELD_VERTEX_CACHE).cast::<DVec3>();
    for ordinal in 0..FIELD_VERTEX_CACHE_RINGS {
        for lane in 0..GRID_LANES-1 {
            unsafe {pointer.add(ordinal*(GRID_LANES-1)+lane).write(
                uncached_field_mesh_vertex(layout,field_ring_identity(ordinal as i32),lane));}
        }
    }
}

pub(super) fn uncached_field_mesh_vertex(layout:FieldLayout,ring:FieldRingIdentity,lane:usize)->DVec3 {
    let theta = (lane % (GRID_LANES - 1)) as f64 / (GRID_LANES - 1) as f64;
    field_point(layout, field_ring_s(layout, ring), theta)
}

pub(super) fn field_edge_points(layout: FieldLayout, identity: FieldEdgeIdentity) -> (DVec3, DVec3) {
    let ordinal = field_ring_ordinal(identity.ring);
    let lane = identity.lane as usize % (GRID_LANES - 1);
    match identity.kind {
        FieldEdgeKind::Axial => (
            field_mesh_vertex(layout, identity.ring, lane),
            field_mesh_vertex(layout, field_ring_identity(ordinal + 1), lane),
        ),
        FieldEdgeKind::Circumferential => (
            field_mesh_vertex(layout, identity.ring, lane),
            field_mesh_vertex(layout, identity.ring, lane + 1),
        ),
    }
}

pub(super) fn star_cell_identity(ordinal: u32) -> StarCellIdentity {
    StarCellIdentity {
        chunk: (ordinal / STAR_CELL_CHUNK_SIZE) as u16,
        cell: (ordinal % STAR_CELL_CHUNK_SIZE) as u8,
    }
}

pub(super) fn star_cell_ordinal(identity: StarCellIdentity) -> u32 {
    identity.chunk as u32 * STAR_CELL_CHUNK_SIZE + identity.cell as u32
}

pub(super) fn star_cell_distance(steps: u32) -> f64 {
    let mut exponent = steps;
    let mut factor = STAR_CELL_RATIO;
    let mut power = 1.0;
    while exponent > 0 {
        if exponent & 1 != 0 {
            power *= factor;
        }
        factor *= factor;
        exponent >>= 1;
    }
    STAR_CELL_BASE_SPACING * (power - 1.0) / (STAR_CELL_RATIO - 1.0)
}

pub(super) fn star_cell_bounds(layout: FieldLayout, identity: StarCellIdentity) -> (f64, f64) {
    let _ = layout;
    let ordinal = star_cell_ordinal(identity);
    if ordinal < STAR_ORIGINAL_CELL_COUNT {
        (
            FIELD_STAR_CATALOGUE_JOIN_S - star_cell_distance(ordinal + 1),
            FIELD_STAR_CATALOGUE_JOIN_S - star_cell_distance(ordinal),
        )
    } else {
        let extension = ordinal - STAR_ORIGINAL_CELL_COUNT;
        (
            FIELD_STAR_CATALOGUE_JOIN_S + star_cell_distance(extension),
            FIELD_STAR_CATALOGUE_JOIN_S + star_cell_distance(extension + 1),
        )
    }
}

pub(super) fn star_object_seed(identity: StarObjectIdentity) -> i32 {
    (star_cell_ordinal(identity.cell) as i32)
        .wrapping_mul(1_009)
        .wrapping_add(identity.object as i32 * 1_747)
}

pub(super) fn star_object_position(layout: FieldLayout, identity: StarObjectIdentity) -> DVec3 {
    let (far_s, near_s) = star_cell_bounds(layout, identity.cell);
    let seed = star_object_seed(identity);
    let axial_amount = (hash(seed, 1_193) as f64 + 0.5) / 256.0;
    let s = far_s + (near_s - far_s) * axial_amount;
    let volume_radius = if star_cell_ordinal(identity.cell) < STAR_ORIGINAL_CELL_COUNT {
        layout
            .radius
            .max((FIELD_STAR_CATALOGUE_JOIN_S - s).max(0.0) * 1.08)
    } else {
        layout.radius
    };
    let x = (hash(seed, 337) - 128) as f64 / 128.0 * volume_radius;
    let y = (hash(seed, 811) - 128) as f64 / 128.0 * volume_radius;
    dvec_add(
        layout.axis_origin,
        dvec_add(
            dvec_scale(layout.forward, s),
            dvec_add(dvec_scale(layout.right, x), dvec_scale(layout.down, y)),
        ),
    )
}

pub(super) fn star_object_exists(layout: FieldLayout, identity: StarObjectIdentity) -> bool {
    let (far_s, near_s) = star_cell_bounds(layout, identity.cell);
    let axial_amount = (hash(star_object_seed(identity), 1_193) as f64 + 0.5) / 256.0;
    let s = far_s + (near_s - far_s) * axial_amount;
    s >= layout.field_start_s
        && s < layout.star_volume_end_s
        && hash(star_object_seed(identity), 2_281) < FIELD_STAR_OBJECT_HASH_LIMIT
}

pub(super) fn background_star_position(universe:Universe,identity:BackgroundStarIdentity)->DVec3 {
    let ordinal=identity.ordinal as f64;
    // Equal-area spherical distribution, not a normalized cube of dots.
    let y=1.0-2.0*(ordinal+0.5)/BACKGROUND_STAR_COUNT as f64;
    let (sn,cs)=flight_math::sin_cos(ordinal*2.399963229728653);
    let r=sqrt_f64(1.0-y*y);
    let depth=BACKGROUND_STAR_NEAR_WORLD+BACKGROUND_STAR_DEPTH_WORLD
        *(hash(identity.ordinal as i32,6157) as f64+0.5)/256.0;
    dvec_add(universe.planet_center,dvec_scale(DVec3{x:r*cs,y,z:r*sn},depth))
}

pub(super) fn local_star_field_visibility(layout:FieldLayout,position:DVec3)->f32 {
    let s=field_axis_coordinate(layout.axis_origin,layout.forward,position);
    (1.0-smootherstep_f64((s-layout.square_start_s)/(layout.square_end_s-layout.square_start_s))) as f32
}

pub(super) fn local_star_object_visibility(
    identity: StarObjectIdentity,
    camera_position: DVec3,
    point: DVec3,
) -> f32 {
    if star_cell_ordinal(identity.cell) < STAR_ORIGINAL_CELL_COUNT {
        return 1.0;
    }
    let distance = dvec_length(dvec_sub(point, camera_position));
    (1.0
        - smootherstep_f64(
            (distance - EXTENSION_STAR_FADE_START_WORLD)
                / (EXTENSION_STAR_MAX_DISTANCE_WORLD - EXTENSION_STAR_FADE_START_WORLD),
        )) as f32
}

pub(super) fn field_exterior_visibility(layout:FieldLayout,point:DVec3)->f32 {
    let p=local_point(point);
    let emergence=smootherstep_f64((p.z+757357884.7961885)/(757357884.7961885-687918663.7148501));
    let s=p.z/METRES_PER_WORLD_UNIT;
    let square=field_square_amount(layout,s);
    let wall=(p.x.abs()/9_000_000.0).max((p.y-AXIS_METRES).abs()/AXIS_METRES);
    let outside=smootherstep_f64((wall-0.88)/0.18);
    (emergence*(1.0-square+square*outside)) as f32
}

pub(super) fn ring_object_segment(
    layout: FieldLayout,
    identity: RingObjectIdentity,
    lane: u16,
) -> (DVec3, DVec3) {
    field_edge_points(
        layout,
        FieldEdgeIdentity {
            ring: identity.ring,
            lane,
            kind: FieldEdgeKind::Circumferential,
        },
    )
}

pub(super) fn rail_object_segment(layout: FieldLayout, identity: RailObjectIdentity) -> (DVec3, DVec3) {
    field_edge_points(
        layout,
        FieldEdgeIdentity {
            ring: identity.first_ring,
            lane: identity.lane,
            kind: FieldEdgeKind::Axial,
        },
    )
}

pub(super) fn rail_lane_start_ordinal(layout: FieldLayout, lane: u16) -> i32 {
    let first = field_first_physical_ring_ordinal(layout);
    let after_transition = field_ring_lower_bound(layout, layout.rails_full_s);
    let transition_rings = (after_transition - first).max(1);
    first + hash(lane as i32, 1_571) * transition_rings / 256
}
