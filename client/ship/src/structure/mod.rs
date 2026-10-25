//! Portable structural authority for #541. No placement UI, mesh, traversal or power graph.
mod assembly;
pub mod construction;
pub use assembly::RoomAssembly;
pub mod funding;
mod grid;
mod persistence;
pub mod placement;
#[cfg(test)]
mod tests;

pub use grid::{CELL_SIZE_METERS, Cell, Direction, Envelope};
pub use persistence::{STRUCTURE_SCHEMA_VERSION, StructureSaveState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct StructureId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructureError {
    InvalidIdentity,
    DuplicateIdentity(StructureId),
    MissingIdentity(StructureId),
    InvalidGeometry,
    InvalidMass,
    EmptyFootprint,
    DuplicateCell(Cell),
    OccupiedCell(Cell),
    MissingCell(Cell),
    Disconnected,
    CoreRemoval,
    ReferencedStructure,
    InvalidFace,
    DuplicateFace,
    UnsupportedVersion(u32),
}
impl std::fmt::Display for StructureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ship structure: {self:?}")
    }
}
impl std::error::Error for StructureError {}

/// Geometry hints, deliberately independent of renderer mesh/material identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberShape {
    Frame,
    FullShell,
    TaperedShell,
    CornerShell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaceKind {
    Wall,
    Floor,
    Ceiling,
    Opening,
    Doorway,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Face {
    pub cell: Cell,
    pub direction: Direction,
    pub kind: FaceKind,
}

/// Existing authored structure is already in dry hull mass. Only new installations add mass.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum StructuralMass {
    IncludedInDryHull,
    AddedKilograms(f64),
}
impl StructuralMass {
    pub fn additional_kg(self) -> Option<f64> {
        match self {
            Self::IncludedInDryHull => None,
            Self::AddedKilograms(kg) => Some(kg),
        }
    }
    fn validate(self) -> Result<(), StructureError> {
        match self {
            Self::IncludedInDryHull => Ok(()),
            Self::AddedKilograms(kg) => validate_mass(kg),
        }
    }
}

/// One durable assembly can span many cells/decks. Mass includes its faces and members once.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralPart {
    pub id: StructureId,
    pub cells: Vec<Cell>,
    pub shape: MemberShape,
    pub faces: Vec<Face>,
    pub space: Envelope,
    pub collision: Vec<Envelope>,
    pub mass: StructuralMass,
}

/// A room groups existing frame cells, adding neither occupied cells nor mass.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Room {
    pub id: StructureId,
    pub cells: Vec<Cell>,
}

/// Equipment occupies space inside supporting cells without replacing their structure.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentFootprint {
    pub id: StructureId,
    pub cells: Vec<Cell>,
    pub space: Envelope,
    pub collision: Vec<Envelope>,
    pub mass: StructuralMass,
}

/// Detached durable data, not a second ship/Core or a simulated physical wreck.
#[derive(Clone, Debug, PartialEq)]
pub struct DetachedStructure {
    pub parts: Vec<StructuralPart>,
    pub rooms: Vec<Room>,
    pub equipment: Vec<EquipmentFootprint>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageOutcome {
    pub destroyed: StructuralPart,
    pub detached: Vec<DetachedStructure>,
    /// Unsupported equipment is handed to the future damage/physical-salvage owner.
    pub unsupported_equipment: Vec<EquipmentFootprint>,
    /// Room metadata crossing destroyed/detached boundaries is invalidated, not duplicated.
    pub invalidated_rooms: Vec<Room>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShipStructure {
    core: StructureId,
    origin_meters: [f64; 3],
    parts: BTreeMap<StructureId, StructuralPart>,
    rooms: BTreeMap<StructureId, Room>,
    equipment: BTreeMap<StructureId, EquipmentFootprint>,
    cells: BTreeMap<Cell, StructureId>,
    retired: BTreeSet<StructureId>,
}

impl ShipStructure {
    /// Origin is an explicit authored migration boundary, never inferred or rounded.
    /// Core-bearing part mass is structural delta only, not another 100 t starter hull.
    pub fn new(origin_meters: [f64; 3], core: StructuralPart) -> Result<Self, StructureError> {
        if origin_meters.iter().any(|v| !v.is_finite()) {
            return Err(StructureError::InvalidGeometry);
        }
        let mut structure = Self {
            core: core.id,
            origin_meters,
            parts: BTreeMap::new(),
            rooms: BTreeMap::new(),
            equipment: BTreeMap::new(),
            cells: BTreeMap::new(),
            retired: BTreeSet::new(),
        };
        structure.insert_part(core, true)?;
        Ok(structure)
    }
    pub fn core_id(&self) -> StructureId {
        self.core
    }
    pub fn origin_meters(&self) -> [f64; 3] {
        self.origin_meters
    }
    pub fn parts(&self) -> impl Iterator<Item = &StructuralPart> {
        self.parts.values()
    }
    pub fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.values()
    }
    pub fn equipment(&self) -> impl Iterator<Item = &EquipmentFootprint> {
        self.equipment.values()
    }
    pub fn occupied_cells(&self) -> impl Iterator<Item = (Cell, StructureId)> + '_ {
        self.cells.iter().map(|(cell, id)| (*cell, *id))
    }

    /// These disjoint entries are the sole installed mass contributions, not a second cached total.
    /// Runtime must transfer physical input ownership before installing them in ShipMass.
    pub fn mass_entries(&self) -> impl Iterator<Item = (crate::MassObjectId, f64)> + '_ {
        self.parts
            .values()
            .filter_map(|p| {
                p.mass
                    .additional_kg()
                    .map(|kg| (crate::MassObjectId::Structure(p.id.0), kg))
            })
            .chain(self.equipment.values().filter_map(|e| {
                e.mass
                    .additional_kg()
                    .map(|kg| (crate::MassObjectId::Structure(e.id.0), kg))
            }))
    }

    pub fn add_part(&mut self, part: StructuralPart) -> Result<(), StructureError> {
        self.insert_part(part, false)
    }

    fn insert_part(&mut self, mut part: StructuralPart, core: bool) -> Result<(), StructureError> {
        self.check_id(part.id)?;
        normalize_cells(&mut part.cells)?;
        validate_envelopes(&part.cells, self.origin_meters, part.space, &part.collision)?;
        part.mass.validate()?;
        self.validate_part_faces(&part)?;
        part.faces.sort_by_key(|f| (f.cell, f.direction));
        for cell in &part.cells {
            if self.cells.contains_key(cell) {
                return Err(StructureError::OccupiedCell(*cell));
            }
        }
        if !core
            && !part
                .cells
                .iter()
                .any(|c| neighbors(*c).any(|n| self.cells.contains_key(&n)))
        {
            return Err(StructureError::Disconnected);
        }
        // Verify combined mass stays representable before publishing any topology.
        let mut ledger = crate::ShipMass::default();
        for (id, mass) in self.mass_entries().chain(
            part.mass
                .additional_kg()
                .map(|kg| (crate::MassObjectId::Structure(part.id.0), kg)),
        ) {
            ledger
                .update(id, Some(mass))
                .map_err(|_| StructureError::InvalidMass)?;
        }
        for cell in &part.cells {
            self.cells.insert(*cell, part.id);
        }
        self.parts.insert(part.id, part);
        Ok(())
    }

    pub fn add_room(&mut self, mut room: Room) -> Result<(), StructureError> {
        self.check_id(room.id)?;
        normalize_cells(&mut room.cells)?;
        self.require_cells(&room.cells)?;
        if room
            .cells
            .iter()
            .any(|c| self.rooms.values().any(|r| r.cells.contains(c)))
        {
            return Err(StructureError::ReferencedStructure);
        }
        self.rooms.insert(room.id, room);
        Ok(())
    }
    pub fn remove_room(&mut self, id: StructureId) -> Result<Room, StructureError> {
        let room = self
            .rooms
            .remove(&id)
            .ok_or(StructureError::MissingIdentity(id))?;
        self.retired.insert(id);
        Ok(room)
    }
    pub fn add_equipment(
        &mut self,
        mut equipment: EquipmentFootprint,
    ) -> Result<(), StructureError> {
        self.check_id(equipment.id)?;
        normalize_cells(&mut equipment.cells)?;
        self.require_cells(&equipment.cells)?;
        validate_envelopes(
            &equipment.cells,
            self.origin_meters,
            equipment.space,
            &equipment.collision,
        )?;
        equipment.mass.validate()?;
        if let Some(cell) = equipment
            .cells
            .iter()
            .find(|c| self.equipment.values().any(|e| e.cells.contains(c)))
        {
            return Err(StructureError::OccupiedCell(*cell));
        }
        let mut ledger = crate::ShipMass::default();
        for (id, mass) in self.mass_entries().chain(
            equipment
                .mass
                .additional_kg()
                .map(|kg| (crate::MassObjectId::Structure(equipment.id.0), kg)),
        ) {
            ledger
                .update(id, Some(mass))
                .map_err(|_| StructureError::InvalidMass)?;
        }
        self.equipment.insert(equipment.id, equipment);
        Ok(())
    }
    pub fn remove_equipment(
        &mut self,
        id: StructureId,
    ) -> Result<EquipmentFootprint, StructureError> {
        let equipment = self
            .equipment
            .remove(&id)
            .ok_or(StructureError::MissingIdentity(id))?;
        self.retired.insert(id);
        Ok(equipment)
    }

    /// Removing a floor/ceiling/partition does not erase its supporting structural frame.
    /// A face's mass belongs to the containing part; this metadata edit does not grant salvage.
    pub fn set_face(
        &mut self,
        id: StructureId,
        cell: Cell,
        direction: Direction,
        kind: Option<FaceKind>,
    ) -> Result<(), StructureError> {
        let part = self
            .parts
            .get(&id)
            .ok_or(StructureError::MissingIdentity(id))?;
        if !part.cells.contains(&cell) {
            return Err(StructureError::MissingCell(cell));
        }
        let mut candidate = part.clone();
        candidate
            .faces
            .retain(|f| f.cell != cell || f.direction != direction);
        if let Some(kind) = kind {
            candidate.faces.push(Face {
                cell,
                direction,
                kind,
            });
        }
        self.validate_part_faces(&candidate)?;
        candidate.faces.sort_by_key(|f| (f.cell, f.direction));
        self.parts.insert(id, candidate);
        Ok(())
    }

    /// Reject a deliberate cut even if the removed assembly itself has several cells.
    pub fn remove_part(&mut self, id: StructureId) -> Result<StructuralPart, StructureError> {
        if id == self.core {
            return Err(StructureError::CoreRemoval);
        }
        let part = self
            .parts
            .get(&id)
            .ok_or(StructureError::MissingIdentity(id))?;
        if self
            .rooms
            .values()
            .any(|r| r.cells.iter().any(|c| part.cells.contains(c)))
            || self
                .equipment
                .values()
                .any(|e| e.cells.iter().any(|c| part.cells.contains(c)))
        {
            return Err(StructureError::ReferencedStructure);
        }
        let remaining: BTreeSet<_> = self
            .cells
            .iter()
            .filter(|(_, owner)| **owner != id)
            .map(|(c, _)| *c)
            .collect();
        if !connected(&remaining) {
            return Err(StructureError::Disconnected);
        }
        let part = self
            .parts
            .remove(&id)
            .ok_or(StructureError::MissingIdentity(id))?;
        for cell in &part.cells {
            self.cells.remove(cell);
        }
        self.retired.insert(id);
        Ok(part)
    }

    /// Damage may cut support. Transfer each disconnected component once, retaining IDs.
    /// Destroying the permanent Core is outside this API and remains explicitly rejected.
    pub fn damage_part(&mut self, id: StructureId) -> Result<DamageOutcome, StructureError> {
        if id == self.core {
            return Err(StructureError::CoreRemoval);
        }
        if !self.parts.contains_key(&id) {
            return Err(StructureError::MissingIdentity(id));
        }
        let mut next = self.clone();
        let destroyed = next
            .parts
            .remove(&id)
            .ok_or(StructureError::MissingIdentity(id))?;
        for cell in &destroyed.cells {
            next.cells.remove(cell);
        }
        next.retired.insert(id);
        let mut components = components(next.cells.keys().copied().collect());
        let core_cell = next.parts[&next.core].cells[0];
        let core_index = components
            .iter()
            .position(|c| c.contains(&core_cell))
            .ok_or(StructureError::Disconnected)?;
        let attached = components.remove(core_index);
        let mut outcome = DamageOutcome {
            destroyed,
            detached: Vec::new(),
            unsupported_equipment: Vec::new(),
            invalidated_rooms: Vec::new(),
        };
        let mut room_components = BTreeMap::<StructureId, usize>::new();
        for room in next.rooms.values() {
            if room.cells.iter().all(|c| attached.contains(c)) {
                continue;
            }
            if let Some(index) = components
                .iter()
                .position(|c| room.cells.iter().all(|v| c.contains(v)))
            {
                room_components.insert(room.id, index);
            } else {
                outcome.invalidated_rooms.push(room.clone());
            }
        }
        let mut equipment_components = BTreeMap::<StructureId, usize>::new();
        for equipment in next.equipment.values() {
            if equipment.cells.iter().all(|c| attached.contains(c)) {
                continue;
            }
            if let Some(index) = components
                .iter()
                .position(|c| equipment.cells.iter().all(|v| c.contains(v)))
            {
                equipment_components.insert(equipment.id, index);
            } else {
                outcome.unsupported_equipment.push(equipment.clone());
            }
        }
        for (index, component) in components.iter().enumerate() {
            let part_ids: BTreeSet<_> = component.iter().map(|c| next.cells[c]).collect();
            let mut detached = DetachedStructure {
                parts: Vec::new(),
                rooms: Vec::new(),
                equipment: Vec::new(),
            };
            for part_id in part_ids {
                let part = next
                    .parts
                    .remove(&part_id)
                    .ok_or(StructureError::MissingIdentity(part_id))?;
                for cell in &part.cells {
                    next.cells.remove(cell);
                }
                next.retired.insert(part_id);
                detached.parts.push(part);
            }
            for (&room_id, _) in room_components.iter().filter(|(_, v)| **v == index) {
                detached.rooms.push(
                    next.rooms
                        .remove(&room_id)
                        .ok_or(StructureError::MissingIdentity(room_id))?,
                );
                next.retired.insert(room_id);
            }
            for (&equipment_id, _) in equipment_components.iter().filter(|(_, v)| **v == index) {
                detached.equipment.push(
                    next.equipment
                        .remove(&equipment_id)
                        .ok_or(StructureError::MissingIdentity(equipment_id))?,
                );
                next.retired.insert(equipment_id);
            }
            outcome.detached.push(detached);
        }
        for room in &outcome.invalidated_rooms {
            next.rooms.remove(&room.id);
            next.retired.insert(room.id);
        }
        for equipment in &outcome.unsupported_equipment {
            next.equipment.remove(&equipment.id);
            next.retired.insert(equipment.id);
        }
        *self = next;
        Ok(outcome)
    }

    fn validate_part_faces(&self, candidate: &StructuralPart) -> Result<(), StructureError> {
        validate_faces(candidate)?;
        let existing: BTreeSet<_> = self
            .parts
            .values()
            .filter(|p| p.id != candidate.id)
            .flat_map(|p| p.faces.iter().map(face_key))
            .collect();
        if candidate
            .faces
            .iter()
            .any(|face| existing.contains(&face_key(face)))
        {
            return Err(StructureError::DuplicateFace);
        }
        Ok(())
    }

    fn check_id(&self, id: StructureId) -> Result<(), StructureError> {
        if id.0 == 0 {
            return Err(StructureError::InvalidIdentity);
        }
        if self.parts.contains_key(&id)
            || self.rooms.contains_key(&id)
            || self.equipment.contains_key(&id)
            || self.retired.contains(&id)
        {
            return Err(StructureError::DuplicateIdentity(id));
        }
        Ok(())
    }
    fn require_cells(&self, cells: &[Cell]) -> Result<(), StructureError> {
        for cell in cells {
            if !self.cells.contains_key(cell) {
                return Err(StructureError::MissingCell(*cell));
            }
        }
        Ok(())
    }
}

fn neighbors(cell: Cell) -> impl Iterator<Item = Cell> {
    Direction::ALL
        .into_iter()
        .filter_map(move |d| cell.neighbor(d))
}
fn normalize_cells(cells: &mut [Cell]) -> Result<(), StructureError> {
    if cells.is_empty() {
        return Err(StructureError::EmptyFootprint);
    }
    cells.sort();
    for pair in cells.windows(2) {
        if pair[0] == pair[1] {
            return Err(StructureError::DuplicateCell(pair[0]));
        }
    }
    if !connected(&cells.iter().copied().collect()) {
        return Err(StructureError::Disconnected);
    }
    Ok(())
}
fn connected(cells: &BTreeSet<Cell>) -> bool {
    let Some(first) = cells.first() else {
        return false;
    };
    reachable(*first, cells) == *cells
}
fn reachable(start: Cell, cells: &BTreeSet<Cell>) -> BTreeSet<Cell> {
    let mut seen = BTreeSet::from([start]);
    let mut pending = vec![start];
    while let Some(cell) = pending.pop() {
        for neighbor in neighbors(cell) {
            if cells.contains(&neighbor) && seen.insert(neighbor) {
                pending.push(neighbor);
            }
        }
    }
    seen
}
fn components(mut cells: BTreeSet<Cell>) -> Vec<BTreeSet<Cell>> {
    let mut result = Vec::new();
    while let Some(&start) = cells.first() {
        let component = reachable(start, &cells);
        for cell in &component {
            cells.remove(cell);
        }
        result.push(component);
    }
    result
}
fn validate_mass(mass: f64) -> Result<(), StructureError> {
    if !mass.is_finite() || mass <= 0.0 {
        Err(StructureError::InvalidMass)
    } else {
        Ok(())
    }
}
fn validate_envelopes(
    cells: &[Cell],
    origin: [f64; 3],
    space: Envelope,
    collision: &[Envelope],
) -> Result<(), StructureError> {
    space.validate()?;
    let bounds: Vec<_> = cells
        .iter()
        .map(|c| c.bounds(origin))
        .collect::<Result<_, _>>()?;
    // Require the supplied space to lie in the footprint's bounding envelope. Nonrectangular
    // footprints still need final mesh/occupancy admission in the placement owner (#542).
    for i in 0..3 {
        let min = bounds
            .iter()
            .map(|b| b.min[i])
            .fold(f64::INFINITY, f64::min);
        let max = bounds
            .iter()
            .map(|b| b.max[i])
            .fold(f64::NEG_INFINITY, f64::max);
        if space.min[i] < min || space.max[i] > max {
            return Err(StructureError::InvalidGeometry);
        }
    }
    for envelope in collision {
        envelope.validate()?;
        if (0..3).any(|i| envelope.min[i] < space.min[i] || envelope.max[i] > space.max[i]) {
            return Err(StructureError::InvalidGeometry);
        }
    }
    Ok(())
}
fn validate_faces(part: &StructuralPart) -> Result<(), StructureError> {
    let mut faces = BTreeSet::new();
    for face in &part.faces {
        if !part.cells.contains(&face.cell)
            || (face.kind == FaceKind::Floor && face.direction != Direction::Down)
            || (face.kind == FaceKind::Ceiling && face.direction != Direction::Up)
        {
            return Err(StructureError::InvalidFace);
        }
        if !faces.insert(face_key(face)) {
            return Err(StructureError::DuplicateFace);
        }
    }
    Ok(())
}

fn face_key(face: &Face) -> (Cell, Direction) {
    match face.cell.neighbor(face.direction) {
        Some(neighbor) if neighbor < face.cell => (neighbor, face.direction.opposite()),
        _ => (face.cell, face.direction),
    }
}
