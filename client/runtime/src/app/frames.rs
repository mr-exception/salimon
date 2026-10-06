//! Domain frame adapters used by gameplay and native automation.

use salimon_character::{ShipFrame, SurfaceFrame};
use salimon_ship::{FlightState, ShipPose, ShipSnapshot};
use salimon_world::{CELESTIAL_BODIES, WorldPosition};

pub(crate) fn character_ship_frame(pose: ShipPose) -> ShipFrame {
    ShipFrame {
        origin_meters: pose.position_meters,
        axes: pose.axes(),
    }
}

pub(crate) fn surface_frame_for_ship(ship: ShipSnapshot) -> SurfaceFrame {
    let body_id = match ship.flight_state {
        FlightState::Landed { body }
        | FlightState::AssistedLanding { body }
        | FlightState::AssistedTakeoff { body } => body,
        FlightState::Flying => {
            CELESTIAL_BODIES
                .iter()
                .filter(|body| body.role == salimon_world::BodyRole::Solid)
                .min_by(|left, right| {
                    distance_squared(left.center.meters(), ship.pose.position_meters).total_cmp(
                        &distance_squared(right.center.meters(), ship.pose.position_meters),
                    )
                })
                .expect("world catalog always contains a solid body")
                .id
        }
    };
    let body = CELESTIAL_BODIES
        .iter()
        .find(|body| body.id == body_id)
        .expect("ship body identifier belongs to the world catalog");
    SurfaceFrame {
        body_center_meters: body.center.meters(),
        radius_meters: body.radius_meters,
    }
}

pub(crate) fn nearby_surface_at(eye: [f64; 3]) -> Option<SurfaceFrame> {
    salimon_world::nearby_solid_body(WorldPosition::new(eye[0], eye[1], eye[2])).map(|(body, _)| {
        SurfaceFrame {
            body_center_meters: body.center.meters(),
            radius_meters: body.radius_meters,
        }
    })
}

fn distance_squared(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(a, b)| (a - b) * (a - b))
        .sum()
}
