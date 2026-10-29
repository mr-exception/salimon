//! Rare natural sites reference existing canonical physical organisms and geological sources.
use super::{
    EnvironmentError as Error,
    activity::ActivityDefinition,
    ecology::BiosphereDefinition,
    landmarks::Landmarks,
    organism_placement::{Organism, Population},
    organisms::Form,
    random,
};
use salimon_universe::synthesis::sha256;
pub const SIGNATURE_SITE_VERSION: u16 = 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    GiantOrganism,
    DenseColony,
    MixedGeological,
    GeothermalEcology,
}
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredSignature {
    pub key: String,
    pub organism_id: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Site {
    pub id: String,
    pub authored_key: Option<String>,
    pub kind: Kind,
    pub organism_ids: Vec<String>,
    pub geology_site_id: Option<String>,
    pub activity_site_id: Option<String>,
    pub realm_id: String,
    pub position_meters: [f64; 3],
    /// Conservative union of the exact physical source part boxes, not another collider.
    pub bounds: [f64; 6],
}
#[derive(Clone, Debug, PartialEq)]
pub struct SignatureSites {
    identity: String,
    sites: Vec<Site>,
}
fn length(p: [f64; 3]) -> f64 {
    p[0].hypot(p[1]).hypot(p[2])
}
fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    length(std::array::from_fn(|i| a[i] - b[i]))
}
impl SignatureSites {
    pub fn publish(
        biosphere: &BiosphereDefinition,
        population: &Population,
        geology: &Landmarks,
        activity: Option<&ActivityDefinition>,
        authored: &[AuthoredSignature],
    ) -> Result<Self, Error> {
        if population.biosphere_identity() != biosphere.identity()
            || geology.geology() != biosphere.climate().geology()
            || activity.is_some_and(|a| a.climate() != biosphere.climate())
        {
            return Err(Error::IncompatibleDefinition);
        }
        if authored.len() > 2
            || authored.iter().any(|s| {
                s.organism_id.len() != 64
                    || !s.organism_id.bytes().all(|c| c.is_ascii_hexdigit())
                    || s.key.is_empty()
                    || s.key.len() > 128
                    || !s
                        .key
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
            })
        {
            return Err(Error::InvalidDefinition);
        }
        let mut signatures = authored.to_vec();
        signatures.sort_by(|a, b| a.key.cmp(&b.key));
        if signatures.windows(2).any(|a| a[0].key == a[1].key)
            || signatures.iter().enumerate().any(|(i, a)| {
                signatures[..i]
                    .iter()
                    .any(|b| b.organism_id == a.organism_id)
            })
        {
            return Err(Error::InvalidDefinition);
        }
        let mut b = format!(
            "natural-sites-{SIGNATURE_SITE_VERSION}:{}:{}:{}:{}",
            biosphere.identity(),
            population.identity(),
            geology.identity(),
            activity.map_or("none", |a| a.identity())
        )
        .into_bytes();
        for s in &signatures {
            b.extend((s.key.len() as u16).to_le_bytes());
            b.extend(s.key.as_bytes());
            b.extend((s.organism_id.len() as u16).to_le_bytes());
            b.extend(s.organism_id.as_bytes());
        }
        let identity = sha256(&b);
        let mut sites = vec![];
        let mut used = std::collections::BTreeSet::new();
        for signature in &signatures {
            let source = population
                .objects()
                .iter()
                .find(|o| o.id == signature.organism_id)
                .ok_or(Error::InvalidDefinition)?;
            let site = make_site(
                &identity,
                biosphere,
                population,
                geology,
                activity,
                source,
                Some(signature.key.clone()),
            )?
            .ok_or(Error::InvalidDefinition)?;
            if site.organism_ids.iter().any(|id| used.contains(id)) {
                return Err(Error::InvalidDefinition);
            }
            used.extend(site.organism_ids.iter().cloned());
            sites.push(site);
        }
        for source in population.objects() {
            if used.contains(&source.id) {
                continue;
            }
            let seed = source
                .id
                .bytes()
                .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(u64::from(b)));
            if random(seed, 0x624) >= 0.125 {
                continue;
            }
            if let Some(site) = make_site(
                &identity, biosphere, population, geology, activity, source, None,
            )? {
                if site.organism_ids.iter().any(|id| used.contains(id)) {
                    continue;
                }
                used.extend(site.organism_ids.iter().cloned());
                sites.push(site);
            }
        }
        sites.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Self { identity, sites })
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn sites(&self) -> &[Site] {
        &self.sites
    }
    pub fn binding(&self) -> Vec<u8> {
        let mut b = b"SLMNAT01".to_vec();
        b.extend(SIGNATURE_SITE_VERSION.to_le_bytes());
        b.extend(self.identity.as_bytes());
        b.extend((self.sites.len() as u32).to_le_bytes());
        for s in &self.sites {
            b.extend(s.id.as_bytes());
            for id in &s.organism_ids {
                b.extend(id.as_bytes());
            }
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
    pub fn nearby(&self, position: [f64; 3], range: f64) -> Result<Vec<&Site>, Error> {
        if !position.iter().all(|v| v.is_finite()) || !range.is_finite() || range < 0. {
            return Err(Error::InvalidDefinition);
        }
        Ok(self
            .sites
            .iter()
            .filter(|s| {
                let delta = std::array::from_fn(|i| {
                    if position[i] < s.bounds[2 * i] {
                        s.bounds[2 * i] - position[i]
                    } else {
                        (position[i] - s.bounds[2 * i + 1]).max(0.)
                    }
                });
                length(delta) <= range
            })
            .collect())
    }
}
fn make_site(
    identity: &str,
    biosphere: &BiosphereDefinition,
    population: &Population,
    geology: &Landmarks,
    activity: Option<&ActivityDefinition>,
    source: &Organism,
    authored_key: Option<String>,
) -> Result<Option<Site>, Error> {
    let n = length(source.position_meters);
    let ecology = biosphere.sample(source.position_meters.map(|v| v / n))?;
    if ecology.macro_cover_fraction <= 0.02 {
        return Ok(None);
    }
    let geological = geology
        .nearby(source.position_meters, 0.)?
        .into_iter()
        .min_by(|a, b| {
            distance(a.position_meters, source.position_meters)
                .total_cmp(&distance(b.position_meters, source.position_meters))
        });
    let thermal = activity.and_then(|a| {
        if a.inputs().heat_flux_w_m2 > 0. {
            a.sites()
                .iter()
                .filter(|s| {
                    distance(s.body_local_position_meters, source.position_meters)
                        <= s.radius_meters
                })
                .min_by(|a, b| a.id.cmp(&b.id))
        } else {
            None
        }
    });
    let mut objects = vec![source];
    let mut kind = if source.specimen.form == Form::Giant {
        Some(Kind::GiantOrganism)
    } else {
        None
    };
    if source.specimen.form == Form::Colony {
        objects = population
            .objects()
            .iter()
            .filter(|o| {
                o.site_id == source.site_id
                    && o.specimen.form == Form::Colony
                    && distance(o.position_meters, source.position_meters) <= 8.
            })
            .collect();
        if objects.len() >= 2 {
            kind = Some(Kind::DenseColony);
        } else {
            objects = vec![source];
        }
    }
    if geological.is_some() {
        kind = Some(Kind::MixedGeological);
    }
    if thermal.is_some() {
        kind = Some(Kind::GeothermalEcology);
    }
    let Some(kind) = kind else {
        return Ok(None);
    };
    objects.sort_by(|a, b| a.id.cmp(&b.id));
    let mut bounds = [
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ];
    for p in objects.iter().flat_map(|o| &o.parts) {
        for i in 0..3 {
            bounds[i * 2] = bounds[i * 2].min(p.bounds[i * 2]);
            bounds[i * 2 + 1] = bounds[i * 2 + 1].max(p.bounds[i * 2 + 1]);
        }
    }
    let organism_ids: Vec<_> = objects.iter().map(|o| o.id.clone()).collect();
    let tag = authored_key.as_deref().unwrap_or("procedural");
    let id = sha256(format!("{identity}:{tag}:{}", organism_ids.join(":")).as_bytes());
    Ok(Some(Site {
        id,
        authored_key,
        kind,
        organism_ids,
        geology_site_id: geological.map(|s| s.id.clone()),
        activity_site_id: thermal.map(|s| s.id.clone()),
        realm_id: ecology.realm_id,
        position_meters: source.position_meters,
        bounds,
    }))
}
#[cfg(test)]
pub(crate) mod tests;
