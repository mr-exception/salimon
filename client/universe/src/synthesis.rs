//! U10 offline canonical fiction. Never called by gameplay or source importers.
use crate::{
    Body, BodyKind, Catalog, CatalogId, Definition, Error, Header, Population, PopulationKind,
    Provenance, Quantity, Record, SeedDomain, Star, System, Uncertainty,
    coordinates::GalacticPosition,
    inference::{AU_METERS, SOLAR_MASS_KG, SOLAR_RADIUS_METERS, YEAR_SECONDS},
    invalid,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const BUILDER: &str = "salimon-fiction-v1";
pub const VERSION: u32 = 1;
pub const NAMESPACE: &str = "salimon-fiction";
pub const PARSEC_METERS: f64 = 3.085677581491367e16;
pub const MAX_SYSTEMS: usize = 2048;
pub const MAX_PUBLICATION_BYTES: usize = 64 * 1024 * 1024;
const G: f64 = 6.67430e-11;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    /// Permanent publication key, lowercase ASCII letters/digits/hyphens, <=32.
    pub key: String,
    /// Caller-reviewed Milky Way region in U02 galactic axes. No galaxy model is
    /// inferred from a name; this cuboid is the full placement support.
    pub center: GalacticPosition,
    pub extent_parsecs: [f64; 3],
    /// Residual *unobserved* single-star system density after source reconciliation.
    /// Must not be total density added on top of Gaia observations.
    pub residual_systems_per_cubic_parsec: f64,
    pub age_gyr: [f64; 2],
    pub metallicity_feh: [f64; 2],
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub seed: u64,
    pub revision: String,
    /// Reviewed model/source/authoring reference; never presented as observation.
    pub rationale: String,
    pub regions: Vec<Region>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StellarSource {
    pub system: CatalogId,
    pub star: CatalogId,
    pub region: String,
    pub position: GalacticPosition,
    pub age_seconds: f64,
    pub metallicity_feh: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Orbit {
    pub object: CatalogId,
    pub parent: CatalogId,
    pub semi_major_axis_meters: f64,
    pub eccentricity: f64,
    pub period_seconds: f64,
    // Phase/orientation/epoch remain absent: U14 owns complete state evaluation.
}
/// Generation inputs, not U11's full terrain/geology definition or evaluator.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BodyRecipe {
    pub builder: String,
    pub object: CatalogId,
    pub parent: CatalogId,
    pub body_kind: BodyKind,
    pub mass_kg: f64,
    pub radius_meters: f64,
    pub bulk_density_kg_per_m3: f64,
    pub age_seconds: f64,
    pub metallicity_feh: f64,
    pub orbit: Orbit,
    pub domains: Vec<SeedDomain>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PopulationRecipe {
    pub builder: String,
    pub object: CatalogId,
    pub parent: CatalogId,
    pub population_kind: PopulationKind,
    /// Remaining unmaterialized members; individually selected bodies are excluded.
    pub estimated_count: u64,
    pub radial_support_meters: [f64; 2],
    pub radius_support_meters: [f64; 2],
    pub bulk_density_kg_per_m3: f64,
    pub distribution: String,
    pub seed: u64,
}
/// Immutable publication; hashes are computed from canonical bytes, not supplied
/// by callers. U12 must package these sidecars along with the schema-1 catalog.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Publication {
    builder: String,
    request: Request,
    input_catalog_sha256: String,
    catalog: Catalog,
    sources: Vec<StellarSource>,
    bodies: BTreeMap<CatalogId, BodyRecipe>,
    populations: BTreeMap<CatalogId, PopulationRecipe>,
}
impl Publication {
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn sources(&self) -> &[StellarSource] {
        &self.sources
    }
    pub fn bodies(&self) -> &BTreeMap<CatalogId, BodyRecipe> {
        &self.bodies
    }
    pub fn populations(&self) -> &BTreeMap<CatalogId, PopulationRecipe> {
        &self.populations
    }
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let bytes = json(self)?;
        if bytes.len() > MAX_PUBLICATION_BYTES {
            return Err(invalid("fiction publication byte limit"));
        }
        Ok(bytes)
    }
    pub fn sha256(&self) -> Result<String, Error> {
        Ok(sha256(&self.encode()?))
    }
    pub fn catalog_sha256(&self) -> Result<String, Error> {
        Ok(sha256(&self.catalog.encode()?))
    }
}
fn json(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|e| Error::Encoding(e.to_string()))
}
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
// Length-prefixed parts prevent ambiguous concatenation. No language hash/RNG,
// time, session or iteration order. Version and named field separate all lanes.
fn lane(seed: u64, key: &str, field: &str) -> u64 {
    let mut h = Sha256::new();
    h.update(seed.to_le_bytes());
    for part in [BUILDER, key, field] {
        h.update((part.len() as u64).to_le_bytes());
        h.update(part.as_bytes());
    }
    u64::from_le_bytes(
        h.finalize()[..8]
            .try_into()
            .expect("SHA256 has eight prefix bytes"),
    )
}
fn unit(seed: u64, key: &str, field: &str) -> f64 {
    // 32-bit dyadic interior sample; exact representable fractions on all hosts.
    ((lane(seed, key, field) >> 32) as f64 + 0.5) / 4294967296.0
}
fn sample(seed: u64, key: &str, field: &str, range: [f64; 2]) -> f64 {
    range[0] + (range[1] - range[0]) * unit(seed, key, field)
}
fn id(key: &str) -> Result<CatalogId, Error> {
    CatalogId::from_source(NAMESPACE, key)
}
fn provenance(r: &Request) -> Provenance {
    Provenance::Fiction {
        builder: BUILDER.into(),
        revision: r.revision.clone(),
        rationale: r.rationale.clone(),
    }
}
fn header(key: &str, r: &Request) -> Result<Header, Error> {
    Ok(Header {
        id: id(key)?,
        display_name: None,
        provenance: provenance(r),
    })
}
fn quantity(value: f64, r: &Request) -> Quantity {
    Quantity {
        value,
        uncertainty: Uncertainty::Unknown,
        provenance: provenance(r),
        validity: None,
    }
}
fn range(v: [f64; 2], min: f64, max: f64) -> bool {
    v.iter().all(|x| x.is_finite()) && v[0] >= min && v[0] <= v[1] && v[1] <= max
}
fn validate(base: &Catalog, r: &mut Request) -> Result<Vec<usize>, Error> {
    if r.version != VERSION || r.regions.is_empty() || r.regions.len() > 32 {
        return Err(invalid("fiction version/region count"));
    }
    if r.revision == base.revision() {
        return Err(invalid("fiction needs new catalog revision"));
    }
    // Reserve the whole namespace, including source dataset IDs. Never dedup or
    // rename a real observation into fiction, even if it occupies the same place.
    if base
        .records()
        .iter()
        .map(Record::id)
        .chain(base.datasets().iter().map(|d| &d.id))
        .any(|x| x.as_str().starts_with(&format!("{NAMESPACE}:")))
    {
        return Err(invalid(
            "fiction namespace already occupied in base catalog",
        ));
    }
    // Validate bounded revision/rationale even when density produces zero records.
    crate::validation::provenance(&provenance(r), &BTreeSet::new())?;
    Catalog::new(r.revision.clone(), vec![], vec![])?;
    r.regions.sort_by(|a, b| a.key.cmp(&b.key));
    let mut keys = BTreeSet::new();
    let mut counts = vec![];
    let mut total = 0;
    for region in &r.regions {
        if region.key.is_empty()
            || region.key.len() > 32
            || !region
                .key
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || !keys.insert(&region.key)
            || !region
                .extent_parsecs
                .iter()
                .all(|x| x.is_finite() && *x > 0.0 && *x <= 1000.0)
            || !region.residual_systems_per_cubic_parsec.is_finite()
            || !(0.0..=1000.0).contains(&region.residual_systems_per_cubic_parsec)
            || !range(region.age_gyr, 0.01, 13.8)
            || !range(region.metallicity_feh, -2.5, 0.5)
        {
            return Err(invalid("fiction region key/extent/density/age/metallicity"));
        }
        let expected = region.extent_parsecs.iter().product::<f64>()
            * region.residual_systems_per_cubic_parsec;
        if !expected.is_finite() || expected > MAX_SYSTEMS as f64 {
            return Err(invalid("fiction expected system limit"));
        }
        let count = expected.floor() as usize
            + usize::from(unit(r.seed, &region.key, "count") < expected.fract());
        total += count;
        if total > MAX_SYSTEMS {
            return Err(invalid("fiction total system limit"));
        }
        counts.push(count);
    }
    for (i, a) in r.regions.iter().enumerate() {
        for b in &r.regions[i + 1..] {
            let delta = a.center.relative_meters(b.center)?;
            if (0..3).all(|axis| {
                delta[axis].abs()
                    < (a.extent_parsecs[axis] + b.extent_parsecs[axis]) * 0.5 * PARSEC_METERS
            }) {
                return Err(invalid(
                    "fiction region supports overlap; reconcile residual density first",
                ));
            }
        }
    }
    Ok(counts)
}
/// Build a new immutable revision. Density is explicitly residual after observed
/// cross-match; geometric proximity alone cannot identify an observed source.
pub fn build(base: &Catalog, mut request: Request) -> Result<Publication, Error> {
    let counts = validate(base, &mut request)?;
    let mut records = base.records().to_vec();
    let mut output = Publication {
        builder: BUILDER.into(),
        input_catalog_sha256: sha256(&base.encode()?),
        catalog: base.clone(),
        request: request.clone(),
        sources: vec![],
        bodies: BTreeMap::new(),
        populations: BTreeMap::new(),
    };
    for (region, count) in request.regions.iter().zip(counts) {
        for slot in 0..count {
            system(&mut output, &mut records, region, slot)?;
        }
    }
    output.sources.sort_by(|a, b| a.system.cmp(&b.system));
    output.catalog = Catalog::new(request.revision.clone(), base.datasets().to_vec(), records)?;
    // Bounds include the observed catalog and all publication sidecars.
    output.catalog.encode()?;
    output.encode()?;
    Ok(output)
}
fn system(
    out: &mut Publication,
    records: &mut Vec<Record>,
    region: &Region,
    slot: usize,
) -> Result<(), Error> {
    let r = out.request.clone();
    let seed = r.seed;
    let key = format!("{}.s{slot:06}", region.key);
    let sys = header(&key, &r)?;
    let star = header(&format!("{key}.star"), &r)?;
    let age = sample(seed, &key, "age-gyr", region.age_gyr);
    let feh = sample(seed, &key, "feh", region.metallicity_feh);
    // Coarse low-mass-dominated IMF proxy, truncated by main-sequence lifetime.
    // Integer milli-solar mass and integer-metre radius freeze numeric outputs.
    let draw = unit(seed, &key, "imf");
    let upper: u32 = if draw < 0.8 {
        500
    } else if draw < 0.98 {
        1000
    } else {
        2000
    };
    let lower = if draw < 0.8 {
        100
    } else if draw < 0.98 {
        500
    } else {
        1000
    };
    let mut mass_milli =
        lower + (lane(seed, &key, "stellar-mass") % u64::from(upper - lower)) as u32;
    while lifetime_gyr(mass_milli) < age {
        mass_milli -= 1;
    }
    let sm = SOLAR_MASS_KG * f64::from(mass_milli) / 1000.0;
    let sr = stellar_radius(mass_milli);
    let displacement = std::array::from_fn(|i| {
        sample(seed, &key, ["x", "y", "z"][i], [-0.5, 0.5])
            * region.extent_parsecs[i]
            * PARSEC_METERS
    });
    out.sources.push(StellarSource {
        system: sys.id.clone(),
        star: star.id.clone(),
        region: region.key.clone(),
        position: region.center.translated(displacement)?,
        age_seconds: age * 1e9 * YEAR_SECONDS,
        metallicity_feh: feh,
    });
    records.push(Record::System(System {
        header: sys.clone(),
    }));
    records.push(Record::Star(Star {
        header: star.clone(),
        system: sys.id.clone(),
        mass_kg: Some(quantity(sm, &r)),
        radius_meters: Some(quantity(sr, &r)),
    }));
    // More metal-rich disks yield more planets and a larger giant fraction.
    let richness = ((feh + 2.5) / 3.0).clamp(0.0, 1.0);
    let n = 1
        + (lane(seed, &key, "planet-count") % 3) as usize
        + usize::from(unit(seed, &key, "rich-disk") < richness);
    for p in 0..n {
        let pk = format!("{key}.p{p:02}");
        let giant = p >= 2 && unit(seed, &pk, "giant") < 0.05 + 0.6 * richness;
        let radius = if giant {
            sample(seed, &pk, "radius", [50e6, 85e6])
        } else {
            sample(seed, &pk, "radius", [2e6, 9e6])
        }
        .round();
        let density = if giant {
            sample(seed, &pk, "density", [500.0, 2000.0])
        } else {
            sample(seed, &pk, "density", [3500.0, 7000.0])
        }
        .round();
        let a =
            AU_METERS * 0.4 * f64::from(6u32.pow(p as u32)) * sample(seed, &pk, "axis", [1.0, 1.1]);
        let pm = density * volume(radius);
        // All adjacent planets have radial ratio >5.4, ensuring >3.5 mutual
        // Hill radii even for the largest admitted giant at the smallest star.
        add_body(
            out,
            records,
            &sys.id,
            &pk,
            &star.id,
            BodyKind::Planet,
            radius,
            density,
            a,
            sm,
            sr,
            age,
            feh,
        )?;
        // One individually materialized major moon, only when Roche/Hill window
        // exists; absence is physical, never a fallback onto an invalid orbit.
        let mr = (radius * if giant { 0.025 } else { 0.08 }).round();
        let md = 3300.0;
        let moon_axis = 10.0 * radius;
        if cube(moon_axis) < cube(0.2 * a) * pm / (3.0 * sm) {
            add_body(
                out,
                records,
                &sys.id,
                &format!("{pk}.m00"),
                &id(&pk)?,
                BodyKind::Moon,
                mr,
                md,
                moon_axis,
                pm,
                radius,
                age,
                feh,
            )?;
        }
    }
    // Selected large belt object is separate from aggregate membership. Place
    // belts beyond the outermost possible planet (max ~95 AU).
    add_body(
        out,
        records,
        &sys.id,
        &format!("{key}.minor00"),
        &star.id,
        BodyKind::MinorBody,
        250_000.0,
        2200.0,
        150.0 * AU_METERS,
        sm,
        sr,
        age,
        feh,
    )?;
    for (suffix, kind, count, support, radii, density) in [
        (
            "belt",
            PopulationKind::Asteroids,
            100_000u64,
            [140.0, 180.0],
            [1.0, 250_000.0],
            2200.0,
        ),
        (
            "cloud",
            PopulationKind::Comets,
            1_000_000u64,
            [1000.0, 10_000.0],
            [1.0, 50_000.0],
            600.0,
        ),
    ] {
        let pop_key = format!("{key}.{suffix}");
        let pop_header = header(&pop_key, &r)?;
        let recipe = PopulationRecipe { builder: BUILDER.into(), object: pop_header.id.clone(), parent: star.id.clone(), population_kind: kind,
            estimated_count: count - u64::from(suffix=="belt"), radial_support_meters: support.map(|x| x*AU_METERS), radius_support_meters: radii,
            bulk_density_kg_per_m3: density, distribution: "uniform radial support; differential radius number proportional to radius^-3.5; not allocated members".into(), seed: lane(seed,&pop_key,"population") };
        let digest = sha256(&json(&recipe)?);
        records.push(Record::Population(Population {
            header: pop_header.clone(),
            system: sys.id.clone(),
            parent: star.id.clone(),
            population_kind: kind,
            estimated_count: recipe.estimated_count,
            descriptor: recipe.distribution.clone(),
            definition: Definition {
                version: VERSION,
                sha256: digest.clone(),
                domains: vec![SeedDomain {
                    name: "population".into(),
                    seed: recipe.seed,
                    algorithm_version: VERSION,
                    definition_sha256: digest,
                    dependencies: vec![],
                }],
            },
        }));
        out.populations.insert(pop_header.id, recipe);
    }
    Ok(())
}
fn lifetime_gyr(milli: u32) -> f64 {
    let m = f64::from(milli) / 1000.0;
    10.0 / (m * m * m.sqrt())
}
// Integer-metre power-law radius without platform libm powf. Monotone search
// for (R/Rsun)^5 <= (M/Msun)^4 uses only basic IEEE operations.
fn stellar_radius(milli: u32) -> f64 {
    let m = f64::from(milli) / 1000.0;
    let target = (m * m) * (m * m);
    let (mut lo, mut hi) = (1u32, 2_000_000_000u32);
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        let x = f64::from(mid) / SOLAR_RADIUS_METERS;
        if (x * x) * (x * x) * x <= target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    f64::from(lo)
}

// Fixed multiplication order: f64::powi permits platform-dependent precision.
fn cube(value: f64) -> f64 {
    (value * value) * value
}

fn volume(r: f64) -> f64 {
    (4.0 / 3.0) * std::f64::consts::PI * cube(r)
}
#[allow(clippy::too_many_arguments)]
fn add_body(
    out: &mut Publication,
    records: &mut Vec<Record>,
    system: &CatalogId,
    key: &str,
    parent: &CatalogId,
    kind: BodyKind,
    radius: f64,
    density: f64,
    axis: f64,
    parent_mass: f64,
    parent_radius: f64,
    age: f64,
    feh: f64,
) -> Result<(), Error> {
    let r = &out.request;
    let h = header(key, r)?;
    let mass = density * volume(radius);
    if axis <= parent_radius + radius || cube(axis) <= cube(2.44 * radius) * (parent_mass / mass) {
        return Err(invalid("fiction orbit violates surface/Roche bound"));
    }
    let orbit = Orbit {
        object: h.id.clone(),
        parent: parent.clone(),
        semi_major_axis_meters: axis,
        eccentricity: 0.0,
        period_seconds: std::f64::consts::TAU * (cube(axis) / (G * (parent_mass + mass))).sqrt(),
    };
    let mut domains = vec![];
    for name in ["resources", "terrain", "visual"] {
        let seed = lane(r.seed, key, name);
        // Domain reference hashes actual reserved input bytes. U11 must publish
        // full algorithm/parameter definitions before any terrain evaluation.
        let bytes = json(&(BUILDER, &h.id, name, seed, radius, density))?;
        domains.push(SeedDomain {
            name: name.into(),
            seed,
            algorithm_version: VERSION,
            definition_sha256: sha256(&bytes),
            dependencies: vec![],
        });
    }
    let recipe = BodyRecipe {
        builder: BUILDER.into(),
        object: h.id.clone(),
        parent: parent.clone(),
        body_kind: kind,
        mass_kg: mass,
        radius_meters: radius,
        bulk_density_kg_per_m3: density,
        age_seconds: age * 1e9 * YEAR_SECONDS,
        metallicity_feh: feh,
        orbit,
        domains: domains.clone(),
    };
    let definition = Definition {
        version: VERSION,
        sha256: sha256(&json(&recipe)?),
        domains,
    };
    records.push(Record::Body(Body {
        header: h.clone(),
        system: system.clone(),
        parent: parent.clone(),
        body_kind: kind,
        mass_kg: Some(quantity(mass, r)),
        radius_meters: Some(quantity(radius, r)),
        definition: Some(definition),
    }));
    out.bodies.insert(h.id, recipe);
    Ok(())
}

#[cfg(test)]
mod tests;
