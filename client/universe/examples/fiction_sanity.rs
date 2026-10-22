//! Fixed synthetic cohort evidence; never a calibrated production galaxy.
use flate2::{Compression, write::ZlibEncoder};
use salimon_universe::{
    BodyKind, Catalog, Record,
    coordinates::GalacticPosition,
    inference::{SOLAR_MASS_KG, YEAR_SECONDS},
    synthesis::{BUILDER, Region, Request, build, sha256},
};
use serde_json::json;
use std::io::Write;

fn compressed(bytes: &[u8]) -> Result<usize, std::io::Error> {
    let mut encoder = ZlibEncoder::new(vec![], Compression::new(6));
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?.len())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().len() != 1 {
        return Err("fiction_sanity takes no arguments; fixed synthetic inputs".into());
    }
    let base = Catalog::new("empty-development-input".into(), vec![], vec![])?;
    let mut regions = vec![];
    for (slot, (key, age, metal)) in [
        ("young-poor", 0.01, -2.5),
        ("old-poor", 12.0, -2.5),
        ("young-rich", 0.01, 0.5),
        ("old-rich", 12.0, 0.5),
    ]
    .into_iter()
    .enumerate()
    {
        regions.push(Region {
            key: key.into(),
            center: GalacticPosition::new([slot as i64 * 10_000_000, 0, 0], [0.0; 3])?,
            extent_parsecs: [4.0; 3],
            residual_systems_per_cubic_parsec: 1.0,
            age_gyr: [age, age],
            metallicity_feh: [metal, metal],
        });
    }
    let request=Request {version:1,seed:42,revision:"fiction-development-v1".into(),rationale:"synthetic density/age/metallicity cohorts; heuristic science, no empirical calibration or production reuse approval".into(),regions};
    let output = build(&base, request.clone())?;
    let repeat = build(&base, request.clone())?;
    let bytes = output.encode()?;
    if bytes != repeat.encode()? {
        return Err("repeated publication bytes differ".into());
    }
    let mut reordered = request.clone();
    reordered.regions.reverse();
    if bytes != build(&base, reordered)?.encode()? {
        return Err("region ordering changed publication".into());
    }
    let mut cohorts = vec![];
    for region in &request.regions {
        let sources: Vec<_> = output
            .sources()
            .iter()
            .filter(|s| s.region == region.key)
            .collect();
        let mut bins = [0usize; 5];
        let mut positions = [0usize; 5];
        let mut mass_sum = 0.0;
        let (mut planets, mut giants, mut moons, mut minors) = (0, 0, 0, 0);
        for s in &sources {
            let Some(Record::Star(star)) = output.catalog().get(&s.star) else {
                return Err("missing host".into());
            };
            let m = star.mass_kg.as_ref().ok_or("missing mass")?.value / SOLAR_MASS_KG;
            mass_sum += m;
            let bin = if m < 0.2 {
                0
            } else if m < 0.5 {
                1
            } else if m < 1.0 {
                2
            } else if m < 1.5 {
                3
            } else {
                4
            };
            bins[bin] += 1;
            if s.age_seconds / YEAR_SECONDS / 1e9 > 10.0 * m.powf(-2.5) + 1e-12 {
                return Err("lifetime constraint".into());
            }
            let x = s.position.relative_meters(region.center)?[0]
                / (4.0 * salimon_universe::synthesis::PARSEC_METERS)
                + 0.5;
            positions[((x * 5.0) as usize).min(4)] += 1;
            for body in output.bodies().values().filter(|b| {
                output
                    .catalog()
                    .get(&b.object)
                    .is_some_and(|r| r.system() == &s.system)
            }) {
                match body.body_kind {
                    BodyKind::Planet => {
                        planets += 1;
                        giants += usize::from(body.radius_meters > 50e6);
                    }
                    BodyKind::Moon => moons += 1,
                    BodyKind::MinorBody => minors += 1,
                }
            }
        }
        if sources.len() != 64
            || minors != 64
            || bins[0] + bins[1] < 40
            || positions.iter().any(|n| *n < 4 || *n > 25)
        {
            return Err("density/low-mass/placement coverage failed".into());
        }
        cohorts.push(json!({"region":region.key,"systems":sources.len(),"age_gyr":region.age_gyr,"feh":region.metallicity_feh,"mean_stellar_mass_solar":mass_sum/sources.len() as f64,"mass_bins_solar_0_1_0_2_0_5_1_1_5_2":bins,"x_position_equal_width_bins":positions,"planets":planets,"gas_giants":giants,"major_moons":moons,"selected_minor_bodies":minors}));
    }
    let catalog = output.catalog().encode()?;
    let implicit: u64 = output
        .populations()
        .values()
        .map(|p| p.estimated_count)
        .sum();
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"builder":BUILDER,"production_eligible":false,"systems":output.sources().len(),"records":output.catalog().records().len(),"population_descriptors":output.populations().len(),"unmaterialized_members":implicit,"catalog_sha256":sha256(&catalog),"publication_sha256":output.sha256()?,"catalog_json_bytes":catalog.len(),"catalog_zlib6_bytes":compressed(&catalog)?,"publication_json_bytes":bytes.len(),"publication_zlib6_bytes":compressed(&bytes)?,"cohorts":cohorts,"repeat_and_reorder_identical":true})
        )?
    );
    Ok(())
}
