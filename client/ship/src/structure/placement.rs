//! Authoritative blueprint reservations. A placeholder is the project, never a solid part.
use super::{Cell, Direction, Envelope, ShipStructure, StructureError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Furniture,
    Wall,
    Door,
    Hull,
    Deck,
    Room,
    Module,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Snap {
    Structural,
    Fine,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Footprint {
    Interior,
    Structural,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub resource: salimon_world::resources::ResourceId,
    /// Exact requested mass; later intake owns physical identity and rounding.
    pub milligrams: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Blueprint {
    pub id: String,
    pub name: String,
    pub category: Category,
    pub snap: Snap,
    /// Fine pitch is catalog data, not a mandatory ship-wide furnishing dimension.
    pub fine_pitch_meters: f64,
    pub footprint: Footprint,
    pub cells: Vec<Cell>,
    /// Relative to the snapped anchor, in ship-local meters.
    pub space: Envelope,
    pub requirements: Vec<Requirement>,
    pub duration_seconds: u32,
    /// Consequences, not category names, determine flight safety.
    pub hull_affecting: bool,
    pub flight_eligible: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema: u32,
    /// Proposed catalogs permit preview/reservation, never authorize material spending.
    pub approved_economy: bool,
    pub blueprints: Vec<Blueprint>,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PlaceholderId(pub u64);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Rotation {
    Forward,
    Port,
    Aft,
    Starboard,
}
impl Rotation {
    pub fn next(self) -> Self {
        match self {
            Self::Forward => Self::Port,
            Self::Port => Self::Aft,
            Self::Aft => Self::Starboard,
            Self::Starboard => Self::Forward,
        }
    }
    fn point(self, [x, y, z]: [f64; 3]) -> [f64; 3] {
        match self {
            Self::Forward => [x, y, z],
            Self::Port => [-z, y, x],
            Self::Aft => [-x, y, -z],
            Self::Starboard => [z, y, -x],
        }
    }
    pub fn cell(self, cell: Cell) -> Option<Cell> {
        let negative = |v: i32| (-1_i32).checked_sub(v);
        Some(match self {
            Self::Forward => cell,
            Self::Port => Cell::new(negative(cell.z)?, cell.y, cell.x),
            Self::Aft => Cell::new(negative(cell.x)?, cell.y, negative(cell.z)?),
            Self::Starboard => Cell::new(cell.z, cell.y, negative(cell.x)?),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placeholder {
    #[serde(default)]
    pub work: super::construction::Work,
    #[serde(default)]
    pub funding: super::funding::Funding,
    pub id: PlaceholderId,
    pub owner: u64,
    pub blueprint: Blueprint,
    pub anchor_meters: [f64; 3],
    pub rotation: Rotation,
    pub origin_meters: [f64; 3],
    pub cells: Vec<Cell>,
    pub space: Envelope,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlightPermission {
    Landed,
    Docked,
    Airborne,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObstacleKind {
    Installed,
    Player,
    Cargo,
    ProtectedRoute,
}
#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub kind: ObstacleKind,
    pub space: Envelope,
}
/// Supplied by trusted session adapters from current authority, never renderer state.
pub struct PlacementContext<'a> {
    pub actor: u64,
    pub owner: u64,
    pub flight: FlightPermission,
    pub eye_meters: [f64; 3],
    pub reach_meters: f64,
    pub reachable: bool,
    pub physical_support: bool,
    pub obstacles: &'a [Obstacle],
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlacementError {
    InvalidCatalog,
    InvalidGeometry,
    InvalidIdentity,
    UnknownBlueprint,
    Unauthorized,
    OutOfReach,
    Unreachable,
    FlightRestricted,
    Unsupported,
    Occupied,
    Blocked(ObstacleKind),
    Overlapping(PlaceholderId),
    MissingPlaceholder,
    IdentityExhausted,
}
impl std::fmt::Display for PlacementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "blueprint placement: {self:?}")
    }
}
impl std::error::Error for PlacementError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Preview {
    pub blueprint: Blueprint,
    pub anchor_meters: [f64; 3],
    pub rotation: Rotation,
    pub cells: Vec<Cell>,
    pub space: Envelope,
    pub rejection: Option<PlacementError>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Placements {
    schema: u32,
    next_id: u64,
    #[serde(with = "project_records")]
    jobs: BTreeMap<PlaceholderId, Placeholder>,
}
mod project_records {
    use super::*;
    pub fn serialize<S: serde::Serializer>(
        jobs: &BTreeMap<PlaceholderId, Placeholder>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(jobs.values())
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<PlaceholderId, Placeholder>, D::Error> {
        let records = Vec::<Placeholder>::deserialize(deserializer)?;
        let mut jobs = BTreeMap::new();
        for record in records {
            if jobs.insert(record.id, record).is_some() {
                return Err(serde::de::Error::custom("duplicate placeholder identity"));
            }
        }
        Ok(jobs)
    }
}

impl<'de> Deserialize<'de> for Placements {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Source {
            schema: u32,
            next_id: u64,
            #[serde(with = "project_records")]
            jobs: BTreeMap<PlaceholderId, Placeholder>,
        }
        let source = Source::deserialize(deserializer)?;
        let result = Self {
            schema: source.schema,
            next_id: source.next_id,
            jobs: source.jobs,
        };
        result.validate_save().map_err(serde::de::Error::custom)?;
        Ok(result)
    }
}

impl Default for Placements {
    fn default() -> Self {
        Self {
            schema: 1,
            next_id: 1,
            jobs: BTreeMap::new(),
        }
    }
}

impl Catalog {
    pub fn validate(&self) -> Result<(), PlacementError> {
        if self.schema != 1 || self.blueprints.is_empty() {
            return Err(PlacementError::InvalidCatalog);
        }
        let mut ids = BTreeSet::new();
        for blueprint in &self.blueprints {
            blueprint.validate()?;
            if !ids.insert(&blueprint.id) {
                return Err(PlacementError::InvalidCatalog);
            }
        }
        Ok(())
    }
    pub fn find(&self, id: &str) -> Result<&Blueprint, PlacementError> {
        self.validate()?;
        self.blueprints
            .iter()
            .find(|b| b.id == id)
            .ok_or(PlacementError::UnknownBlueprint)
    }
}
impl Blueprint {
    fn validate(&self) -> Result<(), PlacementError> {
        let mut cells = self.cells.clone();
        if self.id.is_empty()
            || self.name.is_empty()
            || !self.fine_pitch_meters.is_finite()
            || self.fine_pitch_meters <= 0.
            || self.duration_seconds == 0
            || self.duration_seconds > 300
            || self.space.validate().is_err()
            || super::normalize_cells(&mut cells).is_err()
            || (self.footprint == Footprint::Structural
                && (!self.hull_affecting || self.snap != Snap::Structural))
            || (self.hull_affecting && self.flight_eligible)
        {
            return Err(PlacementError::InvalidCatalog);
        }
        let mut resources = BTreeSet::new();
        if self.requirements.is_empty()
            || self
                .requirements
                .iter()
                .any(|r| r.milligrams == 0 || !resources.insert(r.resource.key()))
        {
            return Err(PlacementError::InvalidCatalog);
        }
        // The declared cells must cover the entire target volume, including multi-deck rooms.
        let actual = covered_cells(self.space, [0.; 3], supplied_limit(self.cells.len()))?;
        let supplied: BTreeSet<_> = cells.into_iter().collect();
        if actual.into_iter().collect::<BTreeSet<_>>() != supplied {
            return Err(PlacementError::InvalidCatalog);
        }
        Ok(())
    }
}
// Rotating and offsetting an authored rectangle can add at most one boundary
// cell per axis. Bound coordinate iteration by its actual authored footprint,
// rather than allowing malformed giant coordinate ranges to allocate unbounded work.
fn supplied_limit(count: usize) -> usize {
    count.saturating_mul(8)
}
fn covered_cells(
    space: Envelope,
    origin: [f64; 3],
    limit: usize,
) -> Result<Vec<Cell>, PlacementError> {
    space
        .validate()
        .map_err(|_| PlacementError::InvalidGeometry)?;
    let low = Cell::containing(space.min, origin).map_err(|_| PlacementError::InvalidGeometry)?;
    let mut hi = [0; 3];
    for i in 0..3 {
        let value = ((space.max[i] - origin[i]) / super::CELL_SIZE_METERS).ceil() - 1.;
        if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
            return Err(PlacementError::InvalidGeometry);
        }
        hi[i] = value as i32;
    }
    let count = (i64::from(hi[0]) - i64::from(low.x) + 1)
        .checked_mul(i64::from(hi[1]) - i64::from(low.y) + 1)
        .and_then(|n| n.checked_mul(i64::from(hi[2]) - i64::from(low.z) + 1));
    if count.is_none_or(|n| n <= 0 || usize::try_from(n).map_or(true, |n| n > limit)) {
        return Err(PlacementError::InvalidGeometry);
    }
    let mut result = Vec::new();
    for x in low.x..=hi[0] {
        for y in low.y..=hi[1] {
            for z in low.z..=hi[2] {
                result.push(Cell::new(x, y, z));
            }
        }
    }
    Ok(result)
}
fn intersects(a: Envelope, b: Envelope) -> bool {
    (0..3).all(|i| a.min[i] < b.max[i] && a.max[i] > b.min[i])
}
fn rotated_space(
    space: Envelope,
    rotation: Rotation,
    anchor: [f64; 3],
) -> Result<Envelope, PlacementError> {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for x in [space.min[0], space.max[0]] {
        for y in [space.min[1], space.max[1]] {
            for z in [space.min[2], space.max[2]] {
                let p = rotation.point([x, y, z]);
                for i in 0..3 {
                    min[i] = min[i].min(p[i] + anchor[i]);
                    max[i] = max[i].max(p[i] + anchor[i]);
                }
            }
        }
    }
    let result = Envelope { min, max };
    result
        .validate()
        .map_err(|_| PlacementError::InvalidGeometry)?;
    Ok(result)
}
impl Placements {
    pub fn jobs(&self) -> impl Iterator<Item = &Placeholder> {
        self.jobs.values()
    }
    pub fn inspect(&self, id: PlaceholderId) -> Option<&Placeholder> {
        self.jobs.get(&id)
    }
    pub fn preview(
        &self,
        catalog: &Catalog,
        id: &str,
        target: [f64; 3],
        rotation: Rotation,
        structure: &ShipStructure,
        context: &PlacementContext<'_>,
    ) -> Result<Preview, PlacementError> {
        let blueprint = catalog.find(id)?.clone();
        if target.iter().any(|v| !v.is_finite()) {
            return Err(PlacementError::InvalidGeometry);
        }
        let origin = structure.origin_meters();
        let pitch = if blueprint.snap == Snap::Structural {
            super::CELL_SIZE_METERS
        } else {
            blueprint.fine_pitch_meters
        };
        let mut anchor =
            std::array::from_fn(|i| origin[i] + ((target[i] - origin[i]) / pitch).floor() * pitch);
        // The trusted host supplies an actual completed support elevation;
        // horizontal fine snapping must not bury furniture in a new deck.
        if blueprint.footprint == Footprint::Interior {
            anchor[1] = target[1];
        }
        let space = rotated_space(blueprint.space, rotation, anchor)?;
        let cells = covered_cells(space, origin, supplied_limit(blueprint.cells.len()))?;
        let mut preview = Preview {
            blueprint,
            anchor_meters: anchor,
            rotation,
            cells,
            space,
            rejection: None,
        };
        preview.rejection = self
            .validate_target(&preview, structure, context, None, false)
            .err();
        Ok(preview)
    }
    fn validate_target(
        &self,
        preview: &Preview,
        structure: &ShipStructure,
        c: &PlacementContext<'_>,
        ignore: Option<PlaceholderId>,
        completion: bool,
    ) -> Result<(), PlacementError> {
        if c.actor == 0 || c.owner == 0 || c.actor != c.owner {
            return Err(PlacementError::Unauthorized);
        }
        if c.eye_meters.iter().any(|v| !v.is_finite())
            || !c.reach_meters.is_finite()
            || c.reach_meters <= 0.
        {
            return Err(PlacementError::InvalidGeometry);
        }
        let distance_squared: f64 = (0..3)
            .map(|i| {
                let near = c.eye_meters[i].clamp(preview.space.min[i], preview.space.max[i]);
                (near - c.eye_meters[i]).powi(2)
            })
            .sum();
        if !distance_squared.is_finite()
            || (!completion && distance_squared > c.reach_meters.powi(2))
        {
            return Err(PlacementError::OutOfReach);
        }
        if !c.reachable {
            return Err(PlacementError::Unreachable);
        }
        if !c.physical_support {
            return Err(PlacementError::Unsupported);
        }
        if c.flight == FlightPermission::Airborne
            && (preview.blueprint.hull_affecting || !preview.blueprint.flight_eligible)
        {
            return Err(PlacementError::FlightRestricted);
        }
        let occupied: BTreeSet<_> = structure.occupied_cells().map(|(cell, _)| cell).collect();
        if preview.blueprint.footprint == Footprint::Interior {
            if preview.cells.iter().any(|cell| !occupied.contains(cell)) {
                return Err(PlacementError::Unsupported);
            }
        } else {
            if preview.cells.iter().any(|cell| occupied.contains(cell)) {
                return Err(PlacementError::Occupied);
            }
            if !preview.cells.iter().any(|cell| {
                [
                    Direction::Forward,
                    Direction::Aft,
                    Direction::Up,
                    Direction::Down,
                    Direction::Port,
                    Direction::Starboard,
                ]
                .into_iter()
                .any(|direction| {
                    cell.neighbor(direction)
                        .is_some_and(|n| occupied.contains(&n))
                })
            }) {
                return Err(PlacementError::Unsupported);
            }
        }
        for obstacle in c.obstacles {
            obstacle
                .space
                .validate()
                .map_err(|_| PlacementError::InvalidGeometry)?;
            if intersects(preview.space, obstacle.space) {
                return Err(PlacementError::Blocked(obstacle.kind));
            }
        }
        for job in self.jobs.values() {
            if !job.work.destroyed
                && Some(job.id) != ignore
                && !(preview.blueprint.footprint == Footprint::Interior
                    && job.work.completed
                    && job.blueprint.footprint == Footprint::Structural)
                && (intersects(preview.space, job.space)
                    || (preview.blueprint.footprint == Footprint::Structural
                        && job.blueprint.footprint == Footprint::Structural
                        && preview.cells.iter().any(|c| job.cells.contains(c))))
            {
                return Err(PlacementError::Overlapping(job.id));
            }
        }
        Ok(())
    }
    /// Recompute every admission condition from current authority; never trust a renderer preview.
    pub fn place(
        &mut self,
        catalog: &Catalog,
        id: &str,
        target: [f64; 3],
        rotation: Rotation,
        structure: &ShipStructure,
        c: &PlacementContext<'_>,
    ) -> Result<PlaceholderId, PlacementError> {
        let preview = self.preview(catalog, id, target, rotation, structure, c)?;
        if let Some(error) = preview.rejection {
            return Err(error);
        }
        let next = self
            .next_id
            .checked_add(1)
            .ok_or(PlacementError::IdentityExhausted)?;
        let id = PlaceholderId(self.next_id);
        self.jobs.insert(
            id,
            Placeholder {
                work: Default::default(),
                funding: Default::default(),
                id,
                owner: c.owner,
                blueprint: preview.blueprint,
                anchor_meters: preview.anchor_meters,
                rotation,
                origin_meters: structure.origin_meters(),
                cells: preview.cells,
                space: preview.space,
            },
        );
        self.next_id = next;
        Ok(id)
    }
    pub fn funding_mut(
        &mut self,
        id: PlaceholderId,
        actor: u64,
    ) -> Result<&mut Placeholder, PlacementError> {
        let p = self
            .jobs
            .get_mut(&id)
            .ok_or(PlacementError::MissingPlaceholder)?;
        if actor == 0 || p.owner != actor {
            return Err(PlacementError::Unauthorized);
        }
        Ok(p)
    }
    pub fn cancel(&mut self, id: PlaceholderId, actor: u64) -> Result<Placeholder, PlacementError> {
        let job = self
            .jobs
            .get(&id)
            .ok_or(PlacementError::MissingPlaceholder)?;
        if actor == 0 || actor != job.owner {
            return Err(PlacementError::Unauthorized);
        }
        if !job.funding.receipts.is_empty() || job.funding.progress_ppm != 0 {
            return Err(PlacementError::Occupied);
        }
        self.jobs
            .remove(&id)
            .ok_or(PlacementError::MissingPlaceholder)
    }
    /// Completion is a separate physical transaction; validate just before installing solids.
    pub fn validate_completion(
        &self,
        id: PlaceholderId,
        structure: &ShipStructure,
        c: &PlacementContext<'_>,
    ) -> Result<(), PlacementError> {
        let p = self
            .jobs
            .get(&id)
            .ok_or(PlacementError::MissingPlaceholder)?;
        if c.actor != p.owner {
            return Err(PlacementError::Unauthorized);
        }
        self.validate_target(
            &Preview {
                blueprint: p.blueprint.clone(),
                anchor_meters: p.anchor_meters,
                rotation: p.rotation,
                cells: p.cells.clone(),
                space: p.space,
                rejection: None,
            },
            structure,
            c,
            Some(id),
            true,
        )
    }
    pub fn validate_save(&self) -> Result<(), PlacementError> {
        if self.schema != 1 || self.next_id == 0 {
            return Err(PlacementError::InvalidIdentity);
        }
        for (id, p) in &self.jobs {
            p.blueprint.validate()?;
            p.funding
                .validate(&p.blueprint.requirements)
                .map_err(|_| PlacementError::InvalidIdentity)?;
            p.work
                .validate(&p.funding, &p.blueprint)
                .map_err(|_| PlacementError::InvalidIdentity)?;
            if *id != p.id || id.0 == 0 || id.0 >= self.next_id || p.owner == 0 {
                return Err(PlacementError::InvalidIdentity);
            }
            let expected = rotated_space(p.blueprint.space, p.rotation, p.anchor_meters)?;
            if p.space != expected
                || p.cells
                    != covered_cells(
                        p.space,
                        p.origin_meters,
                        supplied_limit(p.blueprint.cells.len()),
                    )?
            {
                return Err(PlacementError::InvalidGeometry);
            }
            let mut cells = p.cells.clone();
            super::normalize_cells(&mut cells).map_err(|_| PlacementError::InvalidGeometry)?;
        }
        let jobs: Vec<_> = self.jobs.values().collect();
        for (i, a) in jobs.iter().enumerate() {
            for b in &jobs[i + 1..] {
                let furnishing_in_room = (a.work.completed
                    && a.blueprint.footprint == Footprint::Structural
                    && b.blueprint.footprint == Footprint::Interior)
                    || (b.work.completed
                        && b.blueprint.footprint == Footprint::Structural
                        && a.blueprint.footprint == Footprint::Interior);
                if !a.work.destroyed
                    && !b.work.destroyed
                    && intersects(a.space, b.space)
                    && !furnishing_in_room
                {
                    return Err(PlacementError::Overlapping(b.id));
                }
            }
        }
        Ok(())
    }
}
impl From<StructureError> for PlacementError {
    fn from(_: StructureError) -> Self {
        Self::InvalidGeometry
    }
}

#[cfg(test)]
mod tests;
