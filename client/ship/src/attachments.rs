//! Mechanical attachment identity and ownership, independent of power/rendering.
use salimon_persistence::{SaveError, Validate, invalid};
use salimon_world::resources::FragmentId;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mount {
    pub id: u64,
    /// Stable physical rail identity; rail coordinates are ship local.
    pub rail: u64,
    pub rail_origin: [f64; 3],
    pub offset: [f64; 3],
    pub radius: f64,
    pub secured: Option<FragmentId>,
}
impl Mount {
    pub fn position(&self) -> [f64; 3] {
        std::array::from_fn(|i| self.rail_origin[i] + self.offset[i])
    }
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarryingMount {
    pub id: u64,
    pub world_position: [f64; 3],
}
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attachments {
    pub carried: Option<CarryingMount>,
    pub mounts: Vec<Mount>,
}
impl Attachments {
    pub fn starter() -> Self {
        Self {
            carried: None,
            mounts: vec![
                Mount {
                    id: 2001,
                    rail: 2000,
                    rail_origin: [-1.6, 1.15, 2.7],
                    offset: [0.; 3],
                    radius: 0.15,
                    secured: None,
                },
                Mount {
                    id: 2011,
                    rail: 2010,
                    rail_origin: [-5.8, 0.30, 1.7],
                    offset: [0.; 3],
                    radius: 0.35,
                    secured: None,
                },
            ],
        }
    }
}
impl Validate for Attachments {
    fn validate(&self) -> Result<(), SaveError> {
        if self.mounts.len() > 1024 {
            return Err(invalid("attachment budget"));
        }
        if let Some(c) = &self.carried
            && (c.id != 2001
                || !c.world_position.iter().all(|v| v.is_finite())
                || !self
                    .mounts
                    .iter()
                    .any(|m| m.id == c.id && m.secured.is_none()))
        {
            return Err(invalid("carried accessory source"));
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut items = std::collections::BTreeSet::new();
        for m in &self.mounts {
            let registered = match m.id {
                2001 => {
                    HABITATION_RAILS
                        .iter()
                        .any(|(id, p)| *id == m.rail && *p == m.rail_origin)
                        && m.radius == 0.15
                }
                2011 => m.rail == 2010 && m.rail_origin == [-5.8, 0.30, 1.7] && m.radius == 0.35,
                _ => false,
            };
            if !registered
                || m.offset != [0.; 3]
                || m.id == m.rail
                || !ids.insert(m.id)
                || !m.radius.is_finite()
                || m.radius <= 0.
                || m.radius > 1.
                || m.position()
                    .iter()
                    .chain(m.rail_origin.iter())
                    .chain(m.offset.iter())
                    .any(|v| !v.is_finite() || v.abs() > 10000.)
                || m.secured.is_some_and(|id| !items.insert(id.0))
            {
                return Err(invalid("attachment identity/geometry"));
            }
        }
        Ok(())
    }
}
impl Attachments {
    pub fn is_secured(&self, item: FragmentId) -> bool {
        self.mounts.iter().any(|m| m.secured == Some(item))
    }
    /// Capture the original object in its real pose. No teleport, identity or mass mutation.
    pub fn secure(
        &mut self,
        mount: u64,
        item: FragmentId,
        position: [f64; 3],
        radius: f64,
    ) -> bool {
        if self.is_secured(item) || !radius.is_finite() || radius <= 0. {
            return false;
        }
        let Some(m) = self.mounts.iter_mut().find(|m| m.id == mount) else {
            return false;
        };
        let delta = salimon_math::sub(position, m.position());
        if m.secured.is_some()
            || radius > m.radius
            || delta.iter().any(|v| !v.is_finite())
            || delta[0].abs() + radius > 0.5
            || delta[2].abs() + radius > 0.175
            || (delta[1] - (0.03 + radius)).abs() > 0.02
        {
            return false;
        }
        m.secured = Some(item);
        true
    }
    pub fn release(&mut self, mount: u64) -> Option<FragmentId> {
        self.mounts
            .iter_mut()
            .find(|m| m.id == mount)?
            .secured
            .take()
    }
    /// Removal/damage callers must explicitly release returned children before removing geometry.
    pub fn remove_rail(&mut self, rail: u64) -> Vec<FragmentId> {
        let mut released = Vec::new();
        self.mounts.retain(|m| {
            if m.rail == rail {
                released.extend(m.secured);
                false
            } else {
                true
            }
        });
        released
    }
}
/// Existing portable shelf is included in dry hull mass, not newly fabricated.
/// Physical mass is catalog data; no material cost or construction time is implied.
pub const PORTABLE_SHELF_MASS_KG: f64 = 2.0;
pub const HABITATION_RAILS: &[(u64, [f64; 3])] = &[
    (2000, [-1.6, 1.15, 2.7]),
    (2020, [-2.8, 1.15, 2.7]),
    (2030, [0.0, 1.15, -2.7]),
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_identity_compatibility_uniqueness_and_parent_removal() {
        let mut a = Attachments::starter();
        let p = [-1.6, 1.28, 2.7];
        assert!(a.secure(2001, FragmentId(7), p, 0.1));
        assert!(!a.secure(2011, FragmentId(7), [-5.8, 0.43, 1.7], 0.1));
        assert!(!a.secure(2001, FragmentId(8), p, 0.1));
        assert_eq!(a.remove_rail(2000), vec![FragmentId(7)]);
        assert!(!a.is_secured(FragmentId(7)));
        a.validate().unwrap();
    }
}
