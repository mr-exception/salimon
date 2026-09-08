//! Versioned canonical physical organisms. Rendering density never changes this publication.
use super::{
    EnvironmentError as Error,
    ecology::BiosphereDefinition,
    organisms::{Form, Grammar, Specimen},
    random,
};
use salimon_universe::synthesis::sha256;
pub const PLACEMENT_VERSION: u16 = 1;
#[derive(Clone, Debug, PartialEq)]
pub struct Exclusion {
    pub id: String,
    pub bounds: [f64; 6],
}
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicalPart {
    pub id: u16,
    pub start_meters: [f64; 3],
    pub end_meters: [f64; 3],
    pub bounds: [f64; 6],
}
#[derive(Clone, Debug, PartialEq)]
pub struct Organism {
    pub id: String,
    pub site_id: String,
    pub position_meters: [f64; 3],
    pub axes: [[f64; 3]; 3],
    pub specimen: Specimen,
    pub parts: Vec<PhysicalPart>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Population {
    biosphere_identity: String,
    identity: String,
    objects: Vec<Organism>,
}
fn norm(p: [f64; 3]) -> f64 {
    p[0].hypot(p[1]).hypot(p[2])
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn point(origin: [f64; 3], axes: [[f64; 3]; 3], p: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| origin[i] + (0..3).map(|j| axes[j][i] * p[j]).sum::<f64>())
}
fn overlap(a: [f64; 6], b: [f64; 6]) -> bool {
    (0..3).all(|i| a[i * 2] <= b[i * 2 + 1] && a[i * 2 + 1] >= b[i * 2])
}
fn valid(b: [f64; 6]) -> bool {
    b.iter().all(|v| v.is_finite()) && (0..3).all(|i| b[i * 2] < b[i * 2 + 1])
}
impl Population {
    pub fn publish(
        biosphere: &BiosphereDefinition,
        exclusions: &[Exclusion],
    ) -> Result<Self, Error> {
        if exclusions.len() > 1024
            || exclusions
                .iter()
                .any(|z| z.id.is_empty() || z.id.len() > 160 || !valid(z.bounds))
        {
            return Err(Error::InvalidDefinition);
        }
        let mut zones = exclusions.to_vec();
        zones.sort_by(|a, b| a.id.cmp(&b.id));
        if zones.windows(2).any(|z| z[0].id == z[1].id) {
            return Err(Error::InvalidDefinition);
        }
        let mut recipe = format!(
            "physical-organisms-{PLACEMENT_VERSION}-grammar-{}:{}",
            super::organisms::ORGANISM_VERSION,
            biosphere.identity()
        )
        .into_bytes();
        for z in &zones {
            recipe.extend((z.id.len() as u16).to_le_bytes());
            recipe.extend(z.id.as_bytes());
            for v in z.bounds {
                recipe.extend(v.to_le_bytes());
            }
        }
        let identity = sha256(&recipe);
        let mut objects = vec![];
        if let Some(grammar) = Grammar::from_biosphere(biosphere) {
            for site in biosphere.sites()? {
                for slot in 0..4u8 {
                    let id = sha256(format!("{identity}:{}:{slot}", site.id).as_bytes());
                    let seed = id
                        .bytes()
                        .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(u64::from(b)));
                    let reference = if site.direction[1].abs() > 0.9 {
                        [1., 0., 0.]
                    } else {
                        [0., 1., 0.]
                    };
                    let tangent = cross(site.direction, reference);
                    let t = tangent.map(|v| v / norm(tangent));
                    let u = cross(t, site.direction);
                    let hint = std::array::from_fn(|i| {
                        site.body_local_position_meters[i]
                            + t[i] * (random(seed, 0) - 0.5) * 80.
                            + u[i] * (random(seed, 1) - 0.5) * 80.
                    });
                    let n = norm(hint);
                    let direction = hint.map(|v| v / n);
                    let sample = biosphere.climate().geology().terrain().sample(direction)?;
                    if sample.slope > 0.4 {
                        continue;
                    }
                    let up = sample.outward_normal;
                    let reference = if up[1].abs() > 0.9 {
                        [1., 0., 0.]
                    } else {
                        [0., 1., 0.]
                    };
                    let right = cross(up, reference);
                    let right = right.map(|v| v / norm(right));
                    let forward = cross(right, up);
                    let axes = [right, up, forward];
                    let form = match seed % 12 {
                        0 => Form::Giant,
                        1..=4 => Form::Column,
                        5..=8 => Form::Frond,
                        _ => Form::Colony,
                    };
                    let specimen = grammar.specimen(form, slot)?;
                    let origin = sample.body_local_position_meters;
                    let mut parts = vec![];
                    let mut supported = true;
                    for p in &specimen.parts {
                        let a = point(origin, axes, p.start);
                        let b = point(origin, axes, p.end);
                        let radius = p.radius_start.max(p.radius_end);
                        if p.start[1].abs() < 1e-9 {
                            let n = norm(a);
                            let ground = biosphere
                                .climate()
                                .geology()
                                .terrain()
                                .sample(a.map(|v| v / n))?;
                            let gap = norm(a) - norm(ground.body_local_position_meters);
                            if gap.abs() > 0.35 || ground.slope > 0.4 {
                                supported = false;
                                break;
                            }
                        }
                        let bounds = std::array::from_fn(|i| {
                            if i % 2 == 0 {
                                a[i / 2].min(b[i / 2]) - radius
                            } else {
                                a[i / 2].max(b[i / 2]) + radius
                            }
                        });
                        if zones.iter().any(|z| overlap(bounds, z.bounds)) {
                            supported = false;
                            break;
                        }
                        parts.push(PhysicalPart {
                            id: p.id,
                            start_meters: a,
                            end_meters: b,
                            bounds,
                        });
                    }
                    if supported
                        && !objects.iter().any(|other: &Organism| {
                            other
                                .parts
                                .iter()
                                .any(|a| parts.iter().any(|b| overlap(a.bounds, b.bounds)))
                        })
                    {
                        objects.push(Organism {
                            id,
                            site_id: site.id.clone(),
                            position_meters: origin,
                            axes,
                            specimen,
                            parts,
                        });
                    }
                }
            }
        }
        objects.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Self {
            biosphere_identity: biosphere.identity().into(),
            identity,
            objects,
        })
    }
    pub fn biosphere_identity(&self) -> &str {
        &self.biosphere_identity
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn objects(&self) -> &[Organism] {
        &self.objects
    }
    /// Derived baseline binding. Future mutable harvesting deltas must bind this exact identity.
    pub fn binding(&self) -> Vec<u8> {
        let mut b = b"SLMORG01".to_vec();
        b.extend(PLACEMENT_VERSION.to_le_bytes());
        b.extend(self.identity.as_bytes());
        b.extend((self.objects.len() as u32).to_le_bytes());
        for o in &self.objects {
            b.extend(o.id.as_bytes());
            b.extend((o.parts.len() as u16).to_le_bytes());
            for p in &o.parts {
                b.extend(p.id.to_le_bytes());
            }
        }
        b
    }
    pub fn verify_binding(&self, bytes: &[u8]) -> Result<(), Error> {
        if self.binding() != bytes {
            Err(Error::IncompatibleDefinition)
        } else {
            Ok(())
        }
    }
    pub fn nearby(&self, position: [f64; 3], range_meters: f64) -> Result<Vec<&Organism>, Error> {
        if !position.iter().all(|x| x.is_finite()) || !range_meters.is_finite() || range_meters < 0.
        {
            return Err(Error::InvalidDefinition);
        }
        Ok(self
            .objects
            .iter()
            .filter(|o| {
                let p = std::array::from_fn(|i| o.position_meters[i] - position[i]);
                norm(p) <= range_meters + 20.
            })
            .collect())
    }
}
#[cfg(test)]
mod tests;
