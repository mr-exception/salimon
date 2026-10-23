//! Portable first-person character state and ship/surface traversal.
//!
//! This crate consumes typed input and reference-frame descriptions. It has no
//! dependency on native window events, rendering, or ship simulation state.

mod collision;
mod controller;
mod input;
mod layout;
pub use layout::PLAYER_RADIUS_METERS;
mod math;
mod queries;
mod ramp;
#[allow(dead_code)] // Shared generated geometry; character consumes support dimensions.
mod ramp_geometry;
pub use ramp::{boarding_ramp_occupied, boarding_ramp_tip_meters};
mod points;
mod spatial_contracts;
mod state;
pub use points::{ShipLocalPoint, WorldPoint};

pub use controller::CharacterController;
pub use input::MovementInput;
pub use queries::{
    fabricator_input_sight_obstruction, fabricator_sight_obstruction, ship_contains_object,
    ship_floor_placement, ship_sight_obstruction,
};
pub use spatial_contracts::{
    COCKPIT_SEAT_MARKER_METERS, ENGINE_PORT_ANCHOR_METERS, ENGINE_STARBOARD_ANCHOR_METERS,
    EXIT_DOOR_MARKER_METERS, FABRICATOR_INPUT_MARKER_METERS, FABRICATOR_MONITOR_MARKER_METERS,
    FABRICATOR_TRAY_MARKER_METERS, PLAYER_START_MARKER_METERS,
};
pub use state::{
    CharacterLocation, CharacterSnapshot, GroundQuery, GroundSample, ShipFrame, SurfaceFrame,
};

/// Depth from the authored door interaction marker to the closed door's aft face.
/// Presentation may use this bound to avoid hiding the marker behind its own door.
pub const EXIT_DOOR_MARKER_DEPTH_METERS: f64 =
    EXIT_DOOR_MARKER_METERS[0] - spatial_contracts::AFT_DOOR_BOUNDS[0];

pub const FIXED_GRAVITY_METERS_PER_SECOND_SQUARED: f64 = 9.81;
pub const PLAYER_BODY_HEIGHT_METERS: f64 = 1.80;
pub const PLAYER_EYE_HEIGHT_METERS: f64 = 1.75;
pub const SHIP_CEILING_HEIGHT_METERS: f64 = spatial_contracts::CABIN_TRAVERSAL_BOUNDS[3];

pub const SHIP_FLOOR_HEIGHT_METERS: f64 = spatial_contracts::INTERIOR_FLOOR_BOUNDS[3];

/// Authored sole grids for footprint-aware landing; generated boxes remain canonical.
pub fn landing_sole_samples() -> Vec<[f64; 3]> {
    let mut samples = Vec::new();
    for bounds in spatial_contracts::LANDING_COLLIDERS {
        // Foot boxes are the thin, lowest colliders; legs share their bottom.
        if bounds[3] - bounds[2] > 0.5 {
            continue;
        }
        for x in 0..=4 {
            for z in 0..=4 {
                samples.push([
                    bounds[0] + (bounds[1] - bounds[0]) * f64::from(x) / 4.0,
                    bounds[2],
                    bounds[4] + (bounds[5] - bounds[4]) * f64::from(z) / 4.0,
                ]);
            }
        }
    }
    samples
}
/// Ground-facing deployed ramp tip, sampled across its authored width.
pub fn ramp_tip_support_samples() -> Vec<[f64; 3]> {
    (0..=8)
        .map(|i| {
            [
                ramp::tip_x(),
                ramp::floor_at(ramp::tip_x()),
                ramp_geometry::RAMP_HALF_WIDTH_METERS * (f64::from(i) / 4.0 - 1.0),
            ]
        })
        .collect()
}
/// Conservative lowest non-gear hull plane over the authored exterior envelope.
pub fn hull_ground_samples() -> Vec<[f64; 3]> {
    let mut samples = Vec::new();
    for bounds in [spatial_contracts::CABIN_EXTERIOR_BOUNDS]
        .into_iter()
        .chain(spatial_contracts::WING_COLLIDERS)
        .chain(spatial_contracts::THRUSTER_COLLIDERS)
    {
        for x in 0..=20 {
            for z in 0..=20 {
                samples.push([
                    bounds[0] + (bounds[1] - bounds[0]) * f64::from(x) / 20.0,
                    bounds[2],
                    bounds[4] + (bounds[5] - bounds[4]) * f64::from(z) / 20.0,
                ]);
            }
        }
    }
    samples
}

/// Authored Core center in ship-local meters for mechanical intake targeting.
pub const CORE_INTERACTION_POSITION_METERS: [f64; 3] = [
    (spatial_contracts::CORE_BOUNDS[0] + spatial_contracts::CORE_BOUNDS[1]) * 0.5,
    (spatial_contracts::CORE_BOUNDS[2] + spatial_contracts::CORE_BOUNDS[3]) * 0.5,
    (spatial_contracts::CORE_BOUNDS[4] + spatial_contracts::CORE_BOUNDS[5]) * 0.5,
];

pub use controller::persistence::CharacterSaveState;

/// Raw authored installation envelopes for construction clearance. These are
/// conservative solids, independent of body expansion and the structural grid.
pub fn ship_installation_obstacles() -> Vec<[f64; 6]> {
    use spatial_contracts::*;
    let mut bounds = vec![
        CORE_BOUNDS,
        PILOT_CHAIR_BOUNDS,
        CONSOLE_BOUNDS,
        FABRICATOR_BOUNDS,
        COCKPIT_PORT_HULL_BOUNDS,
        COCKPIT_STARBOARD_HULL_BOUNDS,
        CARGO_HEADER_BOUNDS,
    ];
    bounds.extend(STARTER_COLLIDERS);
    bounds.extend(THRUSTER_COLLIDERS);
    bounds.extend(WING_COLLIDERS);
    bounds.extend(LANDING_COLLIDERS);
    bounds
}

/// Conservative existing cabin/nose hull envelope for exterior construction.
/// Interior placement uses the individual authored fixture/containment queries.
pub fn ship_conservative_hull_envelope() -> [f64; 6] {
    spatial_contracts::CABIN_EXTERIOR_BOUNDS
}
