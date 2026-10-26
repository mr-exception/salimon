//! Atomic installation of a room's individually editable structural cells.
use super::*;

/// One caller-allocated durable identity per cell; room identity is separate.
/// Faces and collision are supplied by the geometry owner, not inferred from a
/// bounding box (which could seal doors or fill a room with solid collision).
#[derive(Clone, Debug)]
pub struct RoomAssembly {
    pub room: Room,
    pub parts: Vec<StructuralPart>,
}

impl ShipStructure {
    /// Preflight the entire assembly in a private graph. Input order need not be
    /// support order. Failure publishes no cells, mass, room or retired IDs.
    pub fn install_room(&mut self, assembly: RoomAssembly) -> Result<(), StructureError> {
        let mut room_cells = assembly.room.cells.clone();
        normalize_cells(&mut room_cells)?;
        let mut part_cells = Vec::new();
        for part in &assembly.parts {
            // A completed room must retain one independently editable part per cell.
            if part.cells.len() != 1 {
                return Err(StructureError::InvalidGeometry);
            }
            part_cells.extend_from_slice(&part.cells);
        }
        part_cells.sort();
        if part_cells != room_cells {
            return Err(StructureError::InvalidGeometry);
        }
        let mut next = self.clone();
        // Validate all identities before topology so duplicate IDs never hide
        // behind a disconnected error or overwrite an earlier installation.
        let mut ids = BTreeSet::from([assembly.room.id]);
        next.check_id(assembly.room.id)?;
        for part in &assembly.parts {
            next.check_id(part.id)?;
            if !ids.insert(part.id) {
                return Err(StructureError::DuplicateIdentity(part.id));
            }
        }
        let mut pending = assembly.parts;
        while !pending.is_empty() {
            let index = pending
                .iter()
                .position(|p| {
                    p.cells
                        .iter()
                        .any(|c| neighbors(*c).any(|n| next.cells.contains_key(&n)))
                })
                .ok_or(StructureError::Disconnected)?;
            next.add_part(pending.remove(index))?;
        }
        next.add_room(assembly.room)?;
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(id: u64, cell: Cell) -> StructuralPart {
        let space = cell.bounds([0.; 3]).unwrap();
        StructuralPart {
            id: StructureId(id),
            cells: vec![cell],
            shape: MemberShape::CornerShell,
            faces: vec![Face {
                cell,
                direction: Direction::Down,
                kind: FaceKind::Floor,
            }],
            space,
            collision: vec![Envelope {
                min: space.min,
                max: [space.max[0], space.min[1] + 0.1, space.max[2]],
            }],
            mass: StructuralMass::AddedKilograms(10.),
        }
    }
    fn ship() -> ShipStructure {
        ShipStructure::new([0.; 3], part(1, Cell::new(0, 0, 0))).unwrap()
    }
    fn assembly() -> RoomAssembly {
        let cells = [
            Cell::new(1, 0, 0),
            Cell::new(1, 1, 0),
            Cell::new(2, 1, 0),
            Cell::new(2, 0, 0),
        ];
        RoomAssembly {
            room: Room {
                id: StructureId(10),
                cells: cells.to_vec(),
            },
            // Deliberately reverse support order.
            parts: cells
                .into_iter()
                .enumerate()
                .rev()
                .map(|(i, c)| part(2 + i as u64, c))
                .collect(),
        }
    }
    #[test]
    fn one_transaction_retains_four_cell_ids_mass_geometry_and_roundtrip() {
        let mut s = ship();
        s.install_room(assembly()).unwrap();
        assert_eq!(s.parts().count(), 5);
        assert_eq!(s.rooms().count(), 1);
        assert_eq!(s.mass_entries().map(|(_, kg)| kg).sum::<f64>(), 50.);
        let json = serde_json::to_string(&s.save_state()).unwrap();
        let restored = ShipStructure::from_save(serde_json::from_str(&json).unwrap()).unwrap();
        assert_eq!(s, restored);
        let cell = Cell::new(1, 1, 0);
        s.set_face(
            StructureId(3),
            cell,
            Direction::Down,
            Some(FaceKind::Opening),
        )
        .unwrap();
        assert_ne!(s, restored);
        assert_eq!(
            s.parts().find(|p| p.id == StructureId(2)),
            restored.parts().find(|p| p.id == StructureId(2))
        );
        let before = s.clone();
        assert!(s.install_room(assembly()).is_err());
        assert_eq!(s, before);
        let damage = s.damage_part(StructureId(3)).unwrap();
        assert_eq!(damage.destroyed.cells, vec![cell]);
        assert!(s.occupied_cells().any(|(c, _)| c == Cell::new(2, 0, 0)));
    }
    #[test]
    fn late_invalid_geometry_and_duplicate_ids_publish_nothing() {
        let mut s = ship();
        let before = s.clone();
        for case in 0..8 {
            let mut a = assembly();
            match case {
                0 => a.parts[0].mass = StructuralMass::AddedKilograms(f64::NAN),
                1 => a.parts[0].id = a.parts[1].id,
                2 => a.room.cells.pop().map(|_| ()).unwrap(),
                3 => a.parts[0].cells.push(Cell::new(8, 0, 0)),
                4 => a.room.id = StructureId(1),
                5 => a.parts[0].collision[0].max[0] = f64::INFINITY,
                6 => a.parts[0].faces[0].cell = Cell::new(99, 0, 0),
                _ => {
                    for p in &mut a.parts {
                        p.mass = StructuralMass::AddedKilograms(f64::MAX);
                    }
                }
            }
            assert!(s.install_room(a).is_err());
            assert_eq!(s, before);
        }
    }

    #[test]
    fn occupied_cells_and_retired_id_cannot_be_reused_by_an_assembly() {
        let mut s = ship();
        s.add_part(part(20, Cell::new(1, 0, 0))).unwrap();
        let before = s.clone();
        assert!(s.install_room(assembly()).is_err());
        assert_eq!(s, before);
        s.remove_part(StructureId(20)).unwrap();
        let before = s.clone();
        let mut a = assembly();
        a.parts[0].id = StructureId(20);
        assert!(s.install_room(a).is_err());
        assert_eq!(s, before);
        // A rejected transaction did not reserve the other proposed IDs.
        s.install_room(assembly()).unwrap();
    }

    #[test]
    fn detached_and_corner_only_rooms_are_rejected_atomically() {
        for cell in [Cell::new(1, 1, 0), Cell::new(9, 0, 0)] {
            let mut s = ship();
            let before = s.clone();
            let a = RoomAssembly {
                room: Room {
                    id: StructureId(10),
                    cells: vec![cell],
                },
                parts: vec![part(2, cell)],
            };
            assert_eq!(s.install_room(a), Err(StructureError::Disconnected));
            assert_eq!(s, before);
        }
    }
}
