//! Platform-neutral controls and normalized planar input.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MovementInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    /// EVA down; ignored during floor/surface walking.
    pub descend: bool,
}

pub(crate) fn axis(positive: bool, negative: bool) -> f64 {
    f64::from(u8::from(positive)) - f64::from(u8::from(negative))
}
pub(crate) fn movement_axes(input: MovementInput) -> [f64; 2] {
    let forward = f64::from(u8::from(input.forward)) - f64::from(u8::from(input.backward));
    let right = f64::from(u8::from(input.right)) - f64::from(u8::from(input.left));
    let length = (forward * forward + right * right).sqrt();
    if length > 1.0 {
        [forward / length, right / length]
    } else {
        [forward, right]
    }
}

pub(crate) fn planar_look(yaw: f64) -> ([f64; 2], [f64; 2]) {
    let forward = [yaw.cos(), yaw.sin()];
    (forward, [-forward[1], forward[0]])
}
pub(crate) fn ship_planar_movement(input: MovementInput, yaw: f64) -> [f64; 2] {
    let movement = movement_axes(input);
    let (forward, right) = planar_look(yaw);
    [
        forward[0] * movement[0] + right[0] * movement[1],
        forward[1] * movement[0] + right[1] * movement[1],
    ]
}
