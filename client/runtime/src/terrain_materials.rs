//! Maps canonical geological observations into renderer-neutral surface factors.
use salimon_world::terrain::geology::{GeologyProfile, GeologySample, Substrate};

/// Luminance only: the existing planet and resource identity palettes retain their hues.
/// Height/thickness are meters; strength is dimensionless. No geometry or physics input.
pub(crate) fn factors(
    profile: GeologyProfile,
    sample: &GeologySample,
    elevation: f64,
) -> (f32, [f32; 3]) {
    if profile == GeologyProfile::HomogeneousRock {
        return (1.0, [0.0, 1.0, 0.0]);
    }
    let ice = sample.substrate == Substrate::WaterIce;
    let albedo = if ice {
        0.98
    } else {
        0.88 + 0.06 * sample.weathering - 0.06 * sample.exposure
    };
    let strength = if ice {
        0.18
    } else {
        0.5 * (1.0 - sample.weathering * 0.5)
    };
    (
        albedo as f32,
        [
            elevation as f32,
            sample.layer_depth_meters.max(0.25) as f32,
            strength as f32,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::{
        CELESTIAL_BODIES, CelestialBodyId,
        terrain::{
            TerrainDefinition,
            geology::{GEOLOGY_VERSION, GeologyDefinition},
        },
    };
    #[test]
    fn physical_and_material_queries_survive_reload() {
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == CelestialBodyId::Earth)
            .unwrap();
        let terrain = TerrainDefinition::representative(body).unwrap();
        for profile in [
            GeologyProfile::DryRock,
            GeologyProfile::WeatheredRock,
            GeologyProfile::Ice,
        ] {
            let geology =
                GeologyDefinition::new(terrain.clone(), GEOLOGY_VERSION, 19, profile).unwrap();
            let restored = GeologyDefinition::from_bytes(body, &geology.to_bytes()).unwrap();
            for d in [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]] {
                let a = geology.sample(d).unwrap();
                let b = restored.sample(d).unwrap();
                assert_eq!(a.surface, terrain.sample(d).unwrap());
                // LOD, quality, light and camera are deliberately absent from this mapping.
                let expected = factors(profile, &a.geology, a.surface.elevation_meters);
                assert_eq!(
                    expected,
                    factors(profile, &b.geology, b.surface.elevation_meters)
                );
                assert!((0.8..=1.0).contains(&expected.0));
                assert!((0.0..=1.0).contains(&expected.1[2]));
            }
        }
    }
    #[test]
    fn vesper_palette_and_existing_surface_are_exactly_preserved() {
        let terrain = TerrainDefinition::canonical_start();
        let geology = terrain.geology();
        for d in [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0]] {
            let sample = geology.sample(d).unwrap();
            assert_eq!(
                factors(
                    geology.profile(),
                    &sample.geology,
                    sample.surface.elevation_meters
                ),
                (1.0, [0.0, 1.0, 0.0])
            );
            assert_eq!(sample.surface, terrain.sample(d).unwrap());
        }
    }
}
