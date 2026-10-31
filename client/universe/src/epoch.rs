//! Split calendar epoch. Time scales are never silently mixed or converted.
use crate::{Error, TimeScale, invalid};
use serde::{Deserialize, Serialize};

const DAY_NS: u64 = 86_400_000_000_000;
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "EpochWire", into = "EpochWire")]
pub struct Epoch {
    day: i64,
    nanos: u64,
    scale: TimeScale,
}
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EpochWire {
    /// Gregorian days since 1970-01-01, midnight in the declared scale.
    day: i64,
    nanos: u64,
    scale: TimeScale,
}
impl TryFrom<EpochWire> for Epoch {
    type Error = Error;
    fn try_from(w: EpochWire) -> Result<Self, Error> {
        if w.nanos >= DAY_NS || w.day.abs_diff(0) > 3_000_000 {
            return Err(invalid("epoch outside supported calendar/day bounds"));
        }
        Ok(Self {
            day: w.day,
            nanos: w.nanos,
            scale: w.scale,
        })
    }
}
impl From<Epoch> for EpochWire {
    fn from(e: Epoch) -> Self {
        Self {
            day: e.day,
            nanos: e.nanos,
            scale: e.scale,
        }
    }
}
impl Epoch {
    /// Proleptic Gregorian years 1..=9999. Leap seconds require an explicit
    /// future leap-table adapter; second 60 is rejected, never normalized away.
    pub fn calendar(date: [i32; 3], time: [u32; 4], scale: TimeScale) -> Result<Self, Error> {
        let [year, month, day] = date;
        let [hour, minute, second, nanos] = time;
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days = [
            31,
            if leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        if !(1..=9999).contains(&year)
            || !(1..=12).contains(&month)
            || day < 1
            || day > days[(month - 1) as usize]
            || hour > 23
            || minute > 59
            || second > 59
            || nanos >= 1_000_000_000
        {
            return Err(invalid("invalid Gregorian epoch"));
        }
        let y = i64::from(year) - i64::from(month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(month) + if month > 2 { -3 } else { 9 };
        let doy = (153 * m + 2) / 5 + i64::from(day) - 1;
        let d = era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468;
        EpochWire {
            day: d,
            nanos: u64::from(hour * 3600 + minute * 60 + second) * 1_000_000_000 + u64::from(nanos),
            scale,
        }
        .try_into()
    }
    /// Exact ordering without pretending UTC days all have 86400 SI seconds.
    pub fn compare(self, other: Self) -> Result<std::cmp::Ordering, Error> {
        if self.scale != other.scale {
            return Err(invalid("epoch time-scale mismatch"));
        }
        Ok((self.day, self.nanos).cmp(&(other.day, other.nanos)))
    }
    pub fn scale(self) -> TimeScale {
        self.scale
    }
    /// Exact uniform-clock phase for deterministic work grids; no float date conversion.
    pub fn nanosecond_of_day(self) -> u64 {
        self.nanos
    }
    /// Uniform scale seconds. UTC subtraction is restricted to a single day:
    /// crossing UTC midnight needs leap-table conversion to TT/TDB first.
    pub fn seconds_since(self, earlier: Self) -> Result<f64, Error> {
        if self.scale != earlier.scale {
            return Err(invalid("epoch time-scale mismatch"));
        }
        if self.scale == TimeScale::Utc && self.day != earlier.day {
            return Err(invalid(
                "cross-day UTC arithmetic requires leap-table adapter",
            ));
        }
        let delta = i128::from(self.day - earlier.day) * i128::from(DAY_NS)
            + i128::from(self.nanos)
            - i128::from(earlier.nanos);
        Ok(delta as f64 * 1e-9)
    }
    /// Advance a uniform TT/TDB calendar clock, including negative durations.
    pub fn advance_nanos(self, elapsed: i64) -> Result<Self, Error> {
        let ns = i128::from(self.nanos) + i128::from(elapsed);
        let day_delta = ns.div_euclid(i128::from(DAY_NS));
        if self.scale == TimeScale::Utc && day_delta != 0 {
            return Err(invalid("UTC advance requires leap-table adapter"));
        }
        EpochWire {
            day: self.day + day_delta as i64,
            nanos: ns.rem_euclid(i128::from(DAY_NS)) as u64,
            scale: self.scale,
        }
        .try_into()
    }
}
