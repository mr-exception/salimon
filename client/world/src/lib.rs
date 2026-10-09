//! Portable compressed Solar System and large-scale camera prototype.
//!
//! Authoritative positions remain `f64` meters. Presentation code must subtract
//! the camera position in `f64` before converting the small relative result to
//! `f32`; [`WorldPosition::camera_relative_f32`] is the canonical conversion.
//! This crate deliberately has no windowing or GPU dependency.

mod camera;
mod catalog;
mod coordinates;
mod geometry;
mod precision;
mod proximity;

pub mod carrying;
pub mod mining;
pub mod resource_distribution;
pub mod resource_fragments;
pub mod resource_generation;
pub mod resource_size;
pub mod resources;

pub use camera::{CameraCommand, CameraPrototype, CameraSnapshot, TransitionPhase, WorldSnapshot};
pub use catalog::{
    BodyRole, CELESTIAL_BODIES, CelestialBody, CelestialBodyId,
    PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND, SURFACE_ANCHOR,
};
pub use coordinates::WorldPosition;
pub use geometry::{LANDING_RANGE_ALTITUDE_RADIUS_FACTOR, sphere_surface_separation};
pub use precision::{PrecisionMarker, PrecisionReport};
pub use proximity::{
    NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS, nearby_solid_body,
    nearest_celestial_body_within_surface_distance,
};
