// Generated from Blender-authored scout spatial contracts; do not edit by hand.
// [forward min/max, up min/max, port min/max] in ship-local meters.
pub(super) const THRUSTER_COLLIDERS: [[f64; 6]; 4] = [
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
pub(super) const WING_COLLIDERS: [[f64; 6]; 2] = [
    // COLLIDER_Wing_Port
    [-8.1, 4.9, 0.193548387, 0.451612903, 4.6, 10.0],
    // COLLIDER_Wing_Starboard
    [-8.1, 4.9, 0.193548387, 0.451612903, -10.0, -4.6],
];
