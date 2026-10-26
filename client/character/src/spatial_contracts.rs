// Generated from Blender-authored scout spatial contracts; do not edit by hand.
// Bounds: [forward min/max, up min/max, port min/max] in ship-local meters.
pub const FABRICATOR_INPUT_MARKER_METERS: [f64; 3] = [-3.0, 1.65, 3.55];
pub const FABRICATOR_MONITOR_MARKER_METERS: [f64; 3] = [-3.8, 2.0, 3.69];
pub const FABRICATOR_TRAY_MARKER_METERS: [f64; 3] = [-2.55, 0.991, 3.4];
pub const COCKPIT_SEAT_MARKER_METERS: [f64; 3] = [2.76, 1.129032258064516, 0.0];
pub const EXIT_DOOR_MARKER_METERS: [f64; 3] = [-7.1, 1.3440860215053763, 0.0];
pub const PLAYER_START_MARKER_METERS: [f64; 3] = [0.5, 1.9973118279569892, -2.2];
pub const ENGINE_PORT_ANCHOR_METERS: [f64; 3] = [-6.4, 1.075268817204301, 7.1];
pub const ENGINE_STARBOARD_ANCHOR_METERS: [f64; 3] = [-6.4, 1.075268817204301, -7.1];
pub(crate) const DOORWAY_TRANSITION_MARKER_METERS: [f64; 3] = [-7.04, 0.24731182795698925, 0.0];
// COLLIDER_Starter_Header
pub(crate) const CARGO_HEADER_BOUNDS: [f64; 6] = [
    -4.750000095367431,
    -4.550000095367432,
    2.4865482862226425,
    2.8064514581926407,
    -1.51,
    1.51,
];
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
    -7.88,
    -7.680000000000001,
    0.24731182795698947,
    2.516129032,
    -1.4,
    1.4,
];
// COLLIDER_Fabricator
pub(crate) const FABRICATOR_BOUNDS: [f64; 6] = [-4.35, -1.95, 0.5, 2.4, 3.1, 4.4];
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
pub(crate) const STARTER_COLLIDERS: [[f64; 6]; 7] = [
    // COLLIDER_Starter_BulkheadPort
    [
        -4.710000095367431,
        -4.590000095367432,
        0.24231177201835052,
        2.806451503128134,
        1.4999999046325683,
        4.199999904632568,
    ],
    // COLLIDER_Starter_BulkheadStarboard
    [
        -4.710000095367431,
        -4.590000095367432,
        0.24231177201835052,
        2.806451503128134,
        -4.199999904632568,
        -1.4999999046325683,
    ],
    // COLLIDER_Starter_BinPort
    [
        -6.899999904632568,
        -5.2999999046325685,
        0.24731184720993044,
        1.0873118472099303,
        2.8500000476837157,
        4.0500000476837155,
    ],
    // COLLIDER_Starter_BinStarboard
    [
        -6.899999904632568,
        -5.2999999046325685,
        0.24731184720993044,
        1.0873118472099303,
        -4.0500000476837155,
        -2.8500000476837157,
    ],
    // COLLIDER_Starter_Bunk
    [
        -1.900000011920929,
        0.29999998807907113,
        0.24500000476837158,
        1.9950000047683716,
        -4.0750000953674315,
        -3.225000095367432,
    ],
    // COLLIDER_Starter_Galley
    [
        -0.2000000238418579,
        1.4999999761581422,
        0.2500000214576721,
        1.3300000214576722,
        3.320000047683716,
        4.080000047683716,
    ],
    // COLLIDER_Starter_Hygiene
    [
        1.1,
        1.9,
        0.24999999284744268,
        1.3899999928474425,
        -4.050000095367432,
        -3.2500000953674317,
    ],
];
pub(crate) const WING_COLLIDERS: [[f64; 6]; 2] = [
    // COLLIDER_Wing_Port
    [-8.1, 4.9, 0.193548387, 0.451612903, 4.6, 10.0],
    // COLLIDER_Wing_Starboard
    [-8.1, 4.9, 0.193548387, 0.451612903, -10.0, -4.6],
];
pub(crate) const LANDING_COLLIDERS: [[f64; 6]; 8] = [
    // COLLIDER_Landing_Aft_Port_Foot
    [
        -6.274999904632568,
        -4.9249999046325685,
        -2.1075268936157228,
        -1.8675268936157225,
        3.0499999046325685,
        4.149999904632568,
    ],
    // COLLIDER_Landing_Aft_Port_Leg
    [
        -6.0499999046325685,
        -5.149999904632568,
        -2.107526867389679,
        0.45247313261032107,
        3.189999904632568,
        4.0099999046325685,
    ],
    // COLLIDER_Landing_Aft_Starboard_Foot
    [
        -6.274999904632568,
        -4.9249999046325685,
        -2.1075268936157228,
        -1.8675268936157225,
        -4.149999904632568,
        -3.0499999046325685,
    ],
    // COLLIDER_Landing_Aft_Starboard_Leg
    [
        -6.0499999046325685,
        -5.149999904632568,
        -2.107526867389679,
        0.45247313261032107,
        -4.0099999046325685,
        -3.189999904632568,
    ],
    // COLLIDER_Landing_Forward_Port_Foot
    [
        4.1250001907348635,
        5.475000190734863,
        -2.1075268936157228,
        -1.8675268936157225,
        3.0499999046325685,
        4.149999904632568,
    ],
    // COLLIDER_Landing_Forward_Port_Leg
    [
        4.350000190734863,
        5.2500001907348635,
        -2.107526867389679,
        0.45247313261032107,
        3.189999904632568,
        4.0099999046325685,
    ],
    // COLLIDER_Landing_Forward_Starboard_Foot
    [
        4.1250001907348635,
        5.475000190734863,
        -2.1075268936157228,
        -1.8675268936157225,
        -4.149999904632568,
        -3.0499999046325685,
    ],
    // COLLIDER_Landing_Forward_Starboard_Leg
    [
        4.350000190734863,
        5.2500001907348635,
        -2.107526867389679,
        0.45247313261032107,
        -4.0099999046325685,
        -3.189999904632568,
    ],
];
