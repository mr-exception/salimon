//! Shared authored dimensions and conservative traversal proxies.
use crate::spatial_contracts::*;
use crate::{PLAYER_EYE_HEIGHT_METERS, SHIP_FLOOR_HEIGHT_METERS};
pub(crate) const PLAYER_RADIUS_METERS: f64 = 0.24;
pub(crate) const SHIP_FLOOR_HEIGHT: f64 = SHIP_FLOOR_HEIGHT_METERS;
// Keep the player's head below the lowest ceiling lamps and the aft lintel.
pub(crate) const SHIP_CEILING_HEIGHT: f64 = CABIN_TRAVERSAL_BOUNDS[3];
pub(crate) const DOORWAY_CEILING_HEIGHT: f64 = AFT_DOOR_BOUNDS[3];
pub(crate) const INTERIOR_FORWARD_MIN: f64 = CABIN_TRAVERSAL_BOUNDS[0];
pub(crate) const AFT_WALL_FORWARD_MIN: f64 = INTERIOR_FORWARD_MIN + PLAYER_RADIUS_METERS;
// The broad deck ends at X=6.56 m. A narrower level floor continues into the
// shortened nose through X=7.85 m. Keep the complete player body over each deck.
pub(crate) const CABIN_FORWARD_MAX: f64 = INTERIOR_FLOOR_BOUNDS[1] - PLAYER_RADIUS_METERS;
pub(crate) const INTERIOR_FORWARD_MAX: f64 = NOSE_FLOOR_BOUNDS[1] - PLAYER_RADIUS_METERS;
pub(crate) const NOSE_SIDE_LIMIT: f64 = NOSE_FLOOR_BOUNDS[5] - PLAYER_RADIUS_METERS;
// Window sills project into the 9.20 m deck to Z = +/-4.20 m.
pub(crate) const INTERIOR_SIDE_LIMIT: f64 = CABIN_TRAVERSAL_BOUNDS[5] - PLAYER_RADIUS_METERS;
pub(crate) const DOORWAY_FORWARD: f64 = DOORWAY_TRANSITION_MARKER_METERS[0];
pub(crate) const DOORWAY_SIDE_LIMIT: f64 = AFT_DOOR_BOUNDS[5] - PLAYER_RADIUS_METERS;
// Body-expanded exterior cabin/nose envelope, independent of door state.
// The aft wall is split around the same clear aperture used inside. Wings and
// engines are excluded so their broad asset bounds cannot obstruct the gate.
pub(crate) const EXTERIOR_AFT: f64 = CABIN_EXTERIOR_BOUNDS[0] - PLAYER_RADIUS_METERS;
pub(crate) const EXTERIOR_FORWARD: f64 = CABIN_EXTERIOR_BOUNDS[1] + PLAYER_RADIUS_METERS;
pub(crate) const EXTERIOR_SIDE: f64 = CABIN_EXTERIOR_BOUNDS[5] + PLAYER_RADIUS_METERS;
pub(crate) const EXTERIOR_BOTTOM: f64 = CABIN_EXTERIOR_BOUNDS[2];
pub(crate) const EXTERIOR_TOP: f64 = CABIN_EXTERIOR_BOUNDS[3];
// A door can close while the body overlaps its proxy. Resolve an active
// crossing to the nearer side of the two existing body-clear stopping planes.
pub(crate) const CLOSED_GATE_MIDPOINT: f64 = (EXTERIOR_AFT + INTERIOR_FORWARD_MIN) * 0.5;
pub(crate) const COLLISION_EPSILON: f64 = 1.0e-7;
pub(crate) const EXTERIOR_OBSTACLES: [[f64; 4]; 3] = [
    [
        DOORWAY_FORWARD,
        EXTERIOR_FORWARD,
        -EXTERIOR_SIDE,
        EXTERIOR_SIDE,
    ],
    [
        EXTERIOR_AFT,
        DOORWAY_FORWARD + PLAYER_RADIUS_METERS,
        -EXTERIOR_SIDE,
        -DOORWAY_SIDE_LIMIT,
    ],
    [
        EXTERIOR_AFT,
        DOORWAY_FORWARD + PLAYER_RADIUS_METERS,
        DOORWAY_SIDE_LIMIT,
        EXTERIOR_SIDE,
    ],
];
pub(crate) const CLOSED_GATE_OBSTACLE: [[f64; 4]; 1] = [[
    EXTERIOR_AFT,
    DOORWAY_FORWARD + PLAYER_RADIUS_METERS,
    -EXTERIOR_SIDE,
    EXTERIOR_SIDE,
]];
// The center Core's two-meter pedestal remains solid, with clear aisles on
// both sides. Inflate its planar footprint by the player's body radius.
pub(crate) const CORE_FORWARD_MIN: f64 = CORE_BOUNDS[0] - PLAYER_RADIUS_METERS;
pub(crate) const CORE_FORWARD_MAX: f64 = CORE_BOUNDS[1] + PLAYER_RADIUS_METERS;
pub(crate) const CORE_SIDE_LIMIT: f64 = CORE_BOUNDS[5] + PLAYER_RADIUS_METERS;
pub(crate) const COCKPIT_CHAIR_OBSTACLE: [f64; 4] = [
    PILOT_CHAIR_BOUNDS[0] - PLAYER_RADIUS_METERS,
    PILOT_CHAIR_BOUNDS[1] + PLAYER_RADIUS_METERS,
    PILOT_CHAIR_BOUNDS[4] - PLAYER_RADIUS_METERS,
    PILOT_CHAIR_BOUNDS[5] + PLAYER_RADIUS_METERS,
];
pub(crate) const COCKPIT_CONSOLE_OBSTACLE: [f64; 4] = [
    CONSOLE_BOUNDS[0] - PLAYER_RADIUS_METERS,
    CONSOLE_BOUNDS[1] + PLAYER_RADIUS_METERS,
    CONSOLE_BOUNDS[4] - PLAYER_RADIUS_METERS,
    CONSOLE_BOUNDS[5] + PLAYER_RADIUS_METERS,
];
// The solid forward hull closes the wide deck at both sides. Its body-expanded
// inner faces keep walkers out of the exterior shell.
pub(crate) const COCKPIT_PORT_HULL_OBSTACLE: [f64; 4] = [
    COCKPIT_PORT_HULL_BOUNDS[0] - PLAYER_RADIUS_METERS,
    INTERIOR_FORWARD_MAX,
    COCKPIT_PORT_HULL_BOUNDS[4] - PLAYER_RADIUS_METERS,
    INTERIOR_SIDE_LIMIT,
];
pub(crate) const COCKPIT_STARBOARD_HULL_OBSTACLE: [f64; 4] = [
    COCKPIT_STARBOARD_HULL_BOUNDS[0] - PLAYER_RADIUS_METERS,
    INTERIOR_FORWARD_MAX,
    -INTERIOR_SIDE_LIMIT,
    COCKPIT_STARBOARD_HULL_BOUNDS[5] + PLAYER_RADIUS_METERS,
];
// The broad deck shoulders stop at X=6.56 m. The narrow center floor continues
// into the nose; these body-expanded shoulders keep both feet over that floor.
pub(crate) const COCKPIT_PORT_NOSE_SHOULDER: [f64; 4] = [
    CABIN_FORWARD_MAX,
    INTERIOR_FORWARD_MAX,
    NOSE_SIDE_LIMIT,
    INTERIOR_SIDE_LIMIT,
];
pub(crate) const COCKPIT_STARBOARD_NOSE_SHOULDER: [f64; 4] = [
    CABIN_FORWARD_MAX,
    INTERIOR_FORWARD_MAX,
    -INTERIOR_SIDE_LIMIT,
    -NOSE_SIDE_LIMIT,
];
// [forward minimum, forward maximum, side minimum, side maximum]. These simple
// body-expanded footprints match the Core, pilot chair, and
// unified console/monitor assembly.
pub(crate) const INTERIOR_OBSTACLES: [[f64; 4]; 7] = [
    [
        CORE_FORWARD_MIN,
        CORE_FORWARD_MAX,
        -CORE_SIDE_LIMIT,
        CORE_SIDE_LIMIT,
    ],
    COCKPIT_CHAIR_OBSTACLE,
    COCKPIT_CONSOLE_OBSTACLE,
    COCKPIT_PORT_HULL_OBSTACLE,
    COCKPIT_STARBOARD_HULL_OBSTACLE,
    COCKPIT_PORT_NOSE_SHOULDER,
    COCKPIT_STARBOARD_NOSE_SHOULDER,
];
pub(crate) const COCKPIT_POSITION: [f64; 3] = [
    COCKPIT_SEAT_MARKER_METERS[0],
    COCKPIT_SEAT_MARKER_METERS[1] + 0.67,
    COCKPIT_SEAT_MARKER_METERS[2],
];
pub(crate) const COCKPIT_VIEW_PITCH_RADIANS: f64 = -0.10;
// Keep the standing eye exactly on the controller floor; glTF marker positions
// are stored as f32 and can differ from that plane by a few nanometers.
pub(crate) const PLAYER_START: [f64; 3] = [
    PLAYER_START_MARKER_METERS[0],
    SHIP_FLOOR_HEIGHT + PLAYER_EYE_HEIGHT_METERS,
    PLAYER_START_MARKER_METERS[2],
];
pub(crate) const LOOK_SENSITIVITY_RADIANS_PER_PIXEL: f64 = 0.0022;
