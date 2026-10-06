//! Character state machine and typed update dispatch.
use crate::collision::*;
use crate::input::{axis, movement_axes, planar_look, ship_planar_movement};
use crate::layout::*;
use crate::math::{lerp, normalize, normalize_or, orthogonal_tangent, reject};
use crate::state::PositionState;
use crate::{
    CharacterLocation, CharacterSnapshot, DOORWAY_GRAVITY_BLEND_DURATION,
    FIXED_GRAVITY_METERS_PER_SECOND_SQUARED, MovementInput, PLAYER_BODY_HEIGHT_METERS,
    PLAYER_EYE_HEIGHT_METERS, ShipFrame, SurfaceFrame,
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
#[derive(Clone, Debug)]
pub struct CharacterController {
    position: PositionState,
    yaw_radians: f64,
    pitch_radians: f64,
    vertical_speed: f64,
    jump_was_down: bool,
    eva_inherited_velocity: [f64; 3],
    eva_control_velocity: [f64; 3],
    eva_axes: [[f64; 3]; 3],
    nearby_surface: Option<SurfaceFrame>,
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
            eva_inherited_velocity: [0.0; 3],
            eva_control_velocity: [0.0; 3],
            eva_axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            nearby_surface: None,
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
        let surface = self.nearby_surface.unwrap_or(surface);
        let (ship, ship_velocity) = ship_motion;
        let seconds = delta.as_secs_f64().min(0.1);
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
