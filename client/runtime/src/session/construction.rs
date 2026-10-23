//! Compose blueprint authority with current ship, player and physical cargo.
use super::{GameplaySession, frames::character_ship_frame};
use salimon_character::{CharacterLocation, SHIP_FLOOR_HEIGHT_METERS};
use salimon_ship::structure::placement::*;
use salimon_ship::structure::{
    Cell, Envelope, MemberShape, ShipStructure, StructuralMass, StructuralPart, StructureId,
};
use serde::{Deserialize, Serialize};

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
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConstructionSave {
    pub(crate) projects: Placements,
}
impl salimon_persistence::Validate for ConstructionSave {
    fn validate(&self) -> Result<(), salimon_persistence::SaveError> {
        self.projects
            .validate_save()
            .map_err(|e| salimon_persistence::invalid(e.to_string()))?;
        let structure = starter_structure();
        for job in self.projects.jobs() {
            if job.owner != OWNER
                || job.origin_meters != ORIGIN
                || registered_catalog().find(&job.blueprint.id).ok() != Some(&job.blueprint)
            {
                return Err(salimon_persistence::invalid(
                    "construction project catalog/owner/origin mismatch",
                ));
            }
            let obstacles = static_obstacles(job.blueprint.footprint);
            let context = PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: job.anchor_meters,
                reach_meters: REACH_METERS,
                reachable: true,
                physical_support: has_physical_support(job.blueprint.footprint, job.space),
                obstacles: &obstacles,
            };
            self.projects
                .validate_completion(job.id, &structure, &context)
                .map_err(|e| salimon_persistence::invalid(e.to_string()))?;
        }
        Ok(())
    }
}
pub(crate) struct Building {
    pub(crate) active: bool,
    selected: usize,
    rotation: Rotation,
    pub(crate) source: ConstructionSave,
    catalog: Catalog,
    structure: ShipStructure,
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
        Ok(Self {
            source,
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
            target[1] = SHIP_FLOOR_HEIGHT_METERS;
        }
        let mut obstacles = static_obstacles(blueprint.footprint);
        let player_bottom = eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS;
        obstacles.push(Obstacle {
            kind: ObstacleKind::Player,
            space: Envelope {
                min: [
                    eye[0] - salimon_character::PLAYER_RADIUS_METERS,
                    player_bottom,
                    eye[2] - salimon_character::PLAYER_RADIUS_METERS,
                ],
                max: [
                    eye[0] + salimon_character::PLAYER_RADIUS_METERS,
                    player_bottom + salimon_character::PLAYER_BODY_HEIGHT_METERS,
                    eye[2] + salimon_character::PLAYER_RADIUS_METERS,
                ],
            },
        });
        for piece in self.mining.session.fragments() {
            let local = frame.world_to_local(piece.transform().position().meters());
            let radius = crate::fragment_physics::sphere_radius(*piece);
            obstacles.push(Obstacle {
                kind: ObstacleKind::Cargo,
                space: Envelope {
                    min: local.map(|p| p - radius),
                    max: local.map(|p| p + radius),
                },
            });
        }
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
        let physical_support = preliminary
            .as_ref()
            .is_some_and(|p| has_physical_support(b.footprint, p.space));
        let reachable = salimon_character::ship_sight_obstruction(
            eye,
            target,
            super::door_passable(self.ship.snapshot()),
        )
        .is_none_or(|hit| hit + 0.02 >= salimon_math::length(salimon_math::sub(target, eye)));
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
        if matches!(action, BuildAction::Place)
            && !self.admit_survival_action(salimon_survival::restrictions::Action::Fabricate)
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
        if matches!(action, BuildAction::Cancel) {
            let nearest = self.aimed_construction();
            self.feedback = Some(if let Some(id) = nearest {
                if self.building.source.projects.cancel(id, OWNER).is_ok() {
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
        let blueprint = hovered.map_or(&p.blueprint, |job| &job.blueprint);
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
        let status = if hovered.is_some() {
            "PLANNED".to_string()
        } else {
            p.rejection
                .map_or_else(|| "CLEAR".to_string(), |e| format!("{e:?}"))
        };
        Some(format!(
            "{} | {}\n{} | {}s (estimate)\n1-7 choose | R rotate | LMB place | RMB cancel | C close",
            hovered.map_or_else(
                || blueprint.name.clone(),
                |job| format!("Project #{} {}", job.id.0, blueprint.name)
            ),
            status,
            resources,
            blueprint.duration_seconds
        ))
    }

    pub(crate) fn construction_inspect(&self) -> serde_json::Value {
        let preview = self.construction_preview();
        serde_json::json!({"active":self.building.active,"approved_economy":self.building.catalog.approved_economy,"selected":self.building.catalog.blueprints[self.building.selected].id,"projects":self.building.source.projects.jobs().collect::<Vec<_>>(),"aimed_project":self.aimed_construction().map(|id|id.0),"preview":preview.map(|p|serde_json::json!({"anchor":p.anchor_meters,"space":p.space,"cells":p.cells,"valid":p.rejection.is_none(),"feedback":p.rejection.map(|e|format!("{e:?}")),"requirements":p.blueprint.requirements,"duration_seconds":p.blueprint.duration_seconds}))})
    }
    pub(crate) fn construction_boxes(&self) -> Vec<(Envelope, [f32; 4])> {
        let mut boxes: Vec<_> = self
            .building
            .source
            .projects
            .jobs()
            .map(|p| (p.space, [0.1, 0.7, 0.8, 1.]))
            .collect();
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
mod tests;
