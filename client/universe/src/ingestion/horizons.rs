//! Horizons API 1.2, geometric ICRF/SSB KM-S vector-table-2 ingestion.
//! Unsupported contexts fail rather than silently rotating/recentering states.
use crate::{
    Body, BodyKind, CatalogId, Dataset, Epoch, Error, Header, Provenance, Quantity, TimeScale,
    Uncertainty, Validity, invalid,
};
use serde::{Deserialize, Serialize};

const MAX_BYTES: usize = 16 * 1024 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    signature: Signature,
    result: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Signature {
    source: String,
    version: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ReferenceFrame {
    IcrfSolarSystemBarycenterGeometric,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct State {
    pub frame: ReferenceFrame,
    pub epoch: Epoch,
    pub julian_day_tdb: f64,
    /// ICRF equatorial axes, solar system barycenter origin, geometric states.
    pub position_meters: [f64; 3],
    pub velocity_mps: [f64; 3],
}
#[derive(Debug, Serialize)]
pub struct Ephemeris {
    target: CatalogId,
    provenance: Provenance,
    raw_response: String,
    samples: Vec<State>,
    /// Physical header measurements retain their own unknown temporal validity.
    mass_kg: Option<Quantity>,
    mean_radius_meters: Option<Quantity>,
}
impl Ephemeris {
    /// Parent/system classification is supplied by the reviewed builder, never
    /// guessed from a JPL number. Publish with Catalog::new for reference checks.
    pub fn body(&self, system: CatalogId, parent: CatalogId, body_kind: BodyKind) -> Body {
        Body {
            header: Header {
                id: self.target.clone(),
                display_name: None,
                provenance: self.provenance.clone(),
            },
            system,
            parent,
            body_kind,
            mass_kg: self.mass_kg.clone(),
            radius_meters: self.mean_radius_meters.clone(),
            definition: None,
        }
    }
    pub fn target(&self) -> &CatalogId {
        &self.target
    }
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }
    pub fn samples(&self) -> &[State] {
        &self.samples
    }
    pub fn raw_response(&self) -> &str {
        &self.raw_response
    }
    pub fn mass_kg(&self) -> Option<&Quantity> {
        self.mass_kg.as_ref()
    }
    pub fn mean_radius_meters(&self) -> Option<&Quantity> {
        self.mean_radius_meters.as_ref()
    }
    /// The imported sample interval, not DE441's full mathematical validity.
    pub fn validity(&self) -> Validity {
        Validity {
            start_julian_day: self.samples[0].julian_day_tdb,
            end_julian_day: self.samples[self.samples.len() - 1].julian_day_tdb,
            scale: TimeScale::Tdb,
        }
    }
    /// Exact imported states only. No interpolation, extrapolation, UTC/TDB
    /// conversion or frozen-edge fallback. U14 supplies evaluated orbit models.
    pub fn state_at(&self, epoch: Epoch) -> Result<&State, Error> {
        let first = self.samples[0].epoch;
        let last = self.samples[self.samples.len() - 1].epoch;
        if epoch.compare(first)?.is_lt() || epoch.compare(last)?.is_gt() {
            return Err(invalid(
                "Horizons epoch outside imported validity; explicit lower-accuracy model required",
            ));
        }
        self.samples
            .iter()
            .find(|s| s.epoch == epoch)
            .ok_or_else(|| {
                invalid("Horizons epoch has no imported sample; evaluated model required")
            })
    }
    pub fn decode(bytes: &[u8], dataset: &Dataset) -> Result<Self, Error> {
        if bytes.len() > MAX_BYTES {
            return Err(invalid("Horizons response exceeds 16 MiB"));
        }
        let response: Response =
            serde_json::from_slice(bytes).map_err(|e| Error::Encoding(e.to_string()))?;
        if response.signature.source != "NASA/JPL Horizons API"
            || response.signature.version != "1.2"
        {
            return Err(invalid("unsupported Horizons API signature"));
        }
        let text = &response.result;
        let (header, rest) = text
            .split_once("$$SOE\n")
            .ok_or_else(|| invalid("Horizons start marker"))?;
        let (block, trailer) = rest
            .split_once("$$EOE")
            .ok_or_else(|| invalid("Horizons end marker"))?;
        if trailer.contains("$$SOE") || trailer.contains("$$EOE") || header.contains("$$SOE") {
            return Err(invalid("multiple Horizons vector blocks"));
        }
        for required in [
            "Output units    : KM-S",
            "Reference frame : ICRF",
            "Output type     : GEOMETRIC cartesian states",
            "Output format   : 2 (position and velocity)",
            "JDTDB",
            "Center-site name: BODY CENTER",
        ] {
            let prefix = required.split_once(':').map_or(required, |(key, _)| key);
            let context: Vec<_> = header
                .lines()
                .filter(|l| l.trim().starts_with(prefix))
                .collect();
            if context.len() != 1 || context[0].trim() != required {
                return Err(invalid(format!("unsupported Horizons context: {required}")));
            }
        }
        let target_key = body_key(header, "Target body name:")?;
        if body_key(header, "Center body name:")? != "0" {
            return Err(invalid(
                "Horizons requires solar-system barycenter center 0",
            ));
        }
        let target = CatalogId::from_source("jpl", &target_key)?;
        let provenance = Provenance::Observed {
            dataset: dataset.id.clone(),
            record_key: target_key,
        };
        let lines: Vec<_> = block.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.is_empty() || lines.len() % 3 != 0 {
            return Err(invalid("Horizons vector-table-2 row count"));
        }
        let mut samples: Vec<State> = Vec::new();
        for rows in lines.as_chunks::<3>().0 {
            let (jd, calendar) = rows[0]
                .split_once(" = A.D. ")
                .ok_or_else(|| invalid("unsupported Horizons epoch label"))?;
            let julian_day_tdb = finite(jd.trim())?;
            let epoch = parse_calendar(calendar.trim())?;
            // JD is redundant with the calendar: 1e-9 day (~86 us) tolerates
            // printed JD/f64 rounding, not a scientific state-error allowance.
            let reference = Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tdb)?;
            let expected_jd = 2451545.0 + epoch.seconds_since(reference)? / 86400.0;
            if (julian_day_tdb - expected_jd).abs() > 1e-9 {
                return Err(invalid("Horizons JD/calendar mismatch"));
            }
            if samples
                .last()
                .is_some_and(|s| epoch.compare(s.epoch).is_ok_and(|o| !o.is_gt()))
            {
                return Err(invalid("Horizons epochs not strictly increasing"));
            }
            samples.push(State {
                frame: ReferenceFrame::IcrfSolarSystemBarycenterGeometric,
                epoch,
                julian_day_tdb,
                position_meters: vector(rows[1], ["X", "Y", "Z"])?,
                velocity_mps: vector(rows[2], ["VX", "VY", "VZ"])?,
            });
        }
        let mean_radius_meters = physical(header, "Vol. Mean Radius (km)", 1000.0, &provenance)?;
        let mass_kg = if let Some((_, tail)) = header.split_once("Mass x10^") {
            let (exponent, tail) = tail
                .split_once(" (kg)")
                .ok_or_else(|| invalid("unsupported Horizons mass units"))?;
            let exponent: i32 = exponent
                .parse()
                .map_err(|_| invalid("Horizons mass exponent"))?;
            Some(parse_physical(tail, 10_f64.powi(exponent), &provenance)?)
        } else {
            None
        };
        Ok(Self {
            target,
            provenance,
            raw_response: response.result,
            samples,
            mass_kg,
            mean_radius_meters,
        })
    }
}
fn body_key(header: &str, prefix: &str) -> Result<String, Error> {
    let mut lines = header.lines().filter(|l| l.starts_with(prefix));
    let line = lines
        .next()
        .ok_or_else(|| invalid(format!("missing Horizons {prefix}")))?;
    if lines.next().is_some() {
        return Err(invalid("duplicate Horizons body header"));
    }
    let (_, tail) = line
        .split_once('(')
        .ok_or_else(|| invalid("Horizons body code"))?;
    let (key, _) = tail
        .split_once(')')
        .ok_or_else(|| invalid("Horizons body code"))?;
    if key.is_empty() || !key.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid("Horizons numeric body code"));
    }
    Ok(key.into())
}
fn finite(text: &str) -> Result<f64, Error> {
    let v: f64 = text
        .parse()
        .map_err(|_| invalid("Horizons invalid number"))?;
    if !v.is_finite() {
        return Err(invalid("Horizons nonfinite number"));
    }
    Ok(v)
}
fn vector(line: &str, labels: [&str; 3]) -> Result<[f64; 3], Error> {
    let mut tail = line.trim();
    let mut out = [0.0; 3];
    for (index, label) in labels.into_iter().enumerate() {
        tail = tail
            .strip_prefix(label)
            .ok_or_else(|| invalid("Horizons vector labels/order"))?
            .trim_start();
        tail = tail
            .strip_prefix('=')
            .ok_or_else(|| invalid("Horizons vector equals"))?
            .trim_start();
        let end = tail.find(char::is_whitespace).unwrap_or(tail.len());
        out[index] = finite(&tail[..end])? * 1000.0;
        if !out[index].is_finite() {
            return Err(invalid("Horizons SI overflow"));
        }
        tail = tail[end..].trim_start();
    }
    if !tail.is_empty() {
        return Err(invalid("Horizons trailing vector text"));
    }
    Ok(out)
}
fn parse_calendar(text: &str) -> Result<Epoch, Error> {
    let parts: Vec<_> = text.split_whitespace().collect();
    if parts.len() != 3 || parts[2] != "TDB" {
        return Err(invalid("Horizons requires TDB AD calendar"));
    }
    let date: Vec<_> = parts[0].split('-').collect();
    let time: Vec<_> = parts[1].split(':').collect();
    if date.len() != 3 || time.len() != 3 {
        return Err(invalid("Horizons date/time"));
    }
    let year: i32 = date[0].parse().map_err(|_| invalid("Horizons year"))?;
    // Mixed Julian/Gregorian before this date requires a separate adapter.
    if year < 1583 {
        return Err(invalid(
            "Horizons mixed-calendar dates before 1583 unsupported",
        ));
    }
    let month = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .iter()
    .position(|m| *m == date[1])
    .ok_or_else(|| invalid("Horizons month"))? as i32
        + 1;
    let day = date[2].parse().map_err(|_| invalid("Horizons day"))?;
    let hour = time[0].parse().map_err(|_| invalid("Horizons hour"))?;
    let minute = time[1].parse().map_err(|_| invalid("Horizons minute"))?;
    let (seconds, fraction) = time[2].split_once('.').unwrap_or((time[2], ""));
    if fraction.len() > 9 || !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid("Horizons fractional second"));
    }
    let second = seconds.parse().map_err(|_| invalid("Horizons second"))?;
    let nanos = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<u32>()
            .map_err(|_| invalid("Horizons nanos"))?
            * 10_u32.pow(9 - fraction.len() as u32)
    };
    Epoch::calendar(
        [year, month, day],
        [hour, minute, second, nanos],
        TimeScale::Tdb,
    )
}
fn physical(
    header: &str,
    label: &str,
    factor: f64,
    provenance: &Provenance,
) -> Result<Option<Quantity>, Error> {
    header
        .split_once(label)
        .map(|(_, tail)| parse_physical(tail, factor, provenance))
        .transpose()
}
fn parse_physical(tail: &str, factor: f64, provenance: &Provenance) -> Result<Quantity, Error> {
    let token = tail
        .trim_start()
        .strip_prefix('=')
        .ok_or_else(|| invalid("Horizons physical equals"))?
        .split_whitespace()
        .next()
        .ok_or_else(|| invalid("Horizons physical number"))?;
    let (value, error) = token
        .split_once("+-")
        .map_or((token, None), |(v, e)| (v, Some(e)));
    let value = finite(value)? * factor;
    if !factor.is_finite() || factor <= 0.0 || !value.is_finite() || value <= 0.0 {
        return Err(invalid("Horizons physical SI range"));
    }
    let uncertainty = if let Some(error) = error {
        let error = finite(error)? * factor;
        if !error.is_finite() || error < 0.0 || error >= value {
            return Err(invalid("Horizons physical error"));
        }
        Uncertainty::Bounds {
            minus: error,
            plus: error,
            confidence: None,
        }
    } else {
        Uncertainty::Unknown
    };
    Ok(Quantity {
        value,
        uncertainty,
        provenance: provenance.clone(),
        validity: None,
    })
}
#[cfg(test)]
mod tests;
