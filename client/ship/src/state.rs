//! Public pose and snapshot contracts.

use crate::orientation::normalized_quaternion;
use crate::{CockpitMessage, EnergyCoreState, NearbyBodyTelemetry};
use salimon_world::CelestialBodyId;

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SteeringInput {
    /// `-1` pitches down, `1` pitches up.
    pub pitch: f64,
    /// `-1` yaws right, `1` yaws left in the ship's rendered frame.
    pub yaw: f64,
    /// `-1` rolls left, `1` rolls right.
    pub roll: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DoorState {
    Closed,
    Open,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FlightState {
    Landed { body: CelestialBodyId },
    AssistedLanding { body: CelestialBodyId },
    AssistedTakeoff { body: CelestialBodyId },
    Flying,
    Bouncing { body: CelestialBodyId },
    GroundedAfterCrash { body: CelestialBodyId },
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShipPose {
    pub position_meters: [f64; 3],
    /// Unit quaternion `[x, y, z, w]` rotating ship-local coordinates to world.
    pub orientation: [f64; 4],
}

impl ShipPose {
    #[must_use]
    pub fn axes(self) -> [[f64; 3]; 3] {
        let [x, y, z, w] = normalized_quaternion(self.orientation);
        [
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y + z * w),
                2.0 * (x * z - y * w),
            ],
            [
                2.0 * (x * y - z * w),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z + x * w),
            ],
            [
                2.0 * (x * z + y * w),
                2.0 * (y * z - x * w),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShipSnapshot {
    pub terrain: salimon_world::terrain::TerrainMode,
    pub pose: ShipPose,
    pub flight_state: FlightState,
    pub door_state: DoorState,
    /// 0 is closed; 1 is raised clear of the aft doorway.
    pub door_open_fraction: f64,
    pub cockpit_control_active: bool,
    pub thruster_percentage: i8,
    pub engines_enabled: bool,
    pub total_mass_kg: f64,
    pub speed_meters_per_second: f64,
    pub velocity_meters_per_second: [f64; 3],
    pub navigation: crate::NavigationTelemetry,
    pub energy_core: EnergyCoreState,
    pub core_status: crate::CoreStatus,
    /// Authoritative power; fractional joules can power systems below display precision.
    pub power_available: bool,
    pub nearby_body: Option<NearbyBodyTelemetry>,
    pub cockpit_message: Option<CockpitMessage>,
    pub steering: SteeringInput,
}
