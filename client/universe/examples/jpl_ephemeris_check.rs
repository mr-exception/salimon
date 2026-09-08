//! Independent withheld Horizons epochs, not samples fitted by the interpolant.
use salimon_universe::{
    CatalogId, Dataset,
    ephemerides::{Model, Sample},
    ingestion::horizons::Ephemeris,
    reference::{self, Budget, Case, Regime, Suite},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dataset = Dataset {
        id: CatalogId::from_source("jpl", "earth-3h-20000101")?,
        version: "Horizons API 1.2 / pinned 2026-10-27".into(),
        reference: "reports/issue-415/cloud/jpl-source.json".into(),
    };
    let data = Ephemeris::decode(
        include_bytes!("../fixtures/jpl-earth-3h-20000101.json"),
        &dataset,
    )?;
    let samples: Vec<_> = data
        .samples()
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, s)| Sample {
            epoch: s.epoch,
            position_meters: s.position_meters,
            velocity_mps: s.velocity_mps,
        })
        .collect();
    let model = Model::Sampled {
        samples,
        max_extrapolation_seconds: 0.0,
        max_interpolation_gap_seconds: 21600.0,
    };
    let mut cases = Vec::new();
    let mut actual = Vec::new();
    for (i, s) in data
        .samples()
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
    {
        let expected = reference::Sample {
            id: format!("earth-withheld-{i}"),
            frame: "ICRF-SSB-geometric".into(),
            epoch: s.epoch,
            position_meters: s.position_meters,
            velocity_mps: s.velocity_mps,
        };
        let (state, _) = model.evaluate(s.epoch)?;
        actual.push(reference::Sample {
            position_meters: state.meters,
            velocity_mps: state.meters_per_second,
            ..expected.clone()
        });
        cases.push(Case{expected,regime:Regime::SystemFlight,budget:Budget{position_meters:1.0,velocity_mps:0.001},budget_basis:"6-hour cubic interpolation model test: 1 m / 1 mm/s; independent of unchanged U03 exact-decoder rounding budgets; not global DE441/model uncertainty".into(),reference:dataset.reference.clone()});
    }
    let comparison = Suite::new(cases)?.compare(&actual)?;
    println!("{}", serde_json::to_string_pretty(&comparison)?);
    if !comparison.passed {
        return Err("withheld JPL interpolation comparison failed".into());
    }
    Ok(())
}
