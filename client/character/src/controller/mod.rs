//! Character state machine and typed update dispatch.
use crate::collision::*;
use crate::input::{axis, movement_axes, planar_look, ship_planar_movement};
use crate::layout::*;
use crate::math::{lerp, normalize, normalize_or, orthogonal_tangent, reject};
use crate::state::PositionState;
use crate::{
    CharacterLocation, CharacterSnapshot, FIXED_GRAVITY_METERS_PER_SECOND_SQUARED, MovementInput,
    PLAYER_BODY_HEIGHT_METERS, PLAYER_EYE_HEIGHT_METERS, ShipFrame, SurfaceFrame,
};
use salimon_math::{add, cross, dot, scale, sub};
use std::f64::consts::{FRAC_PI_2, TAU};
use std::time::Duration;
use surface::{project_eye_to_surface, surface_eye_at_ship_planar_position, surface_view_basis};
mod camera;
#[cfg(test)]
mod collision_tests;
mod doorway;
mod eva;
mod interior;
pub mod persistence;
mod surface;
#[cfg(test)]
mod test_support;
const WALK_SPEED_METERS_PER_SECOND: f64 = 3.8;
const JUMP_SPEED_METERS_PER_SECOND: f64 = 4.4;

// One update samples frames and the jump edge once, then dispatches a single mode.
struct Step {
    delta: Duration,
    seconds: f64,
    input: MovementInput,
    ship: ShipFrame,
    ship_velocity: [f64; 3],
    surface: SurfaceFrame,
    door_open: bool,
    ship_landed: bool,
    doorway_passable: bool,
    jump_started: bool,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterController {
    position: PositionState,
    yaw_radians: f64,
    pitch_radians: f64,
    vertical_speed: f64,
    interior_velocity: [f64; 3],
    #[serde(skip)]
    jump_was_down: bool,
    #[serde(skip, default = "normal_movement_scale")]
    movement_scale: f64,
    eva_inherited_velocity: [f64; 3],
    eva_control_velocity: [f64; 3],
    eva_axes: [[f64; 3]; 3],
    nearby_surface: Option<SurfaceFrame>,
    ramp_moving: bool,
    ship_gravity_enabled: bool,
    #[serde(skip)]
    cargo_door_blocking: bool,
    #[serde(skip)]
    interior_obstacles: Vec<[f64; 6]>,
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
            interior_velocity: [0.0; 3],
            jump_was_down: false,
            movement_scale: 1.0,
            eva_inherited_velocity: [0.0; 3],
            eva_control_velocity: [0.0; 3],
            eva_axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            nearby_surface: None,
            ramp_moving: false,
            ship_gravity_enabled: true,
            cargo_door_blocking: false,
            interior_obstacles: Vec::new(),
        }
    }
}

fn normal_movement_scale() -> f64 {
    1.0
}

impl CharacterController {
    /// Host supplies the independently saved internal door state before stepping.
    pub fn set_cargo_door_blocking(&mut self, blocking: bool) {
        self.cargo_door_blocking = blocking;
    }

    /// Scales voluntary locomotion only, never gravity, elapsed time or inherited drift.
    pub fn set_movement_scale(&mut self, scale: f64) {
        self.movement_scale = if scale.is_finite() {
            scale.clamp(0.0, 1.0)
        } else {
            1.0
        };
    }
    fn walk_speed(&self) -> f64 {
        WALK_SPEED_METERS_PER_SECOND * self.movement_scale
    }

    /// Installed physical fixtures are supplied by composition, never derived from GPU meshes.
    pub fn set_interior_obstacles(&mut self, bounds: Vec<[f64; 6]>) {
        self.interior_obstacles = bounds;
    }
    fn slide_installed(&self, previous: [f64; 3], mut local: [f64; 3]) -> [f64; 3] {
        let bottom = local[1] - PLAYER_EYE_HEIGHT_METERS;
        for b in &self.interior_obstacles {
            if bottom < b[3] && bottom + PLAYER_BODY_HEIGHT_METERS > b[2] {
                local = crate::collision::slide_around_obstacles(
                    previous,
                    local,
                    &[[
                        b[0] - PLAYER_RADIUS_METERS,
                        b[1] + PLAYER_RADIUS_METERS,
                        b[4] - PLAYER_RADIUS_METERS,
                        b[5] + PLAYER_RADIUS_METERS,
                    ]],
                );
            }
        }
        local
    }

    pub fn set_ship_gravity_enabled(&mut self, enabled: bool) {
        if matches!(self.position, PositionState::Inside { .. }) {
            if self.ship_gravity_enabled && !enabled {
                self.interior_velocity[1] = self.vertical_speed;
            } else if !self.ship_gravity_enabled && enabled {
                self.vertical_speed = self.interior_velocity[1];
            }
        }
        self.ship_gravity_enabled = enabled;
    }

    /// Runtime supplies animated-door occupancy policy alongside sampled frames.
    pub fn set_ramp_moving(&mut self, moving: bool) {
        self.ramp_moving = moving;
    }

    #[must_use]
    pub const fn location(&self) -> CharacterLocation {
        match self.position {
            PositionState::Cockpit => CharacterLocation::Cockpit,
            PositionState::Inside { .. } => CharacterLocation::InsideShip,
            PositionState::Doorway { .. } => CharacterLocation::DoorwayBlend,
            PositionState::Surface { .. } => CharacterLocation::Surface,
            PositionState::Space { .. } if self.nearby_surface.is_some() => {
                CharacterLocation::NearbyBody
            }
            PositionState::Space { .. } => CharacterLocation::Space,
        }
    }

    #[must_use]
    pub fn local_ship_position(&self) -> Option<[f64; 3]> {
        match self.position {
            PositionState::Cockpit => Some(COCKPIT_POSITION),
            PositionState::Inside { local } => Some(local),
            PositionState::Doorway { .. }
            | PositionState::Surface { .. }
            | PositionState::Space { .. } => None,
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
        self.advance_with_motion(
            delta,
            input,
            (ship, [0.0; 3]),
            surface,
            door_open,
            ship_landed,
        );
    }

    pub fn advance_with_motion(
        &mut self,
        delta: Duration,
        input: MovementInput,
        ship_motion: (ShipFrame, [f64; 3]),
        surface: SurfaceFrame,
        door_open: bool,
        ship_landed: bool,
    ) {
        let mut remaining = delta;
        while !remaining.is_zero() {
            let tick = remaining.min(Duration::from_millis(100));
            self.advance_tick(tick, input, ship_motion, surface, door_open, ship_landed);
            remaining -= tick;
        }
    }

    fn advance_tick(
        &mut self,
        delta: Duration,
        input: MovementInput,
        ship_motion: (ShipFrame, [f64; 3]),
        surface: SurfaceFrame,
        door_open: bool,
        ship_landed: bool,
    ) {
        let surface = self.nearby_surface.unwrap_or(surface);
        let (ship, ship_velocity) = ship_motion;
        let seconds = delta.as_secs_f64();
        if seconds <= 0.0 {
            return;
        }
        let jump_started = input.jump && !self.jump_was_down;
        self.jump_was_down = input.jump;
        let doorway_passable = door_open && ship_landed;

        let step = Step {
            delta,
            seconds,
            input,
            ship,
            ship_velocity,
            surface,
            door_open,
            ship_landed,
            doorway_passable,
            jump_started,
        };
        match self.position {
            PositionState::Cockpit => {}
            PositionState::Inside { local } => self.advance_interior(local, &step),
            PositionState::Doorway {
                world,
                elapsed,
                leaving_ship,
            } => self.advance_doorway(world, elapsed, leaving_ship, &step),
            PositionState::Space { world } => self.advance_eva(world, &step),
            PositionState::Surface { world } => self.advance_surface(world, &step),
        }
    }
}

#[cfg(test)]
mod timing_tests {
    use super::test_support::{frame, surface};
    use super::*;

    #[test]
    fn long_eva_interval_retains_inherited_travel() {
        let mut whole = CharacterController {
            position: PositionState::Space {
                world: [1000.0, 1000.0, 1000.0],
            },
            eva_inherited_velocity: [10.0, 0.0, 0.0],
            ..CharacterController::default()
        };
        let mut partitioned = whole.clone();
        whole.advance(
            Duration::from_secs(2),
            MovementInput::default(),
            frame(),
            surface(),
            true,
            false,
        );
        for _ in 0..20 {
            partitioned.advance(
                Duration::from_millis(100),
                MovementInput::default(),
                frame(),
                surface(),
                true,
                false,
            );
        }
        let after = whole.snapshot(frame(), surface());
        assert_eq!(
            after.eye_position_meters,
            partitioned.snapshot(frame(), surface()).eye_position_meters
        );
        assert_eq!(after.eye_position_meters[0], 1020.0);
    }
}

#[cfg(test)]
mod locomotion_modifier_tests {
    use super::test_support::{frame, surface};
    use super::*;

    #[test]
    fn movement_multiplier_never_scales_inherited_eva_drift_or_gravity_time() {
        let controller = CharacterController {
            position: PositionState::Space { world: [1000.0; 3] },
            eva_inherited_velocity: [10.0, 0.0, 0.0],
            ..Default::default()
        };
        let mut normal = controller.clone();
        let mut slow = controller;
        slow.set_movement_scale(0.225);
        normal.advance(
            Duration::from_secs(1),
            MovementInput::default(),
            frame(),
            surface(),
            false,
            false,
        );
        slow.advance(
            Duration::from_secs(1),
            MovementInput::default(),
            frame(),
            surface(),
            false,
            false,
        );
        assert_eq!(
            normal.snapshot(frame(), surface()),
            slow.snapshot(frame(), surface())
        );
        assert_eq!(slow.eva_velocity(), Some([10.0, 0.0, 0.0]));
    }

    #[test]
    fn zero_g_acceleration_is_scaled_but_existing_momentum_and_save_source_are_preserved() {
        let mut normal = CharacterController {
            position: PositionState::Inside {
                local: [-2.0, 2.0, 1.0],
            },
            ship_gravity_enabled: false,
            ..Default::default()
        };
        let mut slow = normal.clone();
        slow.set_movement_scale(0.5);
        let input = MovementInput {
            forward: true,
            ..Default::default()
        };
        normal.advance(
            Duration::from_millis(100),
            input,
            frame(),
            surface(),
            false,
            false,
        );
        slow.advance(
            Duration::from_millis(100),
            input,
            frame(),
            surface(),
            false,
            false,
        );
        assert!((slow.interior_velocity[0] / normal.interior_velocity[0] - 0.5).abs() < 1e-12);
        let moving = slow.interior_velocity;
        slow.set_movement_scale(0.1);
        slow.advance(
            Duration::from_millis(100),
            MovementInput::default(),
            frame(),
            surface(),
            false,
            false,
        );
        assert_eq!(slow.interior_velocity, moving);
        let restored = CharacterController::from_save(slow.save_state(), None).unwrap();
        assert_eq!(restored.movement_scale, 1.0);
    }
}
