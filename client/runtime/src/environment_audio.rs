//! Observational, bounded environmental sound. Positions and wind are body-fixed metres.
use crate::{condition_feedback::Settings, session::GameplaySession};
use salimon_math::{dot, length, sub};
use salimon_universe::Epoch;
use salimon_world::environment::{
    activity::{AcousticPath, ActivityDefinition, ActivityInputs},
    climate::{ClimateDefinition, Condensable},
    organism_placement::Population,
    organisms::Form,
};
use std::sync::LazyLock;
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Band {
    pub gain: f32,
    pub pan: f32,
    pub filter: f32,
}
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Frame {
    pub bands: [Band; 5],
    pub warning: Option<&'static str>,
}
#[derive(Clone, Copy)]
pub(crate) struct Listener {
    pub eye: [f64; 3],
    pub right: [f64; 3],
    pub contact: bool,
    /// Actual ramp opening and finite Scout partitions, or suit transmission.
    pub transmission: f64,
    pub filter: f64,
}
static ACTIVITY: LazyLock<ActivityDefinition> = LazyLock::new(|| {
    ActivityDefinition::new(
        salimon_world::canonical::BIOSPHERE.climate().clone(),
        1,
        0,
        ActivityInputs::dormant(),
    )
    .expect("published Vesper has no active sources")
});
type PartFraction<'a> = dyn Fn(&str, u16) -> f64 + 'a;
pub(crate) struct Anatomy<'a> {
    pub population: &'a Population,
    pub remaining: Option<&'a PartFraction<'a>>,
}
/// Explicit physical particulate loading; dry weather alone is not a dust source.
pub(crate) fn sample(
    climate: &ClimateDefinition,
    activity: &ActivityDefinition,
    anatomy: Anatomy<'_>,
    epoch: Epoch,
    listener: Listener,
    dust_kg_m3: f64,
    blocked: impl Fn([f64; 3]) -> bool,
) -> Frame {
    let mut out = Frame::default();
    let n = length(listener.eye);
    if !n.is_finite() || n < 1. || !listener.right.iter().all(|v| v.is_finite()) {
        return out;
    }
    let Ok(weather) = climate.sample(listener.eye.map(|v| v / n), epoch) else {
        return out;
    };
    let altitude = (n
        - climate.geology().terrain().radius_meters()
        - climate
            .geology()
            .terrain()
            .height(listener.eye.map(|v| v / n))
            .unwrap_or(0.))
    .max(0.);
    let air = ((weather.pressure_pascal / 101325.)
        * (-altitude / climate.inputs().scale_height_meters).exp())
    .clamp(0., 1.);
    let wind = length(weather.wind_mps);
    let mut set = |index: usize, gain: f64, pan: f64, filter: f64| {
        out.bands[index] = Band {
            gain: (gain * listener.transmission).clamp(0., 0.35) as f32,
            pan: pan.clamp(-1., 1.) as f32,
            filter: (filter * listener.filter).clamp(0.005, 1.) as f32,
        };
    };
    set(
        0,
        air * (wind / 40.).min(1.) * 0.22,
        if wind > 0. {
            -dot(weather.wind_mps, listener.right) / wind
        } else {
            0.
        },
        0.25,
    );
    let precipitation = if weather.precipitation != Condensable::None {
        weather.precipitation_kg_m2_s * 30.
    } else {
        0.
    };
    let dust = if dust_kg_m3.is_finite() {
        dust_kg_m3.max(0.) * wind * 20.
    } else {
        0.
    };
    set(1, air * (precipitation + dust).min(1.) * 0.18, 0., 0.6);
    if let Ok(events) = activity.events_within(listener.eye, 1000., epoch) {
        for e in events {
            let delta = sub(e.site.body_local_position_meters, listener.eye);
            let distance = length(delta);
            let coupled = listener.contact;
            let airborne = air > 0. && e.acoustic_path == AcousticPath::AirAndContact;
            if !coupled && !airborne {
                continue;
            }
            let path = if coupled { 0.16 } else { air * 0.12 };
            let occlusion = if airborne && blocked(e.site.body_local_position_meters) {
                0.15
            } else {
                1.
            };
            let gain = e.intensity * path * occlusion / (1. + distance * distance / 400.);
            if gain * listener.transmission > f64::from(out.bands[2].gain) {
                out.bands[2] = Band {
                    gain: (gain * listener.transmission) as f32,
                    pan: if distance > 0. {
                        (dot(delta, listener.right) / distance).clamp(-1., 1.) as f32
                    } else {
                        0.
                    },
                    filter: if coupled {
                        0.015
                    } else {
                        (0.25 * listener.filter) as f32
                    },
                };
            }
        }
    }
    // Only published physical flexible material. Never use camera/LOD micro-cover.
    if air > 0. && wind > 0. {
        for o in anatomy.population.objects() {
            if !matches!(o.specimen.form, Form::Frond | Form::Filament) {
                continue;
            }
            let delta = sub(o.position_meters, listener.eye);
            let distance = length(delta);
            if distance > 40. {
                continue;
            }
            let flexible: f64 = o
                .specimen
                .parts
                .iter()
                .filter(|p| p.id > 0)
                .map(|p| anatomy.remaining.map_or(1., |f| f(&o.id, p.id)).sqrt())
                .sum();
            let gain = air * (wind / 20.).min(1.) * (flexible / 20.).min(1.) * 0.12
                / (1. + distance * distance / 9.)
                * if blocked(o.position_meters) { 0.1 } else { 1. };
            if gain * listener.transmission > f64::from(out.bands[3].gain) {
                out.bands[3] = Band {
                    gain: (gain * listener.transmission) as f32,
                    pan: if distance > 0. {
                        (dot(delta, listener.right) / distance).clamp(-1., 1.) as f32
                    } else {
                        0.
                    },
                    filter: (0.5 * listener.filter) as f32,
                };
            }
        }
    }
    out.warning = if weather.storm_fraction > 0.7 && air > 0. {
        Some("LOCAL STORM / REDUCED VISIBILITY")
    } else if out.bands[2].gain > 0.02 {
        Some("LOCAL GROUND ACTIVITY")
    } else {
        None
    };
    out
}
pub(crate) fn shelter(eye: [f64; 3], ramp: f64, cargo: f64, screen: f64) -> (f64, f64) {
    let partitions = crate::scout_audio::acoustics::scout(cargo, screen);
    let t = crate::scout_audio::acoustics::transmission(eye, [-7.1, 1.34, 0.], &partitions);
    (t.0 * (0.025 + 0.975 * ramp), t.1 * (0.04 + 0.56 * ramp))
}
pub(crate) fn capture(s: &GameplaySession) -> Frame {
    if s.mining.terrain != salimon_world::terrain::TerrainMode::Canonical && !s.biology.fixture {
        return Frame::default();
    }
    let state = s.snapshot();
    let eye = state.character.eye_position_meters;
    let look = sub(state.character.look_target_meters, eye);
    let right = salimon_math::cross(look, state.character.up.map(f64::from));
    let norm = length(right);
    let right = if norm > 1e-9 {
        right.map(|v| v / norm)
    } else {
        [0.; 3]
    };
    let inside = matches!(
        state.character.location,
        salimon_character::CharacterLocation::InsideShip
            | salimon_character::CharacterLocation::Cockpit
            | salimon_character::CharacterLocation::DoorwayBlend
    );
    let surface = s.terrain_frame(state.ship);
    let contact = if inside {
        matches!(
            state.ship.flight_state,
            salimon_ship::FlightState::Landed { .. }
        )
    } else {
        state.character.location == salimon_character::CharacterLocation::Surface
            && length(sub(eye, surface.eye_on_ground(eye))) < 0.15
    };
    let (transmission, filter) = if inside {
        let ship_frame = crate::session::frames::character_ship_frame(state.ship.pose);
        shelter(
            ship_frame.world_to_local(eye),
            state.ship.door_open_fraction,
            s.cargo_door.open_fraction,
            s.habitation
                .cabin
                .as_ref()
                .map_or(0., |c| c.motions[2].fraction),
        )
    } else {
        (0.45, 0.35)
    };
    let Ok(epoch) = s.environment_epoch() else {
        return Frame::default();
    };
    let climate = if s.biology.fixture {
        crate::session::biology::FIXTURE.biosphere.climate()
    } else {
        salimon_world::canonical::BIOSPHERE.climate()
    };
    static FIXTURE_ACTIVITY: LazyLock<ActivityDefinition> = LazyLock::new(|| {
        ActivityDefinition::new(
            crate::session::biology::FIXTURE.biosphere.climate().clone(),
            1,
            0,
            ActivityInputs::dormant(),
        )
        .expect("explicit dormant fixture")
    });
    let body_origin = crate::session::biology::origin(s.biology.fixture);
    let remaining = |id: &str, part: u16| s.biological_fraction(id, part);
    sample(
        climate,
        if s.biology.fixture {
            &FIXTURE_ACTIVITY
        } else {
            &ACTIVITY
        },
        Anatomy {
            population: crate::session::biology::population(s.biology.fixture),
            remaining: Some(&remaining),
        },
        epoch,
        Listener {
            eye: sub(eye, body_origin),
            right,
            contact,
            transmission,
            filter,
        },
        0.,
        |p| {
            let p = std::array::from_fn(|i| p[i] + body_origin[i]);
            let ray = salimon_world::mining::MiningRay::new(
                salimon_world::WorldPosition::new(eye[0], eye[1], eye[2]),
                salimon_world::WorldPosition::new(p[0], p[1], p[2]),
            );
            ray.is_some_and(|ray| {
                salimon_world::mining::terrain_obstruction(
                    ray,
                    s.mining.terrain,
                    (length(sub(p, eye)) - 0.5).max(0.),
                )
                .is_some()
            })
        },
    )
}
/// Restrained contact-coupled extraction. Called only after a measured source mass change.
pub(crate) fn harvest(material: salimon_world::resources::ResourceId) -> Band {
    use salimon_world::resources::ResourceId::*;
    Band {
        gain: 0.10,
        pan: 0.,
        filter: match material {
            OrganicBiomass => 0.035,
            MedicinalCompounds => 0.08,
            WaterIce => 0.35,
            IronOre => 0.7,
            SilicateRock => 0.5,
        },
    }
}
pub(crate) fn apply(frame: Frame, control: &mut crate::native_audio::Control, settings: Settings) {
    if !settings.environment_audio || settings.master_mute {
        return;
    }
    for (i, band) in frame.bands.iter().enumerate() {
        let index = 27 + i;
        control.enabled |= 1 << index;
        control.gains[index] = band.gain * f32::from(settings.environment_volume_percent) / 100.;
        control.pans[index] = band.pan;
        control.filters[index] = band.filter;
    }
}
#[cfg(test)]
mod tests;
