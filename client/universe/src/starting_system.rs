//! U11 permanent awakening world. Offline publication, never a new-run lottery.
use crate::{
    BodyKind, Catalog, CatalogId, Definition, Error, Record, SeedDomain,
    coordinates::GalacticPosition,
    inference::{AU_METERS, SOLAR_MASS_KG},
    invalid,
    synthesis::{self, BodyRecipe, Publication, Request},
};
use serde::{Deserialize, Serialize};

pub const VERSION: u32 = 1;
pub const LIGHT_YEAR_METERS: f64 = 9_460_730_472_580_800.0;
pub const SYSTEM_ID: &str = "salimon-fiction:awakening.s000000";
pub const STAR_ID: &str = "salimon-fiction:awakening.s000000.star";
pub const PLANET_ID: &str = "salimon-fiction:awakening.s000000.p00";
pub const CANDIDATE_SHA256: &str =
    "294c3687e044dcd644937b0116b28694f9357be2c5489acc73bb14c58ff8c431";
pub const WORLD_SHA256: &str = "e2bfc551786763078fbe8f48dff0cd71813da7786eb39a6906ba19541fd87bef";
const G: f64 = 6.67430e-11;
const MAX_MANIFEST_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedObject {
    pub id: CatalogId,
    pub display_name: String,
}

/// Reference at publication, not an evolving ephemeris or discovered route.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DistanceProof {
    pub solar_barycenter: GalacticPosition,
    pub system_position: GalacticPosition,
    /// Conservative Earth/SSB envelope; includes Earth's orbital displacement.
    pub earth_envelope_meters: f64,
    pub starting_orbit_envelope_meters: f64,
    pub system_distance_light_years: f64,
    pub minimum_planet_earth_distance_light_years: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Physical {
    pub mass_kg: f64,
    pub radius_meters: f64,
    pub bulk_density_kg_per_m3: f64,
    pub gravity_meters_per_second_squared: f64,
    pub star_mass_kg: f64,
    pub star_radius_meters: f64,
    pub semi_major_axis_meters: f64,
    pub period_seconds: f64,
    pub age_seconds: f64,
    pub metallicity_feh: f64,
    /// Model estimate: M^4 luminosity, Bond albedo 0.3, full redistribution.
    /// Not measured surface temperature, climate or a habitability certificate.
    pub equilibrium_temperature_kelvin: f64,
    /// Authored airless surface. Suit oxygen/thermal gameplay remains downstream.
    pub surface_pressure_pascals: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terrain {
    pub algorithm_version: u32,
    pub seed: u64,
    pub ridge_height_meters: f64,
    pub ridge_frequency: u32,
    /// Authored north-pole cap with constant base radius, no procedural relief.
    pub landing_cap_radius_meters: f64,
    pub transition_width_meters: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Material {
    IronOre,
    SilicateRock,
    WaterIce,
    OrganicBiomass,
    MedicinalCompounds,
}
impl Material {
    /// Existing world/fabricator key; this leaf crate must not depend on world.
    pub fn key(self) -> &'static str {
        match self {
            Self::IronOre => "iron-ore",
            Self::SilicateRock => "silicate-rock",
            Self::WaterIce => "water-ice",
            Self::OrganicBiomass => "organic-biomass",
            Self::MedicinalCompounds => "medicinal-compounds",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcrop {
    pub id: CatalogId,
    pub material: Material,
    /// Body-local tangent x/z at north pole. Height uses the same terrain query.
    pub tangent_xz_meters: [f64; 2],
    /// Intermediate walking points avoid the landed ship's clearance disk.
    pub approach_waypoints_xz_meters: Vec<[f64; 2]>,
    pub initial_mass_kg: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub algorithm_version: u32,
    pub seed: u64,
    pub outcrops: Vec<Outcrop>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Visual {
    pub algorithm_version: u32,
    pub seed: u64,
    pub ground_srgb: [u8; 3],
    pub ridge_srgb: [u8; 3],
    pub ice_srgb: [u8; 3],
    pub description: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arrival {
    pub ship_tangent_xz_meters: [f64; 2],
    pub traveler_tangent_xz_meters: [f64; 2],
    pub ship_clearance_radius_meters: f64,
    pub resource_walk_limit_meters: f64,
}

/// Complete selected-world recipe, distinct from U10's body-recipe reference.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub revision: String,
    pub candidate_publication_sha256: String,
    pub candidate_catalog_sha256: String,
    pub source_recipe_sha256: String,
    pub system: NamedObject,
    pub star: NamedObject,
    pub planet: NamedObject,
    pub distance: DistanceProof,
    pub physical: Physical,
    pub terrain: Terrain,
    pub resources: Resources,
    pub visual: Visual,
    pub arrival: Arrival,
    pub provenance: String,
}

/// Only validated, permanent bytes expose evaluators; mutable input DTOs do not.
#[derive(Clone, Debug, PartialEq)]
pub struct FrozenStart {
    manifest: Manifest,
    definition: Definition,
}
impl FrozenStart {
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn definition(&self) -> &Definition {
        &self.definition
    }
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        json(&self.manifest)
    }
    /// Exact immutable v1 publication only. Future revisions need explicit migration.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(invalid("starting manifest byte limit"));
        }
        let manifest: Manifest =
            serde_json::from_slice(bytes).map_err(|e| Error::Encoding(e.to_string()))?;
        let expected = permanent()?;
        // Compare canonical bytes, including signed-zero representation. Merely
        // comparing floating values would accept some changed definition hashes.
        if json(&manifest)? != expected.encode()? {
            return Err(invalid(
                "unsupported or altered permanent starting definition",
            ));
        }
        Ok(expected)
    }

    /// One radial height evaluator for future render/collision/landing adapters.
    /// Direction is body-local; normalization accepts any finite nonzero scale.
    pub fn height_meters(&self, direction: [f64; 3]) -> Result<f64, Error> {
        height(
            &self.manifest.terrain,
            self.manifest.physical.radius_meters,
            direction,
        )
    }
    pub fn surface_point(&self, tangent_xz_meters: [f64; 2]) -> Result<[f64; 3], Error> {
        surface_point(&self.manifest, tangent_xz_meters)
    }
}

fn json(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|e| Error::Encoding(e.to_string()))
}
fn digest(value: &impl Serialize) -> Result<String, Error> {
    Ok(synthesis::sha256(&json(value)?))
}
pub fn candidate_publication() -> Result<Publication, Error> {
    let request: Request =
        serde_json::from_slice(include_bytes!("../fixtures/starting-candidates-v1.json"))
            .map_err(|e| Error::Encoding(e.to_string()))?;
    synthesis::build(
        &Catalog::new("awakening-empty-v1".into(), vec![], vec![])?,
        request,
    )
}
pub fn permanent() -> Result<FrozenStart, Error> {
    freeze(&candidate_publication()?)
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Assessment {
    pub system: CatalogId,
    pub planet: CatalogId,
    pub minimum_distance_light_years: f64,
    pub gravity_meters_per_second_squared: f64,
    pub equilibrium_temperature_kelvin: f64,
    pub eligible: bool,
}

fn solar_barycenter() -> Result<GalacticPosition, Error> {
    // Authored common galactic frame origin (~8.2 kpc solar radius), not Gaia
    // astrometry. Candidate support and Earth envelope share this exact anchor.
    GalacticPosition::new([230_000_000, 0, 0], [0.0; 3])
}
fn distance(p: &Publication, b: &BodyRecipe) -> Result<DistanceProof, Error> {
    let source = p
        .sources()
        .iter()
        .find(|s| s.star == b.parent)
        .ok_or_else(|| invalid("starting star source missing"))?;
    let solar = solar_barycenter()?;
    let delta = source.position.relative_meters(solar)?;
    let d = ((delta[0] * delta[0] + delta[1] * delta[1]) + delta[2] * delta[2]).sqrt();
    let earth = 2.0 * AU_METERS;
    // Triangle inequality; not a favorable selected phase. Added kilometre is
    // larger than binary64 distance rounding at these ~40 ly separations.
    let orbit = b.orbit.semi_major_axis_meters * (1.0 + b.orbit.eccentricity);
    Ok(DistanceProof {
        solar_barycenter: solar,
        system_position: source.position,
        earth_envelope_meters: earth,
        starting_orbit_envelope_meters: orbit,
        system_distance_light_years: d / LIGHT_YEAR_METERS,
        minimum_planet_earth_distance_light_years: (d - earth - orbit - 1000.0) / LIGHT_YEAR_METERS,
    })
}
fn physical(p: &Publication, b: &BodyRecipe) -> Result<Physical, Error> {
    let Some(Record::Star(star)) = p.catalog().get(&b.parent) else {
        return Err(invalid("starting planet needs single stellar parent"));
    };
    let mass = star
        .mass_kg
        .as_ref()
        .ok_or_else(|| invalid("starting stellar mass"))?
        .value;
    let radius = star
        .radius_meters
        .as_ref()
        .ok_or_else(|| invalid("starting stellar radius"))?
        .value;
    let m = mass / SOLAR_MASS_KG;
    // Fourth-root via square roots fixes arithmetic across platforms. For this
    // selected 0.42 Msun host, M^4 is a coarse luminosity heuristic only.
    let luminosity = ((m * m) * (m * m)) * 0.7;
    let temperature =
        278.3 * luminosity.sqrt().sqrt() / (b.orbit.semi_major_axis_meters / AU_METERS).sqrt();
    Ok(Physical {
        mass_kg: b.mass_kg,
        radius_meters: b.radius_meters,
        bulk_density_kg_per_m3: b.bulk_density_kg_per_m3,
        gravity_meters_per_second_squared: G * b.mass_kg / (b.radius_meters * b.radius_meters),
        star_mass_kg: mass,
        star_radius_meters: radius,
        semi_major_axis_meters: b.orbit.semi_major_axis_meters,
        period_seconds: b.orbit.period_seconds,
        age_seconds: b.age_seconds,
        metallicity_feh: b.metallicity_feh,
        equilibrium_temperature_kelvin: temperature,
        surface_pressure_pascals: 0.0,
    })
}
/// Candidate audit, sorted by stable body ID, with explicit narrow screening.
pub fn assess(p: &Publication) -> Result<Vec<Assessment>, Error> {
    let mut result = vec![];
    for b in p
        .bodies()
        .values()
        .filter(|b| b.body_kind == BodyKind::Planet)
    {
        let phys = physical(p, b)?;
        let proof = distance(p, b)?;
        let Some(Record::Body(body)) = p.catalog().get(&b.object) else {
            return Err(invalid("starting body record missing"));
        };
        result.push(Assessment {
            system: body.system.clone(),
            planet: b.object.clone(),
            minimum_distance_light_years: proof.minimum_planet_earth_distance_light_years,
            gravity_meters_per_second_squared: phys.gravity_meters_per_second_squared,
            equilibrium_temperature_kelvin: phys.equilibrium_temperature_kelvin,
            eligible: proof.minimum_planet_earth_distance_light_years >= 30.0
                && (2e6..=9e6).contains(&b.radius_meters)
                && (3500.0..=7000.0).contains(&b.bulk_density_kg_per_m3)
                && (2.0..=12.0).contains(&phys.gravity_meters_per_second_squared)
                && (120.0..=280.0).contains(&phys.equilibrium_temperature_kelvin),
        });
    }
    Ok(result)
}
/// Bind the fixed chosen identity to the exact reviewed candidate publication.
/// Never fall back to a different candidate when data/model versions change.
pub fn freeze(p: &Publication) -> Result<FrozenStart, Error> {
    if p.sha256()? != CANDIDATE_SHA256 {
        return Err(invalid(
            "permanent start candidate publication digest changed",
        ));
    }
    let assessments = assess(p)?;
    let chosen = assessments
        .iter()
        .find(|a| a.eligible)
        .ok_or_else(|| invalid("no admissible starting planet"))?;
    if chosen.system.as_str() != SYSTEM_ID || chosen.planet.as_str() != PLANET_ID {
        return Err(invalid(
            "permanent start selection changed; explicit migration required",
        ));
    }
    let b = &p.bodies()[&chosen.planet];
    let seed = |name: &str| {
        b.domains
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.seed)
            .ok_or_else(|| invalid("missing starting seed domain"))
    };
    let named = |id: &str, name: &str| -> Result<NamedObject, Error> {
        Ok(NamedObject {
            id: CatalogId::try_from(id.to_owned())?,
            display_name: name.into(),
        })
    };
    let resource_seed = seed("resources")?;
    let mut outcrops = vec![];
    for (slot, material, xz, mass) in [
        ("iron", Material::IronOre, [-28.0, 18.0], 500.0),
        ("silicate", Material::SilicateRock, [28.0, 18.0], 500.0),
        ("ice", Material::WaterIce, [0.0, 35.0], 500.0),
        ("organics", Material::OrganicBiomass, [-24.0, -28.0], 200.0),
        (
            "compounds",
            Material::MedicinalCompounds,
            [24.0, -28.0],
            100.0,
        ),
    ] {
        // Stable semantic site keys; amount/visual/iteration order never rekey.
        outcrops.push(Outcrop {
            id: CatalogId::from_source("salimon-start", &format!("awakening.p00.{slot}"))?,
            material,
            tangent_xz_meters: xz,
            approach_waypoints_xz_meters: match material {
                Material::IronOre => vec![[12.0, 12.0]],
                Material::OrganicBiomass => vec![[12.0, -12.0]],
                _ => vec![],
            },
            initial_mass_kg: mass,
        });
    }
    outcrops.sort_by(|a, b| a.id.cmp(&b.id));
    let manifest = Manifest {
        version: VERSION,
        revision: "awakening-world-v1".into(),
        candidate_publication_sha256: CANDIDATE_SHA256.into(),
        candidate_catalog_sha256: p.catalog_sha256()?,
        source_recipe_sha256: digest(b)?,
        system: named(SYSTEM_ID, "Nacre Reach")?,
        star: named(STAR_ID, "Nacre")?,
        planet: named(PLANET_ID, "Vesper")?,
        distance: distance(p, b)?,
        physical: physical(p, b)?,
        terrain: Terrain {
            algorithm_version: VERSION, seed: seed("terrain")?,
            ridge_height_meters: 1800.0,
            ridge_frequency: 3 + (seed("terrain")? % 5) as u32,
            landing_cap_radius_meters: 80.0, transition_width_meters: 500.0,
        },
        resources: Resources { algorithm_version: VERSION, seed: resource_seed, outcrops },
        visual: Visual {
            algorithm_version: VERSION, seed: seed("visual")?,
            ground_srgb: [37, 82, 87], ridge_srgb: [184, 109, 49], ice_srgb: [181, 222, 229],
            description: "Airless frozen teal mineral plains, amber ridges and pale ice outcrops under a red dwarf; no vegetation or breathable atmosphere claim".into(),
        },
        arrival: Arrival {
            ship_tangent_xz_meters: [0.0, 0.0], traveler_tangent_xz_meters: [14.0, 0.0],
            ship_clearance_radius_meters: 12.0, resource_walk_limit_meters: 55.0,
        },
        provenance: "U11 authored permanent selection from U10 fixed fictional cohort. Physical heuristics and seeds preserved. Terrain/palette and resource outcrops are gameplay authoring, not observations. Frozen carbonaceous feedstocks and medicinal-compound material keys do not establish life or pharmacology. See docs/universe-starting-system.md.".into(),
    };
    validate_arrival(&manifest)?;
    let definition = definition(&manifest)?;
    if definition.sha256 != WORLD_SHA256 {
        return Err(invalid(
            "permanent world digest changed; explicit revision required",
        ));
    }
    Ok(FrozenStart {
        manifest,
        definition,
    })
}

fn definition(m: &Manifest) -> Result<Definition, Error> {
    let mut domains = vec![];
    for (name, seed, sha, dependencies) in [
        (
            "resources",
            m.resources.seed,
            digest(&m.resources)?,
            vec!["terrain".into()],
        ),
        (
            "terrain",
            m.terrain.seed,
            digest(&(m.physical.radius_meters, &m.terrain))?,
            vec![],
        ),
        ("visual", m.visual.seed, digest(&m.visual)?, vec![]),
    ] {
        domains.push(SeedDomain {
            name: name.into(),
            seed,
            algorithm_version: VERSION,
            definition_sha256: sha,
            dependencies,
        });
    }
    Ok(Definition {
        version: VERSION,
        sha256: digest(m)?,
        domains,
    })
}

fn normalized(direction: [f64; 3]) -> Result<[f64; 3], Error> {
    if !direction.iter().all(|v| v.is_finite()) {
        return Err(invalid("starting surface direction is nonfinite"));
    }
    let max = direction.iter().copied().map(f64::abs).fold(0.0, f64::max);
    if max == 0.0 {
        return Err(invalid("starting surface direction is zero"));
    }
    let v = direction.map(|v| v / max);
    let norm = ((v[0] * v[0] + v[1] * v[1]) + v[2] * v[2]).sqrt();
    Ok(v.map(|v| v / norm))
}
fn height(t: &Terrain, radius: f64, direction: [f64; 3]) -> Result<f64, Error> {
    let [x, y, z] = normalized(direction)?;
    // Chord distance from north pole; robust at metre scale (no acos(1-eps)).
    let chord = radius * ((x * x + z * z) + (y - 1.0) * (y - 1.0)).sqrt();
    let taper = ((chord - t.landing_cap_radius_meters) / t.transition_width_meters).clamp(0.0, 1.0);
    let smooth = (taper * taper) * (3.0 - 2.0 * taper);
    // Continuous folded triangular ridges. Seed rotates axes and fixes wavelength;
    // no platform libm, runtime RNG or competing collision surface.
    let phase = (t.seed >> 32) as f64 / 4294967296.0;
    let u = (x + 0.5 * z) * f64::from(t.ridge_frequency) + phase;
    let wave = 1.0 - (2.0 * u.rem_euclid(1.0) - 1.0).abs();
    Ok((t.ridge_height_meters * wave) * smooth)
}
fn surface_point(m: &Manifest, xz: [f64; 2]) -> Result<[f64; 3], Error> {
    if !xz.iter().all(|v| v.is_finite()) || xz[0].hypot(xz[1]) > 1000.0 {
        return Err(invalid(
            "starting tangent point outside bounded north-pole chart",
        ));
    }
    let r = m.physical.radius_meters;
    let direction = normalized([xz[0], r, xz[1]])?;
    let radius = r + height(&m.terrain, r, direction)?;
    Ok(direction.map(|v| v * radius))
}
fn validate_arrival(m: &Manifest) -> Result<(), Error> {
    let a = &m.arrival;
    let length = |v: [f64; 2]| v[0].hypot(v[1]);
    let separation = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
    if length(a.ship_tangent_xz_meters) + a.ship_clearance_radius_meters
        > m.terrain.landing_cap_radius_meters
        || separation(a.ship_tangent_xz_meters, a.traveler_tangent_xz_meters)
            <= a.ship_clearance_radius_meters
    {
        return Err(invalid("starting arrival ship/traveler clearance"));
    }
    for xz in std::iter::once(a.traveler_tangent_xz_meters)
        .chain(m.resources.outcrops.iter().map(|s| s.tangent_xz_meters))
    {
        if length(xz) > m.terrain.landing_cap_radius_meters
            || height(&m.terrain, m.physical.radius_meters, surface_point(m, xz)?)? != 0.0
        {
            return Err(invalid("starting arrival leaves smooth landing cap"));
        }
    }
    for site in &m.resources.outcrops {
        if site.initial_mass_kg <= 0.0
            || separation(site.tangent_xz_meters, a.ship_tangent_xz_meters)
                <= a.ship_clearance_radius_meters
            || separation(site.tangent_xz_meters, a.traveler_tangent_xz_meters)
                > a.resource_walk_limit_meters
        {
            return Err(invalid("starting outcrop unavailable from arrival"));
        }
        let mut from = a.traveler_tangent_xz_meters;
        let mut walked = 0.0;
        for to in site
            .approach_waypoints_xz_meters
            .iter()
            .copied()
            .chain(std::iter::once(site.tangent_xz_meters))
        {
            let delta = [to[0] - from[0], to[1] - from[1]];
            let square = delta[0] * delta[0] + delta[1] * delta[1];
            if square == 0.0 {
                return Err(invalid("starting walk path contains duplicate points"));
            }
            let t = (((a.ship_tangent_xz_meters[0] - from[0]) * delta[0]
                + (a.ship_tangent_xz_meters[1] - from[1]) * delta[1])
                / square)
                .clamp(0.0, 1.0);
            let nearest = [from[0] + t * delta[0], from[1] + t * delta[1]];
            if separation(nearest, a.ship_tangent_xz_meters) <= a.ship_clearance_radius_meters
                || length(to) > m.terrain.landing_cap_radius_meters
            {
                return Err(invalid("starting walk path intersects ship or leaves cap"));
            }
            walked += square.sqrt();
            from = to;
        }
        if walked > a.resource_walk_limit_meters {
            return Err(invalid("starting walk path exceeds resource walk limit"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
