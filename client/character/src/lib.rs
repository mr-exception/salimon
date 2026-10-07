//! Portable first-person character state and ship/surface traversal.
//!
//! This crate consumes typed input and reference-frame descriptions. It has no
//! dependency on native window events, rendering, or ship simulation state.

use std::time::Duration;
mod collision;
mod controller;
mod input;
mod layout;
mod math;
mod queries;
mod spatial_contracts;
mod state;

pub use controller::CharacterController;
pub use input::MovementInput;
pub use queries::{ship_floor_placement, ship_sight_obstruction};
pub use spatial_contracts::{
    COCKPIT_SEAT_MARKER_METERS, ENGINE_PORT_ANCHOR_METERS, ENGINE_STARBOARD_ANCHOR_METERS,
    EXIT_DOOR_MARKER_METERS, PLAYER_START_MARKER_METERS,
};
pub use state::{CharacterLocation, CharacterSnapshot, ShipFrame, SurfaceFrame};

/// Depth from the authored door interaction marker to the closed door's aft face.
/// Presentation may use this bound to avoid hiding the marker behind its own door.
pub const EXIT_DOOR_MARKER_DEPTH_METERS: f64 =
    EXIT_DOOR_MARKER_METERS[0] - spatial_contracts::AFT_DOOR_BOUNDS[0];

pub const FIXED_GRAVITY_METERS_PER_SECOND_SQUARED: f64 = 9.81;
pub const DOORWAY_GRAVITY_BLEND_DURATION: Duration = Duration::from_millis(250);
pub const PLAYER_BODY_HEIGHT_METERS: f64 = 1.80;
pub const PLAYER_EYE_HEIGHT_METERS: f64 = 1.75;
pub const SHIP_FLOOR_HEIGHT_METERS: f64 = spatial_contracts::INTERIOR_FLOOR_BOUNDS[3];
