//! Fixed-seed scientific sanity evidence, not population-frequency calibration.
use salimon_universe::{
    Body, BodyKind, Catalog, CatalogId, Dataset, Header, Provenance, Record, Star, System,
    inference::{
        AU_METERS, EARTH_RADIUS_METERS, MODEL_VERSION, PlanetInput, PlanetModel, SOLAR_MASS_KG,
        StellarModel, SystemInput, YEAR_SECONDS, complete,
    },
};
use serde_json::json;
const N: u64 = 4096;
fn bin(hist: &mut [u64; 5], x: f64, low: f64, high: f64, log: bool) {
    let fraction = if log {
        (x / low).ln() / (high / low).ln()
    } else {
        (x - low) / (high - low)
    };
    hist[((fraction * 5.0).floor() as usize).min(4)] += 1;
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().len() != 1 {
        return Err("inference_sanity takes no arguments; fixed seeds 0..4096".into());
    }
    let id = |s: &str| CatalogId::from_source("sanity", s);
    let provenance = Provenance::Observed {
        dataset: id("source")?,
        record_key: "synthetic-not-observation".into(),
    };
    let header = |s: &str| -> Result<Header, salimon_universe::Error> {
        Ok(Header {
            id: id(s)?,
            display_name: None,
            provenance: provenance.clone(),
        })
    };
    let catalog = Catalog::new(
        "synthetic-input".into(),
        vec![Dataset {
            id: id("source")?,
            version: "v1".into(),
            reference: "synthetic inference workload; no empirical population claim".into(),
        }],
        vec![
            Record::System(System {
                header: header("system")?,
            }),
            Record::Star(Star {
                header: header("star")?,
                system: id("system")?,
                mass_kg: None,
                radius_meters: None,
            }),
            Record::Body(Body {
                header: header("planet")?,
                system: id("system")?,
                parent: id("star")?,
                body_kind: BodyKind::Planet,
                mass_kg: None,
                radius_meters: None,
                definition: None,
            }),
        ],
    )?;
    let mut rows = vec![];
    for model in [PlanetModel::RockyV1, PlanetModel::GasGiantV1] {
        let mut request = SystemInput {
            host: id("star")?,
            model: StellarModel::MainSequenceV1,
            seed: 0,
            age_seconds: None,
            planets: vec![PlanetInput {
                planet: id("planet")?,
                model,
                semi_major_axis_meters: None,
                period_seconds: None,
                eccentricity: None,
                minimum_mass_kg: None,
            }],
        };
        let (mut mass_bins, mut orbit_bins, mut age_bins, mut eccentricity_bins, mut density_bins) =
            ([0; 5], [0; 5], [0; 5], [0; 5], [0; 5]);
        let (mut failures, mut attempts, mut worst_kepler, mut minimum_clearance) =
            (0u64, 0u64, 0.0_f64, f64::MAX);
        let mut first = None;
        for seed in 0..N {
            request.seed = seed;
            let result = complete(&catalog, &request, "sanity-output".into())?;
            attempts += result.accepted_attempt + 1;
            let Some(Record::Star(star)) = result.catalog.get(&request.host) else {
                return Err("missing star".into());
            };
            let Some(Record::Body(planet)) = result.catalog.get(&request.planets[0].planet) else {
                return Err("missing planet".into());
            };
            let sm = star.mass_kg.as_ref().ok_or("missing mass")?.value;
            let sr = star.radius_meters.as_ref().ok_or("missing radius")?.value;
            let pm = planet.mass_kg.as_ref().ok_or("missing mass")?.value;
            let pr = planet.radius_meters.as_ref().ok_or("missing radius")?.value;
            let density = pm / ((4.0 / 3.0) * std::f64::consts::PI * pr.powi(3));
            let o = &result.orbits[0];
            let a = o.semi_major_axis_meters.value;
            let e = o.eccentricity.value;
            let t = o.period_seconds.value;
            let residual = (t * t / (a * a * a) * (6.67430e-11 * (sm + pm))
                / (std::f64::consts::TAU.powi(2))
                - 1.0)
                .abs();
            worst_kepler = worst_kepler.max(residual);
            let clearance = a * (1.0 - e) / (sr + pr).max(2.44 * pr * (sm / pm).cbrt());
            minimum_clearance = minimum_clearance.min(clearance);
            let max_age =
                (10e9 * YEAR_SECONDS * (sm / SOLAR_MASS_KG).powf(-2.5)).min(13.8e9 * YEAR_SECONDS);
            let density_ok = match model {
                PlanetModel::RockyV1 => {
                    (3000.0..=8000.0).contains(&density)
                        && (0.3..=1.6).contains(&(pr / EARTH_RADIUS_METERS))
                }
                PlanetModel::GasGiantV1 => (50.0..=30000.0).contains(&density),
            };
            if !density_ok
                || clearance <= 1.0
                || residual > 1e-12
                || result.age_seconds.value > max_age
                || !(0.0..1.0).contains(&e)
            {
                failures += 1;
            }
            bin(&mut mass_bins, sm / SOLAR_MASS_KG, 0.08, 2.0, true);
            bin(&mut orbit_bins, a / AU_METERS, 0.02, 100.0, true);
            bin(
                &mut age_bins,
                (result.age_seconds.value - 1e6 * YEAR_SECONDS) / (max_age - 1e6 * YEAR_SECONDS),
                0.0,
                1.0,
                false,
            );
            bin(&mut eccentricity_bins, e, 0.0, 0.3, false);
            if model == PlanetModel::RockyV1 {
                bin(&mut density_bins, density, 3000.0, 8000.0, true);
            }
            if first.is_none() {
                first = Some(result);
            }
        }
        // Broad anti-collapse gate: the unconstrained five-bin priors must retain
        // coverage after rejection. This is not an empirical goodness-of-fit test.
        let coverage = [mass_bins, orbit_bins, age_bins, eccentricity_bins]
            .iter()
            .flatten()
            .all(|n| *n > N / 10 && *n < N * 4 / 10);
        rows.push(json!({"model":model,"systems":N,"candidate_attempts":attempts,"constraint_failures":failures,"histogram_coverage_pass":coverage,
            "worst_relative_kepler_residual":worst_kepler,"minimum_pericenter_clearance_ratio":minimum_clearance,
            "histograms_five_equal_bins":{"stellar_mass_log_solar_0_08_to_2":mass_bins,"axis_log_au_0_02_to_100":orbit_bins,"fraction_of_allowed_age_linear":age_bins,"eccentricity_linear_0_to_0_3":eccentricity_bins,"rocky_density_log_kg_m3_3000_to_8000":if model==PlanetModel::RockyV1{Some(density_bins)}else{None}},"first_completion":first}));
        if failures != 0 || !coverage {
            return Err("scientific sanity gate failed".into());
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"format":"salimon.universe.inference-sanity","model_version":MODEL_VERSION,"production_eligible":false,"seed_start":0,"seed_end_exclusive":N,"populations":rows})
        )?
    );
    Ok(())
}
