//! Sparse named observations of the ONE physical heightfield, never a second terrain generator.
use super::{EnvironmentError as Error, random};
use crate::terrain::geology::{GeologicalProcess, GeologyDefinition, GeologyProfile, Substrate};
use salimon_universe::synthesis::sha256;

pub const LANDMARK_VERSION: u16 = 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Geography {
    PublishedFictional,
    Observed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scale {
    Local,
    Regional,
    Planetary,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    ImpactBasin,
    Ridge,
    Scarp,
    IceRidge,
}
/// Explicit authored name/anchor for already published physical relief. No new geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct Signature {
    pub key: String,
    pub direction: [f64; 3],
    pub scale: Scale,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Site {
    pub id: String,
    pub authored_key: Option<String>,
    pub kind: Kind,
    pub scale: Scale,
    pub direction: [f64; 3],
    pub position_meters: [f64; 3],
    pub normal: [f64; 3],
    pub radius_meters: f64,
    pub relief_meters: f64,
    pub province_id: String,
    pub substrate: Substrate,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Landmarks {
    identity: String,
    geology: GeologyDefinition,
    sites: Vec<Site>,
}
fn length(v: [f64; 3]) -> f64 {
    v[0].hypot(v[1]).hypot(v[2])
}
fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = length(v);
    v.map(|x| x / n)
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn radius(scale: Scale, r: f64) -> f64 {
    match scale {
        Scale::Local => 64.0_f64.min(r * 0.01),
        Scale::Regional => (r * 0.001).clamp(256., 10000.).min(r * 0.05),
        Scale::Planetary => (r * 0.02).clamp(1000., 100000.).min(r * 0.1),
    }
}
impl Landmarks {
    pub fn publish(
        geology: GeologyDefinition,
        geography: Geography,
        seed: u64,
        signatures: &[Signature],
    ) -> Result<Self, Error> {
        if signatures.len() > 4
            || signatures.iter().any(|s| {
                s.key.is_empty()
                    || s.key.len() > 128
                    || !s
                        .key
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
                    || !s.direction.iter().all(|x| x.is_finite())
                    || (length(s.direction) - 1.).abs() > 1e-12
            })
        {
            return Err(Error::InvalidDefinition);
        }
        let mut authored = signatures.to_vec();
        authored.sort_by(|a, b| a.key.cmp(&b.key));
        if authored.windows(2).any(|s| s[0].key == s[1].key) {
            return Err(Error::InvalidDefinition);
        }
        let mut bytes = format!(
            "landmarks-{LANDMARK_VERSION}:{}:{geography:?}:{seed}",
            geology.identity()
        )
        .into_bytes();
        for s in &authored {
            bytes.extend((s.key.len() as u16).to_le_bytes());
            bytes.extend(s.key.as_bytes());
            bytes.push(s.scale as u8);
            for x in s.direction {
                bytes.extend(x.to_le_bytes());
            }
        }
        let identity = sha256(&bytes);
        let mut result = Self {
            identity,
            geology,
            sites: vec![],
        };
        // A finite, all-player publication survey. No player position, render level,
        // visit counter, reward quota or mutable RNG enters candidate selection.
        if geography == Geography::PublishedFictional
            && result.geology.profile() != GeologyProfile::HomogeneousRock
        {
            for slot in 0..64u64 {
                if random(seed, slot * 5) >= 0.0625 {
                    continue;
                }
                let z = random(seed, slot * 5 + 1) * 2. - 1.;
                let a = random(seed, slot * 5 + 2) * std::f64::consts::TAU;
                let r = (1. - z * z).sqrt();
                let d = [r * a.cos(), z, r * a.sin()];
                let scale = match slot % 3 {
                    0 => Scale::Local,
                    1 => Scale::Regional,
                    _ => Scale::Planetary,
                };
                let id = sha256(format!("{}:survey:{slot}", result.identity).as_bytes());
                if let Some(site) = result.evaluate(d, scale, id, None)? {
                    result.sites.push(site);
                }
            }
        }
        for s in authored {
            let id = sha256(format!("{}:authored:{}", result.identity, s.key).as_bytes());
            // An authored claim must have actual measurable relief; never install a floating label.
            let site = result
                .evaluate(s.direction, s.scale, id, Some(s.key))?
                .ok_or(Error::InvalidDefinition)?;
            if result.sites.iter().any(|other| {
                length(std::array::from_fn(|i| {
                    other.position_meters[i] - site.position_meters[i]
                })) < other.radius_meters.max(site.radius_meters)
            }) {
                return Err(Error::InvalidDefinition);
            }
            result.sites.push(site);
        }
        result.sites.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(result)
    }
    fn evaluate(
        &self,
        d: [f64; 3],
        scale: Scale,
        id: String,
        authored_key: Option<String>,
    ) -> Result<Option<Site>, Error> {
        let ground = self.geology.sample(d)?;
        let terrain = self.geology.terrain();
        let r = radius(scale, terrain.radius_meters());
        let x = unit(cross(
            d,
            if d[1].abs() > 0.9 {
                [1., 0., 0.]
            } else {
                [0., 1., 0.]
            },
        ));
        let y = cross(d, x);
        let mut heights = vec![ground.surface.elevation_meters];
        for i in 0..8 {
            let a = i as f64 * std::f64::consts::TAU / 8.;
            let q = unit(std::array::from_fn(|j| {
                d[j] * terrain.radius_meters() + r * (x[j] * a.cos() + y[j] * a.sin())
            }));
            heights.push(terrain.sample(q)?.elevation_meters);
        }
        let lo = heights.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = heights.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let relief = hi - lo;
        let threshold = match scale {
            Scale::Local => 2.,
            Scale::Regional => 10.,
            Scale::Planetary => 40.,
        };
        if relief < threshold {
            return Ok(None);
        }
        let g = &ground.geology;
        let impacts = g
            .history
            .iter()
            .any(|h| h.process == GeologicalProcess::Impacts);
        let kind = if impacts && heights[0] < lo + relief * 0.15 {
            Kind::ImpactBasin
        } else if g.substrate == Substrate::WaterIce {
            Kind::IceRidge
        } else if ground.surface.slope > 0.15 {
            Kind::Scarp
        } else {
            Kind::Ridge
        };
        Ok(Some(Site {
            id,
            authored_key,
            kind,
            scale,
            direction: d,
            position_meters: ground.surface.body_local_position_meters,
            normal: ground.surface.outward_normal,
            radius_meters: r,
            relief_meters: relief,
            province_id: g.province_id.clone(),
            substrate: g.substrate,
        }))
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn geology(&self) -> &GeologyDefinition {
        &self.geology
    }
    pub fn sites(&self) -> &[Site] {
        &self.sites
    }
    pub fn nearby(&self, position: [f64; 3], range: f64) -> Result<Vec<&Site>, Error> {
        if !position.iter().all(|x| x.is_finite()) || !range.is_finite() || range < 0. {
            return Err(Error::InvalidDefinition);
        }
        Ok(self
            .sites
            .iter()
            .filter(|s| {
                length(std::array::from_fn(|i| s.position_meters[i] - position[i]))
                    <= range + s.radius_meters
            })
            .collect())
    }
    /// Immutable save/reference binding. Reconstruct from publication inputs before verifying.
    pub fn binding(&self) -> Vec<u8> {
        let mut b = b"SLMLMK01".to_vec();
        b.extend(LANDMARK_VERSION.to_le_bytes());
        b.extend(self.identity.as_bytes());
        b.extend((self.sites.len() as u16).to_le_bytes());
        for s in &self.sites {
            b.extend(s.id.as_bytes());
        }
        b
    }
    pub fn verify_binding(&self, bytes: &[u8]) -> Result<(), Error> {
        if self.binding() == bytes {
            Ok(())
        } else {
            Err(Error::IncompatibleDefinition)
        }
    }
}
#[cfg(test)]
mod tests;
