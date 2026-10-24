//! Physical damage transactions. Receipts remain provenance, never a second inventory.
use super::*;
use salimon_ship::structure::funding::{COMPLETE, milligrams};
use salimon_world::resources::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Damage {
    pub lost: BTreeSet<StructureId>,
    pub damaged: BTreeSet<StructureId>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Target {
    Project(PlaceholderId),
    Part(StructureId),
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Impact {
    pub target: Target,
    pub destroy: bool,
    /// Actual surviving material fraction supplied by the impact producer.
    /// Detached, unhit cells retain all their material. This is not cancellation.
    pub survival_ppm: u32,
}
impl ConstructionSave {
    pub(super) fn validate_damage(&self) -> Result<(), salimon_persistence::SaveError> {
        use salimon_persistence::invalid;
        let mut known = BTreeSet::new();
        for p in self
            .projects
            .jobs()
            .filter(|p| p.work.completed && installed::structural(p))
        {
            for index in 0..p.cells.len() {
                let id =
                    p.id.0
                        .checked_mul(1024)
                        .and_then(|n| n.checked_add(1024 + index as u64))
                        .ok_or_else(|| invalid("cell identity overflow"))?;
                known.insert(StructureId(id));
            }
            if p.work.destroyed
                != (0..p.cells.len()).all(|i| {
                    self.damage
                        .lost
                        .contains(&StructureId(p.id.0 * 1024 + 1024 + i as u64))
                })
            {
                return Err(invalid("destroyed structure/source mismatch"));
            }
        }
        if self.damage.lost.iter().any(|id| !known.contains(id))
            || self
                .damage
                .damaged
                .iter()
                .any(|id| !known.contains(id) || self.damage.lost.contains(id))
        {
            return Err(invalid("unknown, duplicated or lost damaged cell"));
        }
        Ok(())
    }
}
impl GameplaySession {
    pub(in crate::session) fn interrupt_construction_after_crash(&mut self) {
        // Existing hull collision has no per-cell destructive impulse model.
        // Conservatively interrupt sites and mark surviving installed structure;
        // localized destructive producers use the typed Impact boundary.
        let ids: Vec<_> = self
            .building
            .source
            .projects
            .jobs()
            .filter(|p| !p.work.destroyed)
            .map(|p| p.id)
            .collect();
        for id in ids {
            let p = self
                .building
                .source
                .projects
                .inspect(id)
                .expect("collected project");
            if p.work.completed && installed::structural(p) {
                self.building.source.damage.damaged.extend(
                    self.building
                        .structure
                        .parts()
                        .filter(|part| {
                            part.id.0 >= id.0 * 1024 + 1024 && part.id.0 < id.0 * 1024 + 1152
                        })
                        .map(|p| p.id),
                );
            } else if let Ok(p) = self.building.source.projects.funding_mut(id, OWNER) {
                p.work.damaged = true;
            }
            if let Some(m) = self.building.source.modules.get_mut(&id) {
                m.unavailable = true;
                m.fault = Some(modules::Fault::Damage);
            }
        }
        self.building.running = None;
    }

    /// Called by authoritative physical producers. No player damage hotkey or
    /// random salvage roll; duplicate destruction cannot mint another object.
    pub(crate) fn construction_impact(&mut self, hit: Impact) -> Result<(), &'static str> {
        if hit.survival_ppm > COMPLETE || (!hit.destroy && hit.survival_ppm != COMPLETE) {
            return Err("invalid material survival fraction");
        }
        let mut next = self.building.source.clone();
        let mut salvage = Vec::new();
        match hit.target {
            Target::Part(id) => {
                if id.0 < 1024 {
                    return Err("permanent authored Core/hull cannot be removed");
                }
                let mut graph = self.building.structure.clone();
                if !graph.parts().any(|p| p.id == id) {
                    return Err("cell no longer attached");
                }
                if !hit.destroy {
                    next.damage.damaged.insert(id);
                } else {
                    let outcome = graph
                        .damage_part(id)
                        .map_err(|_| "structural damage rejected")?;
                    let parts = std::iter::once((outcome.destroyed, hit.survival_ppm)).chain(
                        outcome
                            .detached
                            .into_iter()
                            .flat_map(|d| d.parts.into_iter().map(|p| (p, COMPLETE))),
                    );
                    for (part, survival) in parts {
                        let project = PlaceholderId((part.id.0 - 1024) / 1024);
                        let p = next.projects.inspect(project).ok_or("cell owner missing")?;
                        let mut cells = p.cells.clone();
                        cells.sort();
                        let index = cells
                            .iter()
                            .position(|c| part.cells.contains(c))
                            .ok_or("cell source missing")?;
                        let n = cells.len() as u64;
                        let center =
                            std::array::from_fn(|i| (part.space.min[i] + part.space.max[i]) / 2.);
                        // Partition each resource in integer milligrams before applying
                        // impact loss. The remainder belongs to the first sorted cells.
                        for requirement in &p.blueprint.requirements {
                            let total = p.funding.delivered(requirement.resource);
                            let mg = total / n + u64::from((index as u64) < total % n);
                            if let Some(receipt) = p
                                .funding
                                .receipts
                                .iter()
                                .find(|f| f.material().resource() == requirement.resource)
                            {
                                salvage.push((*receipt, mg, survival, center));
                            }
                        }
                        next.damage.lost.insert(part.id);
                        next.damage.damaged.remove(&part.id);
                    }
                    let ids: Vec<_> = next
                        .projects
                        .jobs()
                        .filter(|p| {
                            p.work.completed
                                && installed::structural(p)
                                && (0..p.cells.len()).all(|i| {
                                    next.damage
                                        .lost
                                        .contains(&StructureId(p.id.0 * 1024 + 1024 + i as u64))
                                })
                        })
                        .map(|p| p.id)
                        .collect();
                    for id in ids {
                        next.projects
                            .funding_mut(id, OWNER)
                            .map_err(|_| "owner")?
                            .work
                            .destroyed = true;
                    }
                    next.removed_partitions.retain(|edit| {
                        !next.damage.lost.contains(&edit.part)
                            && self
                                .building
                                .structure
                                .parts()
                                .find(|p| p.id == edit.part)
                                .and_then(|p| {
                                    p.faces.iter().find(|f| f.direction == edit.direction)
                                })
                                .and_then(|f| f.cell.neighbor(f.direction))
                                .is_none_or(|neighbor| {
                                    !self.building.structure.occupied_cells().any(|(c, id)| {
                                        c == neighbor && next.damage.lost.contains(&id)
                                    })
                                })
                    });
                    // Unsupported installed furniture becomes physical salvage too;
                    // an unfunded reservation simply pauses until support returns.
                    let unsupported: Vec<_> = next
                        .projects
                        .jobs()
                        .filter(|p| {
                            p.work.completed
                                && !p.work.destroyed
                                && !installed::structural(p)
                                && !has_physical_support(p.blueprint.footprint, p.space)
                                && !installed::supported_by(&graph, p.space)
                        })
                        .map(|p| p.id)
                        .collect();
                    for id in unsupported {
                        retire_project(&mut next, id, COMPLETE, &mut salvage)?;
                    }
                }
            }
            Target::Project(id) => {
                let p = next.projects.inspect(id).ok_or("project missing")?;
                if p.work.destroyed {
                    return Err("project already destroyed");
                }
                if p.work.completed && installed::structural(p) {
                    return Err("address an individual structural cell");
                }
                if hit.destroy {
                    retire_project(&mut next, id, hit.survival_ppm, &mut salvage)?;
                } else {
                    next.projects
                        .funding_mut(id, OWNER)
                        .map_err(|_| "owner")?
                        .work
                        .damaged = true;
                    if let Some(m) = next.modules.get_mut(&id) {
                        m.unavailable = true;
                        m.fault = Some(modules::Fault::Damage);
                    }
                }
            }
        }
        let graph = next
            .structure(None)
            .map_err(|_| "damaged graph cannot reconstruct")?;
        salimon_persistence::Validate::validate(&next).map_err(|_| "damaged source invalid")?;
        let frame = character_ship_frame(self.ship.snapshot().pose);
        let mut pieces = Vec::new();
        for (receipt, mg, survival, center) in salvage {
            let mg = (u128::from(mg) * u128::from(survival) / u128::from(COMPLETE)) as u64;
            if mg == 0 {
                continue;
            }
            let material = RawMaterial::new(receipt.material().resource(), mg as f64 / 1e6)
                .map_err(|_| "salvage material")?;
            let w = frame.local_to_world(center);
            let pose = ResourceTransform::new(
                salimon_world::WorldPosition::new(w[0], w[1], w[2]),
                receipt.transform().orientation_xyzw(),
            )
            .map_err(|_| "salvage pose")?;
            pieces.push((receipt.source(), material, pose));
        }
        // World allocates all IDs or none. No fallible operation follows publication.
        self.mining
            .session
            .import_salvage(&pieces)
            .ok_or("salvage identity allocation")?;
        self.building.source = next;
        self.building.structure = graph;
        self.building.running = None;
        self.sync_mount_collision();
        self.mining.mass_frame = None;
        self.sync_carried();
        self.sync_fabricator_mass();
        Ok(())
    }
    fn aimed_physical_construction(&self) -> Option<Target> {
        if let Some(id) = self.aimed_construction() {
            return Some(Target::Project(id));
        }
        let snap = self.snapshot();
        let f = character_ship_frame(snap.ship.pose);
        let eye = f.world_to_local(snap.character.eye_position_meters);
        let direction = salimon_math::sub(f.world_to_local(snap.character.look_target_meters), eye);
        self.building
            .structure
            .parts()
            .filter(|p| p.id.0 >= 1024)
            .filter_map(|p| {
                p.collision
                    .iter()
                    .filter_map(|b| ray_distance(eye, direction, *b))
                    .min_by(f64::total_cmp)
                    .map(|d| (p.id, d))
            })
            .filter(|(_, d)| {
                salimon_character::ship_sight_obstruction(
                    eye,
                    salimon_math::add(
                        eye,
                        salimon_math::scale(direction, *d / salimon_math::length(direction)),
                    ),
                    super::super::door_passable(snap.ship),
                )
                .is_none_or(|hit| hit + 0.02 >= *d)
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| Target::Part(id))
    }
    pub(super) fn construction_physical_action(&mut self, action: BuildAction) {
        let Some(target) = self.aimed_physical_construction() else {
            self.feedback = Some("Aim at a reachable physical construction object");
            return;
        };
        if matches!(action, BuildAction::Repair) {
            // Mechanical resecure of surviving parts. This never recreates lost
            // material, cells, progress or an unavailable commissioned capability.
            match target {
                Target::Part(id) => {
                    self.building.source.damage.damaged.remove(&id);
                }
                Target::Project(id) => {
                    if let Ok(p) = self.building.source.projects.funding_mut(id, OWNER) {
                        p.work.damaged = false;
                    }
                    if let Some(m) = self.building.source.modules.get_mut(&id) {
                        m.fault = None;
                        if let Some(op) = &mut m.maintenance {
                            op.work.damaged = false;
                        }
                    }
                }
            }
            self.feedback =
                Some("Surviving construction resecured; lost parts require new construction");
        } else {
            self.feedback = Some(if self.dismantle_construction(target).is_ok() {
                "Completed object dismantled into physical salvage"
            } else {
                "Dismantling unsafe: retain support, land and clear occupants"
            });
        }
    }
    pub(in crate::session) fn dismantle_construction(
        &mut self,
        target: Target,
    ) -> Result<(), &'static str> {
        if !matches!(
            self.ship.snapshot().flight_state,
            salimon_ship::FlightState::Landed { .. }
        ) {
            return Err("land first");
        }
        let space = match target {
            Target::Part(id) => {
                let mut next = self.building.structure.clone();
                let outcome = next.damage_part(id).map_err(|_| "protected structure")?;
                if !outcome.detached.is_empty() || !outcome.unsupported_equipment.is_empty() {
                    return Err("disconnects structure");
                }
                let space = outcome.destroyed.space;
                if self.building.source.projects.jobs().any(|p| {
                    p.work.completed
                        && !p.work.destroyed
                        && !installed::structural(p)
                        && !has_physical_support(p.blueprint.footprint, p.space)
                        && !installed::supported_by(&next, p.space)
                }) {
                    return Err("unsupported machinery");
                }
                space
            }
            Target::Project(id) => {
                let p = self
                    .building
                    .source
                    .projects
                    .inspect(id)
                    .ok_or("missing project")?;
                if !p.work.completed {
                    return Err("unfinished: use cancellation");
                }
                p.space
            }
        };
        // Deliberate removal cannot drop the player/cargo through a live floor.
        let snap = self.snapshot();
        let f = character_ship_frame(snap.ship.pose);
        let eye = f.world_to_local(snap.character.eye_position_meters);
        let contains = |p: [f64; 3], r: f64| {
            (0..3).all(|i| p[i] + r > space.min[i] && p[i] - r < space.max[i])
        };
        if contains(eye, salimon_character::PLAYER_EYE_HEIGHT_METERS)
            || self.mining.session.fragments().iter().any(|p| {
                contains(
                    f.world_to_local(p.transform().position().meters()),
                    crate::fragment_physics::sphere_radius(*p),
                )
            })
        {
            return Err("occupied");
        }
        self.construction_impact(Impact {
            target,
            destroy: true,
            survival_ppm: COMPLETE,
        })
    }
}
type Salvage = Vec<(ResourceFragment, u64, u32, [f64; 3])>;
fn retire_project(
    source: &mut ConstructionSave,
    id: PlaceholderId,
    survival: u32,
    out: &mut Salvage,
) -> Result<(), &'static str> {
    let p = source.projects.inspect(id).ok_or("project missing")?;
    if p.work.destroyed {
        return Err("already destroyed");
    }
    let center = std::array::from_fn(|i| (p.space.min[i] + p.space.max[i]) / 2.);
    for f in p.funding.receipts.iter().chain(
        source
            .modules
            .get(&id)
            .into_iter()
            .flat_map(|m| m.receipts()),
    ) {
        out.push((
            *f,
            milligrams(*f).map_err(|_| "invalid receipt")?,
            survival,
            center,
        ));
    }
    source
        .projects
        .funding_mut(id, OWNER)
        .map_err(|_| "owner")?
        .work
        .destroyed = true;
    source.open_doors.retain(|i| *i != id);
    if let Some(m) = source.modules.get_mut(&id) {
        m.unavailable = true;
        m.fault = Some(modules::Fault::Damage);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
