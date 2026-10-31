//! Owner-approved fictional kinematics, separate from immutable surface geography.
//! The local game clock advances uniform SI seconds from an explicit reference;
//! it is not a general UTC/TT or observed-catalog ephemeris conversion service.
use crate::{
    CatalogId, Epoch, Error, TimeScale, coordinates::GalacticPosition, gravity::Source, invalid,
};
use serde::{Deserialize, Serialize};
use std::{sync::LazyLock, time::Duration};

pub const VERSION: u32 = 1;
pub const REFERENCE_UNIX_SECONDS: u64 = 1_767_225_600; // 2026-01-01 UTC
const G: f64 = 6.67430e-11;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub world_sha256: String,
    pub reference_unix_seconds: u64,
    pub reference_epoch: Epoch,
    pub semi_major_axis_meters: f64,
    pub period_seconds: f64,
    pub eccentricity: f64,
    pub phase_radians: f64,
    pub axes: [[f64; 3]; 3],
    pub axial_spin_rps: [f64; 3],
    pub star_mass_kg: f64,
    pub planet_mass_kg: f64,
    pub star_radius_meters: f64,
    pub planet_radius_meters: f64,
    pub surface_pressure_pascals: f64,
}
pub static DEFINITION: LazyLock<Definition> = LazyLock::new(|| {
    let start = crate::starting_system::permanent().expect("checked permanent world publication");
    let p = &start.manifest().physical;
    Definition {
        version: VERSION,
        world_sha256: start.definition().sha256.clone(),
        reference_unix_seconds: REFERENCE_UNIX_SECONDS,
        // At the reference instant TAI-UTC=37 s; TT-TAI=32.184 s.
        reference_epoch: Epoch::calendar([2026, 1, 1], [0, 1, 9, 184_000_000], TimeScale::Tt)
            .expect("fixed reference epoch"),
        semi_major_axis_meters: p.semi_major_axis_meters,
        period_seconds: p.period_seconds,
        eccentricity: 0.,
        phase_radians: 0.,
        axes: [[1., 0., 0.], [0., 0., 1.], [0., -1., 0.]],
        axial_spin_rps: [0.; 3],
        star_mass_kg: p.star_mass_kg,
        planet_mass_kg: p.mass_kg,
        star_radius_meters: p.star_radius_meters,
        planet_radius_meters: p.radius_meters,
        surface_pressure_pascals: p.surface_pressure_pascals,
    }
});
pub static SHA256: LazyLock<String> = LazyLock::new(|| {
    crate::synthesis::sha256(
        &serde_json::to_vec(&*DEFINITION).expect("finite fixed motion publication"),
    )
});

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clock {
    pub version: u32,
    pub definition_sha256: String,
    pub epoch: Epoch,
}
impl Clock {
    pub fn at(unix_seconds: u64) -> Result<Self, Error> {
        let mut epoch = DEFINITION.reference_epoch;
        let mut difference = i128::from(unix_seconds) - i128::from(REFERENCE_UNIX_SECONDS);
        // Checked bounded calendar arithmetic; a malicious u64 clock fails after
        // reaching the supported calendar, never wraps or runs trillions of polls.
        while difference != 0 {
            let chunk = difference.clamp(-86_400_000, 86_400_000) as i64;
            epoch = epoch.advance_nanos(chunk * 1_000_000_000)?;
            difference -= i128::from(chunk);
        }
        Ok(Self {
            version: VERSION,
            definition_sha256: SHA256.clone(),
            epoch,
        })
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != VERSION
            || self.definition_sha256 != *SHA256
            || self.epoch.scale() != TimeScale::Tt
        {
            return Err(invalid(
                "canonical motion version, definition or epoch mismatch",
            ));
        }
        Ok(())
    }
    pub fn advanced(&self, delta: Duration) -> Result<Self, Error> {
        self.validate()?;
        let mut next = self.clone();
        let mut seconds = delta.as_secs();
        while seconds > 0 {
            let step = seconds.min(86_400_000);
            next.epoch = next.epoch.advance_nanos((step * 1_000_000_000) as i64)?;
            seconds -= step;
        }
        next.epoch = next.epoch.advance_nanos(i64::from(delta.subsec_nanos()))?;
        Ok(next)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BodyState {
    pub position_meters: [f64; 3],
    pub velocity_mps: [f64; 3],
    pub acceleration_mps2: [f64; 3],
}
/// Reconstruct the saved translating-frame source without flattening the
/// permanent galactic system anchor into one absolute f64 metre coordinate.
pub fn inertial(
    epoch: Epoch,
    body_relative_meters: [f64; 3],
    body_relative_velocity_mps: [f64; 3],
) -> Result<(GalacticPosition, [f64; 3]), Error> {
    if body_relative_meters
        .into_iter()
        .chain(body_relative_velocity_mps)
        .any(|v| !v.is_finite())
    {
        return Err(invalid("nonfinite body-frame motion source"));
    }
    let body = planet(epoch)?;
    let start = crate::starting_system::permanent()?;
    let local = salimon_math::add(body.position_meters, body_relative_meters);
    Ok((
        start
            .manifest()
            .distance
            .system_position
            .translated(local)?,
        salimon_math::add(body.velocity_mps, body_relative_velocity_mps),
    ))
}
/// Analytic authored circular ephemeris, valid throughout the supported uniform
/// calendar. No imported source is extrapolated and no unknown velocity is zeroed.
pub fn planet(epoch: Epoch) -> Result<BodyState, Error> {
    let p = &*DEFINITION;
    let seconds = epoch.seconds_since(p.reference_epoch)?;
    let angle = (seconds.rem_euclid(p.period_seconds) / p.period_seconds) * std::f64::consts::TAU;
    let angular = std::f64::consts::TAU / p.period_seconds;
    let (s, c) = angle.sin_cos();
    let position_meters = [
        p.semi_major_axis_meters * c,
        0.,
        p.semi_major_axis_meters * s,
    ];
    Ok(BodyState {
        position_meters,
        velocity_mps: [
            -p.semi_major_axis_meters * angular * s,
            0.,
            p.semi_major_axis_meters * angular * c,
        ],
        acceleration_mps2: position_meters.map(|v| -angular * angular * v),
    })
}
/// Complete two-body published field in nonrotating, translating Vesper axes.
/// Callers subtract the frame acceleration when integrating relative velocity.
pub fn sources(epoch: Epoch) -> Result<Vec<Source>, Error> {
    let body = planet(epoch)?;
    Ok(vec![
        Source {
            id: CatalogId::try_from(crate::starting_system::PLANET_ID.to_owned())?,
            position: GalacticPosition::new([0; 3], [0.; 3])?,
            mu_m3_per_s2: G * DEFINITION.planet_mass_kg,
            radius_meters: DEFINITION.planet_radius_meters,
        },
        Source {
            id: CatalogId::try_from(crate::starting_system::STAR_ID.to_owned())?,
            position: GalacticPosition::new([0; 3], body.position_meters.map(|v| -v))?,
            mu_m3_per_s2: G * DEFINITION.star_mass_kg,
            radius_meters: DEFINITION.star_radius_meters,
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_publication_frames_epoch_and_clock_partitions_are_explicit() {
        let start = Clock::at(REFERENCE_UNIX_SECONDS).unwrap();
        assert_eq!(start.epoch, DEFINITION.reference_epoch);
        let a = planet(start.epoch).unwrap();
        assert_eq!(
            a.position_meters,
            [DEFINITION.semi_major_axis_meters, 0., 0.]
        );
        assert_eq!(a.velocity_mps[0], 0.);
        assert!(a.velocity_mps[2] > 0.);
        let one = start.advanced(Duration::from_secs(86400)).unwrap();
        let mut split = start;
        for _ in 0..24 {
            split = split.advanced(Duration::from_secs(3600)).unwrap();
        }
        assert_eq!(one, split);
        assert!(Clock::at(u64::MAX).is_err());
        let mut bad = one;
        bad.definition_sha256 = "unpublished".into();
        assert!(bad.validate().is_err());
        assert_eq!(DEFINITION.axial_spin_rps, [0.; 3]);
    }
}
