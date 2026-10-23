//! Portable Phase 0 ship state and interaction rules.
//!
//! This crate owns inertial force-based flight, smoothed steering, cockpit authority,
//! solid-body collision correction, and landed/open-space airlock access without
//! depending on platform input or GPU presentation.

use salimon_world::LANDING_RANGE_ALTITUDE_RADIUS_FACTOR;
use std::time::Duration;

mod conversion;
pub use conversion::{ConversionConfig, IntakeResult};
mod energy;
pub use energy::{EnergyConfig, EnergyCore, InvalidEnergy};
mod mass;
pub use mass::{MassObjectId, ShipMass};
mod propulsion;
pub mod structure;
pub use propulsion::{
    InvalidFlightMotion, Propulsion, STARTER_DRY_MASS_KG, STARTER_IMPULSE_NS_PER_JOULE,
};

mod controller;
mod orientation;
mod state;

pub use controller::ShipController;
pub use controller::assist::{AssistConfig, InvalidAssistConfig};
pub use controller::cockpit::CockpitMessage;
pub use controller::telemetry::{
    BrakingEstimate, CoreStatus, EnergyCoreState, NavigationTelemetry, NearbyBodyTelemetry,
};
pub use controller::throttle::{THROTTLE_REPEAT_INTERVAL, ThrottleDirection};
pub use state::{DoorState, FlightState, ShipPose, ShipSnapshot, SteeringInput};

mod terrain_landing;
pub use terrain_landing::{LandingFootprint, MAX_LANDING_SLOPE, MAX_SUPPORT_GAP_METERS};
mod landing_geometry;
pub use landing_geometry::{
    LANDED_HULL_CLEARANCE_METERS, LANDED_ORIGIN_HEIGHT_METERS, LANDING_FOOT_LOCAL_Y_METERS,
    LOWEST_LOCAL_Y_METERS,
};
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
/// Takeoff spends two seconds lifting 10 m and four rising another 40 m, ending at rest.
pub const TAKEOFF_ASSIST_DURATION: Duration = Duration::from_secs(6);
/// Inclusive nominal-surface distance for the cockpit's nearby-body sensor.
pub use salimon_world::NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS;
/// Starter Core capacity/charge; watts and idle loads are configurable.
pub const DEFAULT_CORE_ENERGY_CAPACITY_JOULES: u64 = 60_000_000_000;
pub const DEFAULT_CORE_ENERGY_STORED_JOULES: u64 = 45_000_000_000;

pub const DOOR_HINGE_ANIMATION_DURATION: Duration = Duration::from_millis(700);

pub use controller::persistence::ShipSaveState;
