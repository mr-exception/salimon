//! Active-clock paid work and atomic structural/interior completion.
//! Module commissioning remains with #547.
use super::*;
use std::time::Duration;

pub(super) fn supports_completion(p: &Placeholder) -> bool {
    if p.blueprint.id == "two-deck-room" && p.cells.len() < 4 {
        return false;
    }
    matches!(
        p.blueprint.id.as_str(),
        "stool" | "partition" | "hull-cell" | "deck-cell" | "two-deck-room"
    )
}
fn watts(p: &Placeholder) -> f64 {
    static POWER: std::sync::OnceLock<std::collections::BTreeMap<String, f64>> =
        std::sync::OnceLock::new();
    let power = POWER.get_or_init(|| {
        let power: std::collections::BTreeMap<String, f64> =
            serde_json::from_str(include_str!("../../../catalogs/construction-power.json"))
                .expect("checked-in construction watts decode");
        assert!(registered_catalog().blueprints.iter().all(|b| {
            power
                .get(&b.id)
                .is_some_and(|w| w.is_finite() && *w > 0. && *w <= 1_000_000.)
        }));
        power
    });
    power[&p.blueprint.id]
}
impl GameplaySession {
    fn construction_safety(&self, p: &Placeholder) -> Result<(), PlacementError> {
        let obstacles = self.construction_obstacles(p.blueprint.footprint);
        let flight = if matches!(
            self.ship.snapshot().flight_state,
            salimon_ship::FlightState::Landed { .. }
        ) {
            FlightPermission::Landed
        } else {
            FlightPermission::Airborne
        };
        self.building.source.projects.validate_completion(
            p.id,
            &self.building.structure,
            &PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight,
                // Automatic work never depends on the player's aim, tool or distance.
                eye_meters: p.anchor_meters,
                reach_meters: REACH_METERS,
                reachable: true,
                physical_support: if installed::structural(p) {
                    self.structural_support(p.space)
                } else {
                    has_physical_support(p.blueprint.footprint, p.space)
                },
                obstacles: &obstacles,
            },
        )
    }
    pub(super) fn construction_status(&self, p: &Placeholder) -> String {
        if p.work.completed {
            return "COMPLETED".into();
        }
        if !p.funding.funded(&p.blueprint.requirements) {
            return "WAITING FOR MATERIALS".into();
        }
        if p.work.damaged {
            return "PAUSED - damaged".into();
        }
        if let Err(e) = self.construction_safety(p) {
            return format!("PAUSED - {e:?}");
        }
        if !supports_completion(p) && p.work.remaining(&p.funding, &p.blueprint).as_nanos() <= 1_000
        {
            return "PAUSED - installation unavailable".into();
        }
        if self.ship.snapshot().energy_core.stored_joules as f64 <= self.recovery_reserve() {
            return "PAUSED - Core reserve".into();
        }
        "BUILDING".into()
    }
    pub(in crate::session) fn advance_construction(&mut self, delta: Duration) {
        self.building.running = None;
        if delta.is_zero()
            || self.paused
            || self.survival.snapshot().terminal
            || !self.building.catalog.approved_economy
        {
            return;
        }
        let ids: Vec<_> = self.building.source.projects.jobs().map(|p| p.id).collect();
        for id in ids {
            let p = self
                .building
                .source
                .projects
                .inspect(id)
                .expect("collected project identity");
            if p.work.completed
                || p.work.damaged
                || !p.funding.funded(&p.blueprint.requirements)
                || self.construction_safety(p).is_err()
            {
                continue;
            }
            let remaining = p.work.remaining(&p.funding, &p.blueprint).as_nanos() as u64;
            // Leave complex installations cancellable until their own physical
            // transaction exists. No last joule, completed flag or fake collider.
            let allowed = if supports_completion(p) {
                remaining
            } else {
                remaining.saturating_sub(1_000)
            };
            let nanos = (delta.as_nanos().min(u128::from(allowed))) as u64;
            if nanos == 0 {
                continue;
            }
            // Stage the entire physical installation before the final joule.
            let installation = if installed::structural(p) && nanos == remaining {
                match self.building.source.structure(Some(id)) {
                    Ok(next) if self.installation_clear(&next) => Some(next),
                    Ok(_) => continue,
                    Err(_) => continue,
                }
            } else {
                None
            };
            let power = watts(p);
            let center = std::array::from_fn(|i| (p.space.min[i] + p.space.max[i]) / 2.);
            let requested = nanos as f64 / 1e9 * power;
            let funded = self
                .ship
                .fund_auxiliary_work(requested, self.recovery_reserve(), delta)
                .expect("validated construction request");
            let paid = if funded == requested {
                nanos
            } else {
                ((funded / power * 1e9).floor() as u64).min(nanos)
            };
            if paid == 0 {
                continue;
            }
            let p = self
                .building
                .source
                .projects
                .funding_mut(id, OWNER)
                .expect("local project owner");
            p.work
                .advance(&mut p.funding, &p.blueprint, Duration::from_nanos(paid))
                .expect("validated paid progress");
            if p.work.completed
                && let Some(next) = installation
            {
                self.building.structure = next;
            }
            self.building.running = Some(center);
        }
        self.sync_fabricator_mass();
        self.sync_mount_collision();
    }
    /// One physical geometry definition feeds both rendering and collision. No
    /// preview, partial build, or complex unimplemented room becomes solid.
    pub(crate) fn construction_solids(&self) -> Vec<[f64; 6]> {
        let mut out = self.structural_solids();
        for p in self
            .building
            .source
            .projects
            .jobs()
            .filter(|p| p.work.completed)
        {
            let s = p.space;
            if p.blueprint.id == "stool" {
                out.push([
                    s.min[0],
                    s.max[0],
                    s.max[1] - 0.1,
                    s.max[1],
                    s.min[2],
                    s.max[2],
                ]);
                for x in [s.min[0] + 0.05, s.max[0] - 0.05] {
                    for z in [s.min[2] + 0.05, s.max[2] - 0.05] {
                        out.push([
                            x - 0.025,
                            x + 0.025,
                            s.min[1],
                            s.max[1] - 0.1,
                            z - 0.025,
                            z + 0.025,
                        ]);
                    }
                }
            } else if p.blueprint.id == "partition" {
                out.push([s.min[0], s.max[0], s.min[1], s.max[1], s.min[2], s.max[2]]);
            }
        }
        out
    }
    pub(super) fn construction_obstacles(&self, footprint: Footprint) -> Vec<Obstacle> {
        let mut out = static_obstacles(footprint);
        let bounds = self
            .cabin_solids()
            .into_iter()
            .chain(self.construction_solids());
        out.extend(bounds.map(|b| Obstacle {
            kind: ObstacleKind::Installed,
            space: envelope(b),
        }));
        for m in &self.mining.mount_geometry.mounts {
            let p = m.position();
            out.push(Obstacle {
                kind: ObstacleKind::Installed,
                space: Envelope {
                    min: [p[0] - 0.5, p[1] - 0.03, p[2] - 0.175],
                    max: [p[0] + 0.5, p[1] + 0.03, p[2] + 0.175],
                },
            });
        }
        let snapshot = self.snapshot();
        let frame = character_ship_frame(snapshot.ship.pose);
        let eye = frame.world_to_local(snapshot.character.eye_position_meters);
        let r = salimon_character::PLAYER_RADIUS_METERS;
        let y = eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS;
        out.push(Obstacle {
            kind: ObstacleKind::Player,
            space: Envelope {
                min: [eye[0] - r, y, eye[2] - r],
                max: [
                    eye[0] + r,
                    y + salimon_character::PLAYER_BODY_HEIGHT_METERS,
                    eye[2] + r,
                ],
            },
        });
        for f in self.mining.session.fragments() {
            let local = frame.world_to_local(f.transform().position().meters());
            let radius = crate::fragment_physics::sphere_radius(*f);
            out.push(Obstacle {
                kind: ObstacleKind::Cargo,
                space: Envelope {
                    min: local.map(|v| v - radius),
                    max: local.map(|v| v + radius),
                },
            });
        }
        out
    }
}
