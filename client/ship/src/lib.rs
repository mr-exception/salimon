//! Portable Phase 0 ship state and interaction rules.
//!
//! This crate owns direct-speed flight, smoothed steering, cockpit authority,
//! solid-body collision correction, and landed/open-space airlock access without
//! depending on platform input or GPU presentation.

use salimon_world::LANDING_RANGE_ALTITUDE_RADIUS_FACTOR;
use std::time::Duration;

mod controller;
mod orientation;
mod state;

pub use controller::ShipController;
pub use controller::cockpit::CockpitMessage;
pub use controller::telemetry::{EnergyCoreState, NearbyBodyTelemetry};
pub use state::{DoorState, FlightState, ShipPose, ShipSnapshot, SteeringInput};

/// Lowest point of the revised 4 m-tall Task 10 runtime mesh in ship-local Y.
pub const LOWEST_LOCAL_Y_METERS: f64 = -0.107_526_881_720_430_11;
/// Phase 0 steering rate after the short presentation ramp has settled.
pub const STEERING_RATE_RADIANS_PER_SECOND: f64 = 5.0_f64.to_radians();
/// Time for a steering axis to ramp fully on or off. This is not inertia.
pub const STEERING_RAMP_SECONDS: f64 = 0.12;
/// Conservative bounding sphere for the 18.33 x 4.0 x 20.0 meter ship asset.
pub const COLLISION_RADIUS_METERS: f64 = 16.0;
/// Height above a solid surface at which Phase 0 landing assistance is offered.
pub const LANDING_RANGE_RADIUS_FRACTION: f64 = LANDING_RANGE_ALTITUDE_RADIUS_FACTOR;
/// Landing spends two seconds aligning, four approaching, and two touching down.
pub const LANDING_ASSIST_DURATION: Duration = Duration::from_secs(8);
/// Takeoff spends two seconds lifting locally and four clearing the landing volume.
pub const TAKEOFF_ASSIST_DURATION: Duration = Duration::from_secs(6);
/// Inclusive nominal-surface distance for the cockpit's nearby-body sensor.
pub use salimon_world::NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS;
/// Noncanonical Phase 0 fixture values used to exercise live Core telemetry.
pub const DEFAULT_CORE_ENERGY_CAPACITY_JOULES: u64 = 1_000_000_000_000;
pub const DEFAULT_CORE_ENERGY_STORED_JOULES: u64 = 750_000_000_000;

pub const DOOR_HINGE_ANIMATION_DURATION: Duration = Duration::from_millis(700);
