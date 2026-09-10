//! Portable first-person character state and ship/surface traversal.
//!
//! This crate consumes typed input and reference-frame descriptions. It has no
//! dependency on native window events, rendering, or ship simulation state.

use std::f64::consts::{FRAC_PI_2, TAU};
use std::time::Duration;

pub const FIXED_GRAVITY_METERS_PER_SECOND_SQUARED: f64 = 9.81;
pub const DOORWAY_GRAVITY_BLEND_DURATION: Duration = Duration::from_millis(250);
const WALK_SPEED_METERS_PER_SECOND: f64 = 3.8;
const JUMP_SPEED_METERS_PER_SECOND: f64 = 4.4;
const EYE_HEIGHT_METERS: f64 = 1.62;
const SHIP_FLOOR_HEIGHT: f64 = 0.23;
const INTERIOR_FORWARD_MIN: f64 = -3.62;
// Stop walking before the cockpit console; cockpit seating is a contextual
// transition rather than walking through its visible geometry.
const INTERIOR_FORWARD_MAX: f64 = 0.65;
const INTERIOR_SIDE_LIMIT: f64 = 1.43;
const DOORWAY_FORWARD: f64 = -3.52;
const COCKPIT_POSITION: [f64; 3] = [1.38, 1.72, 0.0];
const COCKPIT_VIEW_PITCH_RADIANS: f64 = -0.10;
const PLAYER_START: [f64; 3] = [-0.35, EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT, 0.0];
const LOOK_SENSITIVITY_RADIANS_PER_PIXEL: f64 = 0.0022;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MovementInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShipFrame {
    pub origin_meters: [f64; 3],
    /// Local ship axes in world space: forward (+X), up (+Y), port (+Z).
    pub axes: [[f64; 3]; 3],
}

impl ShipFrame {
    #[must_use]
    pub fn local_to_world(self, local: [f64; 3]) -> [f64; 3] {
        add(
            self.origin_meters,
            add(
                scale(self.axes[0], local[0]),
                add(scale(self.axes[1], local[1]), scale(self.axes[2], local[2])),
            ),
        )
    }

    #[must_use]
    pub fn world_to_local(self, world: [f64; 3]) -> [f64; 3] {
        let relative = sub(world, self.origin_meters);
        [
            dot(relative, self.axes[0]),
            dot(relative, self.axes[1]),
            dot(relative, self.axes[2]),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceFrame {
    pub body_center_meters: [f64; 3],
    pub radius_meters: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterLocation {
    Cockpit,
    InsideShip,
    DoorwayBlend,
    Surface,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterSnapshot {
    pub location: CharacterLocation,
    pub eye_position_meters: [f64; 3],
    pub look_target_meters: [f64; 3],
    pub up: [f32; 3],
    pub local_ship_position_meters: Option<[f64; 3]>,
    pub doorway_blend_fraction: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PositionState {
    Cockpit,
    Inside {
        local: [f64; 3],
    },
    Doorway {
        world: [f64; 3],
        elapsed: Duration,
        leaving_ship: bool,
    },
    Surface {
        world: [f64; 3],
    },
}

#[derive(Clone, Debug)]
pub struct CharacterController {
    position: PositionState,
    yaw_radians: f64,
    pitch_radians: f64,
    vertical_speed: f64,
    jump_was_down: bool,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            position: PositionState::Inside {
                local: PLAYER_START,
            },
            yaw_radians: 0.0,
            pitch_radians: 0.0,
            vertical_speed: 0.0,
            jump_was_down: false,
        }
    }
}

impl CharacterController {
    #[must_use]
    pub const fn location(&self) -> CharacterLocation {
        match self.position {
            PositionState::Cockpit => CharacterLocation::Cockpit,
            PositionState::Inside { .. } => CharacterLocation::InsideShip,
            PositionState::Doorway { .. } => CharacterLocation::DoorwayBlend,
            PositionState::Surface { .. } => CharacterLocation::Surface,
        }
    }

    #[must_use]
    pub fn local_ship_position(&self) -> Option<[f64; 3]> {
        match self.position {
            PositionState::Cockpit => Some(COCKPIT_POSITION),
            PositionState::Inside { local } => Some(local),
            PositionState::Doorway { .. } | PositionState::Surface { .. } => None,
        }
    }

    pub fn apply_mouse_delta(&mut self, delta_x: f64, delta_y: f64) {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return;
        }
        self.yaw_radians =
            (self.yaw_radians + delta_x * LOOK_SENSITIVITY_RADIANS_PER_PIXEL).rem_euclid(TAU);
        self.pitch_radians = (self.pitch_radians - delta_y * LOOK_SENSITIVITY_RADIANS_PER_PIXEL)
            .clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
    }

    pub fn enter_cockpit(&mut self) {
        if matches!(self.position, PositionState::Inside { .. }) {
            self.position = PositionState::Cockpit;
            self.vertical_speed = 0.0;
        }
    }

    pub fn leave_cockpit(&mut self) {
        if matches!(self.position, PositionState::Cockpit) {
            self.position = PositionState::Inside {
                local: [0.75, EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT, 0.0],
            };
        }
    }

    pub fn advance(
        &mut self,
        delta: Duration,
        input: MovementInput,
        ship: ShipFrame,
        surface: SurfaceFrame,
        door_open: bool,
        ship_landed: bool,
    ) {
        let seconds = delta.as_secs_f64().min(0.1);
        if seconds <= 0.0 {
            return;
        }
        let jump_started = input.jump && !self.jump_was_down;
        self.jump_was_down = input.jump;

        match self.position {
            PositionState::Cockpit => {}
            PositionState::Inside { mut local } => {
                let movement = movement_axes(input);
                let (forward, right) = planar_look(self.yaw_radians);
                local[0] += (forward[0] * movement[0] + right[0] * movement[1])
                    * WALK_SPEED_METERS_PER_SECOND
                    * seconds;
                local[2] += (forward[1] * movement[0] + right[1] * movement[1])
                    * WALK_SPEED_METERS_PER_SECOND
                    * seconds;
                if jump_started && local[1] <= EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT + 0.001 {
                    self.vertical_speed = JUMP_SPEED_METERS_PER_SECOND;
                }
                self.vertical_speed -= FIXED_GRAVITY_METERS_PER_SECOND_SQUARED * seconds;
                local[1] += self.vertical_speed * seconds;
                if local[1] <= EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT {
                    local[1] = EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT;
                    self.vertical_speed = 0.0;
                }
                local[2] = local[2].clamp(-INTERIOR_SIDE_LIMIT, INTERIOR_SIDE_LIMIT);
                local[0] = local[0].min(INTERIOR_FORWARD_MAX);
                if local[0] < DOORWAY_FORWARD && door_open && ship_landed {
                    self.position = PositionState::Doorway {
                        world: ship.local_to_world(local),
                        elapsed: Duration::ZERO,
                        leaving_ship: true,
                    };
                } else {
                    local[0] = local[0].max(INTERIOR_FORWARD_MIN);
                    self.position = PositionState::Inside { local };
                }
            }
            PositionState::Doorway {
                mut world,
                mut elapsed,
                leaving_ship,
            } => {
                let movement = movement_axes(input);
                world = add(
                    world,
                    scale(
                        ship.axes[0],
                        movement[0] * WALK_SPEED_METERS_PER_SECOND * seconds,
                    ),
                );
                elapsed = elapsed.saturating_add(delta);
                if elapsed >= DOORWAY_GRAVITY_BLEND_DURATION {
                    if leaving_ship {
                        self.position = PositionState::Surface {
                            world: project_eye_to_surface(world, surface),
                        };
                    } else {
                        let mut local = ship.world_to_local(world);
                        local[1] = EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT;
                        self.position = PositionState::Inside { local };
                    }
                } else {
                    self.position = PositionState::Doorway {
                        world,
                        elapsed,
                        leaving_ship,
                    };
                }
            }
            PositionState::Surface { mut world } => {
                let up = normalize(sub(world, surface.body_center_meters));
                let reference_forward = reject(ship.axes[0], up);
                let forward = normalize_or(reference_forward, ship.axes[2]);
                let right = normalize(cross(forward, up));
                let movement = movement_axes(input);
                let tangent = add(scale(forward, movement[0]), scale(right, movement[1]));
                world = add(
                    world,
                    scale(tangent, WALK_SPEED_METERS_PER_SECOND * seconds),
                );
                world = project_eye_to_surface(world, surface);

                let local = ship.world_to_local(world);
                let near_door =
                    (local[0] - (DOORWAY_FORWARD - 0.12)).abs() < 0.8 && local[2].abs() < 0.8;
                if door_open && ship_landed && near_door && movement[0] > 0.0 {
                    self.position = PositionState::Doorway {
                        world,
                        elapsed: Duration::ZERO,
                        leaving_ship: false,
                    };
                } else {
                    self.position = PositionState::Surface { world };
                }
            }
        }
    }

    #[must_use]
    pub fn snapshot(&self, ship: ShipFrame, surface: SurfaceFrame) -> CharacterSnapshot {
        let (eye, up, blend) = match self.position {
            PositionState::Cockpit => (ship.local_to_world(COCKPIT_POSITION), ship.axes[1], None),
            PositionState::Inside { local } => (ship.local_to_world(local), ship.axes[1], None),
            PositionState::Surface { world } => (
                world,
                normalize(sub(world, surface.body_center_meters)),
                None,
            ),
            PositionState::Doorway {
                world,
                elapsed,
                leaving_ship,
            } => {
                let fraction = (elapsed.as_secs_f64()
                    / DOORWAY_GRAVITY_BLEND_DURATION.as_secs_f64())
                .clamp(0.0, 1.0);
                let surface_up = normalize(sub(world, surface.body_center_meters));
                let amount = if leaving_ship {
                    fraction
                } else {
                    1.0 - fraction
                };
                (
                    world,
                    normalize(lerp(ship.axes[1], surface_up, amount)),
                    Some(fraction),
                )
            }
        };
        let (planar_forward, _) = planar_look(self.yaw_radians);
        let raw_forward = normalize(add(
            scale(ship.axes[0], planar_forward[0]),
            scale(ship.axes[2], planar_forward[1]),
        ));
        let base_forward = if matches!(self.position, PositionState::Surface { .. }) {
            normalize_or(reject(raw_forward, up), reject(ship.axes[2], up))
        } else {
            raw_forward
        };
        let right = normalize(cross(base_forward, up));
        let pitch = self.pitch_radians
            + if matches!(self.position, PositionState::Cockpit) {
                COCKPIT_VIEW_PITCH_RADIANS
            } else {
                0.0
            };
        let look = normalize(add(
            scale(base_forward, pitch.cos()),
            scale(up, pitch.sin()),
        ));
        let corrected_look = normalize(reject(look, right));

        CharacterSnapshot {
            location: self.location(),
            eye_position_meters: eye,
            look_target_meters: add(eye, corrected_look),
            up: up.map(|value| value as f32),
            local_ship_position_meters: self.local_ship_position(),
            doorway_blend_fraction: blend,
        }
    }
}

fn movement_axes(input: MovementInput) -> [f64; 2] {
    let forward = f64::from(u8::from(input.forward)) - f64::from(u8::from(input.backward));
    let right = f64::from(u8::from(input.right)) - f64::from(u8::from(input.left));
    let length = (forward * forward + right * right).sqrt();
    if length > 1.0 {
        [forward / length, right / length]
    } else {
        [forward, right]
    }
}

fn planar_look(yaw: f64) -> ([f64; 2], [f64; 2]) {
    let forward = [yaw.cos(), yaw.sin()];
    (forward, [-forward[1], forward[0]])
}

fn project_eye_to_surface(world: [f64; 3], surface: SurfaceFrame) -> [f64; 3] {
    add(
        surface.body_center_meters,
        scale(
            normalize(sub(world, surface.body_center_meters)),
            surface.radius_meters + EYE_HEIGHT_METERS,
        ),
    )
}

fn add(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn sub(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn scale(vector: [f64; 3], amount: f64) -> [f64; 3] {
    vector.map(|value| value * amount)
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter().zip(right).map(|(a, b)| a * b).sum()
}

fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    normalize_or(vector, [0.0, 1.0, 0.0])
}

fn normalize_or(vector: [f64; 3], fallback: [f64; 3]) -> [f64; 3] {
    let length = dot(vector, vector).sqrt();
    if length > 1.0e-12 {
        scale(vector, length.recip())
    } else {
        fallback
    }
}

fn reject(vector: [f64; 3], normal: [f64; 3]) -> [f64; 3] {
    sub(vector, scale(normal, dot(vector, normal)))
}

fn lerp(from: [f64; 3], to: [f64; 3], amount: f64) -> [f64; 3] {
    add(scale(from, 1.0 - amount), scale(to, amount))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> ShipFrame {
        ShipFrame {
            origin_meters: [0.0, 10.0, 0.0],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    fn surface() -> SurfaceFrame {
        SurfaceFrame {
            body_center_meters: [0.0, -90.0, 0.0],
            radius_meters: 100.0,
        }
    }

    #[test]
    fn starts_inside_landed_ship_and_cockpit_transition_is_instant() {
        let mut controller = CharacterController::default();
        assert_eq!(controller.location(), CharacterLocation::InsideShip);
        controller.enter_cockpit();
        assert_eq!(controller.location(), CharacterLocation::Cockpit);
        controller.leave_cockpit();
        assert_eq!(controller.location(), CharacterLocation::InsideShip);
    }

    #[test]
    fn horizontal_mouse_delta_turns_view_in_screen_direction_inside_and_in_cockpit() {
        let mut right = CharacterController::default();
        right.apply_mouse_delta(10.0, 0.0);
        let right_snapshot = right.snapshot(frame(), surface());
        assert!(right_snapshot.look_target_meters[2] > right_snapshot.eye_position_meters[2]);

        let mut left = CharacterController::default();
        left.apply_mouse_delta(-10.0, 0.0);
        let left_snapshot = left.snapshot(frame(), surface());
        assert!(left_snapshot.look_target_meters[2] < left_snapshot.eye_position_meters[2]);

        right.enter_cockpit();
        let cockpit_snapshot = right.snapshot(frame(), surface());
        assert!(cockpit_snapshot.look_target_meters[2] > cockpit_snapshot.eye_position_meters[2]);
    }

    #[test]
    fn vertical_mouse_delta_keeps_the_existing_pitch_direction() {
        let mut down = CharacterController::default();
        down.apply_mouse_delta(0.0, 10.0);
        let down_snapshot = down.snapshot(frame(), surface());
        assert!(down_snapshot.look_target_meters[1] < down_snapshot.eye_position_meters[1]);

        let mut up = CharacterController::default();
        up.apply_mouse_delta(0.0, -10.0);
        let up_snapshot = up.snapshot(frame(), surface());
        assert!(up_snapshot.look_target_meters[1] > up_snapshot.eye_position_meters[1]);
    }

    #[test]
    fn default_cockpit_view_keeps_the_forward_window_in_sight() {
        let mut controller = CharacterController::default();
        controller.enter_cockpit();
        let snapshot = controller.snapshot(frame(), surface());
        let look = sub(snapshot.look_target_meters, snapshot.eye_position_meters);

        assert!(look[0] > 0.99, "cockpit view must remain primarily forward");
        assert!(
            look[1] > -0.11,
            "cockpit view must clear the console and nose"
        );
        assert!(look[2].abs() < 1.0e-12);
    }

    #[test]
    fn ship_floor_gravity_and_jump_use_the_shared_fixed_strength() {
        let mut controller = CharacterController::default();
        controller.advance(
            Duration::from_millis(16),
            MovementInput {
                jump: true,
                ..MovementInput::default()
            },
            frame(),
            surface(),
            false,
            true,
        );
        let local = controller.local_ship_position().unwrap();
        assert!(local[1] > PLAYER_START[1]);
        assert_eq!(FIXED_GRAVITY_METERS_PER_SECOND_SQUARED, 9.81);
    }

    #[test]
    fn closed_or_flying_door_keeps_player_inside() {
        let mut controller = CharacterController {
            position: PositionState::Inside {
                local: [-3.5, PLAYER_START[1], 0.0],
            },
            ..CharacterController::default()
        };
        let leave = MovementInput {
            backward: true,
            ..MovementInput::default()
        };
        controller.advance(
            Duration::from_secs(1),
            leave,
            frame(),
            surface(),
            false,
            true,
        );
        assert_eq!(controller.location(), CharacterLocation::InsideShip);
        controller.advance(
            Duration::from_secs(1),
            leave,
            frame(),
            surface(),
            true,
            false,
        );
        assert_eq!(controller.location(), CharacterLocation::InsideShip);
    }

    #[test]
    fn landed_open_door_starts_exact_quarter_second_gravity_blend() {
        let mut controller = CharacterController {
            position: PositionState::Inside {
                local: [-3.5, PLAYER_START[1], 0.0],
            },
            ..CharacterController::default()
        };
        let leave = MovementInput {
            backward: true,
            ..MovementInput::default()
        };
        controller.advance(
            Duration::from_millis(50),
            leave,
            frame(),
            surface(),
            true,
            true,
        );
        assert_eq!(controller.location(), CharacterLocation::DoorwayBlend);
        controller.advance(
            Duration::from_millis(249),
            leave,
            frame(),
            surface(),
            true,
            true,
        );
        assert_eq!(controller.location(), CharacterLocation::DoorwayBlend);
        controller.advance(
            Duration::from_millis(1),
            leave,
            frame(),
            surface(),
            true,
            true,
        );
        assert_eq!(controller.location(), CharacterLocation::Surface);
    }

    #[test]
    fn surface_motion_stays_on_full_sphere_with_fixed_eye_height() {
        let mut controller = CharacterController {
            position: PositionState::Surface {
                world: [0.0, 11.62, 0.0],
            },
            ..CharacterController::default()
        };
        let small_surface = SurfaceFrame {
            body_center_meters: [0.0, 0.0, 0.0],
            radius_meters: 10.0,
        };
        controller.advance(
            Duration::from_secs(1),
            MovementInput {
                forward: true,
                ..MovementInput::default()
            },
            frame(),
            small_surface,
            true,
            true,
        );
        let snapshot = controller.snapshot(frame(), small_surface);
        let radius = dot(snapshot.eye_position_meters, snapshot.eye_position_meters).sqrt();
        assert!((radius - 11.62).abs() < 1.0e-9);
    }
}
