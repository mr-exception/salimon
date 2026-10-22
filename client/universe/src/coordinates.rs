//! Hierarchical SI coordinates: exact galactic cells plus bounded local f64.
use crate::{CatalogId, Epoch, Error, invalid};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CELL_METERS: f64 = 1_099_511_627_776.0; // 2^40: local spacing <= 0.0001221 m.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PositionWire", into = "PositionWire")]
pub struct GalacticPosition {
    cell: [i64; 3],
    offset: [f64; 3],
}
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PositionWire {
    cell: [i64; 3],
    offset: [f64; 3],
}
impl TryFrom<PositionWire> for GalacticPosition {
    type Error = Error;
    fn try_from(w: PositionWire) -> Result<Self, Error> {
        Self::new(w.cell, w.offset)
    }
}
impl From<GalacticPosition> for PositionWire {
    fn from(p: GalacticPosition) -> Self {
        Self {
            cell: p.cell,
            offset: p.offset,
        }
    }
}
fn finite(v: [f64; 3]) -> Result<(), Error> {
    if v.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(invalid("nonfinite coordinate/state"))
    }
}
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
impl GalacticPosition {
    pub fn new(mut cell: [i64; 3], mut offset: [f64; 3]) -> Result<Self, Error> {
        finite(offset)?;
        for i in 0..3 {
            let carry = ((offset[i] + CELL_METERS / 2.0) / CELL_METERS).floor();
            if carry < i64::MIN as f64 || carry >= i64::MAX as f64 {
                return Err(invalid("galactic cell overflow"));
            }
            cell[i] = cell[i]
                .checked_add(carry as i64)
                .ok_or_else(|| invalid("galactic cell overflow"))?;
            offset[i] -= carry * CELL_METERS;
        }
        Ok(Self { cell, offset })
    }
    pub fn cells(self) -> [i64; 3] {
        self.cell
    }
    pub fn offset_meters(self) -> [f64; 3] {
        self.offset
    }
    pub fn translated(self, meters: [f64; 3]) -> Result<Self, Error> {
        finite(meters)?;
        // Normalize the displacement independently, so tiny source offsets are
        // not swallowed by a large motion term before cell subtraction.
        let d = Self::new([0; 3], meters)?;
        let mut cells = [0; 3];
        for (i, c) in cells.iter_mut().enumerate() {
            *c = self.cell[i]
                .checked_add(d.cell[i])
                .ok_or_else(|| invalid("cell addition overflow"))?;
        }
        Self::new(cells, add(self.offset, d.offset))
    }
    /// Nearby differences preserve precision before GPU narrowing. Large
    /// separations are approximate f64 distances, never reusable local origins.
    pub fn relative_meters(self, origin: Self) -> Result<[f64; 3], Error> {
        let v = std::array::from_fn(|i| {
            (i128::from(self.cell[i]) - i128::from(origin.cell[i])) as f64 * CELL_METERS
                + (self.offset[i] - origin.offset[i])
        });
        finite(v)?;
        Ok(v)
    }
    pub fn camera_relative_f32(self, camera: Self) -> Result<[f32; 3], Error> {
        let v = self.relative_meters(camera)?.map(|x| x as f32);
        if v.into_iter().all(f32::is_finite) {
            Ok(v)
        } else {
            Err(invalid("GPU relative coordinate overflow"))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum FrameId {
    System(CatalogId),
    Body(CatalogId),
    Local(CatalogId),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub meters: [f64; 3],
    pub meters_per_second: [f64; 3],
}
/// Validated orthonormal right-handed child-to-parent axes (columns).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation([[f64; 3]; 3]);
impl Rotation {
    pub const IDENTITY: Self = Self([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    pub fn new(axes: [[f64; 3]; 3]) -> Result<Self, Error> {
        for a in axes {
            finite(a)?;
        }
        let dot = |a: [f64; 3], b: [f64; 3]| a.into_iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
        for i in 0..3 {
            for j in 0..3 {
                if (dot(axes[i], axes[j]) - f64::from(i == j)).abs() > 1e-12 {
                    return Err(invalid("frame axes not orthonormal"));
                }
            }
        }
        if dot(cross(axes[0], axes[1]), axes[2]) < 1.0 - 1e-12 {
            return Err(invalid("left-handed frame"));
        }
        Ok(Self(axes))
    }
    fn apply(self, v: [f64; 3]) -> [f64; 3] {
        std::array::from_fn(|i| (0..3).map(|j| self.0[j][i] * v[j]).sum())
    }
    fn inverse(self, v: [f64; 3]) -> [f64; 3] {
        self.0
            .map(|a| a.into_iter().zip(v).map(|(x, y)| x * y).sum())
    }
}
/// Instantaneous parent-relative pose and motion at a declared epoch. Angular
/// velocity is in parent axes, radians/s; no orientation extrapolation is assumed.
#[derive(Clone, Debug)]
pub struct FrameLink {
    pub child: FrameId,
    pub parent: FrameId,
    pub epoch: Epoch,
    pub origin_meters: [f64; 3],
    pub velocity_mps: [f64; 3],
    pub rotation: Rotation,
    pub angular_velocity_rps: [f64; 3],
}
impl FrameLink {
    fn outward(&self, s: State) -> State {
        let r = self.rotation.apply(s.meters);
        State {
            meters: add(self.origin_meters, r),
            meters_per_second: add(
                self.velocity_mps,
                add(
                    self.rotation.apply(s.meters_per_second),
                    cross(self.angular_velocity_rps, r),
                ),
            ),
        }
    }
    fn inward(&self, s: State) -> State {
        let r = sub(s.meters, self.origin_meters);
        State {
            meters: self.rotation.inverse(r),
            meters_per_second: self.rotation.inverse(sub(
                sub(s.meters_per_second, self.velocity_mps),
                cross(self.angular_velocity_rps, r),
            )),
        }
    }
}
#[derive(Clone, Debug)]
pub struct SystemAnchor {
    pub system: CatalogId,
    pub epoch: Epoch,
    pub position: GalacticPosition,
    pub velocity_mps: [f64; 3],
    pub rotation: Rotation,
}
impl SystemAnchor {
    /// Linear proper/radial motion in galactic axes. U14 supplies actual orbital
    /// ephemerides; this does not infer missing source velocities.
    pub fn position_at(&self, epoch: Epoch) -> Result<GalacticPosition, Error> {
        finite(self.velocity_mps)?;
        let dt = epoch.seconds_since(self.epoch)?;
        self.position.translated(self.velocity_mps.map(|v| v * dt))
    }
}
/// One system's instantaneous hierarchy. Identity is runtime checked, not merely
/// a vector label; unknown, duplicate, cyclic and disconnected frames reject.
pub struct FrameTree {
    anchor: SystemAnchor,
    links: BTreeMap<FrameId, FrameLink>,
}
impl FrameTree {
    pub fn new(anchor: SystemAnchor, links: Vec<FrameLink>) -> Result<Self, Error> {
        finite(anchor.velocity_mps)?;
        let root = FrameId::System(anchor.system.clone());
        let mut map = BTreeMap::new();
        for l in links {
            finite(l.origin_meters)?;
            finite(l.velocity_mps)?;
            finite(l.angular_velocity_rps)?;
            if l.epoch != anchor.epoch
                || matches!(l.child, FrameId::System(_))
                || map.insert(l.child.clone(), l).is_some()
            {
                return Err(invalid("frame epoch/root/duplicate"));
            }
        }
        let tree = Self { anchor, links: map };
        for key in tree.links.keys() {
            let path = tree.path(key)?;
            if path.last().map(|l| &l.parent) != Some(&root) {
                return Err(invalid("disconnected frame"));
            }
        }
        Ok(tree)
    }
    fn path(&self, id: &FrameId) -> Result<Vec<&FrameLink>, Error> {
        let root = FrameId::System(self.anchor.system.clone());
        let mut path = Vec::new();
        let mut current = id;
        while current != &root {
            let link = self
                .links
                .get(current)
                .ok_or_else(|| invalid("unknown frame"))?;
            if path.len() >= self.links.len() {
                return Err(invalid("cyclic frame tree"));
            }
            path.push(link);
            current = &link.parent;
        }
        Ok(path)
    }
    pub fn transform(
        &self,
        from: &FrameId,
        to: &FrameId,
        state: State,
        epoch: Epoch,
    ) -> Result<State, Error> {
        finite(state.meters)?;
        finite(state.meters_per_second)?;
        if epoch != self.anchor.epoch {
            return Err(invalid("frame tree requires evaluated epoch"));
        }
        let mut out = state;
        // Cancel common ancestry before arithmetic: body-local detail is never
        // added to a multi-light-year system anchor and then subtracted again.
        let mut a = self.path(from)?;
        let mut b = self.path(to)?;
        while a.last().map(|l| &l.child) == b.last().map(|l| &l.child)
            && !a.is_empty()
            && !b.is_empty()
        {
            a.pop();
            b.pop();
        }
        for l in a {
            out = l.outward(out);
        }
        for l in b.into_iter().rev() {
            out = l.inward(out);
        }
        finite(out.meters)?;
        finite(out.meters_per_second)?;
        Ok(out)
    }
    pub fn galactic_position(
        &self,
        frame: &FrameId,
        state: State,
        epoch: Epoch,
    ) -> Result<GalacticPosition, Error> {
        let system = self.transform(
            frame,
            &FrameId::System(self.anchor.system.clone()),
            state,
            epoch,
        )?;
        self.anchor
            .position_at(epoch)?
            .translated(self.anchor.rotation.apply(system.meters))
    }
}

impl FrameTree {
    pub fn from_galactic(
        &self,
        frame: &FrameId,
        position: GalacticPosition,
        velocity_mps: [f64; 3],
        epoch: Epoch,
    ) -> Result<State, Error> {
        finite(velocity_mps)?;
        let root = FrameId::System(self.anchor.system.clone());
        let system = State {
            meters: self
                .anchor
                .rotation
                .inverse(position.relative_meters(self.anchor.position_at(epoch)?)?),
            meters_per_second: self
                .anchor
                .rotation
                .inverse(sub(velocity_mps, self.anchor.velocity_mps)),
        };
        self.transform(&root, frame, system, epoch)
    }
}

#[cfg(test)]
mod tests;
