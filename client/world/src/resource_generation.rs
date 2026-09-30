//! Stateless, bounded cube-sphere sampling. No full-planet allocation or GPU types.
use std::f64::consts::PI;

use crate::resource_distribution::BodyResourceDistribution;
use crate::resources::{DepositId, RawMaterial, ResourceDeposit};
use crate::{BodyRole, CelestialBody, WorldPosition};

/// A materialized deposit with authoritative body-local geometry in f64 meters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceDeposit {
    pub deposit: ResourceDeposit,
    pub body_local_position_meters: [f64; 3],
    /// Spherical physical bound derived from initial solid volume (not a mesh).
    pub bounds_radius_meters: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationError {
    InvalidArea,
    InvalidMaterial,
    BodyMismatch,
    UnsupportedResolution,
    CandidateBudgetExceeded,
}

/// Maximum candidate cells examined per request, across all materials and faces.
pub const MAX_CANDIDATES: usize = 65_536;
/// The caller replaces its active list on each successful query; keeping mined
/// state across unload/reload is a separate local-state layer (#50).
/// Radius selects deposit centers by body-local chord distance from the supplied
/// player position, including altitude. Empty/far-space queries allocate no planet.
pub fn materialize_nearby_deposits(
    body: CelestialBody,
    profile: &BodyResourceDistribution,
    world_seed: u64,
    biome: Option<&str>,
    player: WorldPosition,
    radius_meters: f64,
) -> Result<Vec<SurfaceDeposit>, GenerationError> {
    if profile.body() != body.id {
        return Err(GenerationError::BodyMismatch);
    }
    if !player.is_finite()
        || !body.center.is_finite()
        || !body.radius_meters.is_finite()
        || body.radius_meters <= 0.0
        || !radius_meters.is_finite()
        || radius_meters <= 0.0
    {
        return Err(GenerationError::InvalidArea);
    }
    let local = player.offset_from(body.center);
    let length = norm(local);
    if !length.is_finite() {
        return Err(GenerationError::InvalidArea);
    }
    if body.role != BodyRole::Solid || (length - body.radius_meters).abs() > radius_meters {
        return Ok(vec![]);
    }
    if length == 0.0 {
        return Err(GenerationError::InvalidArea);
    }
    let direction = local.map(|v| v / length);
    // Triangle inequality bounds every surface point in the query ball around
    // the radial projection, including when the player is above/below ground.
    let chord =
        ((radius_meters + (length - body.radius_meters).abs()) / body.radius_meters).min(2.0);
    let inputs = profile.generation_inputs(world_seed, biome);
    let max_weight = inputs
        .iter()
        .map(|i| i.distribution.weight())
        .fold(0.0, f64::max);
    let total_weight: f64 = inputs
        .iter()
        .map(|i| i.distribution.weight() / max_weight)
        .sum();
    let mut output = Vec::new();
    let mut candidates = 0;
    for input in inputs {
        let distribution = input.distribution;
        let resolution = (2.0 * body.radius_meters / distribution.spacing_meters())
            .ceil()
            .max(1.0);
        if !resolution.is_finite() || resolution > 100_000_000.0 {
            return Err(GenerationError::UnsupportedResolution);
        }
        let cells = resolution as u64;
        for face in 0..6_u64 {
            let axis = face as usize / 2;
            let sign = if face % 2 == 0 { 1.0 } else { -1.0 };
            let dominant = direction[axis] * sign;
            // On this face the normalized dominant component is >= 1/sqrt(3).
            if dominant + chord < 1.0 / 3.0_f64.sqrt() {
                continue;
            }
            let lower = (dominant - chord).max(1.0 / 3.0_f64.sqrt());
            let upper = (dominant + chord).min(1.0);
            let uv_axes = [(axis + 1) % 3, (axis + 2) % 3];
            let intervals = uv_axes.map(|a| {
                let lo = direction[a] - chord;
                let hi = direction[a] + chord;
                let ratios = [lo / lower, lo / upper, hi / lower, hi / upper];
                let min = ratios
                    .into_iter()
                    .fold(f64::INFINITY, f64::min)
                    .clamp(-1.0, 1.0);
                let max = ratios
                    .into_iter()
                    .fold(f64::NEG_INFINITY, f64::max)
                    .clamp(-1.0, 1.0);
                let index = |v: f64| (((v + 1.0) * 0.5 * resolution).floor() as u64).min(cells - 1);
                [index(min), index(max)]
            });
            let count = (intervals[0][1] - intervals[0][0] + 1)
                .saturating_mul(intervals[1][1] - intervals[1][0] + 1);
            if count > (MAX_CANDIDATES - candidates) as u64 {
                return Err(GenerationError::CandidateBudgetExceeded);
            }
            candidates += count as usize;
            for u in intervals[0][0]..=intervals[0][1] {
                for v in intervals[1][0]..=intervals[1][1] {
                    let seed = cell_seed(input.seed, cells, face, u, v);
                    let probability = (distribution.weight() / max_weight) / total_weight;
                    if sample(seed, 0) >= probability {
                        continue;
                    }
                    let mut cube = [0.0; 3];
                    cube[axis] = sign;
                    cube[uv_axes[0]] =
                        -1.0 + 2.0 * (u as f64 + 0.1 + 0.8 * sample(seed, 1)) / resolution;
                    cube[uv_axes[1]] =
                        -1.0 + 2.0 * (v as f64 + 0.1 + 0.8 * sample(seed, 2)) / resolution;
                    let scale = body.radius_meters / norm(cube);
                    let position = cube.map(|x| x * scale);
                    if norm(std::array::from_fn(|a| position[a] - local[a])) > radius_meters {
                        continue;
                    }
                    let [min, max] = distribution.mass_range_kg();
                    let mass = min + (max - min) * sample(seed, 3);
                    let material = RawMaterial::new(distribution.resource(), mass)
                        .map_err(|_| GenerationError::InvalidMaterial)?;
                    let deposit = ResourceDeposit::new(
                        DepositId {
                            body: body.id,
                            local: seed,
                        },
                        material,
                        body.center.translated(position),
                        mass,
                    )
                    .map_err(|_| GenerationError::InvalidArea)?;
                    output.push(SurfaceDeposit {
                        deposit,
                        body_local_position_meters: position,
                        bounds_radius_meters: (3.0 * material.volume_m3() / (4.0 * PI)).cbrt(),
                    });
                }
            }
        }
    }
    Ok(output)
}

fn norm(vector: [f64; 3]) -> f64 {
    vector[0].hypot(vector[1]).hypot(vector[2])
}

// Versioned explicit arithmetic, never DefaultHasher or process RNG. IDs are
// 64-bit content keys; configuration/seed changes define a different world.
fn cell_seed(seed: u64, cells: u64, face: u64, u: u64, v: u64) -> u64 {
    [1_u64, cells, face, u, v]
        .into_iter()
        .fold(seed, |mut hash, value| {
            for byte in value.to_le_bytes() {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
            hash
        })
}
fn sample(seed: u64, lane: u64) -> f64 {
    let mut value = seed.wrapping_add(lane.wrapping_add(1).wrapping_mul(0x9e3779b97f4a7c15));
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((value ^ (value >> 31)) >> 11) as f64 / (1_u64 << 53) as f64
}
