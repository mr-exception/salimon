//! Strict U08 adapter: source limits/unspecified flags require caller review.
use super::*;
use crate::ingestion::exoplanets::{Limit, PlanetObservation, ReportedQuantity};

fn measured(q: &Option<ReportedQuantity>) -> Result<Option<Quantity>, Error> {
    q.as_ref().map(|q| {
        if q.limit != Limit::Measurement {
            return Err(invalid("inference adapter requires reviewed measurements; upper/lower/unspecified source limits are not point measurements"));
        }
        Ok(q.quantity.clone())
    }).transpose()
}
fn merge(slot: &mut Option<Quantity>, value: Option<Quantity>) -> Result<(), Error> {
    if let Some(value) = value {
        match slot {
            Some(old) if old != &value => {
                return Err(invalid(
                    "conflicting observed physical quantities require source reconciliation",
                ));
            }
            Some(_) => {}
            None => *slot = Some(value),
        }
    }
    Ok(())
}
/// Imports relevant U08 measurements before inference, preserving source errors
/// and true/minimum-mass semantics. Exactly one observation per requested planet;
/// additional epochs/orientations/raw fields remain in the original U08 sidecar.
/// Conflicting host measurements reject instead of selecting an arbitrary row.
pub fn complete_observed(
    catalog: &Catalog,
    input: &SystemInput,
    observations: &[PlanetObservation],
    revision: String,
) -> Result<Completion, Error> {
    let mut request = input.clone();
    if observations.len() != request.planets.len() {
        return Err(invalid(
            "U08 inference requires exactly one observation per requested planet",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut records = catalog.records().to_vec();
    for observation in observations {
        if observation.host != request.host || !seen.insert(&observation.planet) {
            return Err(invalid(
                "U08 inference host mismatch or duplicate observation",
            ));
        }
        let p = request
            .planets
            .iter_mut()
            .find(|p| p.planet == observation.planet)
            .ok_or_else(|| invalid("unrequested U08 inference planet"))?;
        merge(
            &mut p.semi_major_axis_meters,
            measured(&observation.semi_major_axis_meters)?,
        )?;
        merge(
            &mut p.period_seconds,
            measured(&observation.orbital_period_seconds)?,
        )?;
        merge(&mut p.eccentricity, measured(&observation.eccentricity)?)?;
        merge(
            &mut p.minimum_mass_kg,
            measured(&observation.minimum_mass_kg)?,
        )?;
        let Some(Record::Star(host)) = records.iter_mut().find(|r| r.id() == &request.host) else {
            return Err(invalid("U08 inference host missing"));
        };
        merge(&mut host.mass_kg, measured(&observation.host_mass_kg)?)?;
        merge(
            &mut host.radius_meters,
            measured(&observation.host_radius_meters)?,
        )?;
        let Some(Record::Body(body)) = records.iter_mut().find(|r| r.id() == &p.planet) else {
            return Err(invalid("U08 inference planet missing"));
        };
        merge(&mut body.mass_kg, measured(&observation.planet_mass_kg)?)?;
        merge(
            &mut body.radius_meters,
            measured(&observation.planet_radius_meters)?,
        )?;
    }
    let base = Catalog::new(
        catalog.revision().into(),
        catalog.datasets().to_vec(),
        records,
    )?;
    complete(&base, &request, revision)
}
