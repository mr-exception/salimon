use super::*;
use salimon_persistence::{SaveGame, Validate};
use std::time::Duration;

fn part(id: u64, cells: &[[i32; 3]]) -> StructuralPart {
    let cells: Vec<_> = cells.iter().map(|c| Cell::new(c[0], c[1], c[2])).collect();
    let bounds: Vec<_> = cells.iter().map(|c| c.bounds([0.; 3]).unwrap()).collect();
    let space = Envelope {
        min: std::array::from_fn(|i| {
            bounds
                .iter()
                .map(|b| b.min[i])
                .fold(f64::INFINITY, f64::min)
        }),
        max: std::array::from_fn(|i| {
            bounds
                .iter()
                .map(|b| b.max[i])
                .fold(f64::NEG_INFINITY, f64::max)
        }),
    };
    StructuralPart {
        id: StructureId(id),
        cells,
        shape: MemberShape::Frame,
        faces: vec![],
        space,
        collision: vec![space],
        mass: StructuralMass::AddedKilograms(12.5),
    }
}
fn ship() -> ShipStructure {
    ShipStructure::new([0.; 3], part(1, &[[0, 0, 0]])).unwrap()
}
fn equipment(id: u64, cells: &[[i32; 3]]) -> EquipmentFootprint {
    let p = part(id, cells);
    EquipmentFootprint {
        id: p.id,
        cells: p.cells,
        space: p.space,
        collision: p.collision,
        mass: StructuralMass::AddedKilograms(40.),
    }
}

#[test]
fn all_six_faces_attach_but_edges_and_corners_do_not() {
    for x in -1_i32..=1 {
        for y in -1_i32..=1 {
            for z in -1_i32..=1 {
                if [x, y, z] == [0, 0, 0] {
                    continue;
                }
                let mut state = ship();
                let before = state.clone();
                let result = state.add_part(part(2, &[[x, y, z]]));
                assert_eq!(
                    result.is_ok(),
                    x.abs() + y.abs() + z.abs() == 1,
                    "{x},{y},{z}"
                );
                if result.is_err() {
                    assert_eq!(state, before);
                }
            }
        }
    }
}

#[test]
fn grid_queries_preserve_precision_and_handle_negative_and_extreme_coordinates() {
    let origin = [0.125, -2.75, 8.03125];
    let c = Cell::new(-2, 3, -1);
    let bounds = c.bounds(origin).unwrap();
    assert_eq!(bounds.min, [-5.875, 6.25, 5.03125]);
    assert_eq!(Cell::containing(bounds.min, origin), Ok(c));
    assert_eq!(
        Cell::containing(bounds.max, origin),
        Ok(Cell::new(-1, 4, 0))
    );
    assert_eq!(
        Cell::containing([-0.01, 0., 0.], [0.; 3]),
        Ok(Cell::new(-1, 0, 0))
    );
    assert_eq!(Cell::new(i32::MAX, 0, 0).neighbor(Direction::Forward), None);
    assert_eq!(Cell::new(i32::MIN, 0, 0).neighbor(Direction::Aft), None);
    let extreme = Cell::new(i32::MAX, i32::MIN, 0);
    assert_eq!(
        Cell::containing(extreme.bounds([0.; 3]).unwrap().min, [0.; 3]),
        Ok(extreme)
    );
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
        assert!(Cell::containing([invalid, 0., 0.], [0.; 3]).is_err());
        assert!(Cell::new(0, 0, 0).bounds([invalid, 0., 0.]).is_err());
    }
    let mut core = part(1, &[[0, 0, 0]]);
    core.space = Cell::new(0, 0, 0).bounds(origin).unwrap();
    core.collision = vec![core.space];
    assert_eq!(
        ShipStructure::new(origin, core).unwrap().origin_meters(),
        origin
    );
}

#[test]
fn multi_deck_large_footprints_rooms_and_equipment_have_distinct_ownership() {
    let mut s = ship();
    let cells: Vec<_> = (1..=6)
        .flat_map(|x| (0..=2).flat_map(move |y| (-1..=1).map(move |z| [x, y, z])))
        .collect();
    let mut large = part(2, &cells);
    large.shape = MemberShape::TaperedShell;
    s.add_part(large).unwrap();
    s.add_room(Room {
        id: StructureId(3),
        cells: vec![Cell::new(1, 0, 0), Cell::new(1, 1, 0), Cell::new(2, 1, 0)],
    })
    .unwrap();
    s.add_equipment(equipment(4, &[[1, 0, 0], [1, 1, 0]]))
        .unwrap();
    assert_eq!(s.occupied_cells().count(), 55);
    assert_eq!(s.mass_entries().count(), 3); // Rooms add no second mass/occupancy.
    let before = s.clone();
    assert_eq!(
        s.remove_part(StructureId(2)),
        Err(StructureError::ReferencedStructure)
    );
    assert_eq!(s, before);
    assert!(s.add_equipment(equipment(5, &[[1, 1, 0]])).is_err());
    s.remove_equipment(StructureId(4)).unwrap();
    s.remove_room(StructureId(3)).unwrap();
    s.remove_part(StructureId(2)).unwrap();
    assert_eq!(s.occupied_cells().count(), 1);
}

#[test]
fn face_changes_do_not_disconnect_frames_and_shared_faces_have_one_owner() {
    let mut s = ship();
    s.add_part(part(2, &[[1, 0, 0]])).unwrap();
    s.set_face(
        StructureId(1),
        Cell::new(0, 0, 0),
        Direction::Forward,
        Some(FaceKind::Doorway),
    )
    .unwrap();
    let before = s.clone();
    assert_eq!(
        s.set_face(
            StructureId(2),
            Cell::new(1, 0, 0),
            Direction::Aft,
            Some(FaceKind::Wall)
        ),
        Err(StructureError::DuplicateFace)
    );
    assert_eq!(s, before);
    s.set_face(StructureId(1), Cell::new(0, 0, 0), Direction::Forward, None)
        .unwrap();
    s.set_face(
        StructureId(2),
        Cell::new(1, 0, 0),
        Direction::Down,
        Some(FaceKind::Floor),
    )
    .unwrap();
    s.set_face(
        StructureId(2),
        Cell::new(1, 0, 0),
        Direction::Up,
        Some(FaceKind::Ceiling),
    )
    .unwrap();
    s.set_face(StructureId(2), Cell::new(1, 0, 0), Direction::Down, None)
        .unwrap();
    s.set_face(StructureId(2), Cell::new(1, 0, 0), Direction::Up, None)
        .unwrap();
    assert_eq!(s.occupied_cells().count(), 2);
    assert_eq!(s.parts().count(), 2);
    assert_eq!(s.mass_entries().map(|(_, m)| m).sum::<f64>(), 25.);
}

#[test]
fn safe_remove_rejects_bridge_cut_but_accepts_a_redundant_path() {
    let mut s = ship();
    s.add_part(part(2, &[[1, 0, 0]])).unwrap();
    s.add_part(part(3, &[[2, 0, 0]])).unwrap();
    let before = s.clone();
    assert_eq!(
        s.remove_part(StructureId(2)),
        Err(StructureError::Disconnected)
    );
    assert_eq!(s, before);
    s.add_part(part(4, &[[0, 1, 0], [1, 1, 0], [2, 1, 0]]))
        .unwrap();
    s.remove_part(StructureId(2)).unwrap();
    assert_eq!(s.parts().count(), 3);
    assert_eq!(
        s.remove_part(StructureId(1)),
        Err(StructureError::CoreRemoval)
    );
    assert_eq!(
        s.damage_part(StructureId(1)),
        Err(StructureError::CoreRemoval)
    );
}

#[test]
fn damage_transfers_multiple_components_ids_equipment_and_room_metadata_once() {
    let mut s = ship();
    s.add_part(part(2, &[[1, 0, 0]])).unwrap();
    s.add_part(part(3, &[[2, 0, 0]])).unwrap();
    s.add_part(part(4, &[[1, 1, 0]])).unwrap();
    s.add_room(Room {
        id: StructureId(5),
        cells: vec![Cell::new(2, 0, 0)],
    })
    .unwrap();
    s.add_room(Room {
        id: StructureId(6),
        cells: vec![Cell::new(1, 0, 0), Cell::new(1, 1, 0)],
    })
    .unwrap();
    s.add_equipment(equipment(7, &[[2, 0, 0]])).unwrap();
    s.add_equipment(equipment(8, &[[1, 0, 0]])).unwrap();
    let original_mass = s.mass_entries().map(|(_, m)| m).sum::<f64>();
    let result = s.damage_part(StructureId(2)).unwrap();
    assert_eq!(result.detached.len(), 2);
    assert_eq!(result.invalidated_rooms[0].id, StructureId(6));
    assert_eq!(result.unsupported_equipment[0].id, StructureId(8));
    assert_eq!(
        s.parts().map(|p| p.id).collect::<Vec<_>>(),
        vec![StructureId(1)]
    );
    let mut ids = BTreeSet::new();
    let mut returned_mass = result.destroyed.mass.additional_kg().unwrap_or(0.);
    ids.insert(result.destroyed.id);
    for detached in &result.detached {
        for p in &detached.parts {
            assert!(ids.insert(p.id));
            returned_mass += p.mass.additional_kg().unwrap_or(0.);
        }
        for e in &detached.equipment {
            assert!(ids.insert(e.id));
            returned_mass += e.mass.additional_kg().unwrap_or(0.);
        }
        for r in &detached.rooms {
            assert!(ids.insert(r.id));
        }
    }
    for e in &result.unsupported_equipment {
        assert!(ids.insert(e.id));
        returned_mass += e.mass.additional_kg().unwrap_or(0.);
    }
    assert_eq!(
        original_mass,
        returned_mass + s.mass_entries().map(|(_, m)| m).sum::<f64>()
    );
    assert!(s.damage_part(StructureId(2)).is_err());
    assert_eq!(
        s.add_part(part(3, &[[1, 0, 0]])),
        Err(StructureError::DuplicateIdentity(StructureId(3)))
    );
    assert_eq!(ShipStructure::from_save(s.save_state()).unwrap(), s);
}

#[test]
fn invalid_inputs_and_duplicate_identity_reject_atomically() {
    let mut s = ship();
    let before = s.clone();
    let valid = part(2, &[[1, 0, 0]]);
    let mut candidates = vec![];
    let mut p = valid.clone();
    p.id = StructureId(0);
    candidates.push(p);
    let mut p = valid.clone();
    p.id = StructureId(1);
    candidates.push(p);
    let mut p = valid.clone();
    p.cells.clear();
    candidates.push(p);
    let mut p = valid.clone();
    p.cells.push(p.cells[0]);
    candidates.push(p);
    let mut p = valid.clone();
    p.cells.push(Cell::new(2, 1, 0));
    candidates.push(p);
    let mut p = valid.clone();
    p.space.min[0] = f64::NAN;
    candidates.push(p);
    let mut p = valid.clone();
    p.space.max[0] = 20.;
    candidates.push(p);
    let mut p = valid.clone();
    p.collision[0].min[0] = 0.;
    candidates.push(p);
    let mut p = valid.clone();
    p.faces.push(Face {
        cell: p.cells[0],
        direction: Direction::Port,
        kind: FaceKind::Floor,
    });
    candidates.push(p);
    for mass in [0., -1., f64::NAN, f64::INFINITY] {
        let mut p = valid.clone();
        p.mass = StructuralMass::AddedKilograms(mass);
        candidates.push(p);
    }
    for p in candidates {
        assert!(s.add_part(p).is_err());
        assert_eq!(s, before);
    }
    s.add_room(Room {
        id: StructureId(9),
        cells: vec![Cell::new(0, 0, 0)],
    })
    .unwrap();
    assert_eq!(
        s.add_equipment(equipment(9, &[[0, 0, 0]])),
        Err(StructureError::DuplicateIdentity(StructureId(9)))
    );
    let mut huge = ship();
    huge.add_part({
        let mut p = valid.clone();
        p.mass = StructuralMass::AddedKilograms(f64::MAX);
        p
    })
    .unwrap();
    let before = huge.clone();
    assert_eq!(
        huge.add_part({
            let mut p = part(3, &[[2, 0, 0]]);
            p.mass = StructuralMass::AddedKilograms(f64::MAX);
            p
        }),
        Err(StructureError::InvalidMass)
    );
    assert_eq!(huge, before);
}

#[test]
fn save_round_trip_rebuilds_topology_instead_of_requiring_id_order() {
    let mut s = ship();
    s.add_part(part(30, &[[1, 0, 0]])).unwrap();
    s.add_part(part(2, &[[2, 0, 0]])).unwrap();
    s.add_room(Room {
        id: StructureId(6),
        cells: vec![Cell::new(2, 0, 0)],
    })
    .unwrap();
    s.add_equipment(equipment(7, &[[2, 0, 0]])).unwrap();
    let state = s.save_state();
    state.validate().unwrap();
    let envelope = SaveGame::new(state.clone(), Duration::ZERO, Duration::ZERO, 123).unwrap();
    let bytes = envelope.encode().unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    for invalid_coordinate in ["2147483648", "-2147483649", "1.5", "null"] {
        let malformed = text.replacen("\"x\":0", &format!("\"x\":{invalid_coordinate}"), 1);
        assert_ne!(malformed, text);
        assert!(SaveGame::<StructureSaveState>::decode(malformed.as_bytes()).is_err());
    }
    let decoded = SaveGame::<StructureSaveState>::decode(&bytes).unwrap();
    assert_eq!(ShipStructure::from_save(decoded.state).unwrap(), s);
    assert_eq!(envelope.encode().unwrap(), bytes);
    let mut unordered = state.clone();
    unordered.parts.reverse();
    assert_eq!(
        ShipStructure::from_save(unordered).unwrap().save_state(),
        state
    );
    let mut corrupt = state.clone();
    corrupt.schema_version += 1;
    assert!(corrupt.validate().is_err());
    let mut corrupt = state.clone();
    corrupt.parts.push(corrupt.parts[0].clone());
    assert!(corrupt.validate().is_err());
    let mut corrupt = state.clone();
    corrupt.core = StructureId(99);
    assert!(corrupt.validate().is_err());
    let mut corrupt = state.clone();
    corrupt.parts.retain(|p| p.id != StructureId(30));
    assert!(corrupt.validate().is_err());
    let mut corrupt = state.clone();
    corrupt.equipment[0].id = corrupt.rooms[0].id;
    assert!(corrupt.validate().is_err());
    let mut corrupt = state.clone();
    corrupt.retired = vec![StructureId(7)];
    assert!(corrupt.validate().is_err());
    let mut corrupt = state;
    corrupt.retired = vec![StructureId(90), StructureId(90)];
    assert!(corrupt.validate().is_err());
}

#[test]
fn structural_mass_namespace_coexists_with_cargo_and_legacy_equipment() {
    let s = ship();
    let mut mass = crate::ShipMass::default();
    mass.update(crate::MassObjectId::Fragment(1), Some(2.))
        .unwrap();
    mass.update(crate::MassObjectId::Equipment(1), Some(3.))
        .unwrap();
    for (id, kg) in s.mass_entries() {
        assert!(mass.update(id, Some(kg)).unwrap());
        assert!(!mass.update(id, Some(kg)).unwrap());
    }
    assert_eq!(mass.total_kg(), crate::STARTER_DRY_MASS_KG + 17.5);
    for (id, _) in s.mass_entries() {
        mass.update(id, None).unwrap();
    }
    assert_eq!(mass.total_kg(), crate::STARTER_DRY_MASS_KG + 5.);
}

#[test]
fn authored_baseline_structure_never_counts_dry_hull_mass_again() {
    let mut core = part(1, &[[0, 0, 0]]);
    core.mass = StructuralMass::IncludedInDryHull;
    let mut s = ShipStructure::new([0.; 3], core).unwrap();
    let mut authored = part(2, &[[1, 0, 0]]);
    authored.mass = StructuralMass::IncludedInDryHull;
    s.add_part(authored).unwrap();
    s.add_equipment(equipment(3, &[[1, 0, 0]])).unwrap();
    let entries: Vec<_> = s.mass_entries().collect();
    assert_eq!(entries, vec![(crate::MassObjectId::Structure(3), 40.)]);
    let mut ledger = crate::ShipMass::default();
    for (id, kg) in entries {
        ledger.update(id, Some(kg)).unwrap();
    }
    assert_eq!(ledger.total_kg(), crate::STARTER_DRY_MASS_KG + 40.);
    assert_eq!(ShipStructure::from_save(s.save_state()).unwrap(), s);
}

#[test]
fn all_connected_two_by_two_by_two_layouts_obey_cut_and_damage_contracts() {
    // Independent Manhattan-distance closure, with no production neighbor/component helpers.
    fn closure(cells: &[[i32; 3]]) -> BTreeSet<[i32; 3]> {
        let mut seen = BTreeSet::from([[0, 0, 0]]);
        loop {
            let before = seen.len();
            for cell in cells {
                if seen
                    .iter()
                    .any(|other| (0..3).map(|i| cell[i].abs_diff(other[i])).sum::<u32>() == 1)
                {
                    seen.insert(*cell);
                }
            }
            if seen.len() == before {
                return seen;
            }
        }
    }
    let candidates: Vec<_> = (0..2)
        .flat_map(|x| (0..2).flat_map(move |y| (0..2).map(move |z| [x, y, z])))
        .filter(|c| *c != [0, 0, 0])
        .collect();
    let mut checked = 0;
    for mask in 0..128 {
        let cells: Vec<_> = std::iter::once([0, 0, 0])
            .chain(
                candidates
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, c)| *c),
            )
            .collect();
        if closure(&cells).len() != cells.len() {
            continue;
        }
        let mut s = ship();
        let mut remaining: Vec<_> = cells[1..]
            .iter()
            .enumerate()
            .map(|(i, c)| ((i + 2) as u64, *c))
            .collect();
        while !remaining.is_empty() {
            let mut admitted = false;
            for index in 0..remaining.len() {
                let (id, cell) = remaining[index];
                if s.add_part(part(id, &[cell])).is_ok() {
                    remaining.remove(index);
                    admitted = true;
                    break;
                }
            }
            assert!(admitted);
        }
        for p in s.parts().filter(|p| p.id != s.core_id()) {
            let removed = p.cells[0];
            let others: Vec<_> = cells
                .iter()
                .copied()
                .filter(|c| *c != [removed.x, removed.y, removed.z])
                .collect();
            let supported = closure(&others);
            let mut normal = s.clone();
            assert_eq!(
                normal.remove_part(p.id).is_ok(),
                supported.len() == others.len()
            );
            if normal.parts().count() == s.parts().count() {
                assert_eq!(normal, s);
            }
            let mut damaged = s.clone();
            let outcome = damaged.damage_part(p.id).unwrap();
            let attached: BTreeSet<_> = damaged
                .occupied_cells()
                .map(|(c, _)| [c.x, c.y, c.z])
                .collect();
            assert_eq!(attached, supported);
            let mut returned = attached;
            for component in outcome.detached {
                for p in component.parts {
                    for c in p.cells {
                        assert!(returned.insert([c.x, c.y, c.z]));
                    }
                }
            }
            assert_eq!(returned, others.iter().copied().collect());
            checked += 1;
        }
    }
    assert!(checked > 200);
}

#[test]
fn opening_a_shared_divider_preserves_independent_room_finishes() {
    use crate::finishes::{Finishes, Group};
    let mut s = ship();
    s.add_part(part(2, &[[1, 0, 0]])).unwrap();
    s.add_room(Room {
        id: StructureId(700),
        cells: vec![Cell::new(0, 0, 0)],
    })
    .unwrap();
    s.add_room(Room {
        id: StructureId(701),
        cells: vec![Cell::new(1, 0, 0)],
    })
    .unwrap();
    let mut f = Finishes::default();
    assert!(f.add_room(700));
    assert!(f.add_room(701));
    f.cycle_group(700, Group::Walls);
    f.cycle_group(701, Group::Floor);
    let before = f.clone();
    s.set_face(
        StructureId(1),
        Cell::new(0, 0, 0),
        Direction::Forward,
        Some(FaceKind::Wall),
    )
    .unwrap();
    s.set_face(StructureId(1), Cell::new(0, 0, 0), Direction::Forward, None)
        .unwrap();
    assert_eq!(f, before);
    assert_ne!(f.rooms[&700], f.rooms[&701]);
    let restored: Finishes = serde_json::from_slice(&serde_json::to_vec(&f).unwrap()).unwrap();
    assert_eq!(restored, f);
}
