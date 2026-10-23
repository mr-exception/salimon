//! Independent domain schema; whole-game publication/migration belongs to #550.
use super::*;
use salimon_persistence::{SaveError, Validate, invalid};

pub const STRUCTURE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructureSaveState {
    pub(super) schema_version: u32,
    pub(super) core: StructureId,
    pub(super) origin_meters: [f64; 3],
    pub(super) parts: Vec<StructuralPart>,
    pub(super) rooms: Vec<Room>,
    pub(super) equipment: Vec<EquipmentFootprint>,
    pub(super) retired: Vec<StructureId>,
}
impl Validate for StructureSaveState {
    fn validate(&self) -> Result<(), SaveError> {
        ShipStructure::from_save(self.clone())
            .map(|_| ())
            .map_err(|e| invalid(e.to_string()))
    }
}
impl ShipStructure {
    pub fn save_state(&self) -> StructureSaveState {
        StructureSaveState {
            schema_version: STRUCTURE_SCHEMA_VERSION,
            core: self.core,
            origin_meters: self.origin_meters,
            parts: self.parts.values().cloned().collect(),
            rooms: self.rooms.values().cloned().collect(),
            equipment: self.equipment.values().cloned().collect(),
            retired: self.retired.iter().copied().collect(),
        }
    }

    /// Validate into a temporary graph and rebuild indexes; never trust saved caches.
    pub fn from_save(mut state: StructureSaveState) -> Result<Self, StructureError> {
        if state.schema_version != STRUCTURE_SCHEMA_VERSION {
            return Err(StructureError::UnsupportedVersion(state.schema_version));
        }
        let core_index = state
            .parts
            .iter()
            .position(|p| p.id == state.core)
            .ok_or(StructureError::MissingIdentity(state.core))?;
        let core = state.parts.remove(core_index);
        let mut structure = Self::new(state.origin_meters, core)?;
        // Saved IDs are not topological order. Admit any reachable part each pass;
        // no-progress means unsupported structure, not a silently dropped assembly.
        while !state.parts.is_empty() {
            let index = state.parts.iter().position(|p| {
                p.cells
                    .iter()
                    .any(|c| neighbors(*c).any(|n| structure.cells.contains_key(&n)))
            });
            let Some(index) = index else {
                return Err(StructureError::Disconnected);
            };
            structure.add_part(state.parts.remove(index))?;
        }
        for room in state.rooms {
            structure.add_room(room)?;
        }
        for equipment in state.equipment {
            structure.add_equipment(equipment)?;
        }
        for id in state.retired {
            structure.check_id(id)?;
            structure.retired.insert(id);
        }
        Ok(structure)
    }
}
