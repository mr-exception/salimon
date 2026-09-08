//! Save logical support identity; runtime rebinds the pure sampled-ground queries.
use crate::{SphereSurface, Surface};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Support {
    Floor {
        height_meters: f64,
    },
    UnpoweredFloor {
        height_meters: f64,
        ceiling_meters: f64,
    },
    Sphere(SphereSurface),
    Sampled(SphereSurface),
    Unsupported,
}
pub fn serialize<S: serde::Serializer>(value: &Surface, s: S) -> Result<S::Ok, S::Error> {
    match *value {
        Surface::Floor { height_meters } => Support::Floor { height_meters },
        Surface::UnpoweredFloor {
            height_meters,
            ceiling_meters,
        } => Support::UnpoweredFloor {
            height_meters,
            ceiling_meters,
        },
        Surface::Sphere(body) => Support::Sphere(body),
        Surface::Sampled { body, .. } => Support::Sampled(body),
        Surface::Unsupported => Support::Unsupported,
    }
    .serialize(s)
}
pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Surface, D::Error> {
    Ok(match Support::deserialize(d)? {
        Support::Floor { height_meters } => Surface::Floor { height_meters },
        Support::UnpoweredFloor {
            height_meters,
            ceiling_meters,
        } => Surface::UnpoweredFloor {
            height_meters,
            ceiling_meters,
        },
        // A temporary query-free surface. Only SleepTracker::from_save may publish
        // it, after the caller has rebound environment/geometry from its own state.
        Support::Sphere(body) | Support::Sampled(body) => Surface::Sphere(body),
        Support::Unsupported => Surface::Unsupported,
    })
}
