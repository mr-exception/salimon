//! Ship-local grid coordinates; authored anchors are never snapped implicitly.
use super::StructureError;
use serde::{Deserialize, Serialize};

pub const CELL_SIZE_METERS: f64 = 3.0;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Cell {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn neighbor(self, direction: Direction) -> Option<Self> {
        let [dx, dy, dz] = direction.delta();
        Some(Self::new(
            self.x.checked_add(dx)?,
            self.y.checked_add(dy)?,
            self.z.checked_add(dz)?,
        ))
    }

    /// Half-open cell bounds relative to a separately supplied, precise ship-local origin.
    pub fn bounds(self, origin_meters: [f64; 3]) -> Result<Envelope, StructureError> {
        if origin_meters.iter().any(|v| !v.is_finite()) {
            return Err(StructureError::InvalidGeometry);
        }
        let indices = [self.x, self.y, self.z];
        let min =
            std::array::from_fn(|i| origin_meters[i] + f64::from(indices[i]) * CELL_SIZE_METERS);
        let max = min.map(|v| v + CELL_SIZE_METERS);
        let envelope = Envelope { min, max };
        envelope.validate()?;
        Ok(envelope)
    }

    /// Explicit placement query, not a migration of any existing asset anchor.
    pub fn containing(
        position_meters: [f64; 3],
        origin_meters: [f64; 3],
    ) -> Result<Self, StructureError> {
        let mut indices = [0; 3];
        for i in 0..3 {
            let index = ((position_meters[i] - origin_meters[i]) / CELL_SIZE_METERS).floor();
            if !position_meters[i].is_finite()
                || !origin_meters[i].is_finite()
                || !index.is_finite()
                || index < f64::from(i32::MIN)
                || index > f64::from(i32::MAX)
            {
                return Err(StructureError::InvalidGeometry);
            }
            indices[i] = index as i32;
        }
        Ok(Self::new(indices[0], indices[1], indices[2]))
    }
}

/// X forward, Y up, Z port. Only face adjacency establishes structural support.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Direction {
    Forward,
    Aft,
    Up,
    Down,
    Port,
    Starboard,
}
impl Direction {
    pub const ALL: [Self; 6] = [
        Self::Forward,
        Self::Aft,
        Self::Up,
        Self::Down,
        Self::Port,
        Self::Starboard,
    ];
    const fn delta(self) -> [i32; 3] {
        match self {
            Self::Forward => [1, 0, 0],
            Self::Aft => [-1, 0, 0],
            Self::Up => [0, 1, 0],
            Self::Down => [0, -1, 0],
            Self::Port => [0, 0, 1],
            Self::Starboard => [0, 0, -1],
        }
    }
    pub const fn opposite(self) -> Self {
        match self {
            Self::Forward => Self::Aft,
            Self::Aft => Self::Forward,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Port => Self::Starboard,
            Self::Starboard => Self::Port,
        }
    }
}

/// Conservative ship-local space/collision bounds. These are not final mesh colliders.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub min: [f64; 3],
    pub max: [f64; 3],
}
impl Envelope {
    pub(super) fn validate(&self) -> Result<(), StructureError> {
        if (0..3).any(|i| {
            !self.min[i].is_finite() || !self.max[i].is_finite() || self.min[i] >= self.max[i]
        }) {
            return Err(StructureError::InvalidGeometry);
        }
        Ok(())
    }
}
