//! Biological input/clock/frame composition; source debit and publication remain world-owned.
use super::{GameplaySession, ViewMode};
use salimon_persistence::{SaveError, invalid};
use salimon_physics::static_boxes::{StaticBox, StaticField};
use salimon_world::{
    WorldPosition,
    environment::{
        harvest::{Catalog, Journal, Key, Method},
        organism_placement::Population,
    },
    mining::MiningRay,
    resources::{ResourceId, ResourceTransform},
};
use std::{
    cell::RefCell,
    sync::{Arc, LazyLock},
    time::Duration,
};
pub(crate) static FIXTURE: LazyLock<salimon_world::environment::harvest_fixture::Fixture> =
    LazyLock::new(salimon_world::environment::harvest_fixture::publish);
static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
    Catalog::publish(
        &salimon_world::canonical::ORGANISMS,
        salimon_world::CelestialBodyId::Vesper,
        &[],
    )
    .expect("sterile world has no yields")
});
#[derive(Default)]
pub(crate) struct State {
    pub fixture: bool,
    pub journal: Option<Journal>,
    pub pending: Option<Pull>,
    field: RefCell<Option<(Option<Journal>, Arc<StaticField>)>>,
    pub contact_audio: (u64, Option<ResourceId>),
}
pub(crate) struct Pull {
    pub key: Key,
    pub revision: u64,
    pub elapsed: Duration,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Save {
    fixture: bool,
    journal: Journal,
}
pub(crate) fn catalog(fixture: bool) -> &'static Catalog {
    if fixture { &FIXTURE.catalog } else { &CATALOG }
}
pub(crate) fn population(fixture: bool) -> &'static Population {
    if fixture {
        &FIXTURE.population
    } else {
        &salimon_world::canonical::ORGANISMS
    }
}
pub(crate) fn origin(fixture: bool) -> [f64; 3] {
    if fixture {
        salimon_world::CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Moon)
            .expect("fixture body")
            .center
            .meters()
    } else {
        [0.; 3]
    }
}
fn pos(p: [f64; 3]) -> WorldPosition {
    WorldPosition::new(p[0], p[1], p[2])
}
impl Save {
    pub(crate) fn validate(
        &self,
        world: &salimon_world::mining::MiningSaveState,
        terrain: salimon_world::terrain::TerrainMode,
    ) -> Result<(), SaveError> {
        if self.fixture && terrain != salimon_world::terrain::TerrainMode::Representative {
            return Err(invalid("biological fixture world mismatch"));
        }
        let c = catalog(self.fixture);
        self.journal
            .validate(c)
            .map_err(|_| invalid("biological publication/deltas"))?;
        for (id, remaining) in &world.remaining {
            if let Some(s) = c.sources().iter().find(|s| s.deposit == *id)
                && (s.method != Method::Embedded || *remaining != self.journal.remaining(s))
            {
                return Err(invalid("biological mining source debit"));
            }
        }
        for d in self.journal.deltas() {
            let s = c.get(&d.key).ok_or_else(|| invalid("biological source"))?;
            if s.method == Method::Embedded
                && !world
                    .remaining
                    .iter()
                    .any(|(id, m)| *id == s.deposit && *m == d.remaining_kg)
            {
                return Err(invalid("missing biological mining debit"));
            }
            if let Some(id) = d.detached
                && let Some(f) = world.fragments.iter().find(|f| f.id() == id)
                && (f.natural_shape() != Some(s.shape) || f.source() != s.deposit)
            {
                return Err(invalid(
                    "biological historical handle reused by another object",
                ));
            }
            if d.detached.is_some_and(|id| id.0 > world.next_id) {
                return Err(invalid("biological allocator history"));
            }
        }
        let mut biological_extracted = 0.;
        for source in c.sources().iter().filter(|s| s.method == Method::Embedded) {
            let removed = source.initial_mass_kg - self.journal.remaining(source);
            let retained: f64 = world
                .fragments
                .iter()
                .filter(|f| f.source() == source.deposit)
                .map(|f| f.material().mass_kg())
                .sum();
            if retained > removed + 1e-9 {
                return Err(invalid("biological physical mass exceeds source debit"));
            }
            biological_extracted += removed;
        }
        if biological_extracted > world.extracted_mass_kg + 1e-9 {
            return Err(invalid(
                "biological extraction history exceeds mining total",
            ));
        }
        for f in &world.fragments {
            if let Some(shape) = f.natural_shape() {
                let s = c
                    .sources()
                    .iter()
                    .find(|s| s.shape == shape)
                    .ok_or_else(|| invalid("biological fragment anatomy/source binding"))?;
                if f.source() != s.deposit
                    || f.material().resource() != s.material
                    || f.material().mass_kg() != s.initial_mass_kg
                    || (world.carried == Some(f.id()) && !s.carriable())
                    || self
                        .journal
                        .delta(&s.key)
                        .is_none_or(|d| d.detached != Some(f.id()))
                {
                    return Err(invalid("duplicated or uncommitted biological part"));
                }
            } else if let Some(s) = c.sources().iter().find(|s| s.deposit == f.source())
                && (s.method != Method::Embedded || s.material != f.material().resource())
            {
                return Err(invalid("biological fragment method/material"));
            }
        }
        Ok(())
    }
}
impl GameplaySession {
    pub(crate) fn biology_capture(&self) -> Result<Save, SaveError> {
        let s = Save {
            fixture: self.biology.fixture,
            journal: self
                .biology
                .journal
                .clone()
                .unwrap_or_else(|| Journal::new(catalog(self.biology.fixture))),
        };
        s.validate(&self.mining.session.save_state(), self.mining.terrain)?;
        Ok(s)
    }
    pub(crate) fn biology_restore(
        save: Option<Save>,
        schema: u32,
        world: &salimon_world::mining::MiningSaveState,
        terrain: salimon_world::terrain::TerrainMode,
    ) -> Result<State, SaveError> {
        let s = match save {
            Some(s) => s,
            None if schema < 29 => Save {
                fixture: false,
                journal: Journal::new(catalog(false)),
            },
            None => return Err(invalid("missing biological state")),
        };
        if schema < 29 && (s.fixture || !s.journal.deltas().is_empty()) {
            return Err(invalid("biological deltas predate schema 29"));
        }
        s.validate(world, terrain)?;
        Ok(State {
            fixture: s.fixture,
            journal: Some(s.journal),
            ..Default::default()
        })
    }
    pub(crate) fn biological_fraction(&self, object: &str, part: u16) -> f64 {
        let key = Key {
            organism: object.to_owned(),
            part,
        };
        catalog(self.biology.fixture).get(&key).map_or(1., |s| {
            self.biology
                .journal
                .as_ref()
                .map_or(1., |j| j.remaining(s) / s.initial_mass_kg)
        })
    }
    pub(crate) fn biological_bounds(&self) -> Vec<(Key, [f64; 6])> {
        let base = origin(self.biology.fixture);
        population(self.biology.fixture)
            .objects()
            .iter()
            .flat_map(|o| {
                o.parts.iter().filter_map(|p| {
                    let fraction = self.biological_fraction(&o.id, p.id);
                    if fraction <= 0. {
                        return None;
                    }
                    let part = o
                        .specimen
                        .parts
                        .iter()
                        .find(|part| part.id == p.id)
                        .expect("published part");
                    let r = part.radius_start.max(part.radius_end) * fraction.sqrt();
                    let bounds = std::array::from_fn(|i| {
                        base[i / 2]
                            + if i % 2 == 0 {
                                p.start_meters[i / 2].min(p.end_meters[i / 2]) - r
                            } else {
                                p.start_meters[i / 2].max(p.end_meters[i / 2]) + r
                            }
                    });
                    Some((
                        Key {
                            organism: o.id.clone(),
                            part: p.id,
                        },
                        bounds,
                    ))
                })
            })
            .collect()
    }
    pub(crate) fn organism_field(&self) -> Arc<StaticField> {
        let mut cache = self.biology.field.borrow_mut();
        if let Some((j, field)) = &*cache
            && *j == self.biology.journal
        {
            return field.clone();
        }
        let boxes = self
            .biological_bounds()
            .into_iter()
            .map(|(_, b)| StaticBox::new(b).expect("published finite shape"))
            .collect();
        let field = Arc::new(StaticField::new(boxes).expect("bounded physical publication"));
        *cache = Some((self.biology.journal.clone(), field.clone()));
        field
    }
    /// First solid hit wins, including non-harvestable anatomy. No through-root pickup.
    pub(crate) fn biological_target(&self) -> Option<Key> {
        if self.view_mode != ViewMode::Gameplay
            || self.building.active
            || self.surface_chart.open
            || self.crafting_ui.open
            || self.archive_reader.open
            || self.wheel.open
            || self.is_paused()
            || self.snapshot().character.location != salimon_character::CharacterLocation::Surface
        {
            return None;
        }
        let state = self.snapshot();
        let eye = state.character.eye_position_meters;
        let look = state.character.look_target_meters;
        let ray = MiningRay::new(pos(eye), pos(look))?;
        let far = std::array::from_fn(|i| eye[i] + ray.direction[i] * 4.);
        let frame = super::frames::character_ship_frame(state.ship.pose);
        let blocked = salimon_world::mining::terrain_obstruction(ray, self.mining.terrain, 4.)
            .into_iter()
            .chain(salimon_character::ship_sight_obstruction(
                frame.world_to_local(eye),
                frame.world_to_local(far),
                super::door_passable(state.ship),
            ))
            .chain(self.mining.interior_solids.iter().filter_map(|b| {
                salimon_character::cargo::box_sight_obstruction(
                    frame.world_to_local(eye),
                    frame.world_to_local(far),
                    *b,
                )
            }))
            .chain(
                self.mining
                    .session
                    .fragments()
                    .iter()
                    .filter(|f| Some(f.id()) != self.mining.session.attached_sphere_id())
                    .filter_map(|f| {
                        ray.sphere_distance(f.transform().position(), f.contact_radius_meters())
                    }),
            )
            .min_by(f64::total_cmp)
            .unwrap_or(f64::INFINITY);
        let (key, distance) = self
            .biological_bounds()
            .into_iter()
            .filter_map(|(key, b)| {
                salimon_character::cargo::box_sight_obstruction(eye, far, b).map(|d| (key, d))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)))?;
        let s = catalog(self.biology.fixture).get(&key)?;
        let limit = if s.method == Method::Detach { 3. } else { 4. };
        (distance <= limit && distance < blocked).then_some(key)
    }
    pub(crate) fn begin_biological_pull(&mut self) -> bool {
        let Some(key) = self.biological_target() else {
            return false;
        };
        let c = catalog(self.biology.fixture);
        let source = c.get(&key).expect("target source");
        if source.method != Method::Detach {
            return false;
        }
        if self.mining.session.carried_object().is_some()
            || self.mounts.carried.is_some()
            || !self.admit_survival_action(salimon_survival::restrictions::Action::Carry)
        {
            return true;
        }
        if !source.carriable() {
            self.feedback = Some("Attached part too large or heavy to carry");
            return true;
        }
        self.equipment.stow();
        self.mining.stop_mining();
        let journal = self.biology.journal.get_or_insert_with(|| Journal::new(c));
        self.biology.pending = Some(Pull {
            revision: journal.revision(&key),
            key,
            elapsed: Duration::ZERO,
        });
        self.feedback = Some("Gripping biological part - separating");
        true
    }
    /// True when biology handled this tick; ordinary mineral mining otherwise proceeds.
    pub(crate) fn advance_biology(&mut self, delta: Duration) -> bool {
        if let Some(mut pull) = self.biology.pending.take() {
            if self.biological_target().as_ref() != Some(&pull.key)
                || !self
                    .restrictions()
                    .permits(salimon_survival::restrictions::Action::Carry)
                || self.mining.session.carried_object().is_some()
            {
                self.feedback = Some("Separation interrupted - source intact");
                return true;
            }
            pull.elapsed += delta;
            if pull.elapsed < Duration::from_millis(250) {
                self.biology.pending = Some(pull);
                return true;
            }
            let c = catalog(self.biology.fixture);
            let source = c.get(&pull.key).expect("bound pull");
            let base = origin(self.biology.fixture);
            let center = std::array::from_fn(|i| source.center[i] + base[i]);
            let pose =
                ResourceTransform::new(pos(center), [0., 0., 0., 1.]).expect("published pose");
            let j = self.biology.journal.get_or_insert_with(|| Journal::new(c));
            match self
                .mining
                .session
                .detach_biological(c, j, &pull.key, pull.revision, pose)
            {
                Ok(_) => {
                    self.equipment.sync_carrying(true);
                    self.biology.contact_audio = (
                        self.biology.contact_audio.0.wrapping_add(1),
                        Some(source.material),
                    );
                    self.feedback = Some("Biological part separated - F to drop");
                }
                Err(_) => self.feedback = Some("Source changed - separation declined"),
            }
            return true;
        }
        if self.mining.input.held && self.equipment.mining_equipped() {
            let Some(key) = self.biological_target() else {
                return false;
            };
            let c = catalog(self.biology.fixture);
            let source = c.get(&key).expect("bound source");
            if source.method != Method::Embedded {
                return false;
            }
            let Some(pose) = crate::mining::attachment_pose(self.snapshot().character) else {
                return true;
            };
            let config = self.mining.config;
            let dt = self.mining_delta(delta);
            let j = self.biology.journal.get_or_insert_with(|| Journal::new(c));
            let revision = j.revision(&key);
            if self
                .mining
                .session
                .mine_biological(c, j, (&key, revision), dt, config, pose)
                .is_err()
            {
                self.mining.session.interrupt_spherical();
            }
            return true;
        }
        false
    }
    pub(crate) fn biological_hint(&self) -> Option<String> {
        if let Some(p) = &self.biology.pending {
            return Some(format!(
                "GRIPPING / SEPARATING {}%",
                (p.elapsed.as_secs_f64() / 0.25 * 100.).min(100.) as u8
            ));
        }
        if self.mining.session.carried_object().is_some() {
            return None;
        }
        let key = self.biological_target()?;
        let s = catalog(self.biology.fixture).get(&key)?;
        Some(match s.method {
            Method::Detach if !s.carriable() => "PART TOO LARGE / HEAVY".into(),
            Method::Detach => "F GRIP AND SEPARATE BIOLOGICAL PART".into(),
            Method::Embedded => format!(
                "{} / LEFT CLICK MINE {}",
                if self.equipment.mining_equipped() {
                    "TOOL READY"
                } else {
                    "1 EQUIP TOOL"
                },
                s.material.key()
            ),
        })
    }
}

mod fixture;
pub(crate) use fixture::initialize_fixture;

#[cfg(test)]
mod tests;
