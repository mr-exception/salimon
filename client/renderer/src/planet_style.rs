//! Compact presentation recipes, independent of authoritative terrain definitions.
//! Colors are linear RGB, matching the shared Rgba8Unorm texture array.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlanetVisualStyle {
    pub version: u8,
    pub seed: u32,
    pub palette: [[f64; 3]; 2],
    /// Oceanic material's land palette; selected by the same seeded broad field.
    pub land: Option<[[f64; 3]; 2]>,
}

// Indexed by renderer SurfaceMaterial, not celestial ID. Runtime owns that mapping.
// Explicit seeds survive enum/name refactors. Future external bodies can choose
// these styles without sending height maps or changing terrain physics seeds.
pub(crate) const STYLES: [PlanetVisualStyle; 6] = [
    PlanetVisualStyle {
        version: 1,
        seed: 91,
        palette: [[0.25, 0.23, 0.21], [0.55, 0.51, 0.45]],
        land: None,
    },
    PlanetVisualStyle {
        version: 1,
        seed: 734_378,
        palette: [[0.38, 0.24, 0.10], [0.72, 0.53, 0.25]],
        land: None,
    },
    PlanetVisualStyle {
        version: 1,
        seed: 1_468_665,
        palette: [[0.03, 0.12, 0.25], [0.08, 0.30, 0.46]],
        land: Some([[0.18, 0.27, 0.18], [0.43, 0.48, 0.28]]),
    },
    PlanetVisualStyle {
        version: 1,
        seed: 2_202_952,
        palette: [[0.24, 0.27, 0.30], [0.56, 0.57, 0.53]],
        land: None,
    },
    PlanetVisualStyle {
        version: 1,
        seed: 2_937_239,
        palette: [[0.32, 0.13, 0.07], [0.66, 0.34, 0.17]],
        land: None,
    },
    // Sun keeps its existing unquantized emissive surface recipe.
    PlanetVisualStyle {
        version: 1,
        seed: 3_671_526,
        palette: [[0.95, 0.31, 0.035], [1.0, 0.86, 0.22]],
        land: None,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn styles_are_versioned_distinct_opaque_palettes_with_stable_material_mapping() {
        use crate::SurfaceMaterial;
        assert_eq!(STYLES[SurfaceMaterial::Rust as usize].seed, 2_937_239);
        assert!(STYLES[SurfaceMaterial::Oceanic as usize].land.is_some());
        for (i, style) in STYLES.iter().enumerate() {
            assert_eq!(style.version, 1);
            assert!(!STYLES[..i].iter().any(|s| s.seed == style.seed));
            for palette in std::iter::once(&style.palette).chain(style.land.iter()) {
                for color in palette {
                    assert!(
                        color
                            .iter()
                            .all(|c| c.is_finite() && (0.0..=1.0).contains(c))
                    );
                }
                assert_ne!(palette[0], palette[1]);
            }
        }
    }
}
