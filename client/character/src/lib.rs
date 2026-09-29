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
pub const PLAYER_BODY_HEIGHT_METERS: f64 = 1.80;
pub const PLAYER_EYE_HEIGHT_METERS: f64 = 1.75;
const PLAYER_RADIUS_METERS: f64 = 0.24;
const SHIP_FLOOR_HEIGHT: f64 = 0.247_311_827_956_989_25;
// Keep the player's head below the lowest ceiling lamps and the aft lintel.
const SHIP_CEILING_HEIGHT: f64 = 2.505 * (4.0 / 3.72);
const DOORWAY_CEILING_HEIGHT: f64 = 2.34 * (4.0 / 3.72);
const INTERIOR_FORWARD_MIN: f64 = -7.24;
const AFT_WALL_FORWARD_MIN: f64 = -7.24 + PLAYER_RADIUS_METERS;
// The walkable deck ends at local X = 6.56 m. Keep the complete player body
// inside the forward hull instead of excluding the whole cockpit.
const INTERIOR_FORWARD_MAX: f64 = 6.56 - PLAYER_RADIUS_METERS;
// Window sills project into the 9.20 m deck to Z = +/-4.20 m.
const INTERIOR_SIDE_LIMIT: f64 = 4.20 - PLAYER_RADIUS_METERS;
const DOORWAY_FORWARD: f64 = -7.04;
const DOORWAY_SIDE_LIMIT: f64 = 1.40 - PLAYER_RADIUS_METERS;
// Body-expanded exterior cabin/nose envelope, independent of door state.
// The aft wall is split around the same clear aperture used inside. Wings and
// engines are excluded so their broad asset bounds cannot obstruct the gate.
const EXTERIOR_AFT: f64 = -7.92 - PLAYER_RADIUS_METERS;
const EXTERIOR_FORWARD: f64 = 10.50 + PLAYER_RADIUS_METERS;
const EXTERIOR_SIDE: f64 = 5.10 + PLAYER_RADIUS_METERS;
const EXTERIOR_BOTTOM: f64 = -0.10 * (4.0 / 3.72);
const EXTERIOR_TOP: f64 = 2.73 * (4.0 / 3.72);
// A door can close while the body overlaps its proxy. Resolve an active
// crossing to the nearer side of the two existing body-clear stopping planes.
const CLOSED_GATE_MIDPOINT: f64 = (EXTERIOR_AFT + INTERIOR_FORWARD_MIN) * 0.5;
const COLLISION_EPSILON: f64 = 1.0e-7;
const EXTERIOR_OBSTACLES: [[f64; 4]; 3] = [
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
const CLOSED_GATE_OBSTACLE: [[f64; 4]; 1] = [[
    EXTERIOR_AFT,
    DOORWAY_FORWARD + PLAYER_RADIUS_METERS,
    -EXTERIOR_SIDE,
    EXTERIOR_SIDE,
]];
// The center Core's two-meter pedestal remains solid, with clear aisles on
// both sides. Inflate its planar footprint by the player's body radius.
const CORE_FORWARD_MIN: f64 = -2.0 - PLAYER_RADIUS_METERS;
const CORE_FORWARD_MAX: f64 = PLAYER_RADIUS_METERS;
const CORE_SIDE_LIMIT: f64 = 1.0 + PLAYER_RADIUS_METERS;
const COCKPIT_CHAIR_OBSTACLE: [f64; 4] = [
    1.77 - PLAYER_RADIUS_METERS,
    3.30 + PLAYER_RADIUS_METERS,
    -0.79 - PLAYER_RADIUS_METERS,
    0.79 + PLAYER_RADIUS_METERS,
];
const COCKPIT_CENTER_CONSOLE_OBSTACLE: [f64; 4] = [
    4.3126 - PLAYER_RADIUS_METERS,
    6.12 + PLAYER_RADIUS_METERS,
    -1.06 - PLAYER_RADIUS_METERS,
    1.06 + PLAYER_RADIUS_METERS,
];
const COCKPIT_PORT_CONSOLE_OBSTACLE: [f64; 4] = [
    3.65 - PLAYER_RADIUS_METERS,
    5.96 + PLAYER_RADIUS_METERS,
    1.70 - PLAYER_RADIUS_METERS,
    2.90 + PLAYER_RADIUS_METERS,
];
const COCKPIT_STARBOARD_CONSOLE_OBSTACLE: [f64; 4] = [
    3.65 - PLAYER_RADIUS_METERS,
    5.96 + PLAYER_RADIUS_METERS,
    -2.90 - PLAYER_RADIUS_METERS,
    -1.70 + PLAYER_RADIUS_METERS,
];
// The solid forward hull closes the deck outside the side consoles. Expanding
// its inner faces by the body radius prevents a walker from bypassing a console
// through the exterior shell while retaining the aisle between it and the chair.
const COCKPIT_PORT_HULL_OBSTACLE: [f64; 4] = [
    2.50 - PLAYER_RADIUS_METERS,
    INTERIOR_FORWARD_MAX,
    2.90 - PLAYER_RADIUS_METERS,
    INTERIOR_SIDE_LIMIT,
];
const COCKPIT_STARBOARD_HULL_OBSTACLE: [f64; 4] = [
    2.50 - PLAYER_RADIUS_METERS,
    INTERIOR_FORWARD_MAX,
    -INTERIOR_SIDE_LIMIT,
    -2.90 + PLAYER_RADIUS_METERS,
];
// [forward minimum, forward maximum, side minimum, side maximum]. These simple
// body-expanded footprints match the Core, cabin furniture, pilot chair, and
// console/monitor assemblies. A single proxy covers each console and its
// attached monitor because their floor-plane footprints overlap.
const INTERIOR_OBSTACLES: [[f64; 4]; 9] = [
    [
        CORE_FORWARD_MIN,
        CORE_FORWARD_MAX,
        -CORE_SIDE_LIMIT,
        CORE_SIDE_LIMIT,
    ],
    [
        -5.70 - PLAYER_RADIUS_METERS,
        -2.24 + PLAYER_RADIUS_METERS,
        3.60 - PLAYER_RADIUS_METERS,
        4.44 + PLAYER_RADIUS_METERS,
    ],
    [
        -5.96 - PLAYER_RADIUS_METERS,
        -3.00 + PLAYER_RADIUS_METERS,
        -4.48 - PLAYER_RADIUS_METERS,
        -3.58 + PLAYER_RADIUS_METERS,
    ],
    COCKPIT_CHAIR_OBSTACLE,
    COCKPIT_CENTER_CONSOLE_OBSTACLE,
    COCKPIT_PORT_CONSOLE_OBSTACLE,
    COCKPIT_STARBOARD_CONSOLE_OBSTACLE,
    COCKPIT_PORT_HULL_OBSTACLE,
    COCKPIT_STARBOARD_HULL_OBSTACLE,
];
const COCKPIT_POSITION: [f64; 3] = [2.76, 1.799_032_258_064_516, 0.0];
const COCKPIT_VIEW_PITCH_RADIANS: f64 = -0.10;
const PLAYER_START: [f64; 3] = [0.50, PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT, -2.20];
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
                local: PLAYER_START,
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
        let doorway_passable = door_open && ship_landed;

        match self.position {
            PositionState::Cockpit => {}
            PositionState::Inside { mut local } => {
                let previous = local;
                let movement = ship_planar_movement(input, self.yaw_radians);
                local[0] += movement[0] * WALK_SPEED_METERS_PER_SECOND * seconds;
                local[2] += movement[1] * WALK_SPEED_METERS_PER_SECOND * seconds;
                if jump_started && local[1] <= PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT + 0.001
                {
                    self.vertical_speed = JUMP_SPEED_METERS_PER_SECOND;
                }
                self.vertical_speed -= FIXED_GRAVITY_METERS_PER_SECOND_SQUARED * seconds;
                local[1] += self.vertical_speed * seconds;
                if local[1] <= PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT {
                    local[1] = PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT;
                    self.vertical_speed = 0.0;
                }
                local[2] = local[2].clamp(-INTERIOR_SIDE_LIMIT, INTERIOR_SIDE_LIMIT);
                local[0] = local[0].min(INTERIOR_FORWARD_MAX);
                local = slide_around_fixtures(previous, local);
                let ceiling = if local[0] < -6.88 && local[2].abs() < 1.84 {
                    DOORWAY_CEILING_HEIGHT
                } else {
                    SHIP_CEILING_HEIGHT
                };
                let maximum_eye = ceiling - (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS);
                if local[1] > maximum_eye {
                    local[1] = maximum_eye;
                    self.vertical_speed = self.vertical_speed.min(0.0);
                }
                let within_doorway = local[2].abs() <= DOORWAY_SIDE_LIMIT;
                if local[0] < DOORWAY_FORWARD && within_doorway && doorway_passable {
                    self.position = PositionState::Doorway {
                        world: ship.local_to_world(local),
                        elapsed: Duration::ZERO,
                        leaving_ship: true,
                    };
                } else {
                    let aft_limit = if within_doorway {
                        INTERIOR_FORWARD_MIN
                    } else {
                        AFT_WALL_FORWARD_MIN
                    };
                    local[0] = local[0].max(aft_limit);
                    self.position = PositionState::Inside { local };
                }
            }
            PositionState::Doorway {
                mut world,
                mut elapsed,
                leaving_ship,
            } => {
                if !doorway_passable {
                    // Recheck before moving: an in-progress gravity blend does
                    // not grant passage through a door that has since closed.
                    let mut local = ship.world_to_local(world);
                    // Preserve doorway body clearance after the world round-trip;
                    // a rounded jamb contact must not become an aft-wall overlap.
                    local[2] = local[2].clamp(-DOORWAY_SIDE_LIMIT, DOORWAY_SIDE_LIMIT);
                    self.vertical_speed = 0.0;
                    self.position = if local[0] < CLOSED_GATE_MIDPOINT {
                        local[0] = local[0].min(EXTERIOR_AFT);
                        PositionState::Surface {
                            world: surface_eye_at_ship_planar_position(local, ship, surface),
                        }
                    } else {
                        local[0] = local[0].max(INTERIOR_FORWARD_MIN);
                        local[1] = local[1].max(PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT);
                        PositionState::Inside { local }
                    };
                    return;
                }
                let movement = ship_planar_movement(input, self.yaw_radians);
                world = add(
                    world,
                    scale(
                        add(
                            scale(ship.axes[0], movement[0]),
                            scale(ship.axes[2], movement[1]),
                        ),
                        WALK_SPEED_METERS_PER_SECOND * seconds,
                    ),
                );
                // Keep body clearance while oblique movement slides along the jamb.
                let mut local = ship.world_to_local(world);
                local[2] = local[2].clamp(-DOORWAY_SIDE_LIMIT, DOORWAY_SIDE_LIMIT);
                world = ship.local_to_world(local);
                elapsed = elapsed.saturating_add(delta);
                // A slow/diagonal entry may finish its gravity blend while still
                // in the gate. Keep walking through it before selecting the cabin
                // state; otherwise the next interior tick treats it as a new exit.
                let reached_destination =
                    leaving_ship || local[0] >= DOORWAY_FORWARD || local[0] < EXTERIOR_AFT;
                if elapsed >= DOORWAY_GRAVITY_BLEND_DURATION && reached_destination {
                    if local[0] < DOORWAY_FORWARD {
                        self.position = PositionState::Surface {
                            world: project_eye_to_surface(world, surface),
                        };
                    } else {
                        local[1] = PLAYER_EYE_HEIGHT_METERS + SHIP_FLOOR_HEIGHT;
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
                let mut previous = ship.world_to_local(world);
                if !doorway_passable
                    && overlaps_exterior_hull(previous)
                    && previous[0] > EXTERIOR_AFT
                    && previous[0] < DOORWAY_FORWARD + PLAYER_RADIUS_METERS
                    && previous[2].abs() <= DOORWAY_SIDE_LIMIT + world_round_trip_tolerance(world)
                {
                    // A stopped exit can finish its blend within the gate.
                    // Closing around that surface walker must first clear the
                    // overlap; swept collision alone only handles new contact.
                    previous[0] = EXTERIOR_AFT;
                    world = surface_eye_at_ship_planar_position(previous, ship, surface);
                    // Keep the exact resolved plane for the sweep. At catalog
                    // coordinates, a world round-trip can decode just inside
                    // it and incorrectly admit the next inward walk step.
                }
                let up = normalize(sub(world, surface.body_center_meters));
                let (forward, right) = surface_view_basis(ship.axes, up, self.yaw_radians);
                let movement = movement_axes(input);
                let tangent = add(scale(forward, movement[0]), scale(right, movement[1]));
                world = add(
                    world,
                    scale(tangent, WALK_SPEED_METERS_PER_SECOND * seconds),
                );
                world = project_eye_to_surface(world, surface);

                let proposed = ship.world_to_local(world);
                let overlaps_hull = overlaps_exterior_hull(proposed);
                let mut local = proposed;
                if overlaps_hull {
                    local = slide_around_obstacles(previous, local, &EXTERIOR_OBSTACLES);
                    if !doorway_passable {
                        local = slide_around_obstacles(previous, local, &CLOSED_GATE_OBSTACLE);
                    }
                    if local[0] != proposed[0] || local[2] != proposed[2] {
                        // Preserve the collision-resolved X/Z while putting the eye
                        // back on the sphere. Radial projection here would shrink
                        // those coordinates and push the player through a wall.
                        world = surface_eye_at_ship_planar_position(local, ship, surface);
                        local = ship.world_to_local(world);
                    }
                }
                let through_gate = overlaps_hull
                    && previous[0] < DOORWAY_FORWARD
                    && local[0] > EXTERIOR_AFT
                    && local[0] <= DOORWAY_FORWARD + COLLISION_EPSILON
                    && previous[2].abs() <= DOORWAY_SIDE_LIMIT + COLLISION_EPSILON
                    && local[2].abs() <= DOORWAY_SIDE_LIMIT + COLLISION_EPSILON
                    && local[1]
                        <= DOORWAY_CEILING_HEIGHT
                            - (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS);
                // Re-entry follows actual inward movement, regardless of which
                // key produces it. Ignore rounding noise from parallel strafing.
                let moving_into_ship = dot(tangent, ship.axes[0]) > 1.0e-6;
                if doorway_passable && through_gate && moving_into_ship {
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
        let base_forward = if matches!(self.position, PositionState::Surface { .. }) {
            surface_view_basis(ship.axes, up, self.yaw_radians).0
        } else {
            let (planar_forward, _) = planar_look(self.yaw_radians);
            normalize(add(
                scale(ship.axes[0], planar_forward[0]),
                scale(ship.axes[2], planar_forward[1]),
            ))
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

fn overlaps_exterior_hull(local: [f64; 3]) -> bool {
    local[1] + (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS) > EXTERIOR_BOTTOM
        && local[1] - PLAYER_EYE_HEIGHT_METERS < EXTERIOR_TOP
}

fn world_round_trip_tolerance(world: [f64; 3]) -> f64 {
    // Conversion to/from a translated, rotated frame incurs world-coordinate
    // rounding. Allow a few ULPs when recognizing an existing jamb contact,
    // without widening the passable aperture or changing the stopping plane.
    let magnitude = world.into_iter().map(f64::abs).fold(0.0, f64::max);
    COLLISION_EPSILON.max(4.0 * f64::EPSILON * magnitude)
}

fn slide_around_fixtures(previous: [f64; 3], proposed: [f64; 3]) -> [f64; 3] {
    slide_around_obstacles(previous, proposed, &INTERIOR_OBSTACLES)
}

fn slide_around_obstacles(
    previous: [f64; 3],
    mut proposed: [f64; 3],
    obstacles: &[[f64; 4]],
) -> [f64; 3] {
    // Resolve one planar axis at a time so diagonal input slides along the
    // fixtures instead of stopping the player or tunneling through a corner.
    for &[forward_min, forward_max, side_min, side_max] in obstacles {
        if previous[2] > side_min && previous[2] < side_max {
            proposed[0] = stop_at_obstacle(previous[0], proposed[0], forward_min, forward_max);
        }
    }
    for &[forward_min, forward_max, side_min, side_max] in obstacles {
        if proposed[0] > forward_min && proposed[0] < forward_max {
            proposed[2] = stop_at_obstacle(previous[2], proposed[2], side_min, side_max);
        }
    }
    proposed
}

fn stop_at_obstacle(previous: f64, proposed: f64, minimum: f64, maximum: f64) -> f64 {
    if previous <= minimum + COLLISION_EPSILON && proposed > minimum {
        minimum
    } else if previous >= maximum - COLLISION_EPSILON && proposed < maximum {
        maximum
    } else {
        proposed
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

fn ship_planar_movement(input: MovementInput, yaw: f64) -> [f64; 2] {
    let movement = movement_axes(input);
    let (forward, right) = planar_look(yaw);
    [
        forward[0] * movement[0] + right[0] * movement[1],
        forward[1] * movement[0] + right[1] * movement[1],
    ]
}

fn surface_view_basis(
    ship_axes: [[f64; 3]; 3],
    up: [f64; 3],
    yaw_radians: f64,
) -> ([f64; 3], [f64; 3]) {
    let (planar_forward, _) = planar_look(yaw_radians);
    let raw_forward = add(
        scale(ship_axes[0], planar_forward[0]),
        scale(ship_axes[2], planar_forward[1]),
    );
    let forward = normalize_or(
        reject(raw_forward, up),
        normalize_or(reject(ship_axes[2], up), orthogonal_tangent(up)),
    );
    let right = normalize(cross(forward, up));
    (forward, right)
}

fn orthogonal_tangent(normal: [f64; 3]) -> [f64; 3] {
    let reference = if normal[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    normalize(cross(normal, reference))
}

fn project_eye_to_surface(world: [f64; 3], surface: SurfaceFrame) -> [f64; 3] {
    add(
        surface.body_center_meters,
        scale(
            normalize(sub(world, surface.body_center_meters)),
            surface.radius_meters + PLAYER_EYE_HEIGHT_METERS,
        ),
    )
}

fn surface_eye_at_ship_planar_position(
    mut local: [f64; 3],
    ship: ShipFrame,
    surface: SurfaceFrame,
) -> [f64; 3] {
    let center = ship.world_to_local(surface.body_center_meters);
    let radius = surface.radius_meters + PLAYER_EYE_HEIGHT_METERS;
    let forward = local[0] - center[0];
    let side = local[2] - center[2];
    let height_squared = radius * radius - forward * forward - side * side;
    if height_squared < 0.0 {
        return project_eye_to_surface(ship.local_to_world(local), surface);
    }
    local[1] = center[1] + height_squared.sqrt().copysign(local[1] - center[1]);
    ship.local_to_world(local)
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
    fn anchors_and_walk_bounds_match_the_wider_ship() {
        assert_eq!(PLAYER_BODY_HEIGHT_METERS, 1.80);
        assert_eq!(PLAYER_EYE_HEIGHT_METERS, 1.75);
        assert_eq!(PLAYER_START, [0.50, 1.997_311_827_956_989_2, -2.20]);
        assert_eq!(COCKPIT_POSITION, [2.76, 1.799_032_258_064_516, 0.0]);
        assert_eq!(SHIP_FLOOR_HEIGHT, 0.247_311_827_956_989_25);
        assert_eq!(INTERIOR_FORWARD_MIN, -7.24);
        assert!((INTERIOR_FORWARD_MAX - 6.32).abs() < 1.0e-12);
        assert!((INTERIOR_SIDE_LIMIT - 3.96).abs() < 1.0e-12);
        assert_eq!(DOORWAY_FORWARD, -7.04);
    }

    fn inside_at(x: f64, z: f64) -> CharacterController {
        CharacterController {
            position: PositionState::Inside {
                local: [x, PLAYER_START[1], z],
            },
            ..CharacterController::default()
        }
    }

    fn walk_steps(controller: &mut CharacterController, input: MovementInput, steps: u32) {
        for _ in 0..steps {
            controller.advance(
                Duration::from_millis(100),
                input,
                frame(),
                surface(),
                false,
                true,
            );
        }
    }

    #[test]
    fn cockpit_exit_returns_to_clear_aisle_without_a_position_snap() {
        let mut controller = CharacterController::default();
        controller.enter_cockpit();
        controller.leave_cockpit();
        assert_eq!(controller.local_ship_position(), Some(PLAYER_START));
        walk_steps(&mut controller, MovementInput::default(), 1);
        assert_eq!(controller.local_ship_position(), Some(PLAYER_START));
    }

    #[test]
    fn center_core_blocks_approaches_from_all_four_sides() {
        for (x, z, input, expected_x, expected_z) in [
            (
                -3.0,
                0.0,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                CORE_FORWARD_MIN,
                0.0,
            ),
            (
                1.2,
                0.0,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
                CORE_FORWARD_MAX,
                0.0,
            ),
            (
                -1.0,
                -2.0,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                -1.0,
                -CORE_SIDE_LIMIT,
            ),
            (
                -1.0,
                2.0,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                -1.0,
                CORE_SIDE_LIMIT,
            ),
        ] {
            let mut controller = inside_at(x, z);
            walk_steps(&mut controller, input, 20);
            let local = controller.local_ship_position().unwrap();
            assert_eq!(local[0], expected_x);
            assert_eq!(local[2], expected_z);
        }
    }

    #[test]
    fn diagonal_movement_slides_along_core_and_can_round_its_corner() {
        let mut controller = inside_at(CORE_FORWARD_MIN, 0.0);
        let input = MovementInput {
            forward: true,
            right: true,
            ..MovementInput::default()
        };
        walk_steps(&mut controller, input, 3);
        let alongside = controller.local_ship_position().unwrap();
        assert_eq!(alongside[0], CORE_FORWARD_MIN);
        assert!(alongside[2] > 0.7);
        walk_steps(&mut controller, input, 8);
        let around_corner = controller.local_ship_position().unwrap();
        assert!(around_corner[0] > CORE_FORWARD_MIN + 1.0);
        assert!(around_corner[2] > CORE_SIDE_LIMIT);
    }

    #[test]
    fn both_cabin_aisles_enter_the_cockpit_until_the_side_consoles() {
        for side in [-2.4, 2.4] {
            let mut controller = inside_at(-6.0, side);
            walk_steps(
                &mut controller,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                30,
            );
            let local = controller.local_ship_position().unwrap();
            assert_eq!(local[0], COCKPIT_PORT_CONSOLE_OBSTACLE[0]);
            assert!(local[0] > 1.30, "the old broad cockpit exclusion is gone");
            assert_eq!(local[2], side);
        }
    }

    #[test]
    fn cockpit_side_routes_pass_the_chair_and_reach_the_monitor_console() {
        for side in [-1.20, 1.20] {
            let mut controller = inside_at(0.50, side);
            walk_steps(
                &mut controller,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                20,
            );

            let local = controller.local_ship_position().unwrap();
            assert_eq!(local[0], COCKPIT_CENTER_CONSOLE_OBSTACLE[0]);
            assert!(local[0] > COCKPIT_CHAIR_OBSTACLE[1]);
            assert_eq!(local[2], side);
        }
    }

    #[test]
    fn cockpit_chair_blocks_walkers_from_every_planar_direction() {
        let [forward_min, forward_max, side_min, side_max] = COCKPIT_CHAIR_OBSTACLE;
        for (x, z, input, expected_x, expected_z) in [
            (
                0.5,
                0.0,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                forward_min,
                0.0,
            ),
            (
                3.8,
                0.0,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
                forward_max,
                0.0,
            ),
            (
                2.5,
                -1.5,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                2.5,
                side_min,
            ),
            (
                2.5,
                1.5,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                2.5,
                side_max,
            ),
        ] {
            let mut controller = inside_at(x, z);
            walk_steps(&mut controller, input, 20);
            let local = controller.local_ship_position().unwrap();
            assert_eq!(local[0], expected_x);
            assert_eq!(local[2], expected_z);
        }
    }

    #[test]
    fn cockpit_console_proxies_cover_the_monitor_bodies() {
        for (obstacle, x, z) in [
            (COCKPIT_CENTER_CONSOLE_OBSTACLE, 3.7, 0.0),
            (COCKPIT_PORT_CONSOLE_OBSTACLE, 3.0, 2.3),
            (COCKPIT_STARBOARD_CONSOLE_OBSTACLE, 3.0, -2.3),
        ] {
            let mut controller = inside_at(x, z);
            walk_steps(
                &mut controller,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                20,
            );
            assert_eq!(controller.local_ship_position().unwrap()[0], obstacle[0]);
        }
    }

    #[test]
    fn reduced_center_monitor_blocks_walking_and_jumping_before_its_front_face() {
        // Asset version 8's complete center assembly begins at X=4.3126 m,
        // slightly in front of the dashboard's pilot-facing edge at X=4.33 m.
        for jump in [false, true] {
            let mut controller = inside_at(3.7, 0.0);
            walk_steps(
                &mut controller,
                MovementInput {
                    forward: true,
                    jump,
                    ..MovementInput::default()
                },
                20,
            );
            let eye = controller.local_ship_position().unwrap();
            assert!((eye[0] + PLAYER_RADIUS_METERS - 4.3126).abs() < 1.0e-9);
        }
    }

    #[test]
    fn cockpit_collision_slides_around_the_chair_and_allows_retreat() {
        let mut controller = inside_at(COCKPIT_CHAIR_OBSTACLE[0], 0.0);
        walk_steps(
            &mut controller,
            MovementInput {
                forward: true,
                right: true,
                ..MovementInput::default()
            },
            8,
        );
        let around_corner = controller.local_ship_position().unwrap();
        assert!(around_corner[0] > COCKPIT_CHAIR_OBSTACLE[0]);
        assert!(around_corner[2] > COCKPIT_CHAIR_OBSTACLE[3]);

        walk_steps(
            &mut controller,
            MovementInput {
                backward: true,
                ..MovementInput::default()
            },
            3,
        );
        assert!(controller.local_ship_position().unwrap()[0] < around_corner[0]);
    }

    #[test]
    fn cockpit_side_hull_blocks_bypassing_the_consoles() {
        for (side, outward, hull, expected_side) in [
            (
                1.0,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                COCKPIT_PORT_HULL_OBSTACLE,
                COCKPIT_PORT_HULL_OBSTACLE[2],
            ),
            (
                -1.0,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                COCKPIT_STARBOARD_HULL_OBSTACLE,
                COCKPIT_STARBOARD_HULL_OBSTACLE[3],
            ),
        ] {
            let mut across_hull = inside_at(2.0, side * 3.2);
            walk_steps(
                &mut across_hull,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                10,
            );
            assert_eq!(across_hull.local_ship_position().unwrap()[0], hull[0]);

            let mut around_console = inside_at(3.0, side * 2.4);
            walk_steps(&mut around_console, outward, 10);
            assert_eq!(
                around_console.local_ship_position().unwrap()[2],
                expected_side
            );
        }
    }

    #[test]
    fn interior_walls_and_forward_hull_keep_the_complete_player_inside() {
        for (start, input, axis, expected) in [
            (
                [1.0, 0.0],
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                2,
                INTERIOR_SIDE_LIMIT,
            ),
            (
                [1.0, 0.0],
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                2,
                -INTERIOR_SIDE_LIMIT,
            ),
            (
                [5.0, 1.4],
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                0,
                INTERIOR_FORWARD_MAX,
            ),
        ] {
            let mut controller = inside_at(start[0], start[1]);
            walk_steps(&mut controller, input, 20);
            assert_eq!(controller.local_ship_position().unwrap()[axis], expected);
        }
    }

    #[test]
    fn cockpit_chair_collision_preserves_instant_seating_and_safe_exit() {
        let mut controller = inside_at(COCKPIT_CHAIR_OBSTACLE[0], 0.0);
        controller.enter_cockpit();
        assert_eq!(controller.location(), CharacterLocation::Cockpit);
        assert_eq!(controller.local_ship_position(), Some(COCKPIT_POSITION));

        controller.leave_cockpit();
        assert_eq!(controller.local_ship_position(), Some(PLAYER_START));
        walk_steps(&mut controller, MovementInput::default(), 1);
        assert_eq!(controller.local_ship_position(), Some(PLAYER_START));
    }

    #[test]
    fn wider_cabin_allows_window_approaches_and_stops_before_window_sills() {
        for (side, input) in [
            (
                -1.0,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
            ),
            (
                1.0,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
            ),
        ] {
            let mut controller = inside_at(-6.5, 0.0);
            walk_steps(&mut controller, input, 20);
            let local = controller.local_ship_position().unwrap();
            assert!((local[2] - side * 3.96).abs() < 1.0e-12);
            assert!(local[2].abs() > 3.9);
        }
    }

    #[test]
    fn port_sofa_and_starboard_worktop_block_walkers_with_body_clearance() {
        for (input, expected_side) in [
            (
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                3.36,
            ),
            (
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                -3.34,
            ),
        ] {
            let mut controller = inside_at(-4.5, 0.0);
            walk_steps(&mut controller, input, 20);
            let local = controller.local_ship_position().unwrap();
            assert!((local[2] - expected_side).abs() < 1.0e-12);
        }
        for (side, expected_forward) in [(3.5, -5.94), (-3.5, -6.20)] {
            let mut controller = inside_at(-6.5, side);
            walk_steps(
                &mut controller,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                20,
            );
            assert!(
                (controller.local_ship_position().unwrap()[0] - expected_forward).abs() < 1.0e-12
            );
        }
    }

    #[test]
    fn jump_keeps_player_head_below_ceiling_lamps_and_aft_lintel() {
        for (x, z, ceiling) in [
            (0.5, -2.2, SHIP_CEILING_HEIGHT),
            (-7.0, 0.0, DOORWAY_CEILING_HEIGHT),
        ] {
            let mut controller = inside_at(x, z);
            let mut highest_eye = PLAYER_START[1];
            let head_above_eye = PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS;
            for _ in 0..100 {
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
                let eye = controller.local_ship_position().unwrap()[1];
                highest_eye = highest_eye.max(eye);
                assert!(eye + head_above_eye <= ceiling + 1.0e-12);
            }
            assert!((highest_eye + head_above_eye - ceiling).abs() < 1.0e-12);
            assert_eq!(
                controller.local_ship_position().unwrap()[1],
                PLAYER_START[1]
            );
        }
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
                local: [-7.0, PLAYER_START[1], 0.0],
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
    fn open_door_does_not_allow_walking_through_rear_windows_or_door_jambs() {
        for side in [-4.0, -1.20, 1.20, 4.0] {
            let mut controller = inside_at(-7.0, side);
            for _ in 0..10 {
                controller.advance(
                    Duration::from_millis(100),
                    MovementInput {
                        backward: true,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    true,
                    true,
                );
            }
            assert_eq!(controller.location(), CharacterLocation::InsideShip);
            assert_eq!(
                controller.local_ship_position().unwrap()[0],
                AFT_WALL_FORWARD_MIN
            );
        }
    }

    #[test]
    fn doorway_aperture_accepts_player_body_clearance_on_both_sides() {
        for side in [-1.15, 0.0, 1.15] {
            let mut controller = inside_at(-7.0, side);
            controller.advance(
                Duration::from_millis(50),
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
                frame(),
                surface(),
                true,
                true,
            );
            assert_eq!(controller.location(), CharacterLocation::DoorwayBlend);
        }
    }

    #[test]
    fn surface_reentry_uses_same_clear_doorway_and_reaches_cabin_floor() {
        for side in [-1.30_f64, -1.15, 1.15, 1.30] {
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(
                        frame().local_to_world([-8.0, PLAYER_START[1], side]),
                        surface(),
                    ),
                },
                ..CharacterController::default()
            };
            let input = MovementInput {
                forward: true,
                ..MovementInput::default()
            };
            controller.advance(
                Duration::from_millis(100),
                input,
                frame(),
                surface(),
                true,
                true,
            );
            if side.abs() > DOORWAY_SIDE_LIMIT {
                assert_eq!(controller.location(), CharacterLocation::Surface);
            } else {
                assert_eq!(controller.location(), CharacterLocation::DoorwayBlend);
                for _ in 0..16 {
                    controller.advance(
                        Duration::from_millis(16),
                        input,
                        frame(),
                        surface(),
                        true,
                        true,
                    );
                }
                assert_eq!(controller.location(), CharacterLocation::InsideShip);
                let local = controller.local_ship_position().unwrap();
                assert!(local[0] > DOORWAY_FORWARD);
                assert!(local[2].abs() < DOORWAY_SIDE_LIMIT);
                assert_eq!(local[1], PLAYER_START[1]);
            }
        }
    }

    #[test]
    fn surface_walkers_cannot_enter_through_the_hull_with_either_door_state() {
        let cases = [
            ([0.0, PLAYER_START[1], -6.0], FRAC_PI_2, 2, -5.34),
            ([0.0, PLAYER_START[1], 6.0], -FRAC_PI_2, 2, 5.34),
            (
                [DOORWAY_FORWARD, PLAYER_START[1], -6.0],
                FRAC_PI_2,
                2,
                -5.34,
            ),
            ([DOORWAY_FORWARD, PLAYER_START[1], 6.0], -FRAC_PI_2, 2, 5.34),
            ([12.0, PLAYER_START[1], 0.0], std::f64::consts::PI, 0, 10.74),
            ([-9.0, PLAYER_START[1], -3.0], 0.0, 0, -8.16),
            ([-9.0, PLAYER_START[1], 3.0], 0.0, 0, -8.16),
            ([-9.0, PLAYER_START[1], -1.17], 0.0, 0, -8.16),
            ([-9.0, PLAYER_START[1], 1.17], 0.0, 0, -8.16),
        ];
        for axes in [
            frame().axes,
            [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
            [[0.6, 0.8, 0.0], [-0.8, 0.6, 0.0], [0.0, 0.0, 1.0]],
        ] {
            let ship = ShipFrame {
                axes,
                origin_meters: [60_000_000.0, 7_000_000.0, -3_000_000.0],
            };
            let surface = SurfaceFrame {
                body_center_meters: sub(ship.origin_meters, scale(ship.axes[1], 6_000_000.0)),
                radius_meters: 6_000_000.0,
            };
            for door_open in [false, true] {
                for (start, yaw_radians, axis, contact) in cases {
                    let mut controller = CharacterController {
                        position: PositionState::Surface {
                            world: project_eye_to_surface(ship.local_to_world(start), surface),
                        },
                        yaw_radians,
                        ..CharacterController::default()
                    };
                    for _ in 0..100 {
                        controller.advance(
                            Duration::from_millis(100),
                            MovementInput {
                                forward: true,
                                ..MovementInput::default()
                            },
                            ship,
                            surface,
                            door_open,
                            true,
                        );
                        assert_eq!(controller.location(), CharacterLocation::Surface);
                    }
                    let snapshot = controller.snapshot(ship, surface);
                    let local = ship.world_to_local(snapshot.eye_position_meters);
                    assert!(
                        (local[axis] - contact).abs() < 1.0e-7,
                        "hull approach {start:?}, door open {door_open}: {local:?}"
                    );
                    let radius = sub(snapshot.eye_position_meters, surface.body_center_meters);
                    assert!(
                        (dot(radius, radius).sqrt()
                            - surface.radius_meters
                            - PLAYER_EYE_HEIGHT_METERS)
                            .abs()
                            < 1.0e-7
                    );
                }
            }
        }
    }

    #[test]
    fn closed_or_unlanded_gate_blocks_surface_entry() {
        for (door_open, landed) in [(false, true), (true, false)] {
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(
                        frame().local_to_world([-9.0, PLAYER_START[1], 0.0]),
                        surface(),
                    ),
                },
                ..CharacterController::default()
            };
            for _ in 0..100 {
                controller.advance(
                    Duration::from_millis(100),
                    MovementInput {
                        forward: true,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    door_open,
                    landed,
                );
                assert_eq!(controller.location(), CharacterLocation::Surface);
            }
            let local =
                frame().world_to_local(controller.snapshot(frame(), surface()).eye_position_meters);
            assert!((local[0] + 8.16).abs() < 1.0e-9);
        }
    }

    #[test]
    fn closing_after_a_short_exit_clears_the_gate_before_surface_movement() {
        let mut controller = inside_at(-7.0, 0.0);
        controller.advance(
            Duration::from_millis(50),
            MovementInput {
                backward: true,
                ..MovementInput::default()
            },
            frame(),
            surface(),
            true,
            true,
        );
        controller.advance(
            DOORWAY_GRAVITY_BLEND_DURATION,
            MovementInput::default(),
            frame(),
            surface(),
            true,
            true,
        );
        assert_eq!(controller.location(), CharacterLocation::Surface);
        let outside = controller.snapshot(frame(), surface());
        assert!(frame().world_to_local(outside.eye_position_meters)[0] > EXTERIOR_AFT);

        for step in 0..30 {
            controller.advance(
                Duration::from_millis(16),
                MovementInput {
                    forward: true,
                    jump: step % 2 == 0,
                    ..MovementInput::default()
                },
                frame(),
                surface(),
                false,
                true,
            );
            let snapshot = controller.snapshot(frame(), surface());
            let local = frame().world_to_local(snapshot.eye_position_meters);
            assert_eq!(snapshot.location, CharacterLocation::Surface);
            assert_eq!(snapshot.doorway_blend_fraction, None);
            assert!(local[0] <= EXTERIOR_AFT + COLLISION_EPSILON, "{local:?}");
            let radial = sub(snapshot.eye_position_meters, surface().body_center_meters);
            assert!(
                (dot(radial, radial).sqrt() - surface().radius_meters - PLAYER_EYE_HEIGHT_METERS)
                    .abs()
                    < 1.0e-9
            );
            assert_eq!(snapshot.up, normalize(radial).map(|axis| axis as f32));
        }
    }

    #[test]
    fn closing_during_an_outside_crossing_cancels_the_gravity_blend() {
        for leaving_ship in [false, true] {
            for (door_open, landed) in [(false, true), (true, false)] {
                let mut controller = CharacterController {
                    position: PositionState::Doorway {
                        world: frame().local_to_world([-8.0, PLAYER_START[1], 0.0]),
                        elapsed: Duration::from_millis(240),
                        leaving_ship,
                    },
                    ..CharacterController::default()
                };
                controller.advance(
                    Duration::from_millis(16),
                    MovementInput {
                        forward: true,
                        jump: true,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    door_open,
                    landed,
                );
                let snapshot = controller.snapshot(frame(), surface());
                assert_eq!(snapshot.location, CharacterLocation::Surface);
                assert_eq!(snapshot.doorway_blend_fraction, None);
                assert!(
                    frame().world_to_local(snapshot.eye_position_meters)[0]
                        <= EXTERIOR_AFT + COLLISION_EPSILON
                );
                controller.enter_cockpit();
                assert_eq!(controller.location(), CharacterLocation::Surface);
            }
        }
    }

    fn rotated_catalog_frames() -> (ShipFrame, SurfaceFrame) {
        let ship = ShipFrame {
            origin_meters: [1.0e12, -750.0e9, 250.0e9],
            axes: [
                [
                    0.904_961_439_349_810_7,
                    0.408_776_116_798_125_6,
                    -0.118_096_907_772_236_55,
                ],
                [
                    0.425_493_587_836_431_4,
                    -0.869_405_870_275_219_2,
                    0.251_174_519_888_204_05,
                ],
                [0.0, -0.277_552_732_046_423_83, -0.960_710_404_301_715_7],
            ],
        };
        let surface = SurfaceFrame {
            body_center_meters: sub(ship.origin_meters, scale(ship.axes[1], 6_000_000.0)),
            radius_meters: 6_000_000.0,
        };
        (ship, surface)
    }

    #[test]
    fn closed_gate_remains_solid_in_a_rotated_frame_at_catalog_coordinates() {
        let (ship, surface) = rotated_catalog_frames();
        let mut controller = CharacterController::default();
        for (steps, milliseconds, input) in [
            (
                18,
                100,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
            ),
            (
                6,
                100,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
            ),
            (
                2,
                100,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
            ),
            (
                10,
                16,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
            ),
            (20, 16, MovementInput::default()),
        ] {
            for _ in 0..steps {
                controller.advance(
                    Duration::from_millis(milliseconds),
                    input,
                    ship,
                    surface,
                    true,
                    true,
                );
            }
        }
        assert_eq!(controller.location(), CharacterLocation::Surface);
        let outside = controller.clone();
        for side in [-DOORWAY_SIDE_LIMIT, 0.08, DOORWAY_SIDE_LIMIT] {
            let mut controller = outside.clone();
            let mut local =
                ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
            local[2] = side;
            controller.position = PositionState::Surface {
                world: surface_eye_at_ship_planar_position(local, ship, surface),
            };
            for step in 0..100 {
                controller.advance(
                    Duration::from_millis(16),
                    MovementInput {
                        forward: step > 0,
                        jump: step % 2 == 0,
                        ..MovementInput::default()
                    },
                    ship,
                    surface,
                    false,
                    true,
                );
                let snapshot = controller.snapshot(ship, surface);
                let local = ship.world_to_local(snapshot.eye_position_meters);
                assert_eq!(snapshot.location, CharacterLocation::Surface);
                assert_eq!(snapshot.doorway_blend_fraction, None);
                // At 1e12 m, a world-coordinate ULP is about 0.12 mm. The gate
                // must remain stable within that precision, never admit a walk step.
                assert!(
                    (local[0] - EXTERIOR_AFT).abs() < 0.001,
                    "step {step}: {local:?}"
                );
            }
        }
    }

    #[test]
    fn closing_at_a_rotated_cabin_jamb_preserves_the_interior_stopping_plane() {
        let (ship, surface) = rotated_catalog_frames();
        for side in [-DOORWAY_SIDE_LIMIT, DOORWAY_SIDE_LIMIT] {
            for leaving_ship in [false, true] {
                let mut controller = CharacterController {
                    position: PositionState::Doorway {
                        world: ship.local_to_world([-7.4, PLAYER_START[1], side]),
                        elapsed: Duration::from_millis(100),
                        leaving_ship,
                    },
                    ..CharacterController::default()
                };
                for _ in 0..30 {
                    controller.advance(
                        Duration::from_millis(16),
                        MovementInput {
                            backward: true,
                            ..MovementInput::default()
                        },
                        ship,
                        surface,
                        false,
                        true,
                    );
                    assert_eq!(controller.location(), CharacterLocation::InsideShip);
                    let local = controller.local_ship_position().unwrap();
                    assert_eq!(local[0], INTERIOR_FORWARD_MIN);
                }
            }
        }
    }

    #[test]
    fn closing_on_the_cabin_side_keeps_the_player_inside_with_body_clearance() {
        for leaving_ship in [false, true] {
            for eye_height in [PLAYER_START[1] - 0.3, PLAYER_START[1] + 0.2] {
                let mut controller = CharacterController {
                    position: PositionState::Doorway {
                        world: frame().local_to_world([-7.4, eye_height, 1.15]),
                        elapsed: Duration::from_millis(100),
                        leaving_ship,
                    },
                    ..CharacterController::default()
                };
                controller.advance(
                    Duration::from_millis(16),
                    MovementInput {
                        backward: true,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    false,
                    true,
                );
                let snapshot = controller.snapshot(frame(), surface());
                assert_eq!(snapshot.location, CharacterLocation::InsideShip);
                assert_eq!(snapshot.doorway_blend_fraction, None);
                let local = snapshot.local_ship_position_meters.unwrap();
                assert!(local[0] >= INTERIOR_FORWARD_MIN);
                assert_eq!(local[1], eye_height.max(PLAYER_START[1]));
                assert_eq!(local[2], 1.15);
            }
        }
    }

    #[test]
    fn closed_gate_blocks_diagonal_and_jump_input_at_both_edges() {
        for side in [-1.0, 1.0] {
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(
                        frame().local_to_world([-9.0, PLAYER_START[1], side * 1.15]),
                        surface(),
                    ),
                },
                ..CharacterController::default()
            };
            for step in 0..100 {
                controller.advance(
                    Duration::from_millis(16),
                    MovementInput {
                        forward: true,
                        right: side > 0.0,
                        left: side < 0.0,
                        jump: step % 2 == 0,
                        ..MovementInput::default()
                    },
                    frame(),
                    surface(),
                    false,
                    true,
                );
                let snapshot = controller.snapshot(frame(), surface());
                let local = frame().world_to_local(snapshot.eye_position_meters);
                assert_eq!(snapshot.location, CharacterLocation::Surface);
                assert_eq!(snapshot.doorway_blend_fraction, None);
                assert!(
                    local[0] <= EXTERIOR_AFT + COLLISION_EPSILON
                        || local[2].abs() >= EXTERIOR_SIDE - COLLISION_EPSILON,
                    "{local:?}"
                );
            }
        }
    }

    #[test]
    fn repeated_door_cycles_restore_entry_and_exit_at_center_and_edges() {
        for side in [-1.15, 0.0, 1.15] {
            let mut controller = inside_at(-7.0, side);
            for _ in 0..3 {
                for _ in 0..6 {
                    controller.advance(
                        Duration::from_millis(100),
                        MovementInput {
                            backward: true,
                            ..MovementInput::default()
                        },
                        frame(),
                        surface(),
                        true,
                        true,
                    );
                }
                assert_eq!(controller.location(), CharacterLocation::Surface);

                for step in 0..40 {
                    controller.advance(
                        Duration::from_millis(16),
                        MovementInput {
                            forward: true,
                            jump: step % 2 == 0,
                            ..MovementInput::default()
                        },
                        frame(),
                        surface(),
                        false,
                        true,
                    );
                    let snapshot = controller.snapshot(frame(), surface());
                    assert_eq!(snapshot.location, CharacterLocation::Surface);
                    assert_eq!(snapshot.doorway_blend_fraction, None);
                    assert!(
                        frame().world_to_local(snapshot.eye_position_meters)[0]
                            <= EXTERIOR_AFT + COLLISION_EPSILON
                    );
                }
                let mut entered_blend = false;
                for _ in 0..40 {
                    controller.advance(
                        Duration::from_millis(16),
                        MovementInput {
                            forward: true,
                            ..MovementInput::default()
                        },
                        frame(),
                        surface(),
                        true,
                        true,
                    );
                    let snapshot = controller.snapshot(frame(), surface());
                    if !entered_blend && snapshot.location == CharacterLocation::DoorwayBlend {
                        assert_eq!(snapshot.doorway_blend_fraction, Some(0.0));
                        entered_blend = true;
                    }
                    if snapshot.location == CharacterLocation::InsideShip {
                        break;
                    }
                }
                assert!(entered_blend);
                assert_eq!(controller.location(), CharacterLocation::InsideShip);
                for _ in 0..10 {
                    controller.advance(
                        Duration::from_millis(16),
                        MovementInput {
                            backward: true,
                            ..MovementInput::default()
                        },
                        frame(),
                        surface(),
                        false,
                        true,
                    );
                    assert_eq!(controller.location(), CharacterLocation::InsideShip);
                }
            }
        }
    }

    #[test]
    fn surface_walkers_can_round_the_hull_and_enter_only_at_the_open_gate() {
        for side in [-1.0, 1.0] {
            let ship = frame();
            let surface = SurfaceFrame {
                body_center_meters: sub(ship.origin_meters, [0.0, 6_000_000.0, 0.0]),
                radius_meters: 6_000_000.0,
            };
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(
                        ship.local_to_world([0.0, PLAYER_START[1], side * 6.0]),
                        surface,
                    ),
                },
                yaw_radians: -side * 3.0 * std::f64::consts::FRAC_PI_4,
                ..CharacterController::default()
            };
            let input = MovementInput {
                forward: true,
                ..MovementInput::default()
            };
            // Push diagonally against the side, slide aft, and round the corner.
            for _ in 0..40 {
                controller.advance(Duration::from_millis(100), input, ship, surface, true, true);
                assert_eq!(controller.location(), CharacterLocation::Surface);
                let local =
                    ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
                assert!(local[0] <= -8.16 + 1.0e-7 || local[2].abs() >= 5.34 - 1.0e-7);
            }
            controller.yaw_radians = -side * FRAC_PI_2;
            for _ in 0..100 {
                let local =
                    ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
                if local[2].abs() < 0.05 {
                    break;
                }
                controller.advance(Duration::from_millis(16), input, ship, surface, true, true);
                assert_eq!(controller.location(), CharacterLocation::Surface);
            }
            controller.yaw_radians = 0.0;
            let mut entered = false;
            for _ in 0..100 {
                controller.advance(Duration::from_millis(16), input, ship, surface, true, true);
                let local =
                    ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
                if controller.location() == CharacterLocation::InsideShip {
                    entered = true;
                    assert!(local[0] >= DOORWAY_FORWARD);
                } else {
                    assert!(
                        !entered,
                        "held entry movement must not start another exit blend"
                    );
                }
            }
            assert!(entered);
        }
    }

    #[test]
    fn open_gate_cannot_capture_surface_walkers_below_the_ship() {
        let ship = ShipFrame {
            origin_meters: add(frame().origin_meters, [0.0, 50.0, 0.0]),
            ..frame()
        };
        let mut controller = CharacterController {
            position: PositionState::Surface {
                world: project_eye_to_surface(
                    frame().local_to_world([-7.5, PLAYER_START[1], 0.0]),
                    surface(),
                ),
            },
            ..CharacterController::default()
        };
        for _ in 0..50 {
            controller.advance(
                Duration::from_millis(100),
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                ship,
                surface(),
                true,
                true,
            );
            assert_eq!(controller.location(), CharacterLocation::Surface);
        }
        let local = ship.world_to_local(controller.snapshot(ship, surface()).eye_position_meters);
        assert!(local[0] > 10.0);
    }

    #[test]
    fn doorway_exit_follows_held_wasd_across_ship_orientations() {
        let inputs = [
            (
                std::f64::consts::PI,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
            ),
            (
                0.0,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
            ),
            (
                FRAC_PI_2,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
            ),
            (
                -FRAC_PI_2,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
            ),
        ];
        for axes in [
            frame().axes,
            [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
        ] {
            let ship = ShipFrame { axes, ..frame() };
            let surface = SurfaceFrame {
                body_center_meters: sub(ship.origin_meters, scale(ship.axes[1], 6_000_000.0)),
                radius_meters: 6_000_000.0,
            };
            for (yaw_radians, input) in inputs {
                let mut controller = CharacterController {
                    yaw_radians,
                    ..inside_at(-7.0, 0.0)
                };
                let mut previous_forward = -7.0;
                let mut crossed_doorway = false;
                for _ in 0..60 {
                    controller.advance(Duration::from_millis(16), input, ship, surface, true, true);
                    let local =
                        ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
                    assert!(
                        local[0] < previous_forward,
                        "held exit input must keep moving aft, including during the blend"
                    );
                    previous_forward = local[0];
                    crossed_doorway |= controller.location() == CharacterLocation::DoorwayBlend;
                }
                assert!(crossed_doorway);
                assert_eq!(controller.location(), CharacterLocation::Surface);
                assert!(previous_forward < -10.0);
            }
        }
    }

    #[test]
    fn doorway_blend_preserves_camera_relative_diagonal_movement() {
        let mut controller = CharacterController {
            position: PositionState::Doorway {
                world: frame().local_to_world([-7.2, PLAYER_START[1], 0.0]),
                elapsed: Duration::ZERO,
                leaving_ship: true,
            },
            yaw_radians: std::f64::consts::PI,
            ..CharacterController::default()
        };
        let before = controller.snapshot(frame(), surface());
        let forward = normalize(sub(before.look_target_meters, before.eye_position_meters));
        let right = cross(forward, before.up.map(f64::from));
        let expected = scale(
            normalize(add(forward, right)),
            WALK_SPEED_METERS_PER_SECOND * 0.016,
        );
        controller.advance(
            Duration::from_millis(16),
            MovementInput {
                forward: true,
                right: true,
                ..MovementInput::default()
            },
            frame(),
            surface(),
            true,
            true,
        );
        let actual = sub(
            controller.snapshot(frame(), surface()).eye_position_meters,
            before.eye_position_meters,
        );
        assert!(dot(sub(actual, expected), sub(actual, expected)).sqrt() < 1.0e-9);
    }

    #[test]
    fn oblique_doorway_crossings_slide_along_both_jambs() {
        let ship = ShipFrame {
            axes: [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
            ..frame()
        };
        let surface = SurfaceFrame {
            body_center_meters: sub(ship.origin_meters, scale(ship.axes[1], 6_000_000.0)),
            radius_meters: 6_000_000.0,
        };
        let input = MovementInput {
            forward: true,
            ..MovementInput::default()
        };
        for side in [-1.0, 1.0] {
            for leaving_ship in [false, true] {
                let position = if leaving_ship {
                    PositionState::Inside {
                        local: [-7.0, PLAYER_START[1], side * 1.10],
                    }
                } else {
                    PositionState::Surface {
                        world: project_eye_to_surface(
                            ship.local_to_world([-7.5, PLAYER_START[1], side * 1.10]),
                            surface,
                        ),
                    }
                };
                let mut controller = CharacterController {
                    position,
                    yaw_radians: side * FRAC_PI_2 * if leaving_ship { 1.5 } else { 0.5 },
                    ..CharacterController::default()
                };
                controller.advance(Duration::from_millis(16), input, ship, surface, true, true);
                assert_eq!(controller.location(), CharacterLocation::DoorwayBlend);

                let mut touched_jamb = false;
                for _ in 0..16 {
                    controller.advance(Duration::from_millis(16), input, ship, surface, true, true);
                    let local =
                        ship.world_to_local(controller.snapshot(ship, surface).eye_position_meters);
                    assert!(
                        local[2].abs() <= DOORWAY_SIDE_LIMIT + 1.0e-9,
                        "the complete player body must remain clear of the jamb during a blend"
                    );
                    touched_jamb |= (local[2].abs() - DOORWAY_SIDE_LIMIT).abs() < 1.0e-9;
                }
                assert!(touched_jamb, "oblique movement must slide along the jamb");
                if leaving_ship {
                    assert_eq!(controller.location(), CharacterLocation::Surface);
                } else {
                    assert_eq!(controller.location(), CharacterLocation::InsideShip);
                    let local = controller.local_ship_position().unwrap();
                    assert!(local[0] > DOORWAY_FORWARD);
                    assert_eq!(local[1], PLAYER_START[1]);
                }
            }
        }
    }

    #[test]
    fn surface_reentry_depends_on_ship_relative_direction_instead_of_key() {
        let cases = [
            (
                0.0,
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
            ),
            (
                std::f64::consts::PI,
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
            ),
            (
                -FRAC_PI_2,
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
            ),
            (
                FRAC_PI_2,
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
            ),
        ];
        for (inward_yaw, input) in cases {
            for entering in [false, true] {
                let mut controller = CharacterController {
                    position: PositionState::Surface {
                        world: project_eye_to_surface(
                            frame().local_to_world([-7.5, PLAYER_START[1], 0.0]),
                            surface(),
                        ),
                    },
                    yaw_radians: inward_yaw + if entering { 0.0 } else { std::f64::consts::PI },
                    ..CharacterController::default()
                };
                controller.advance(
                    Duration::from_millis(16),
                    input,
                    frame(),
                    surface(),
                    true,
                    true,
                );
                assert_eq!(
                    controller.location(),
                    if entering {
                        CharacterLocation::DoorwayBlend
                    } else {
                        CharacterLocation::Surface
                    }
                );
                for _ in 0..16 {
                    controller.advance(
                        Duration::from_millis(16),
                        input,
                        frame(),
                        surface(),
                        true,
                        true,
                    );
                }
                assert_eq!(
                    controller.location(),
                    if entering {
                        CharacterLocation::InsideShip
                    } else {
                        CharacterLocation::Surface
                    }
                );
                if entering {
                    let local = controller.local_ship_position().unwrap();
                    assert!(local[0] > DOORWAY_FORWARD);
                    assert_eq!(local[1], PLAYER_START[1]);
                }
            }
        }
    }

    #[test]
    fn surface_parallel_or_idle_movement_does_not_trigger_reentry() {
        for input in [
            MovementInput::default(),
            MovementInput {
                forward: true,
                ..MovementInput::default()
            },
        ] {
            let mut controller = CharacterController {
                position: PositionState::Surface {
                    world: project_eye_to_surface(
                        frame().local_to_world([-7.5, PLAYER_START[1], 0.0]),
                        surface(),
                    ),
                },
                yaw_radians: FRAC_PI_2,
                ..CharacterController::default()
            };
            controller.advance(
                Duration::from_millis(16),
                input,
                frame(),
                surface(),
                true,
                true,
            );
            assert_eq!(controller.location(), CharacterLocation::Surface);
        }
    }

    #[test]
    fn landed_open_door_starts_exact_quarter_second_gravity_blend() {
        let mut controller = CharacterController {
            position: PositionState::Inside {
                local: [-7.0, PLAYER_START[1], 0.0],
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
                world: [0.0, 11.75, 0.0],
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
        assert!((radius - 11.75).abs() < 1.0e-9);
    }

    #[test]
    fn surface_wasd_follows_camera_tangent_basis_across_bodies_and_orientations() {
        let cases = [
            ("Mercury", 2_400_000.0, [0.0, 1.0, 0.0], 0.0),
            ("Venus", 5_500_000.0, [1.0, 0.0, 0.0], 0.65),
            ("Earth", 6_000_000.0, [0.0, 0.0, 1.0], 1.4),
            ("Mars", 3_500_000.0, normalize([1.0, 2.0, -3.0]), 2.2),
            ("Moon", 1_600_000.0, normalize([-2.0, 1.0, 1.0]), 5.1),
        ];
        let inputs = [
            (
                "forward",
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                [1.0, 0.0],
            ),
            (
                "backward",
                MovementInput {
                    backward: true,
                    ..MovementInput::default()
                },
                [-1.0, 0.0],
            ),
            (
                "right",
                MovementInput {
                    right: true,
                    ..MovementInput::default()
                },
                [0.0, 1.0],
            ),
            (
                "left",
                MovementInput {
                    left: true,
                    ..MovementInput::default()
                },
                [0.0, -1.0],
            ),
        ];

        for (body, radius_meters, up, yaw_radians) in cases {
            let surface = SurfaceFrame {
                body_center_meters: [125.0, -80.0, 45.0],
                radius_meters,
            };
            let start = add(
                surface.body_center_meters,
                scale(up, radius_meters + PLAYER_EYE_HEIGHT_METERS),
            );

            for (direction, input, expected_axes) in inputs {
                let mut controller = CharacterController {
                    position: PositionState::Surface { world: start },
                    yaw_radians,
                    ..CharacterController::default()
                };
                let before = controller.snapshot(frame(), surface);
                let camera_forward = normalize(reject(
                    sub(before.look_target_meters, before.eye_position_meters),
                    up,
                ));
                let camera_right = normalize(cross(camera_forward, up));
                let expected = add(
                    scale(camera_forward, expected_axes[0]),
                    scale(camera_right, expected_axes[1]),
                );

                controller.advance(
                    Duration::from_millis(10),
                    input,
                    frame(),
                    surface,
                    false,
                    true,
                );

                let after = controller.snapshot(frame(), surface);
                let traveled = normalize(reject(
                    sub(after.eye_position_meters, before.eye_position_meters),
                    up,
                ));
                assert!(
                    dot(traveled, expected) > 0.999_999,
                    "{body} {direction} movement must follow the camera tangent basis"
                );
                let altitude = dot(
                    sub(after.eye_position_meters, surface.body_center_meters),
                    sub(after.eye_position_meters, surface.body_center_meters),
                )
                .sqrt();
                assert!(
                    (altitude - radius_meters - PLAYER_EYE_HEIGHT_METERS).abs() < 1.0e-8,
                    "{body} movement must preserve surface eye height"
                );
            }
        }
    }

    #[test]
    fn surface_forward_direction_tracks_camera_yaw_changes() {
        let surface = SurfaceFrame {
            body_center_meters: [0.0, 0.0, 0.0],
            radius_meters: 100.0,
        };
        let start = [0.0, 101.75, 0.0];
        let mut before_turn = CharacterController {
            position: PositionState::Surface { world: start },
            ..CharacterController::default()
        };
        let mut after_turn = before_turn.clone();
        after_turn.apply_mouse_delta(400.0, 0.0);

        for controller in [&mut before_turn, &mut after_turn] {
            let before = controller.snapshot(frame(), surface);
            let expected = normalize(reject(
                sub(before.look_target_meters, before.eye_position_meters),
                [0.0, 1.0, 0.0],
            ));
            controller.advance(
                Duration::from_millis(10),
                MovementInput {
                    forward: true,
                    ..MovementInput::default()
                },
                frame(),
                surface,
                false,
                true,
            );
            let after = controller.snapshot(frame(), surface);
            let traveled = normalize(reject(
                sub(after.eye_position_meters, before.eye_position_meters),
                [0.0, 1.0, 0.0],
            ));
            assert!(dot(traveled, expected) > 0.999_999);
        }

        let first = before_turn.snapshot(frame(), surface).eye_position_meters;
        let turned = after_turn.snapshot(frame(), surface).eye_position_meters;
        assert!(
            sub(first, start)[2].abs() < 1.0e-8,
            "zero yaw should move along ship forward"
        );
        assert!(
            sub(turned, start)[2] > 0.0,
            "positive yaw should rotate surface-forward movement toward screen right"
        );
    }
}
