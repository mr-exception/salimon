//! Swept, body-clear walking on completed modular decks and stair treads.
use super::*;
impl CharacterController {
    /// The original hull path is used everywhere except completed aft rooms and
    /// their existing gate connection. Closed gate admission remains authoritative.
    pub(super) fn advance_modular(&mut self, previous: [f64; 3], step: &Step) -> bool {
        if self.modular.volumes.is_empty() || previous[0] > -6.8 {
            return false;
        }
        let near_port = previous[0] > -8.5;
        if !self.modular.contains(previous) && !near_port {
            return false;
        }
        if !self.ship_gravity_enabled {
            self.advance_modular_weightless(previous, step);
            return true;
        }
        let foot = previous[1] - PLAYER_EYE_HEIGHT_METERS;
        let standing = self
            .modular
            .support(previous, PLAYER_RADIUS_METERS, foot + 0.001)
            .or_else(|| (previous[0] >= -7.24).then_some(crate::SHIP_FLOOR_HEIGHT_METERS))
            .is_some_and(|h| (h - foot).abs() < 0.002);
        if step.jump_started && standing {
            self.vertical_speed = JUMP_SPEED_METERS_PER_SECOND;
        }
        let movement = ship_planar_movement(step.input, self.yaw_radians);
        let distance = self.walk_speed() * step.seconds;
        let proposed = [
            previous[0] + movement[0] * distance,
            previous[1],
            previous[2] + movement[1] * distance,
        ];
        // Subdivide independently of tick size so a fast tick cannot tunnel
        // through a thin partition or skip a stair riser.
        let count = ((distance / 0.04).ceil() as usize).max(1);
        let mut local = previous;
        if !step.door_open && local[0] > -8.16 && local[0] < -7.24 {
            local[0] = if local[0] > -7.7 { -7.24 } else { -8.16 };
        }
        for _ in 0..count {
            for axis in [0, 2] {
                let mut next = local;
                next[axis] += (proposed[axis] - previous[axis]) / count as f64;
                if !step.door_open
                    && ((local[0] >= -7.24 && next[0] < -7.24)
                        || (local[0] <= -8.16 && next[0] > -8.16))
                {
                    continue;
                }
                if next[0] > -6.8 {
                    local = next;
                    continue;
                }
                if let Some([tip, _, _, _]) = self.modular.exit
                    && next[0] < tip - PLAYER_RADIUS_METERS
                    && local[0] >= tip - PLAYER_RADIUS_METERS
                    && next[2].abs() < 0.46
                {
                    self.vertical_speed = 0.;
                    self.interior_velocity = [0.; 3];
                    let world = step.ship.local_to_world(next);
                    if step.ship_landed {
                        self.position = PositionState::Surface {
                            world: step.surface.eye_on_ground(world),
                        };
                    } else {
                        self.eva_inherited_velocity = step.ship_velocity;
                        self.eva_control_velocity = [0.; 3];
                        self.eva_axes = step.ship.axes;
                        self.nearby_surface = None;
                        self.position = PositionState::Space { world };
                    }
                    return true;
                }
                if !self.modular.contains(next) {
                    continue;
                }
                let foot = local[1] - PLAYER_EYE_HEIGHT_METERS;
                if self.vertical_speed == 0. {
                    if let Some(floor) = self
                        .modular
                        .support(next, PLAYER_RADIUS_METERS, foot + 0.205)
                        .or_else(|| {
                            (next[0] >= -7.24 && next[2].abs() < 1.2)
                                .then_some(crate::SHIP_FLOOR_HEIGHT_METERS)
                        })
                    {
                        // Steps only lift from an actually supported pose. They
                        // do not teleport a falling body onto a higher deck.
                        if floor < foot - 0.205 {
                            continue;
                        }
                        next[1] = floor + PLAYER_EYE_HEIGHT_METERS;
                    } else {
                        continue;
                    }
                }
                if self.modular_body_clear(next) {
                    local = next;
                }
            }
        }
        if self.vertical_speed != 0. || !standing {
            self.vertical_speed -= FIXED_GRAVITY_METERS_PER_SECOND_SQUARED * step.seconds;
            let travel = self.vertical_speed * step.seconds;
            let ticks = ((travel.abs() / 0.03).ceil() as usize).max(1);
            for _ in 0..ticks {
                let mut next = local;
                next[1] += travel / ticks as f64;
                let foot = local[1] - PLAYER_EYE_HEIGHT_METERS;
                if self.vertical_speed < 0.
                    && let Some(floor) =
                        self.modular
                            .support(next, PLAYER_RADIUS_METERS, foot + 0.001)
                    && next[1] - PLAYER_EYE_HEIGHT_METERS <= floor
                {
                    next[1] = floor + PLAYER_EYE_HEIGHT_METERS;
                    self.vertical_speed = 0.;
                }
                if !self.modular_body_clear(next) {
                    self.vertical_speed = 0.;
                    break;
                }
                local = next;
            }
        }
        self.interior_velocity = std::array::from_fn(|i| (local[i] - previous[i]) / step.seconds);
        self.position = PositionState::Inside { local };
        true
    }
    fn advance_modular_weightless(&mut self, previous: [f64; 3], step: &Step) {
        let forward = [
            self.pitch_radians.cos() * self.yaw_radians.cos(),
            self.pitch_radians.sin(),
            self.pitch_radians.cos() * self.yaw_radians.sin(),
        ];
        let right = [-self.yaw_radians.sin(), 0., self.yaw_radians.cos()];
        let movement = add(
            add(
                scale(forward, axis(step.input.forward, step.input.backward)),
                scale(right, axis(step.input.right, step.input.left)),
            ),
            [0., axis(step.input.jump, step.input.descend), 0.],
        );
        let acceleration = scale(
            movement,
            self.walk_speed() / dot(movement, movement).sqrt().max(1.),
        );
        let before = self.interior_velocity;
        let mut velocity = add(before, scale(acceleration, step.seconds));
        let limit = dot(before, before).sqrt().max(self.walk_speed());
        velocity = scale(velocity, limit / dot(velocity, velocity).sqrt().max(limit));
        let travel = scale(add(before, velocity), step.seconds * 0.5);
        let count = ((dot(travel, travel).sqrt() / 0.03).ceil() as usize).max(1);
        let mut local = previous;
        if !step.door_open && local[0] > -8.16 && local[0] < -7.24 {
            local[0] = if local[0] > -7.7 { -7.24 } else { -8.16 };
        }
        for _ in 0..count {
            for a in 0..3 {
                let mut next = local;
                next[a] += travel[a] / count as f64;
                let gate = !step.door_open
                    && ((local[0] >= -7.24 && next[0] < -7.24)
                        || (local[0] <= -8.16 && next[0] > -8.16));
                if !gate && self.modular_body_clear(next) {
                    local = next;
                } else {
                    velocity[a] = 0.;
                }
            }
        }
        self.interior_velocity = velocity;
        self.vertical_speed = velocity[1];
        self.position = PositionState::Inside { local };
    }
    fn modular_body_clear(&self, eye: [f64; 3]) -> bool {
        let bottom = eye[1] - PLAYER_EYE_HEIGHT_METERS;
        let top = bottom + PLAYER_BODY_HEIGHT_METERS;
        let r = PLAYER_RADIUS_METERS;
        // Adjacent cells form one volume; keep every body corner inside it.
        if ![-r, r].into_iter().all(|x| {
            [-r, r].into_iter().all(|z| {
                [bottom + 0.001, top - 0.001].into_iter().all(|y| {
                    self.modular.contains([eye[0] + x, y, eye[2] + z])
                        || (eye[0] + x >= -7.24
                            && (eye[2] + z).abs() < 1.2
                            && (crate::SHIP_FLOOR_HEIGHT_METERS..=DOORWAY_CEILING_HEIGHT)
                                .contains(&y))
                })
            })
        }) {
            return false;
        }
        self.interior_obstacles.iter().all(|b| {
            !(eye[0] + r > b[0] + 1e-7
                && eye[0] - r < b[1] - 1e-7
                && eye[2] + r > b[4] + 1e-7
                && eye[2] - r < b[5] - 1e-7
                && bottom < b[3] - 1e-7
                && top > b[2] + 1e-7)
        })
    }
}

impl CharacterController {
    /// Conservative body sweep against completed panels in ship-local axes.
    /// Exterior surface/EVA paths retain their own gravity and never acquire
    /// interior state merely by touching a new panel.
    pub(super) fn sweep_modular_exterior(
        &self,
        previous: [f64; 3],
        proposed: [f64; 3],
    ) -> [f64; 3] {
        let mut out = previous;
        let r = PLAYER_RADIUS_METERS;
        for axis in [0, 2, 1] {
            let mut value = proposed[axis];
            for b in &self.modular.solids {
                let lo = [
                    b[0] - r,
                    b[2] - (PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS),
                    b[4] - r,
                ];
                let hi = [b[1] + r, b[3] + PLAYER_EYE_HEIGHT_METERS, b[5] + r];
                if (0..3)
                    .filter(|a| *a != axis)
                    .all(|a| out[a] > lo[a] + 1e-7 && out[a] < hi[a] - 1e-7)
                {
                    if previous[axis] <= lo[axis] + 1e-7 && value > lo[axis] {
                        value = value.min(lo[axis]);
                    }
                    if previous[axis] >= hi[axis] - 1e-7 && value < hi[axis] {
                        value = value.max(hi[axis]);
                    }
                }
            }
            out[axis] = value;
        }
        out
    }
}

impl CharacterController {
    pub(super) fn enter_modular_stairs(&mut self, previous: [f64; 3], local: [f64; 3]) -> bool {
        let Some([tip, _, bottom, _]) = self.modular.exit else {
            return false;
        };
        let plane = tip - PLAYER_RADIUS_METERS;
        if previous[0] <= plane + 0.001
            && local[0] >= plane - 1e-7
            && local[0] > previous[0] + 1e-7
            && local[2].abs() <= 0.46
            && (local[1] - PLAYER_EYE_HEIGHT_METERS - bottom).abs() < 0.3
        {
            let mut next = local;
            next[0] = next[0].max(plane + 0.002);
            let Some(floor) = self
                .modular
                .support(next, PLAYER_RADIUS_METERS, bottom + 0.3)
            else {
                return false;
            };
            next[1] = floor + PLAYER_EYE_HEIGHT_METERS;
            if self.modular_body_clear(next) {
                self.vertical_speed = 0.;
                self.interior_velocity = [0.; 3];
                self.position = PositionState::Inside { local: next };
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exterior_sweep_blocks_vertical_and_planar_tunneling_without_granting_entry() {
        let mut c = CharacterController::default();
        c.set_modular_interior(crate::modular::ModularInterior {
            volumes: vec![],
            solids: vec![[-12., -9., 3., 3.1, -1.5, 1.5]],
            aft_port: false,
            exit: None,
        });
        let above = c.sweep_modular_exterior([-10., 7., 0.], [-10., 0., 0.]);
        assert!((above[1] - 3.1 - PLAYER_EYE_HEIGHT_METERS).abs() < 1e-12);
        let below = c.sweep_modular_exterior([-10., 2., 0.], [-10., 5., 0.]);
        assert!(
            (below[1] - 3. + PLAYER_BODY_HEIGHT_METERS - PLAYER_EYE_HEIGHT_METERS).abs() < 1e-12
        );
        let side = c.sweep_modular_exterior([-14., 3.5, 0.], [-8., 3.5, 0.]);
        assert_eq!(side[0], -12. - PLAYER_RADIUS_METERS);
        let clear = c.sweep_modular_exterior([-14., 1., 0.], [-8., 1., 0.]);
        assert_eq!(clear, [-8., 1., 0.]);
    }
}
