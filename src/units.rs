//! One fixed conversion for the whole universe; never camera-relative scaling.

pub const METERS_PER_MILE: f64 = 1_609.344;
pub(super) const TERRAIN_CHART_UNITS_PER_RADIAN: f64 = 13_500.0;
pub(super) const METRES_PER_WORLD_UNIT: f64 = 70_947_663.29049851 / 18.0;
pub(super) const CLOSE_ORBIT_ALTITUDE_MILES: f64 = 100.0;
pub const FINAL_ALTITUDE_METERS: f64 = 20.0;
pub(super) const FLOW_PLANET_RADIUS_WORLD: f64 = 6_371_000.0 / METRES_PER_WORLD_UNIT;
pub(super) const FLOW_MAX_RELIEF_MILES: f64 = 12.0;
pub(super) const GRID_BOTTOM_HEIGHT_WORLD: f64 = 7_801_802.502068698 / METRES_PER_WORLD_UNIT;
pub(super) const FLOW_ATMOSPHERE_HEIGHT_MILES: f64 = 100_000.0 / METERS_PER_MILE;
pub(super) const FLOW_ATMOSPHERE_RADIUS_WORLD: f64 =
    FLOW_PLANET_RADIUS_WORLD + FLOW_ATMOSPHERE_HEIGHT_MILES * WORLD_UNITS_PER_MILE;
pub const WORLD_UNITS_PER_MILE: f64 = METERS_PER_MILE / METRES_PER_WORLD_UNIT;
pub const FLOW_CLOSE_ORBIT_RADIUS_WORLD: f64 =
    FLOW_PLANET_RADIUS_WORLD + CLOSE_ORBIT_ALTITUDE_MILES * WORLD_UNITS_PER_MILE;
pub const FLOW_FINAL_ALTITUDE_WORLD: f64 =
    FINAL_ALTITUDE_METERS / METERS_PER_MILE * WORLD_UNITS_PER_MILE;
pub const fn miles_to_world(miles: f64) -> f64 {
    miles * WORLD_UNITS_PER_MILE
}

pub const fn world_to_miles(world: f64) -> f64 {
    world / WORLD_UNITS_PER_MILE
}

pub const fn world_to_meters(world: f64) -> f64 {
    world_to_miles(world) * METERS_PER_MILE
}
