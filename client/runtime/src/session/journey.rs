//! Authoritative travel accounting. Only admitted physical tick endpoints enter
//! this module: render/camera coordinates and time between saves never do.
use salimon_character::{CharacterLocation, CharacterSnapshot};
use salimon_persistence::{SaveError, Validate, invalid};
#[cfg(test)]
use salimon_world::CELESTIAL_BODIES;
use salimon_world::CelestialBodyId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct JourneyState {
    pub(crate) distance_meters: f64,
    /// None is accepted only while decoding the schema-2 migration.
    #[serde(default)]
    pub(crate) tracking: Option<TrackingState>,
}
impl Default for JourneyState {
    fn default() -> Self {
        Self {
            distance_meters: 0.0,
            tracking: Some(TrackingState::default()),
        }
    }
}
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TrackingState {
    last_ship_position_meters: Option<[f64; 3]>,
    last_character_position: Option<CharacterPosition>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
enum CharacterPosition {
    ShipLocal([f64; 3]),
    World([f64; 3]),
}
impl CharacterPosition {
    fn capture(s: CharacterSnapshot) -> Self {
        match s.local_ship_position_meters {
            Some(local)
                if matches!(
                    s.location,
                    CharacterLocation::InsideShip | CharacterLocation::Cockpit
                ) =>
            {
                Self::ShipLocal(local)
            }
            _ => Self::World(s.eye_position_meters),
        }
    }
    fn meters(&self) -> [f64; 3] {
        match self {
            Self::ShipLocal(p) | Self::World(p) => *p,
        }
    }
}
impl Validate for JourneyState {
    fn validate(&self) -> Result<(), SaveError> {
        if !self.distance_meters.is_finite() || self.distance_meters < 0.0 {
            return Err(invalid("journey distance must be finite and nonnegative"));
        }
        if let Some(t) = &self.tracking {
            for p in [
                t.last_ship_position_meters,
                t.last_character_position
                    .as_ref()
                    .map(CharacterPosition::meters),
            ]
            .into_iter()
            .flatten()
            {
                if !p.into_iter().all(f64::is_finite) {
                    return Err(invalid("journey sample must be finite"));
                }
            }
        }
        Ok(())
    }
}
impl JourneyState {
    pub(super) fn initialize_tracking(&mut self) {
        self.tracking = Some(TrackingState::default());
    }
    pub(super) fn record(
        &mut self,
        ship_start: [f64; 3],
        ship_end: [f64; 3],
        character_start: CharacterSnapshot,
        character_end: CharacterSnapshot,
        active: bool,
    ) {
        let start = CharacterPosition::capture(character_start);
        let end = CharacterPosition::capture(character_end);
        // The ship-local path preserves submillimetre cabin movement at solar
        // coordinates. A frame crossing uses world endpoints in the SAME ship
        // frame, so the doorway segment remains physical rather than a jump.
        let walking = match (&start, &end) {
            (CharacterPosition::ShipLocal(a), CharacterPosition::ShipLocal(b)) => distance(*a, *b),
            _ => distance(
                character_start.eye_position_meters,
                character_end.eye_position_meters,
            ),
        };
        let travel = distance(ship_start, ship_end) + walking;
        if active && travel.is_finite() && (self.distance_meters + travel).is_finite() {
            self.distance_meters += travel;
        }
        // Persist the latest valid authority, but never bridge it to a later
        // tick: commands, fixture teleports and restore can change that baseline.
        if ship_end.into_iter().all(f64::is_finite) && end.meters().into_iter().all(f64::is_finite)
        {
            self.tracking = Some(TrackingState {
                last_ship_position_meters: Some(ship_end),
                last_character_position: Some(end),
            });
        }
    }
}
fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    (b[0] - a[0]).hypot(b[1] - a[1]).hypot(b[2] - a[2])
}

/// Stable body identity plus a presentation name. Space retains nearest-body
/// context without claiming the player is on that body's surface.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NamedLocation {
    pub(crate) name: String,
    pub(crate) nearest_body: CelestialBodyId,
    pub(crate) altitude_meters: f64,
}
impl NamedLocation {
    #[cfg(test)]
    pub(crate) fn at(position: [f64; 3]) -> Self {
        Self::with_terrain(position, salimon_world::terrain::TerrainMode::Flat)
    }
    pub(crate) fn with_terrain(
        position: [f64; 3],
        mode: salimon_world::terrain::TerrainMode,
    ) -> Self {
        let body = mode
            .bodies()
            .iter()
            .min_by(|a, b| {
                (distance(position, a.center.meters()) - a.radius_meters)
                    .max(0.0)
                    .total_cmp(&(distance(position, b.center.meters()) - b.radius_meters).max(0.0))
            })
            .expect("nonempty world catalog");
        let altitude = distance(position, body.center.meters()) - body.radius_meters;
        // Share the world's nearby-body envelope, rather than inventing a
        // second proximity radius for Journey Ended.
        let near = altitude <= salimon_world::NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS;
        Self {
            name: if near {
                format!("Near {}", body.name)
            } else {
                format!("Space near {}", body.name)
            },
            nearest_body: body.id,
            altitude_meters: altitude,
        }
    }
}

#[cfg(test)]
mod tests;
