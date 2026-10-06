// Generated from Blender-authored scout spatial contracts; do not edit by hand.
// Bounds: [forward min/max, up min/max, port min/max] in ship-local meters.
pub const COCKPIT_SEAT_MARKER_METERS: [f64; 3] = [2.76, 1.129032258064516, 0.0];
pub const EXIT_DOOR_MARKER_METERS: [f64; 3] = [-7.1, 1.3440860215053763, 0.0];
pub const PLAYER_START_MARKER_METERS: [f64; 3] = [0.5, 1.9973118279569892, -2.2];
pub const ENGINE_PORT_ANCHOR_METERS: [f64; 3] = [-6.4, 1.075268817204301, 7.1];
pub const ENGINE_STARBOARD_ANCHOR_METERS: [f64; 3] = [-6.4, 1.075268817204301, -7.1];
pub(crate) const DOORWAY_TRANSITION_MARKER_METERS: [f64; 3] = [-7.04, 0.24731182795698925, 0.0];
// COLLIDER_InteriorFloor
pub(crate) const INTERIOR_FLOOR_BOUNDS: [f64; 6] = [
    -7.56,
    6.56,
    0.13978494623655913,
    0.24731182795698925,
    -4.6,
    4.6,
];
// COLLIDER_CockpitNoseFloor
pub(crate) const NOSE_FLOOR_BOUNDS: [f64; 6] =
    [6.5600000000000005, 7.85, 0.14, 0.247311828, -1.1, 1.1];
// COLLIDER_CabinTraversalEnvelope
pub(crate) const CABIN_TRAVERSAL_BOUNDS: [f64; 6] = [
    -7.24,
    6.5600000000000005,
    0.24731182795698903,
    2.693548387096774,
    -4.2,
    4.2,
];
// COLLIDER_CabinExteriorEnvelope
pub(crate) const CABIN_EXTERIOR_BOUNDS: [f64; 6] = [
    -7.92,
    8.53,
    -0.10752688172043001,
    2.9354838709677415,
    -5.1,
    5.1,
];
// COLLIDER_AftDoor
pub(crate) const AFT_DOOR_BOUNDS: [f64; 6] = [
    -7.819999999999999,
    -7.54,
    0.24731182795698925,
    2.516129032258064,
    -1.4,
    1.4,
];
// COLLIDER_EnergyCore
pub(crate) const CORE_BOUNDS: [f64; 6] =
    [-2.0, 0.0, 0.24731182795698947, 2.053763440860215, -1.0, 1.0];
// COLLIDER_PilotChair
pub(crate) const PILOT_CHAIR_BOUNDS: [f64; 6] = [
    1.7700000000000002,
    3.3,
    0.24731182795698925,
    1.73,
    -0.79,
    0.79,
];
// COLLIDER_CockpitCenterConsole
pub(crate) const CONSOLE_BOUNDS: [f64; 6] = [3.9000000000000004, 6.0, 0.25, 1.27, -1.35, 1.35];
// COLLIDER_CockpitPortHull
pub(crate) const COCKPIT_PORT_HULL_BOUNDS: [f64; 6] = [
    2.5,
    7.85,
    0.24731182795698947,
    2.9354838709677415,
    2.8999999999999995,
    4.2,
];
// COLLIDER_CockpitStarboardHull
pub(crate) const COCKPIT_STARBOARD_HULL_BOUNDS: [f64; 6] = [
    2.5,
    7.85,
    0.24731182795698947,
    2.9354838709677415,
    -4.2,
    -2.8999999999999995,
];
pub(crate) const THRUSTER_COLLIDERS: [[f64; 6]; 4] = [
    // COLLIDER_Engine_Port_Body
    [
        -9.8,
        -3.0,
        0.30107526881720426,
        1.8494623655913978,
        5.66,
        8.54,
    ],
    // COLLIDER_Engine_Port_SweptFin
    [
        -7.64,
        -4.3,
        0.6236559139784945,
        2.8494623655913975,
        6.989999999999999,
        7.21,
    ],
    // COLLIDER_Engine_Starboard_Body
    [
        -9.8,
        -3.0,
        0.30107526881720426,
        1.8494623655913978,
        -8.54,
        -5.66,
    ],
    // COLLIDER_Engine_Starboard_SweptFin
    [
        -7.64,
        -4.3,
        0.6236559139784945,
        2.8494623655913975,
        -7.21,
        -6.989999999999999,
    ],
];
pub(crate) const WING_COLLIDERS: [[f64; 6]; 2] = [
    // COLLIDER_Wing_Port
    [-8.1, 4.9, 0.193548387, 0.451612903, 4.6, 10.0],
    // COLLIDER_Wing_Starboard
    [-8.1, 4.9, 0.193548387, 0.451612903, -10.0, -4.6],
];
