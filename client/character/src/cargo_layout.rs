// Generated from Blender-authored scout spatial contracts; do not edit by hand.
pub const CARGO_ROOM_MIN_METERS: [f64; 3] = [-3.2, 0.24731182795698925, 5.1];
pub const CARGO_ROOM_MAX_METERS: [f64; 3] = [2.2, 2.806451612903226, 10.600000000000001];
// [forward min/max, port min/max]; includes the cabin partition at the passage.
pub(super) const CARGO_WALLS: [[f64; 4]; 5] = [
    [
        -7.5600000000000005,
        0.3999999999999999,
        4.200000000000001,
        5.1,
    ],
    [
        2.1999999999999997,
        6.5600000000000005,
        4.200000000000001,
        5.1,
    ],
    [-3.3999999999999995, -3.2, 5.1000000000000005, 11.0],
    [2.2, 2.3999999999999995, 5.1000000000000005, 11.0],
    [-3.4, 2.4, 10.600000000000001, 11.0],
];
