//! Deterministic room geometry. Every element is addressed by (cell part, face/index).
use super::*;

#[derive(Clone, Debug)]
pub struct ShellRoom {
    pub project: u64,
    pub cells: Vec<Cell>,
    pub mass_kg: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionEdit {
    pub part: StructureId,
    pub direction: Direction,
}

/// Build completed source into independently owned cells. IDs do not depend on
/// completion order. Existing authored hull is retained verbatim.
pub fn build_shells(
    starter: &ShipStructure,
    rooms: &[ShellRoom],
    edits: &[PartitionEdit],
) -> Result<ShipStructure, StructureError> {
    let mut owners = BTreeMap::new();
    for room in rooms {
        if room.project == 0 {
            return Err(StructureError::InvalidIdentity);
        }
        if room.cells.len() > 128 || room.cells.is_empty() {
            return Err(StructureError::InvalidGeometry);
        }
        let mut cells = room.cells.clone();
        normalize_cells(&mut cells)?;
        for (index, cell) in cells.iter().enumerate() {
            let id = room
                .project
                .checked_mul(1024)
                .and_then(|n| n.checked_add(1024 + index as u64))
                .ok_or(StructureError::InvalidIdentity)?;
            if owners
                .insert(*cell, (room.project, StructureId(id)))
                .is_some()
            {
                return Err(StructureError::OccupiedCell(*cell));
            }
        }
    }
    if !owners.is_empty() {
        let port = Cell::new(-3, 0, 1);
        let cells: BTreeSet<_> = owners.keys().copied().collect();
        if !cells.contains(&port) || reachable(port, &cells) != cells {
            return Err(StructureError::Disconnected);
        }
    }
    let exit_cell = owners
        .keys()
        .filter(|c| c.y == 0 && c.z == 1)
        .min_by_key(|c| c.x)
        .copied();
    let mut applied = BTreeSet::new();
    let mut assemblies = Vec::new();
    for room in rooms {
        let mut cells = room.cells.clone();
        cells.sort();
        let low_x = cells.iter().map(|c| c.x).min().unwrap();
        let high_x = cells.iter().map(|c| c.x).max().unwrap();
        let mut parts = Vec::new();
        for cell in &cells {
            let id = owners[cell].1;
            let space = cell.bounds(starter.origin_meters())?;
            let mut faces = Vec::new();
            let mut collision = Vec::new();
            for direction in Direction::ALL {
                let neighbor = cell.neighbor(direction);
                let other = neighbor.and_then(|c| owners.get(&c));
                if direction == Direction::Up && other.is_some() {
                    continue;
                }
                // Horizontal shared faces have exactly one owner. The upper cell
                // owns inter-deck floors; no duplicate lower ceiling is emitted.
                if !matches!(direction, Direction::Up | Direction::Down)
                    && neighbor.is_some_and(|n| owners.contains_key(&n) && n < *cell)
                {
                    continue;
                }
                let port = *cell == Cell::new(-3, 0, 1) && direction == Direction::Forward;
                let mut kind = match direction {
                    Direction::Down => FaceKind::Floor,
                    Direction::Up => FaceKind::Ceiling,
                    _ if port || (Some(*cell) == exit_cell && direction == Direction::Aft) => {
                        FaceKind::Doorway
                    }
                    _ if other.is_some_and(|(project, _)| *project == room.project) => {
                        FaceKind::Opening
                    }
                    _ if other.is_some() => FaceKind::Doorway,
                    _ => FaceKind::Wall,
                };
                for (index, edit) in edits.iter().enumerate() {
                    if edit.part == id && edit.direction == direction {
                        if kind != FaceKind::Doorway || other.is_none() || !applied.insert(index) {
                            return Err(StructureError::InvalidFace);
                        }
                        kind = FaceKind::Opening;
                    }
                }
                faces.push(Face {
                    cell: *cell,
                    direction,
                    kind,
                });
                let stair_above = direction == Direction::Down
                    && cell
                        .neighbor(Direction::Down)
                        .is_some_and(|n| cells.contains(&n));
                face_boxes(
                    space,
                    direction,
                    kind,
                    stair_above,
                    cell.x == low_x,
                    &mut collision,
                );
            }
            // Treads span two horizontal cells. Each tread belongs to its cell,
            // and the matching upper-deck hatch is cut from the floor geometry.
            if cell
                .neighbor(Direction::Up)
                .is_some_and(|n| cells.contains(&n))
            {
                if high_x - low_x < 1 {
                    return Err(StructureError::InvalidGeometry);
                }
                let x0 = starter.origin_meters()[0] + f64::from(low_x) * 3. + 0.4;
                let x1 = starter.origin_meters()[0] + (f64::from(high_x) + 1.) * 3. - 0.4;
                let count = 20;
                for step in 0..count {
                    let a = x0 + (x1 - x0) * f64::from(step) / f64::from(count);
                    let b = x0 + (x1 - x0) * f64::from(step + 1) / f64::from(count);
                    let lo = a.max(space.min[0]);
                    let hi = b.min(space.max[0]);
                    if lo < hi {
                        collision.push(Envelope {
                            min: [lo, space.min[1], space.min[2] + 0.25],
                            max: [
                                hi,
                                (space.min[1]
                                    + 0.1
                                    + 3. * f64::from(count - step) / f64::from(count))
                                .min(space.max[1]),
                                space.min[2] + 1.4,
                            ],
                        });
                    }
                }
            }
            if cell
                .neighbor(Direction::Down)
                .is_some_and(|n| cells.contains(&n))
            {
                let start = space.min[0] + if cell.x == low_x { 0.9 } else { 0. };
                let end = space.max[0] - 0.1;
                if start < end {
                    collision.push(Envelope {
                        min: [start, space.min[1] + 1.05, space.min[2] + 1.45],
                        max: [end, space.min[1] + 1.1, space.min[2] + 1.5],
                    });
                    let mut x = start;
                    while x < end {
                        collision.push(Envelope {
                            min: [x, space.min[1] + 0.1, space.min[2] + 1.45],
                            max: [(x + 0.05).min(end), space.min[1] + 1.05, space.min[2] + 1.5],
                        });
                        x += 0.8;
                    }
                }
            }
            // Clip exposed corners into a stepped bevel; the upper cap tapers
            // farther inward. These are real sealed panels, not a render-only hint.
            for xdir in [Direction::Aft, Direction::Forward] {
                for zdir in [Direction::Starboard, Direction::Port] {
                    if cell.neighbor(xdir).is_some_and(|n| owners.contains_key(&n))
                        || cell.neighbor(zdir).is_some_and(|n| owners.contains_key(&n))
                    {
                        continue;
                    }
                    clipped_corner(
                        space,
                        xdir == Direction::Forward,
                        zdir == Direction::Port,
                        &mut collision,
                    );
                }
            }
            parts.push(StructuralPart {
                id,
                cells: vec![*cell],
                shape: MemberShape::CornerShell,
                faces,
                space,
                collision,
                mass: StructuralMass::AddedKilograms(room.mass_kg / cells.len() as f64),
            });
        }
        let room_id = room
            .project
            .checked_mul(1024)
            .and_then(|n| n.checked_add(2047))
            .ok_or(StructureError::InvalidIdentity)?;
        assemblies.push(RoomAssembly {
            room: Room {
                id: StructureId(room_id),
                cells,
            },
            parts,
        });
    }
    if applied.len() != edits.len() {
        return Err(StructureError::InvalidFace);
    }
    let mut result = starter.clone();
    while !assemblies.is_empty() {
        let index = assemblies
            .iter()
            .position(|a| {
                a.room
                    .cells
                    .iter()
                    .any(|c| neighbors(*c).any(|n| result.cells.contains_key(&n)))
            })
            .ok_or(StructureError::Disconnected)?;
        result.install_room(assemblies.remove(index))?;
    }
    Ok(result)
}

fn face_boxes(
    s: Envelope,
    d: Direction,
    kind: FaceKind,
    hatch: bool,
    landing_cell: bool,
    out: &mut Vec<Envelope>,
) {
    if kind == FaceKind::Opening {
        return;
    }
    let axis = match d {
        Direction::Forward | Direction::Aft => 0,
        Direction::Up | Direction::Down => 1,
        _ => 2,
    };
    let positive = matches!(d, Direction::Forward | Direction::Up | Direction::Port);
    let mut slab = s;
    if positive {
        slab.min[axis] = slab.max[axis] - 0.1;
    } else {
        slab.max[axis] = slab.min[axis] + 0.1;
    }
    if hatch {
        // The stair lane is open; a landing at the aft end connects to the deck.
        slab.min[2] = s.min[2] + 1.5;
        out.push(slab);
        let mut landing = s;
        landing.max[0] = s.min[0] + 0.75;
        landing.max[1] = s.min[1] + 0.1;
        if landing_cell {
            out.push(landing);
        }
    } else if kind == FaceKind::Doorway {
        let horizontal = if axis == 0 { 2 } else { 0 };
        let mid = (s.min[horizontal] + s.max[horizontal]) / 2.;
        let mut left = slab;
        left.max[horizontal] = mid - 0.7;
        let mut right = slab;
        right.min[horizontal] = mid + 0.7;
        let mut lintel = slab;
        lintel.min[1] = s.min[1] + 2.3;
        out.extend([left, right, lintel]);
    } else {
        out.push(slab);
    }
}

fn clipped_corner(s: Envelope, high_x: bool, high_z: bool, boxes: &mut Vec<Envelope>) {
    let mapped = |x0: f64, x1: f64, y0: f64, y1: f64, z0: f64, z1: f64| Envelope {
        min: [
            if high_x { s.max[0] - x1 } else { s.min[0] + x0 },
            y0,
            if high_z { s.max[2] - z1 } else { s.min[2] + z0 },
        ],
        max: [
            if high_x { s.max[0] - x0 } else { s.min[0] + x1 },
            y1,
            if high_z { s.max[2] - z0 } else { s.min[2] + z1 },
        ],
    };
    for (lo, hi, bevel) in [
        (s.min[1], s.max[1] - 0.3, 0.3),
        (s.max[1] - 0.3, s.max[1], 0.45),
    ] {
        for cut in [
            mapped(0., bevel / 2., lo, hi, 0., bevel),
            mapped(bevel / 2., bevel, lo, hi, 0., bevel / 2.),
        ] {
            *boxes = boxes.iter().flat_map(|b| subtract(*b, cut)).collect();
        }
        // Two short orthogonal panels close the clipped edge, sharing the same
        // exact geometry with collision. A cap joins the two taper sections.
        boxes.extend([
            mapped(0., bevel / 2. + 0.08, lo, hi, bevel, bevel + 0.08),
            mapped(
                bevel / 2.,
                bevel / 2. + 0.08,
                lo,
                hi,
                bevel / 2.,
                bevel + 0.08,
            ),
            mapped(
                bevel / 2.,
                bevel + 0.08,
                lo,
                hi,
                bevel / 2.,
                bevel / 2. + 0.08,
            ),
            mapped(bevel, bevel + 0.08, lo, hi, 0., bevel / 2. + 0.08),
        ]);
    }
    boxes.push(mapped(
        0.15,
        0.53,
        s.max[1] - 0.35,
        s.max[1] - 0.25,
        0.15,
        0.53,
    ));
}
fn subtract(b: Envelope, cut: Envelope) -> Vec<Envelope> {
    let lo: [f64; 3] = std::array::from_fn(|i| b.min[i].max(cut.min[i]));
    let hi: [f64; 3] = std::array::from_fn(|i| b.max[i].min(cut.max[i]));
    if (0..3).any(|i| lo[i] >= hi[i]) {
        return vec![b];
    }
    let mut core = b;
    let mut out = Vec::new();
    for i in 0..3 {
        if core.min[i] < lo[i] {
            let mut piece = core;
            piece.max[i] = lo[i];
            out.push(piece);
            core.min[i] = lo[i];
        }
        if core.max[i] > hi[i] {
            let mut piece = core;
            piece.min[i] = hi[i];
            out.push(piece);
            core.max[i] = hi[i];
        }
    }
    out
}
