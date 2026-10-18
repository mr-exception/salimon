//! Render-only history. Authority and input always read the live session.
use super::{GameplaySession, Snapshot, frames::character_ship_frame};
use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
use salimon_math::{add, dot, sub};
use salimon_ship::ShipPose;

#[derive(Default)]
pub(crate) struct History {
    previous: Option<Snapshot>,
    completed: Option<Snapshot>,
}

pub(crate) struct Presentation {
    current: Snapshot,
    pub(crate) ship: salimon_ship::ShipSnapshot,
    pub(crate) character: CharacterSnapshot,
    pub(crate) alpha: f64,
}

impl History {
    pub(crate) fn begin(&mut self, snapshot: Snapshot) {
        self.previous = Some(snapshot);
        self.completed = None;
    }

    pub(crate) fn complete(&mut self, snapshot: Snapshot) -> bool {
        let continuous = self.previous.as_ref().is_some_and(|previous| {
            previous.character.location == snapshot.character.location
                && snapshot.character.location != CharacterLocation::DoorwayBlend
                && previous.ship.flight_state == snapshot.ship.flight_state
        });
        if continuous {
            self.completed = Some(snapshot);
        } else {
            *self = Self::default();
        }
        continuous
    }

    fn sample(&self, current: Snapshot, alpha: f64) -> Presentation {
        let mut result = Presentation {
            current,
            ship: current.ship,
            character: current.character,
            alpha: 1.0,
        };
        let (Some(previous), Some(completed)) = (&self.previous, &self.completed) else {
            return result;
        };
        // Out-of-tick teleports/fixture changes must never consume stale history.
        // Look targets intentionally differ: mouse look is never interpolated.
        if completed.ship.pose != current.ship.pose
            || completed.ship.flight_state != current.ship.flight_state
            || completed.character.location != current.character.location
            || completed.character.eye_position_meters != current.character.eye_position_meters
            || completed.character.local_ship_position_meters
                != current.character.local_ship_position_meters
            || alpha >= 1.0
        {
            return result;
        }
        result.alpha = alpha.clamp(0.0, 1.0);
        result.ship.pose = ShipPose {
            position_meters: lerp(
                previous.ship.pose.position_meters,
                current.ship.pose.position_meters,
                result.alpha,
            ),
            orientation: orientation(
                previous.ship.pose.orientation,
                current.ship.pose.orientation,
                result.alpha,
            ),
        };
        result.ship.door_open_fraction = previous.ship.door_open_fraction
            + (current.ship.door_open_fraction - previous.ship.door_open_fraction) * result.alpha;
        let direction = sub(
            current.character.look_target_meters,
            current.character.eye_position_meters,
        );
        let (eye, direction, up) = if matches!(
            current.character.location,
            CharacterLocation::Cockpit | CharacterLocation::InsideShip
        ) {
            let old = character_ship_frame(previous.ship.pose);
            let live = character_ship_frame(current.ship.pose);
            let render = character_ship_frame(result.ship.pose);
            let local_eye = lerp(
                old.world_to_local(previous.character.eye_position_meters),
                live.world_to_local(current.character.eye_position_meters),
                result.alpha,
            );
            result.character.local_ship_position_meters = current
                .character
                .local_ship_position_meters
                .map(|_| local_eye);
            (
                render.local_to_world(local_eye),
                rotate_vector(direction, live, render),
                rotate_vector(current.character.up.map(f64::from), live, render),
            )
        } else {
            (
                lerp(
                    previous.character.eye_position_meters,
                    current.character.eye_position_meters,
                    result.alpha,
                ),
                direction,
                current.character.up.map(f64::from),
            )
        };
        result.character.eye_position_meters = eye;
        result.character.look_target_meters = add(eye, direction);
        result.character.up = up.map(|v| v as f32);
        result
    }
}

pub(crate) fn lerp<const N: usize>(a: [f64; N], b: [f64; N], alpha: f64) -> [f64; N] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * alpha)
}

// Same shortest-sign normalized linear quaternion policy as loose fragments.
fn orientation(a: [f64; 4], mut b: [f64; 4], alpha: f64) -> [f64; 4] {
    if a.iter().zip(b).map(|(a, b)| a * b).sum::<f64>() < 0.0 {
        b = b.map(|v| -v);
    }
    let q = lerp(a, b, alpha);
    let norm = q.iter().map(|v| v * v).sum::<f64>().sqrt();
    q.map(|v| v / norm)
}

fn rotate_vector(vector: [f64; 3], from: ShipFrame, to: ShipFrame) -> [f64; 3] {
    let local = from.axes.map(|axis| dot(vector, axis));
    std::array::from_fn(|i| (0..3).map(|j| to.axes[j][i] * local[j]).sum())
}

impl GameplaySession {
    pub(crate) fn reset_presentation(&mut self) {
        self.presentation = History::default();
        self.mining.fragment_previous.clear();
    }

    pub(crate) fn presentation(&self) -> Presentation {
        let current = self.snapshot();
        if self.interpolate_presentation {
            self.presentation
                .sample(current, self.update_clock.interpolation())
        } else {
            Presentation {
                current,
                ship: current.ship,
                character: current.character,
                alpha: 1.0,
            }
        }
    }
}

impl Presentation {
    pub(crate) fn fragment_mesh(
        &self,
        session: &GameplaySession,
        fragment: salimon_world::resources::ResourceFragment,
    ) -> salimon_renderer::ResourceMeshInstance {
        let live = self.current;
        let frame = character_ship_frame(live.ship.pose);
        let render = character_ship_frame(self.ship.pose);
        let mut mesh = crate::fragment_physics::interpolated_mesh(
            &session.mining,
            fragment,
            frame,
            self.alpha,
        );
        if self.alpha >= 1.0 {
            return mesh;
        }
        if session.mining.session.carried_id() == Some(fragment.id()) {
            let offset = sub(mesh.center_meters, live.character.eye_position_meters);
            let offset = if matches!(
                live.character.location,
                CharacterLocation::InsideShip | CharacterLocation::Cockpit
            ) {
                rotate_vector(offset, frame, render)
            } else {
                offset
            };
            mesh.center_meters = add(self.character.eye_position_meters, offset);
        } else if session.mining.ship_fragments.contains_key(&fragment.id()) {
            mesh.center_meters = render.local_to_world(frame.world_to_local(mesh.center_meters));
        } else {
            return mesh;
        }
        if matches!(
            live.character.location,
            CharacterLocation::InsideShip | CharacterLocation::Cockpit
        ) || session.mining.ship_fragments.contains_key(&fragment.id())
        {
            mesh.orientation_xyzw = crate::fragment_physics::to_world_orientation(
                render,
                crate::fragment_physics::to_local_orientation(frame, mesh.orientation_xyzw),
            );
        }
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Command, ControlKey};
    use crate::update_clock::GAME_TICK;
    use salimon_ship::ShipController;
    use std::time::Duration;

    fn flying() -> GameplaySession {
        let mut session = GameplaySession {
            ship: ShipController::flying_with_velocity(
                ShipPose {
                    position_meters: [1e12, 2e12, -3e12],
                    orientation: [0.0, 0.0, 0.0, 1.0],
                },
                0,
                [40.0, 0.0, 0.0],
            )
            .unwrap(),
            ..Default::default()
        };
        session.character.enter_cockpit();
        session.ship.set_cockpit_control(true);
        session
    }

    fn close(a: [f64; 3], b: [f64; 3], tolerance: f64) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() <= tolerance, "{a:?} != {b:?}");
        }
    }

    #[test]
    fn synthetic_60_120_144_hz_rendering_never_advances_authority() {
        for hz in [60, 120, 144] {
            let mut session = flying();
            let origin = session.ship.snapshot().pose.position_meters;
            let interval = Duration::from_nanos(1_000_000_000 / hz);
            let mut admitted = Duration::ZERO;
            let mut zero_ticks = 0;
            let mut multi_ticks = 0;
            for _ in 0..hz * 2 {
                let before_time = session.elapsed;
                let before_position = session.ship.snapshot().pose.position_meters;
                session.advance_automation(interval);
                admitted += interval;
                let ticks = (session.elapsed - before_time).as_millis() / 16;
                zero_ticks += usize::from(ticks == 0);
                multi_ticks += usize::from(ticks > 1);
                let authority = session.snapshot();
                let shown = session.presentation();
                let expected_time = if session.elapsed.is_zero() {
                    0.0
                } else {
                    session.elapsed.saturating_sub(GAME_TICK).as_secs_f64()
                        + authority.backlog.as_secs_f64()
                };
                close(
                    shown.ship.pose.position_meters,
                    add(origin, [40.0 * expected_time, 0.0, 0.0]),
                    0.02,
                );
                assert_eq!(shown.ship, session.presentation().ship);
                assert_eq!(session.snapshot().ship, authority.ship);
                assert_eq!(session.snapshot().character, authority.character);
                assert_eq!(session.elapsed + session.update_clock.backlog(), admitted);
                if ticks == 0 {
                    assert_eq!(before_position, authority.ship.pose.position_meters);
                }
                // Camera and ship must share the same moving frame at distant f64 anchors.
                close(
                    character_ship_frame(shown.ship.pose)
                        .world_to_local(shown.character.eye_position_meters),
                    character_ship_frame(authority.ship.pose)
                        .world_to_local(authority.character.eye_position_meters),
                    0.001,
                );
            }
            if hz > 60 {
                assert!(zero_ticks > 0);
            } else {
                assert!(multi_ticks > 0);
            }
        }
    }

    #[test]
    fn overload_fractional_backlog_and_multiple_ticks_use_the_last_pair() {
        let mut session = flying();
        session.update_clock.accumulate(Duration::from_millis(101));
        for _ in 0..2 {
            let tick = session.update_clock.take_tick().unwrap();
            session.advance_game(tick);
        }
        assert_eq!(session.presentation().alpha, 1.0);
        assert_eq!(session.presentation().ship, session.snapshot().ship);
        while let Some(tick) = session.update_clock.take_tick() {
            session.advance_game(tick);
        }
        assert_eq!(session.presentation().alpha, 5.0 / 16.0);
        let before = session.presentation.previous.unwrap();
        let current = session.snapshot();
        close(
            session.presentation().ship.pose.position_meters,
            lerp(
                before.ship.pose.position_meters,
                current.ship.pose.position_meters,
                5.0 / 16.0,
            ),
            0.0,
        );
    }

    #[test]
    fn immediate_look_rotation_telemetry_and_automation_bypass() {
        let mut session = flying();
        session.command(Command::Control(ControlKey::Right, true));
        session.advance_automation(Duration::from_millis(24));
        let before = session.presentation();
        session.command(Command::Look(120.0, -25.0));
        let after = session.presentation();
        assert_eq!(
            before.character.eye_position_meters,
            after.character.eye_position_meters
        );
        assert_ne!(
            before.character.look_target_meters,
            after.character.look_target_meters
        );
        let live = session.snapshot();
        let render_frame = character_ship_frame(after.ship.pose);
        let live_frame = character_ship_frame(live.ship.pose);
        close(
            rotate_vector(
                sub(
                    after.character.look_target_meters,
                    after.character.eye_position_meters,
                ),
                render_frame,
                live_frame,
            ),
            sub(
                live.character.look_target_meters,
                live.character.eye_position_meters,
            ),
            0.001,
        );
        assert_eq!(after.ship.navigation, live.ship.navigation);
        assert_eq!(after.ship.energy_core, live.ship.energy_core);
        assert_ne!(after.ship.pose.orientation, live.ship.pose.orientation);
        session.interpolate_presentation = false;
        assert_eq!(session.presentation().ship, live.ship);
        assert_eq!(session.presentation().character, live.character);
        assert_eq!(session.presentation().alpha, 1.0);
        assert_eq!(
            orientation([0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, -1.0], 0.5),
            [0.0, 0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn lifecycle_teleports_views_and_location_transitions_bypass_all_history() {
        let mut session = flying();
        assert_eq!(session.presentation().alpha, 1.0);
        session.advance_automation(Duration::from_millis(24));
        assert_eq!(session.presentation().alpha, 0.5);
        session.command(Command::Interact);
        assert_eq!(session.presentation().alpha, 1.0);
        session.advance_automation(Duration::from_millis(16));
        session.reset_presentation();
        assert_eq!(session.presentation().alpha, 1.0);
        session.advance_automation(Duration::from_millis(16));
        session.command(Command::ToggleView);
        assert_eq!(session.presentation().alpha, 1.0);
        session.command(Command::ToggleView);
        session.advance_automation(Duration::from_millis(16));
        session.ship = ShipController::flying(
            ShipPose {
                position_meters: [-1e12, 0.0, 0.0],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        assert_eq!(session.presentation().ship, session.snapshot().ship);
        assert_eq!(session.presentation().alpha, 1.0);
        for location in [
            CharacterLocation::InsideShip,
            CharacterLocation::DoorwayBlend,
            CharacterLocation::Surface,
            CharacterLocation::Space,
            CharacterLocation::NearbyBody,
            CharacterLocation::Cockpit,
        ] {
            let start = session.snapshot();
            let mut end = start;
            end.character.location = location;
            end.character.eye_position_meters = [10.0; 3];
            session.presentation.begin(start);
            let continuous = session.presentation.complete(end);
            if location != start.character.location || location == CharacterLocation::DoorwayBlend {
                assert!(!continuous);
                assert_eq!(
                    session.presentation.sample(end, 0.25).character,
                    end.character
                );
            }
        }
    }
}

#[cfg(test)]
mod frame_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn surface_motion_and_ship_local_walking_use_distinct_frames_without_smear() {
        let session = GameplaySession::default();
        for location in [
            CharacterLocation::Surface,
            CharacterLocation::Space,
            CharacterLocation::NearbyBody,
            CharacterLocation::InsideShip,
        ] {
            let mut start = session.snapshot();
            start.character.location = location;
            start.ship.pose.position_meters = [1e12, 0.0, 0.0];
            start.ship.pose.orientation = [0.0, 0.0, 0.0, 1.0];
            start.character.eye_position_meters = [1e12 + 1.0, 2.0, 0.0];
            let mut end = start;
            end.ship.pose.position_meters[0] += 20.0;
            end.ship.pose.orientation = [0.0, 1.0, 0.0, 0.0];
            end.character.eye_position_meters = if location == CharacterLocation::InsideShip {
                character_ship_frame(end.ship.pose).local_to_world([3.0, 2.0, 0.0])
            } else {
                [1e12 + 5.0, 2.0, 0.0]
            };
            end.character.look_target_meters =
                add(end.character.eye_position_meters, [0.0, 0.0, 5.0]);
            let mut history = History::default();
            history.begin(start);
            assert!(history.complete(end));
            let shown = history.sample(end, 0.5);
            let expected = if location == CharacterLocation::InsideShip {
                character_ship_frame(shown.ship.pose).local_to_world([2.0, 2.0, 0.0])
            } else {
                [1e12 + 3.0, 2.0, 0.0]
            };
            assert_eq!(shown.character.eye_position_meters, expected);
            assert!(
                (shown
                    .ship
                    .pose
                    .orientation
                    .iter()
                    .map(|v| v * v)
                    .sum::<f64>()
                    - 1.0)
                    .abs()
                    < 1e-12
            );
            let mut teleported = end;
            teleported.character.eye_position_meters[0] += 100.0;
            assert_eq!(history.sample(teleported, 0.0).alpha, 1.0);
        }
    }

    #[test]
    fn local_fragments_new_objects_and_carried_items_share_the_render_frame() {
        let mut session = GameplaySession::default();
        session.character.enter_cockpit();
        session.ship = salimon_ship::ShipController::flying_with_velocity(
            ShipPose {
                position_meters: [1e12, 2e12, -3e12],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
            [40.0, 0.0, 0.0],
        )
        .unwrap();
        crate::flight_regression::initialize_cargo(&mut session, 1).unwrap();
        session.advance_automation(Duration::from_millis(24));
        let piece = session.mining.session.fragments()[0];
        let id = piece.id();
        let shown = session.presentation();
        let frame = character_ship_frame(shown.ship.pose);
        let current_frame = character_ship_frame(session.snapshot().ship.pose);
        let interpolated =
            crate::fragment_physics::interpolated_mesh(&session.mining, piece, current_frame, 0.5);
        assert_eq!(
            shown.fragment_mesh(&session, piece).center_meters,
            frame.local_to_world(current_frame.world_to_local(interpolated.center_meters))
        );
        // Created/released objects bypass object history, but keep their ship-local attachment.
        session.mining.fragment_previous.clear();
        assert_eq!(
            shown.fragment_mesh(&session, piece).center_meters,
            frame.local_to_world(
                current_frame.world_to_local(piece.transform().position().meters())
            )
        );
        assert!(session.mining.session.pick_up(id));
        session.sync_carried();
        let shown = session.presentation();
        let piece = session.mining.session.fragment(id).unwrap();
        let offset = sub(
            piece.transform().position().meters(),
            session.snapshot().character.eye_position_meters,
        );
        assert_eq!(
            shown.fragment_mesh(&session, *piece).center_meters,
            add(shown.character.eye_position_meters, offset)
        );
        // Removal never emits a ghost: the presentation holds no entity collection.
        session
            .ship
            .intake_carried_fragment(&mut session.mining.session);
        assert!(session.mining.session.fragments().is_empty());
    }
}
