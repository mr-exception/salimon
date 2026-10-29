//! World-authored procedural specimens to neutral GPU segments; no physical population authority.
use salimon_renderer::OrganismSegment;
use salimon_world::environment::organisms::Specimen;
pub(super) fn segments(
    specimen: &Specimen,
    origin: [f64; 3],
    axes: [[f64; 3]; 3],
) -> Vec<OrganismSegment> {
    let point = |p: [f64; 3]| {
        std::array::from_fn(|i| origin[i] + (0..3).map(|j| axes[j][i] * p[j]).sum::<f64>())
    };
    specimen
        .parts
        .iter()
        .map(|part| OrganismSegment {
            start_meters: point(part.start),
            end_meters: point(part.end),
            radius_start_meters: part.radius_start,
            radius_end_meters: part.radius_end,
            color: specimen.color,
        })
        .collect()
}
/// The current canonical Vesper publication is sterile; no runtime reseeding or
/// implied life is allowed. Other definitions use the explicit specimen adapter.
pub(super) fn canonical(camera: [f64; 3]) -> Vec<OrganismSegment> {
    use salimon_world::canonical::BIOSPHERE;
    use salimon_world::environment::organisms::Grammar;
    if Grammar::from_biosphere(&BIOSPHERE).is_none() {
        return vec![];
    }
    let n = camera[0].hypot(camera[1]).hypot(camera[2]);
    if n <= 0. {
        return vec![];
    }
    let d = camera.map(|v| v / n);
    let (face, u, v) = if d[0].abs() >= d[1].abs().max(d[2].abs()) {
        if d[0] >= 0. {
            (0, d[1] / d[0], d[2] / d[0])
        } else {
            (1, d[1] / -d[0], d[2] / d[0])
        }
    } else if d[1].abs() >= d[2].abs() {
        if d[1] >= 0. {
            (2, d[0] / d[1], d[2] / d[1])
        } else {
            (3, d[0] / -d[1], d[2] / d[1])
        }
    } else if d[2] >= 0. {
        (4, d[0] / d[2], d[1] / d[2])
    } else {
        (5, d[0] / d[2], d[1] / -d[2])
    };
    let tile = salimon_world::environment::organisms::CoverTile {
        face,
        u: (((u + 1.) * 32768.).floor() as u32).min(65535) as u16,
        v: (((v + 1.) * 32768.).floor() as u32).min(65535) as u16,
    };
    let cover = salimon_world::environment::organisms::cosmetic_tile(&BIOSPHERE, tile, 64)
        .expect("validated fixed tile and published biosphere");
    let mut out = Vec::new();
    for c in cover {
        let up = c.outward_normal;
        let reference = if up[1].abs() > 0.9 {
            [1., 0., 0.]
        } else {
            [0., 1., 0.]
        };
        let x = [
            up[1] * reference[2] - up[2] * reference[1],
            up[2] * reference[0] - up[0] * reference[2],
            up[0] * reference[1] - up[1] * reference[0],
        ];
        let length = x[0].hypot(x[1]).hypot(x[2]);
        let x = x.map(|v| v / length);
        let z = [
            x[1] * up[2] - x[2] * up[1],
            x[2] * up[0] - x[0] * up[2],
            x[0] * up[1] - x[1] * up[0],
        ];
        out.extend(segments(&c.specimen, c.position_meters, [x, up, z]));
    }
    for object in salimon_world::canonical::ORGANISMS
        .nearby(camera, 5000.)
        .expect("finite admitted camera")
    {
        out.extend(segments(
            &object.specimen,
            object.position_meters,
            object.axes,
        ));
    }
    out
}
#[cfg(test)]
mod tests {
    #[test]
    fn sterile_vesper_never_gets_visible_organisms() {
        assert!(super::canonical([0., 3e6, 0.]).is_empty());
    }
    #[test]
    fn organism_segment_adapter_preserves_local_shape_and_large_f64_anchor() {
        use salimon_world::environment::organisms::{Form, Part, Specimen};
        let s = Specimen {
            id: "test".into(),
            grammar_id: "test".into(),
            form: Form::Column,
            color: [0.3; 3],
            parts: vec![Part {
                id: 0,
                start: [0.; 3],
                end: [0., 2., 0.],
                radius_start: 0.2,
                radius_end: 0.1,
            }],
        };
        let segments = super::segments(&s, [1e12; 3], [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]);
        assert_eq!(segments[0].end_meters[1] - segments[0].start_meters[1], 2.);
        assert_eq!(segments[0].radius_start_meters, 0.2);
    }
}

#[cfg(test)]
mod evidence {
    #[test]
    #[ignore = "exports actual world grammar/runtime geometry; set SALIMON_ORGANISMS_FILE"]
    fn organism_specimen_export() {
        use salimon_world::{
            environment::{
                climate::{Chemistry, ClimateDefinition, ClimateInputs, Condensable},
                ecology::{BiosphereDecision, BiosphereDefinition, BodyNature},
                organisms::{Form, Grammar},
            },
            terrain::TerrainDefinition,
        };
        let body = *salimon_world::CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Moon)
            .unwrap();
        let mut p = ClimateInputs::vacuum(86400., 31557600., 1361.);
        p.chemistry = Chemistry::Nitrogen;
        p.pressure_pascal = 101325.;
        p.scale_height_meters = 8000.;
        p.condensable = Condensable::Water;
        p.condensable_fraction = 0.8;
        p.greenhouse_kelvin = 30.;
        let climate = ClimateDefinition::new(
            TerrainDefinition::new(body, 1, 5, vec![], vec![])
                .unwrap()
                .geology(),
            1,
            37,
            p,
        )
        .unwrap();
        let biosphere = BiosphereDefinition::new(
            salimon_universe::CatalogId::from_source("fixture", "fictional-biology").unwrap(),
            BodyNature::Fictional,
            BiosphereDecision::Developed,
            climate,
            1,
            17,
        )
        .unwrap();
        let grammar = Grammar::from_biosphere(&biosphere).unwrap();
        let mut specimens = vec![];
        for form in [
            Form::Mat,
            Form::Column,
            Form::Frond,
            Form::Filament,
            Form::Colony,
            Form::Giant,
        ] {
            for variant in [0, 1] {
                let s = grammar.specimen(form, variant).unwrap();
                let segments =
                    super::segments(&s, [0.; 3], [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]);
                let data:Vec<_>=segments.iter().map(|s|serde_json::json!({"start":s.start_meters,"end":s.end_meters,"r0":s.radius_start_meters,"r1":s.radius_end_meters,"color":s.color})).collect();
                specimens.push(serde_json::json!({"name":format!("{form:?}-{variant}"),"id":s.id,"grammar":s.grammar_id,"segments":data}));
            }
        }
        std::fs::write(
            std::env::var("SALIMON_ORGANISMS_FILE").unwrap(),
            serde_json::to_vec_pretty(&specimens).unwrap(),
        )
        .unwrap();
    }
}
