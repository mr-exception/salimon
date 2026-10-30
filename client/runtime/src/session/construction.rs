//! Compose blueprint authority with current ship, player and physical cargo.
mod boarding;
pub(crate) mod damage;
mod doors;
mod evidence;
pub(super) mod save;
pub(crate) use evidence::evidence_module;
mod flight;
mod installed;
mod modules;
mod work;
use super::{GameplaySession, frames::character_ship_frame};
use salimon_character::{CharacterLocation, SHIP_FLOOR_HEIGHT_METERS};
use salimon_ship::structure::placement::*;
use salimon_ship::structure::{
    Cell, Envelope, MemberShape, ShipStructure, StructuralMass, StructuralPart, StructureId,
};
use serde::{Deserialize, Serialize};
use work::supports_completion;

const OWNER: u64 = 1; // The existing client-only run has one local ship owner.
const REACH_METERS: f64 = 5.;
const ORIGIN: [f64; 3] = [-2.5, SHIP_FLOOR_HEIGHT_METERS, -4.5];
fn registered_catalog() -> &'static Catalog {
    static CATALOG: std::sync::OnceLock<Catalog> = std::sync::OnceLock::new();
    CATALOG.get_or_init(|| {
        let catalog: Catalog = serde_json::from_str(include_str!("../../catalogs/blueprints.json"))
            .expect("checked-in blueprint catalog must decode");
        catalog
            .validate()
            .expect("checked-in blueprint catalog must validate");
        catalog
    })
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum BuildAction {
    Toggle,
    Select(usize),
    Rotate,
    Place,
    Cancel,
    RemovePartition,
    StopResume,
    Module,
    Repair,
    Dismantle,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConstructionSave {
    #[serde(default)]
    pub(super) modules: std::collections::BTreeMap<PlaceholderId, modules::Module>,
    pub(crate) projects: Placements,
    #[serde(default)]
    pub(crate) damage: damage::Damage,
    #[serde(default)]
    pub(crate) removed_partitions: Vec<salimon_ship::structure::shell::PartitionEdit>,
    #[serde(default)]
    pub(crate) open_doors: Vec<PlaceholderId>,
}
impl salimon_persistence::Validate for ConstructionSave {
    fn validate(&self) -> Result<(), salimon_persistence::SaveError> {
        self.validate_damage()?;
        self.projects
            .validate_save()
            .map_err(|e| salimon_persistence::invalid(e.to_string()))?;
        let structure = self
            .structure(None)
            .map_err(|e| salimon_persistence::invalid(e.to_string()))?;
        let mut door_ids = std::collections::BTreeSet::new();
        for id in &self.open_doors {
            if !door_ids.insert(*id)
                || self.projects.inspect(*id).is_none_or(|p| {
                    p.work.destroyed || !p.work.completed || p.blueprint.id != "internal-door"
                })
            {
                return Err(salimon_persistence::invalid("invalid completed door state"));
            }
        }
        for job in self.projects.jobs() {
            if job.owner != OWNER || job.origin_meters != ORIGIN || !matches_catalog(job) {
                return Err(salimon_persistence::invalid(
                    "construction project catalog/owner/origin mismatch",
                ));
            }
            if job.work.destroyed {
                continue;
            }
            let mut obstacles = static_obstacles(job.blueprint.footprint);
            if job.blueprint.footprint == Footprint::Interior {
                obstacles.extend(structure.parts().filter(|p| p.id.0 >= 1024).flat_map(|p| {
                    p.collision.iter().map(|b| Obstacle {
                        kind: ObstacleKind::Installed,
                        space: *b,
                    })
                }));
            }
            let context = PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: job.anchor_meters,
                reach_meters: REACH_METERS,
                reachable: true,
                physical_support: has_physical_support(job.blueprint.footprint, job.space)
                    || (job.blueprint.footprint == Footprint::Interior
                        && installed::supported_by(&structure, job.space)),
                obstacles: &obstacles,
            };
            if job.work.completed && installed::structural(job) {
                if obstacles.iter().any(|o| {
                    (0..3).all(|i| {
                        job.space.min[i] < o.space.max[i] && job.space.max[i] > o.space.min[i]
                    })
                }) {
                    return Err(salimon_persistence::invalid(
                        "completed structure overlaps authored geometry",
                    ));
                }
            } else {
                let result = self
                    .projects
                    .validate_completion(job.id, &structure, &context);
                let unsupported_reservation = !job.work.completed
                    && !self.damage.lost.is_empty()
                    && matches!(result, Err(PlacementError::Unsupported))
                    && !obstacles.iter().any(|o| {
                        (0..3).all(|i| {
                            job.space.min[i] < o.space.max[i] && job.space.max[i] > o.space.min[i]
                        })
                    });
                if !unsupported_reservation {
                    result.map_err(|e| salimon_persistence::invalid(e.to_string()))?;
                }
            }
        }
        self.validate_modules()?;
        Ok(())
    }
}
pub(crate) struct Building {
    pub(crate) active: bool,
    selected: usize,
    rotation: Rotation,
    pub(crate) source: ConstructionSave,
    catalog: Catalog,
    pub(super) structure: ShipStructure,
    pub(crate) running: Option<[f64; 3]>,
}
fn starter_structure() -> ShipStructure {
    let origin = ORIGIN;
    let core = Cell::new(0, 0, 1);
    let space = core.bounds(origin).expect("finite starter support cell");
    let part = |id, cells: Vec<Cell>, space| StructuralPart {
        id: StructureId(id),
        cells,
        shape: MemberShape::Frame,
        faces: vec![],
        space,
        collision: vec![],
        mass: StructuralMass::IncludedInDryHull,
    };
    let mut structure = ShipStructure::new(origin, part(1, vec![core], space))
        .expect("authored baseline support root");
    let cells = (-2..=2)
        .flat_map(|x| (0..=2).map(move |z| Cell::new(x, 0, z)))
        .filter(|c| *c != core)
        .collect();
    structure
        .add_part(part(
            2,
            cells,
            Envelope {
                min: [-8.5, SHIP_FLOOR_HEIGHT_METERS, -4.5],
                max: [6.5, SHIP_FLOOR_HEIGHT_METERS + 3., 4.5],
            },
        ))
        .expect("connected baseline support envelope");
    structure
}

impl Default for Building {
    fn default() -> Self {
        let catalog = registered_catalog().clone();
        // Explicit coarse support graph; it neither rounds authored anchors nor
        // generates solid hull/colliders. New installations still test actual geometry.
        let structure = starter_structure();
        Self {
            active: false,
            running: None,
            selected: 0,
            rotation: Rotation::Forward,
            source: Default::default(),
            catalog,
            structure,
        }
    }
}
impl Building {
    pub(crate) fn from_source(
        source: ConstructionSave,
    ) -> Result<Self, salimon_persistence::SaveError> {
        salimon_persistence::Validate::validate(&source)?;
        let structure = source
            .structure(None)
            .map_err(|e| salimon_persistence::invalid(e.to_string()))?;
        Ok(Self {
            source,
            structure,
            ..Default::default()
        })
    }
}
fn envelope(b: [f64; 6]) -> Envelope {
    Envelope {
        min: [b[0], b[2], b[4]],
        max: [b[1], b[3], b[5]],
    }
}
fn static_obstacles(footprint: Footprint) -> Vec<Obstacle> {
    let mut obstacles: Vec<_> = salimon_character::ship_installation_obstacles()
        .into_iter()
        .map(|b| Obstacle {
            kind: ObstacleKind::Installed,
            space: envelope(b),
        })
        .collect();
    if footprint == Footprint::Structural {
        obstacles.push(Obstacle {
            kind: ObstacleKind::Installed,
            space: envelope(salimon_character::ship_conservative_hull_envelope()),
        });
    }
    // Existing essential carrying/escape lanes remain protected at final completion too.
    for bounds in [
        [
            -7.24,
            -2.24,
            SHIP_FLOOR_HEIGHT_METERS,
            SHIP_FLOOR_HEIGHT_METERS + 1.8,
            -0.8,
            0.8,
        ],
        [
            -3.8,
            6.32,
            SHIP_FLOOR_HEIGHT_METERS,
            SHIP_FLOOR_HEIGHT_METERS + 1.8,
            -2.6,
            -1.5,
        ],
        [
            -3.8,
            6.32,
            SHIP_FLOOR_HEIGHT_METERS,
            SHIP_FLOOR_HEIGHT_METERS + 1.8,
            1.5,
            2.6,
        ],
        [
            -3.6,
            -2.4,
            SHIP_FLOOR_HEIGHT_METERS,
            SHIP_FLOOR_HEIGHT_METERS + 1.8,
            -3.5,
            -2.4,
        ],
    ] {
        obstacles.push(Obstacle {
            kind: ObstacleKind::ProtectedRoute,
            space: envelope(bounds),
        });
    }
    obstacles
}
fn has_physical_support(footprint: Footprint, space: Envelope) -> bool {
    if footprint == Footprint::Structural {
        return true;
    }
    space.max[1] <= salimon_character::SHIP_CEILING_HEIGHT_METERS
        && [space.min[0], space.max[0]].into_iter().all(|x| {
            [space.min[2], space.max[2]].into_iter().all(|z| {
                salimon_character::ship_floor_placement([x, SHIP_FLOOR_HEIGHT_METERS, z], 0.001)
                    .is_some()
            })
        })
}

fn ray_distance(eye: [f64; 3], direction: [f64; 3], space: Envelope) -> Option<f64> {
    let length = salimon_math::length(direction);
    if !length.is_finite() || length <= 1e-12 {
        return None;
    }
    let direction = salimon_math::scale(direction, 1. / length);
    let mut near = 0.0_f64;
    let mut far = REACH_METERS;
    for i in 0..3 {
        if direction[i].abs() < 1e-12 {
            if eye[i] < space.min[i] || eye[i] > space.max[i] {
                return None;
            }
        } else {
            let a = (space.min[i] - eye[i]) / direction[i];
            let b = (space.max[i] - eye[i]) / direction[i];
            near = near.max(a.min(b));
            far = far.min(a.max(b));
            if near > far {
                return None;
            }
        }
    }
    Some(near)
}

impl GameplaySession {
    pub(super) fn contribute_construction(&mut self) {
        if self.mining.carry_blocked
            || !self.admit_survival_action(salimon_survival::restrictions::Action::Fabricate)
        {
            return;
        }
        let Some(id) = self.aimed_construction() else {
            self.feedback = Some("Aim at a reachable placeholder");
            return;
        };
        if self.building.source.projects.inspect(id).is_some_and(|p| {
            p.work.completed && !p.work.destroyed && p.blueprint.id == "internal-door"
        }) {
            self.toggle_completed_door(id);
            return;
        }
        if self.feed_module(id) {
            return;
        }
        if self
            .building
            .source
            .projects
            .inspect(id)
            .is_some_and(|p| p.work.completed)
        {
            self.feedback = Some("G: upgrade / restoration / commissioning");
            return;
        }
        let approved = self.building.catalog.approved_economy;
        let Ok(p) = self.building.source.projects.funding_mut(id, OWNER) else {
            return;
        };
        use salimon_ship::structure::funding::FundingError::*;
        self.feedback = Some(
            match p.funding.contribute(
                &mut self.mining.session,
                &p.blueprint.requirements,
                approved,
            ) {
                Ok(()) => {
                    if p.funding.funded(&p.blueprint.requirements) {
                        "Materials funded - construction starts automatically when safe"
                    } else {
                        "Material delivered to placeholder"
                    }
                }
                Err(Unapproved) => "Construction economy awaits owner approval; nothing consumed",
                Err(Oversized) => {
                    "Fragment exceeds remaining cost; bring a smaller piece (nothing consumed)"
                }
                Err(WrongMaterial) => "Placeholder requires a different material",
                _ => "Cannot accept this material; check carry slot and whole-milligram quantity",
            },
        );
        self.sync_carried();
        self.sync_fabricator_mass();
    }
    fn return_construction_materials(
        &mut self,
        mut refunds: Vec<salimon_world::resources::ResourceFragment>,
    ) -> Result<(), ()> {
        let state = self.snapshot();
        let frame = character_ship_frame(state.ship.pose);
        let eye = frame.world_to_local(state.character.eye_position_meters);
        let mut solids = self.cabin_solids();
        solids.extend(self.construction_solids());
        if self.cargo_door.open_fraction < 1. {
            solids.push(salimon_character::cargo::cargo_door_bounds(
                self.cargo_door.open_fraction,
            ));
        }
        let mut placed: Vec<([f64; 3], f64)> = Vec::new();
        for f in &mut refunds {
            let radius = f.contact_radius_meters();
            let mut destination = None;
            'search: for x in -6..=6 {
                for z in -6..=6 {
                    let requested = [
                        eye[0] + f64::from(x) * 0.5,
                        SHIP_FLOOR_HEIGHT_METERS,
                        eye[2] + f64::from(z) * 0.5,
                    ];
                    let Some(p) = salimon_character::ship_floor_placement(requested, radius) else {
                        continue;
                    };
                    if (p[0] - eye[0]).hypot(p[2] - eye[2]) < radius + 0.5
                        || salimon_math::length(salimon_math::sub(p, eye)) > REACH_METERS
                        || salimon_character::ship_sight_obstruction(
                            eye,
                            p,
                            super::door_passable(state.ship),
                        )
                        .is_some_and(|d| d + 0.02 < salimon_math::length(salimon_math::sub(p, eye)))
                        || solids
                            .iter()
                            .any(|b| salimon_character::cargo::sphere_overlaps_box(p, radius, *b))
                        || self.mining.session.fragments().iter().any(|other| {
                            salimon_math::length(salimon_math::sub(
                                p,
                                frame.world_to_local(other.transform().position().meters()),
                            )) < radius + crate::fragment_physics::sphere_radius(*other) + 0.01
                        })
                        || placed.iter().any(|(q, r)| {
                            salimon_math::length(salimon_math::sub(p, *q)) < radius + r + 0.01
                        })
                    {
                        continue;
                    }
                    destination = Some(p);
                    break 'search;
                }
            }
            let p = destination.ok_or(())?;
            placed.push((p, radius));
            let w = frame.local_to_world(p);
            f.set_transform(
                salimon_world::resources::ResourceTransform::new(
                    salimon_world::WorldPosition::new(w[0], w[1], w[2]),
                    f.transform().orientation_xyzw(),
                )
                .map_err(|_| ())?,
            );
        }
        // Both owners preflight before publication; no fallible geometry or partial return afterward.
        if !self.mining.session.return_reserved_loose(&refunds) {
            return Err(());
        }
        Ok(())
    }
    fn cancel_construction(&mut self, id: PlaceholderId) -> Result<(), ()> {
        if self
            .building
            .source
            .modules
            .get(&id)
            .is_some_and(|m| m.maintenance.is_some())
        {
            return self.cancel_module(id);
        }
        let job = self.building.source.projects.inspect(id).ok_or(())?;
        if job.work.destroyed {
            return Err(());
        }
        let refunds = job
            .funding
            .refunds(&job.blueprint.requirements)
            .map_err(|_| ())?;
        self.return_construction_materials(refunds)?;
        let p = self
            .building
            .source
            .projects
            .funding_mut(id, OWNER)
            .map_err(|_| ())?;
        p.funding = Default::default();
        self.building
            .source
            .projects
            .cancel(id, OWNER)
            .map_err(|_| ())?;
        self.building.source.modules.remove(&id);
        self.sync_carried();
        self.sync_fabricator_mass();
        Ok(())
    }
    pub(super) fn validate_construction_receipts(
        &self,
    ) -> Result<(), salimon_persistence::SaveError> {
        use salimon_persistence::invalid;
        let next = self.mining.session.save_state().next_id;
        let core_ids: std::collections::HashSet<_> = self
            .core_upgrades
            .pending
            .iter()
            .chain(
                self.core_upgrades
                    .applied
                    .iter()
                    .flat_map(|a| a.receipts.iter()),
            )
            .map(|f| f.id())
            .collect();
        let mut ids = std::collections::HashSet::new();
        for p in self.building.source.projects.jobs() {
            if p.work.completed && !supports_completion(p) {
                return Err(invalid("unsupported completed construction"));
            }
            for f in p.funding.receipts.iter().chain(
                self.building
                    .source
                    .modules
                    .get(&p.id)
                    .into_iter()
                    .flat_map(|m| m.receipts()),
            ) {
                if !ids.insert(f.id())
                    || core_ids.contains(&f.id())
                    || f.id().0 > next
                    || self.mining.session.fragment(f.id()).is_some()
                {
                    return Err(invalid(
                        "construction receipt duplicates an owner or exceeds allocator",
                    ));
                }
            }
        }
        Ok(())
    }
    fn construction_inputs(&self) -> ([f64; 3], [f64; 3], Vec<Obstacle>, FlightPermission, bool) {
        let snapshot = self.snapshot();
        let frame = character_ship_frame(snapshot.ship.pose);
        let eye = frame.world_to_local(snapshot.character.eye_position_meters);
        let look = frame.world_to_local(snapshot.character.look_target_meters);
        let direction = salimon_math::sub(look, eye);
        let length = salimon_math::length(direction);
        let direction = salimon_math::scale(direction, 1. / length.max(1e-12));
        let floor_distance = (SHIP_FLOOR_HEIGHT_METERS - eye[1]) / direction[1];
        let distance = if direction[1] < -1e-6 && (0.0..=REACH_METERS).contains(&floor_distance) {
            floor_distance
        } else {
            3.
        };
        let mut target = salimon_math::add(eye, salimon_math::scale(direction, distance));
        // Interior ground-mounted prototypes target the real deck, not eye height.
        let blueprint = &self.building.catalog.blueprints[self.building.selected];
        if blueprint.footprint == Footprint::Interior {
            target[1] = self
                .mining
                .modular_interior
                .support(
                    target,
                    0.01,
                    eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS + 0.21,
                )
                .unwrap_or(SHIP_FLOOR_HEIGHT_METERS);
        }
        let obstacles = self.construction_obstacles(blueprint.footprint);
        let flight = if matches!(
            snapshot.ship.flight_state,
            salimon_ship::FlightState::Landed { .. }
        ) {
            FlightPermission::Landed
        } else {
            FlightPermission::Airborne
        };
        let authorized = matches!(
            self.character.location(),
            CharacterLocation::InsideShip
                | CharacterLocation::Surface
                | CharacterLocation::Space
                | CharacterLocation::NearbyBody
        );
        (eye, target, obstacles, flight, authorized)
    }
    fn construction_context<'a>(
        &self,
        eye: [f64; 3],
        target: [f64; 3],
        obstacles: &'a [Obstacle],
        flight: FlightPermission,
        authorized: bool,
    ) -> PlacementContext<'a> {
        let b = &self.building.catalog.blueprints[self.building.selected];
        let preliminary = self
            .building
            .source
            .projects
            .preview(
                &self.building.catalog,
                &b.id,
                target,
                self.building.rotation,
                &self.building.structure,
                &PlacementContext {
                    actor: OWNER,
                    owner: OWNER,
                    flight,
                    eye_meters: eye,
                    reach_meters: REACH_METERS,
                    reachable: true,
                    physical_support: true,
                    obstacles: &[],
                },
            )
            .ok();
        let physical_support = preliminary.as_ref().is_some_and(|p| {
            if b.footprint == Footprint::Structural {
                self.structural_support(p.space)
            } else {
                has_physical_support(b.footprint, p.space)
            }
        });
        let reachable = salimon_character::ship_sight_obstruction(
            eye,
            target,
            super::door_passable(self.ship.snapshot()),
        )
        .is_none_or(|hit| hit + 0.02 >= salimon_math::length(salimon_math::sub(target, eye)))
            && self.construction_solids().iter().all(|b| {
                salimon_character::cargo::box_sight_obstruction(eye, target, *b).is_none_or(|hit| {
                    hit + 0.02 >= salimon_math::length(salimon_math::sub(target, eye))
                })
            });
        PlacementContext {
            actor: if authorized { OWNER } else { 0 },
            owner: OWNER,
            flight,
            eye_meters: eye,
            reach_meters: REACH_METERS,
            reachable,
            physical_support,
            obstacles,
        }
    }
    fn aimed_construction(&self) -> Option<PlaceholderId> {
        let snapshot = self.snapshot();
        let frame = character_ship_frame(snapshot.ship.pose);
        let eye = frame.world_to_local(snapshot.character.eye_position_meters);
        let direction = salimon_math::sub(
            frame.world_to_local(snapshot.character.look_target_meters),
            eye,
        );
        self.building
            .source
            .projects
            .jobs()
            .filter(|p| !p.work.destroyed && !(p.work.completed && installed::structural(p)))
            .filter_map(|p| ray_distance(eye, direction, p.space).map(|distance| (p.id, distance)))
            .filter(|(_, distance)| {
                let target = salimon_math::add(
                    eye,
                    salimon_math::scale(direction, *distance / salimon_math::length(direction)),
                );
                salimon_character::ship_sight_obstruction(
                    eye,
                    target,
                    super::door_passable(snapshot.ship),
                )
                .is_none_or(|hit| hit + 0.02 >= *distance)
                    && self.construction_solids().iter().all(|b| {
                        ray_distance(eye, direction, envelope(*b))
                            .is_none_or(|hit| hit + 0.02 >= *distance)
                    })
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|p| p.0)
    }
    pub(crate) fn construction_preview(&self) -> Option<Preview> {
        if !self.building.active {
            return None;
        }
        let (eye, target, obstacles, flight, authorized) = self.construction_inputs();
        let context = self.construction_context(eye, target, &obstacles, flight, authorized);
        self.building
            .source
            .projects
            .preview(
                &self.building.catalog,
                &self.building.catalog.blueprints[self.building.selected].id,
                target,
                self.building.rotation,
                &self.building.structure,
                &context,
            )
            .ok()
    }
    pub(crate) fn construction_action(&mut self, action: BuildAction) {
        if self
            .habitation
            .cabin
            .as_ref()
            .is_some_and(|c| c.rest.is_some())
        {
            self.feedback = Some("Get up before placing construction projects");
            return;
        }
        if matches!(
            action,
            BuildAction::Place
                | BuildAction::RemovePartition
                | BuildAction::Module
                | BuildAction::Repair
                | BuildAction::Dismantle
        ) && !self.admit_survival_action(salimon_survival::restrictions::Action::Fabricate)
        {
            return;
        }
        match action {
            BuildAction::Toggle => {
                if self.character.location() == CharacterLocation::Cockpit {
                    self.feedback = Some("Leave cockpit control to use blueprints");
                    return;
                }
                self.building.active = !self.building.active;
                if self.building.active {
                    self.equipment.stow();
                }
                self.mining.clear_input();
                self.feedback = Some(if self.building.active {
                    "Blueprints opened"
                } else {
                    "Construction closed"
                });
                return;
            }
            BuildAction::Select(index) => {
                if self.building.active && index < self.building.catalog.blueprints.len() {
                    self.building.selected = index;
                }
                return;
            }
            BuildAction::Rotate => {
                if self.building.active {
                    self.building.rotation = self.building.rotation.next();
                }
                return;
            }
            _ => {}
        }
        if !self.building.active {
            return;
        }
        let (eye, target, obstacles, flight, authorized) = self.construction_inputs();
        if !authorized {
            self.feedback = Some("Leave cockpit control to place or cancel a blueprint");
            return;
        }
        if matches!(action, BuildAction::Repair | BuildAction::Dismantle) {
            self.construction_physical_action(action);
            return;
        }
        if matches!(action, BuildAction::Module) {
            self.module_action();
            return;
        }
        if matches!(action, BuildAction::StopResume) {
            self.stop_resume_construction();
            return;
        }
        if matches!(action, BuildAction::RemovePartition) {
            self.remove_partition();
            return;
        }
        if matches!(action, BuildAction::Cancel) {
            let nearest = self.aimed_construction();
            self.feedback = Some(if let Some(id) = nearest {
                if self.cancel_construction(id).is_ok() {
                    "Placeholder cancelled"
                } else {
                    "Cannot cancel this placeholder"
                }
            } else {
                "No reachable placeholder"
            });
            return;
        }
        let context = self.construction_context(eye, target, &obstacles, flight, authorized);
        let id = &self.building.catalog.blueprints[self.building.selected].id;
        self.feedback = Some(
            match self.building.source.projects.place(
                &self.building.catalog,
                id,
                target,
                self.building.rotation,
                &self.building.structure,
                &context,
            ) {
                Ok(_) => "Placeholder placed",
                Err(_) => "Placement declined - inspect the preview feedback",
            },
        );
    }
    pub(crate) fn construction_hint(&self) -> Option<String> {
        let p = self.construction_preview()?;
        let hovered = self
            .aimed_construction()
            .and_then(|id| self.building.source.projects.inspect(id));
        let maintenance_blueprint = hovered.and_then(|job| self.module_blueprint(job.id));
        let blueprint = maintenance_blueprint
            .as_ref()
            .unwrap_or_else(|| hovered.map_or(&p.blueprint, |job| &job.blueprint));
        let resources = blueprint
            .requirements
            .iter()
            .map(|r| {
                format!(
                    "{} {:.3}kg",
                    r.resource.key(),
                    r.milligrams as f64 / 1_000_000.
                )
            })
            .collect::<Vec<_>>()
            .join(" / ");
        let status = if let Some(job) = hovered {
            if let Some(status) = self.module_status(job.id) {
                status
            } else if job.work.completed {
                if job.blueprint.id == "internal-door" {
                    "COMPLETED | E open/close".to_string()
                } else {
                    "COMPLETED".to_string()
                }
            } else if job.work.stopped || job.funding.funded(&job.blueprint.requirements) {
                format!(
                    "{} | {:.1}%",
                    self.construction_status(job),
                    f64::from(job.funding.progress_ppm) / 10_000.
                )
            } else {
                format!("WAITING | supplied {:.3}kg", job.funding.mass_kg())
            }
        } else {
            p.rejection
                .map_or_else(|| "CLEAR".to_string(), |e| format!("{e:?}"))
        };
        Some(format!(
            "{} | {}\n{} | {}s (estimate)\n1-7 select | R rotate | LMB place | E feed | RMB cancel | T stop | G module | H secure | X dismantle | Del partition | C close",
            hovered.map_or_else(
                || blueprint.name.clone(),
                |job| format!("Project #{} {}", job.id.0, blueprint.name)
            ),
            status,
            resources,
            blueprint.duration_seconds
        ))
    }

    pub(in crate::session) fn construction_safety_record(
        &self,
    ) -> std::collections::BTreeMap<u64, String> {
        self.building
            .source
            .projects
            .jobs()
            .map(|p| (p.id.0, self.construction_status(p)))
            .collect()
    }
    pub(crate) fn construction_inspect(&self) -> serde_json::Value {
        let preview = self.construction_preview();
        serde_json::json!({"offline_clock":self.offline,"active":self.building.active,"modules":self.building.source.modules,"recovery_multiplier":self.utility_recovery_multiplier(),"approved_economy":self.building.catalog.approved_economy,"selected":self.building.catalog.blueprints[self.building.selected].id,"damage":self.building.source.damage,"structure":self.building.structure.save_state(),"volumes":self.structural_volumes(),"solids":self.structural_solids(),"installed_solids":self.construction_solids(),"projects":self.building.source.projects.jobs().collect::<Vec<_>>(),"status":self.building.source.projects.jobs().map(|p|(p.id.0.to_string(),self.construction_status(p))).collect::<std::collections::BTreeMap<_,_>>(),"aimed_project":self.aimed_construction().map(|id|id.0),"preview":preview.map(|p|serde_json::json!({"anchor":p.anchor_meters,"space":p.space,"cells":p.cells,"valid":p.rejection.is_none(),"feedback":p.rejection.map(|e|format!("{e:?}")),"requirements":p.blueprint.requirements,"duration_seconds":p.blueprint.duration_seconds}))})
    }
    pub(crate) fn construction_boxes(&self) -> Vec<(Envelope, [f32; 4])> {
        let mut boxes: Vec<_> = self
            .building
            .source
            .projects
            .jobs()
            .filter(|p| !p.work.completed && !p.work.destroyed)
            .map(|p| {
                let mut space = p.space;
                if p.funding.progress_ppm > 0 {
                    space.max[1] = space.min[1]
                        + (space.max[1] - space.min[1])
                            * (0.08 + 0.92 * f64::from(p.funding.progress_ppm) / 1_000_000.);
                }
                (
                    space,
                    if p.work.damaged {
                        [1., 0.15, 0.05, 1.]
                    } else if p.funding.progress_ppm > 0 {
                        [0.85, 0.65, 0.25, 1.]
                    } else {
                        [0.1, 0.7, 0.8, 1.]
                    },
                )
            })
            .collect();
        boxes.extend(
            self.building
                .structure
                .parts()
                .filter(|p| self.building.source.damage.damaged.contains(&p.id))
                .map(|p| (p.space, [1., 0.15, 0.05, 1.])),
        );
        boxes.extend(
            self.building
                .source
                .projects
                .jobs()
                .filter(|p| p.work.completed && !p.work.destroyed && p.work.damaged)
                .map(|p| (p.space, [1., 0.15, 0.05, 1.])),
        );
        if self.aimed_construction().is_none()
            && let Some(preview) = self.construction_preview()
        {
            boxes.push((
                preview.space,
                if preview.rejection.is_some() {
                    [1., 0.25, 0.1, 1.]
                } else {
                    [0.2, 1., 0.4, 1.]
                },
            ));
        }
        boxes
    }
}

#[cfg(test)]
pub(in crate::session) mod room_tests;
#[cfg(test)]
mod tests;

/// Explicit evidence setup: placed stool and one seeded carried iron sphere.
/// Full funding uses 1 kg; the earlier material-only route uses 0.5 kg.
pub(crate) fn evidence_initial_conditions(
    s: &mut GameplaySession,
    full: bool,
) -> Result<(), String> {
    use super::{Command, ControlKey};
    s.command(Command::Control(ControlKey::Right, true));
    s.advance_automation(std::time::Duration::from_millis(72 * 16));
    s.command(Command::Control(ControlKey::Right, false));
    let snapshot = s.snapshot();
    let frame = character_ship_frame(snapshot.ship.pose);
    let eye = frame.world_to_local(snapshot.character.eye_position_meters);
    let d = salimon_math::sub([-1.4, SHIP_FLOOR_HEIGHT_METERS, 3.2], eye);
    s.command(Command::Look(
        d[2].atan2(d[0]) / 0.0022,
        -d[1].atan2(d[0].hypot(d[2])) / 0.0022,
    ));
    s.command(Command::Build(BuildAction::Toggle));
    s.command(Command::Build(BuildAction::Place));
    s.building.catalog.approved_economy = true;
    use salimon_world::{resources::*, spherical_mining::*};
    let w = frame.local_to_world([-1., 1.3, 2.5]);
    let pose = ResourceTransform::new(
        salimon_world::WorldPosition::new(w[0], w[1], w[2]),
        [0., 0., 0., 1.],
    )
    .map_err(|e| e.to_string())?;
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: salimon_world::CelestialBodyId::Earth,
            local: 543,
        },
        RawMaterial::new(ResourceId::IronOre, if full { 1. } else { 0.5 })
            .map_err(|e| e.to_string())?,
        pose.position(),
        if full { 1. } else { 0.5 },
    )
    .map_err(|e| e.to_string())?;
    let id = (0..400)
        .find_map(|_| {
            s.mining.session.tick_spherical(
                true,
                Some(&mut deposit),
                SPHERE_TICK,
                MiningConfig::default(),
                pose,
            )
        })
        .ok_or("evidence sphere absent")?;
    if !s.mining.session.pick_up(id) {
        return Err("evidence carry occupied".into());
    }
    s.equipment.stow();
    s.sync_carried();
    Ok(())
}

pub(super) fn is_structural(p: &Placeholder) -> bool {
    installed::structural(p)
}

fn matches_catalog(job: &Placeholder) -> bool {
    let Ok(current) = registered_catalog().find(&job.blueprint.id) else {
        return false;
    };
    if current == &job.blueprint {
        return true;
    }
    // Preserve pre-18 two-cell reservations and receipts verbatim. Their unsafe
    // narrow staircase footprint cannot complete; cancel/replan rather than
    // silently enlarging an already reserved volume or changing its price.
    if job.blueprint.id == "two-deck-room" && !job.work.completed {
        let mut legacy = current.clone();
        legacy.cells = vec![Cell::new(0, 0, 0), Cell::new(0, 1, 0)];
        legacy.space.max[0] = 3.;
        return legacy == job.blueprint;
    }
    false
}

pub(in crate::session) use boarding::boarding_exit;
pub(crate) use evidence::{evidence_cancellation, evidence_room};
