//! Mutable held state owned by portable input; observers receive copies.
use salimon_character::MovementInput;
use salimon_ship::SteeringInput;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ShipControlInput {
    pub(crate) pitch_up: bool,
    pub(crate) pitch_down: bool,
    pub(crate) yaw_left: bool,
    pub(crate) yaw_right: bool,
    pub(crate) roll_left: bool,
    pub(crate) roll_right: bool,
}
impl ShipControlInput {
    pub(crate) fn steering(self) -> SteeringInput {
        SteeringInput {
            pitch: f64::from(i8::from(self.pitch_up) - i8::from(self.pitch_down)),
            yaw: f64::from(i8::from(self.yaw_left) - i8::from(self.yaw_right)),
            roll: f64::from(i8::from(self.roll_right) - i8::from(self.roll_left)),
        }
    }
}
#[derive(Default)]
pub(crate) struct State {
    pub(in crate::session) movement: MovementInput,
    pub(in crate::session) steering: ShipControlInput,
}
impl State {
    pub(crate) fn movement(&self) -> MovementInput {
        self.movement
    }
    pub(crate) fn steering(&self) -> ShipControlInput {
        self.steering
    }
    pub(in crate::session) fn clear(&mut self) {
        *self = Self::default();
    }
}
