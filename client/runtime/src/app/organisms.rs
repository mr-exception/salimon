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
            material: material(specimen, part.id),
            flex_meters: [0.; 3],
            coating: [0.; 3],
        })
        .collect()
}
/// Species traits derive from immutable grammar identity, not visit time or camera.
fn material(specimen: &Specimen, part: u16) -> [f32; 4] {
    let key = specimen
        .grammar_id
        .bytes()
        .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(u64::from(b)));
    let family = key % 6;
    let roughness = [0.95, 0.3, 0.8, 0.6, 0.4, 0.7][family as usize];
    // Sparse terminal organs alone emit; the rest of the plant remains dark at night.
    let emission = if key.is_multiple_of(5) && part > 0 && part.is_multiple_of(7) {
        0.7
    } else {
        0.
    };
    [
        roughness,
        if family == 3 { 0.6 } else { 0. },
        if family == 4 { 0.35 } else { 0. },
        emission,
    ]
}
/// Stateless shared-TT response. No physical endpoints, identities or roots are changed.
pub(super) fn respond(
    segments: &mut [OrganismSegment],
    specimen: &Specimen,
    weather: &salimon_world::environment::climate::WeatherSample,
    time_seconds: f64,
) {
    use salimon_world::environment::climate::Condensable;
    let wind = weather.wind_mps;
    let speed = wind[0].hypot(wind[1]).hypot(wind[2]);
    let airborne = weather.pressure_pascal > 0.;
    let wet = if airborne
        && weather.precipitation == Condensable::Water
        && weather.temperature_kelvin > 273.15
    {
        (weather.precipitation_kg_m2_s * 100.).clamp(0., 1.) as f32
    } else {
        0.
    };
    let frost = if airborne
        && weather.precipitation == Condensable::Water
        && weather.temperature_kelvin <= 273.15
    {
        (weather.precipitation_kg_m2_s * 100.).clamp(0., 1.) as f32
    } else {
        0.
    };
    let dust = if airborne && weather.precipitation == Condensable::None {
        weather.storm_fraction.clamp(0., 1.) as f32
    } else {
        0.
    };
    for (segment, part) in segments.iter_mut().zip(&specimen.parts) {
        segment.coating = [wet, frost, dust];
        let flexible = matches!(
            specimen.form,
            salimon_world::environment::organisms::Form::Frond
                | salimon_world::environment::organisms::Form::Filament
        );
        // Structural stem remains fixed; only fine flexible appendages move inside colliders.
        let gain = if airborne && flexible && part.id > 0 && speed > 0. {
            segment.radius_start_meters.min(segment.radius_end_meters)
                * 0.08
                * (speed / 20.).min(1.)
                * (0.65 + 0.35 * (time_seconds * 1.7 + f64::from(part.id)).sin())
                / speed
        } else {
            0.
        };
        segment.flex_meters = wind.map(|v| (v * gain) as f32);
    }
}
/// The current canonical Vesper publication is sterile; no runtime reseeding or
/// implied life is allowed. Other definitions use the explicit specimen adapter.
pub(super) fn canonical(camera: [f64; 3], epoch: salimon_universe::Epoch) -> Vec<OrganismSegment> {
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
        let mut geometry = segments(&c.specimen, c.position_meters, [x, up, z]);
        let source_radius = c.position_meters[0]
            .hypot(c.position_meters[1])
            .hypot(c.position_meters[2]);
        let weather = BIOSPHERE
            .climate()
            .sample(c.position_meters.map(|x| x / source_radius), epoch)
            .expect("published climate and TT epoch");
        respond(&mut geometry, &c.specimen, &weather, super_time(epoch));
        out.extend(geometry);
    }
    for object in salimon_world::canonical::ORGANISMS
        .nearby(camera, 5000.)
        .expect("finite admitted camera")
    {
        let mut geometry = segments(&object.specimen, object.position_meters, object.axes);
        let n = object
            .position_meters
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt();
        let weather = BIOSPHERE
            .climate()
            .sample(object.position_meters.map(|x| x / n), epoch)
            .expect("published climate");
        respond(&mut geometry, &object.specimen, &weather, super_time(epoch));
        out.extend(geometry);
    }
    out
}
fn super_time(epoch: salimon_universe::Epoch) -> f64 {
    let origin = salimon_universe::Epoch::calendar(
        [2000, 1, 1],
        [12, 0, 0, 0],
        salimon_universe::TimeScale::Tt,
    )
    .expect("fixed TT epoch");
    epoch.seconds_since(origin).expect("same TT scale")
}
#[cfg(test)]
mod tests {
    #[test]
    fn sterile_vesper_never_gets_visible_organisms() {
        assert!(
            super::canonical(
                [0., 3e6, 0.],
                salimon_universe::Epoch::calendar(
                    [2000, 1, 1],
                    [12, 0, 0, 0],
                    salimon_universe::TimeScale::Tt
                )
                .unwrap()
            )
            .is_empty()
        );
    }
    #[test]
    fn biological_weather_response_keeps_source_bounds_and_stems() {
        use salimon_world::environment::{
            climate::{Condensable, WeatherSample},
            organisms::{Form, Part, Specimen},
        };
        let specimen = Specimen {
            id: "source".into(),
            grammar_id: "species".into(),
            form: Form::Frond,
            color: [0.3; 3],
            parts: (0..3)
                .map(|id| Part {
                    id,
                    start: [0.; 3],
                    end: [0., 2., 0.],
                    radius_start: 0.2,
                    radius_end: 0.1,
                })
                .collect(),
        };
        let epoch = salimon_universe::Epoch::calendar(
            [2000, 1, 1],
            [12, 0, 0, 0],
            salimon_universe::TimeScale::Tt,
        )
        .unwrap();
        let mut weather = WeatherSample {
            definition_id: "test".into(),
            province_id: "test".into(),
            epoch,
            climate_temperature_kelvin: 290.,
            temperature_kelvin: 290.,
            pressure_pascal: 1e5,
            cloud_fraction: 1.,
            storm_fraction: 0.8,
            wind_mps: [10., 0., 0.],
            precipitation: Condensable::Water,
            precipitation_kg_m2_s: 0.01,
            extinction_per_meter: 0.001,
        };
        let original = super::segments(
            &specimen,
            [1e12; 3],
            [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        );
        let mut geometry = original.clone();
        super::respond(&mut geometry, &specimen, &weather, 3.);
        assert_eq!(geometry[0].flex_meters, [0.; 3]);
        assert!(geometry[1].flex_meters[0] > 0.);
        assert_eq!(geometry[1].coating, [1., 0., 0.]);
        for (before, after) in original.iter().zip(&geometry) {
            assert_eq!(before.start_meters, after.start_meters);
            assert_eq!(before.end_meters, after.end_meters);
            assert!(after.flex_meters[0] <= after.radius_end_meters as f32 * 0.1);
        }
        let mut repeat = original.clone();
        super::respond(&mut repeat, &specimen, &weather, 3.);
        assert_eq!(repeat, geometry);
        weather.temperature_kelvin = 260.;
        super::respond(&mut geometry, &specimen, &weather, 3.);
        assert_eq!(geometry[1].coating, [0., 1., 0.]);
        weather.precipitation = Condensable::None;
        super::respond(&mut geometry, &specimen, &weather, 3.);
        assert_eq!(geometry[1].coating, [0., 0., 0.8]);
        weather.pressure_pascal = 0.;
        super::respond(&mut geometry, &specimen, &weather, 3.);
        assert_eq!(geometry[1].coating, [0.; 3]);
        assert_eq!(geometry[1].flex_meters, [0.; 3]);
        // All six material families stay bounded; emission never covers entire anatomy.
        let mut families = std::collections::BTreeSet::new();
        for key in 0..100 {
            let mut source = specimen.clone();
            source.grammar_id = format!("species-{key}");
            let root = super::material(&source, 0);
            assert_eq!(root[3], 0.);
            families.insert((root[0] * 100.) as u32);
            assert!(
                super::material(&source, 7)
                    .iter()
                    .all(|x| (0.0..=1.0).contains(x))
            );
        }
        assert_eq!(families.len(), 6);
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
                let data:Vec<_>=segments.iter().map(|s|serde_json::json!({"start":s.start_meters,"end":s.end_meters,"r0":s.radius_start_meters,"r1":s.radius_end_meters,"color":s.color,"material":s.material,"flex":s.flex_meters,"coating":s.coating})).collect();
                specimens.push(serde_json::json!({"name":format!("{form:?}-{variant}"),"id":s.id,"grammar":s.grammar_id,"segments":data}));
            }
        }
        let mut families = std::collections::BTreeSet::new();
        for seed in 0..100 {
            let b = BiosphereDefinition::new(
                biosphere.catalog_body().clone(),
                BodyNature::Fictional,
                BiosphereDecision::Developed,
                biosphere.climate().clone(),
                1,
                seed,
            )
            .unwrap();
            let g = Grammar::from_biosphere(&b).unwrap();
            let source = g.specimen(Form::Colony, 0).unwrap();
            let family = (super::material(&source, 0)[0] * 100.) as u32;
            if !families.insert(family) {
                continue;
            }
            for (name, light) in [
                ("day", [1., 1., 1.]),
                ("red", [1., 0.3, 0.15]),
                ("night", [0.; 3]),
            ] {
                let geometry =
                    super::segments(&source, [0.; 3], [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]);
                let data:Vec<_>=geometry.iter().map(|s|serde_json::json!({"start":s.start_meters,"end":s.end_meters,"r0":s.radius_start_meters,"r1":s.radius_end_meters,"color":s.color,"material":s.material,"flex":s.flex_meters,"coating":s.coating})).collect();
                specimens.push(serde_json::json!({"name":format!("material-{family}-{name}"),"id":source.id,"grammar":source.grammar_id,"light":light,"segments":data}));
            }
        }
        let source = grammar.specimen(Form::Frond, 0).unwrap();
        let epoch = salimon_universe::Epoch::calendar(
            [2000, 1, 1],
            [12, 0, 0, 0],
            salimon_universe::TimeScale::Tt,
        )
        .unwrap();
        for (name, temperature, precipitation) in [
            ("wet", 290., Condensable::Water),
            ("frost", 260., Condensable::Water),
            ("dust", 290., Condensable::None),
        ] {
            // Declared weather-response fixture drives the actual runtime adapter.
            let mut weather = biosphere.climate().sample([1., 0., 0.], epoch).unwrap();
            weather.wind_mps = [15., 0., 0.];
            weather.temperature_kelvin = temperature;
            weather.precipitation = precipitation;
            weather.precipitation_kg_m2_s = 0.01;
            weather.storm_fraction = 0.8;
            let mut geometry =
                super::segments(&source, [0.; 3], [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]);
            super::respond(&mut geometry, &source, &weather, 3.);
            let data:Vec<_>=geometry.iter().map(|s|serde_json::json!({"start":s.start_meters,"end":s.end_meters,"r0":s.radius_start_meters,"r1":s.radius_end_meters,"color":s.color,"material":s.material,"flex":s.flex_meters,"coating":s.coating})).collect();
            specimens.push(serde_json::json!({"name":name,"id":source.id,"grammar":source.grammar_id,"fixture_weather":true,"segments":data}));
        }
        std::fs::write(
            std::env::var("SALIMON_ORGANISMS_FILE").unwrap(),
            serde_json::to_vec_pretty(&specimens).unwrap(),
        )
        .unwrap();
    }
}

#[cfg(test)]
mod natural_site_evidence {
    use super::*;
    use salimon_world::{
        environment::{
            climate::{Chemistry, ClimateDefinition, ClimateInputs, Condensable},
            ecology::{BiosphereDecision, BiosphereDefinition, BodyNature},
            landmarks::{Geography, Landmarks},
            organism_placement::Population,
            signature_sites::{Kind, SignatureSites},
        },
        terrain::TerrainDefinition,
    };
    fn sources(seed: u64) -> (BiosphereDefinition, Population, Landmarks) {
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
        let c = ClimateDefinition::new(
            TerrainDefinition::new(body, 1, 5, vec![], vec![])
                .unwrap()
                .geology(),
            1,
            37,
            p,
        )
        .unwrap();
        let b = BiosphereDefinition::new(
            salimon_universe::CatalogId::from_source("fixture", "fictional-biology").unwrap(),
            BodyNature::Fictional,
            BiosphereDecision::Developed,
            c,
            1,
            seed,
        )
        .unwrap();
        let population = Population::publish(&b, &[]).unwrap();
        let geology = Landmarks::publish(
            b.climate().geology().clone(),
            Geography::PublishedFictional,
            0,
            &[],
        )
        .unwrap();
        (b, population, geology)
    }
    #[test]
    fn natural_site_render_and_collision_parts_survive_quality_and_reconstruction() {
        let mut kinds = std::collections::BTreeSet::new();
        for seed in 0..128 {
            let (b, p, g) = sources(seed);
            let sites = SignatureSites::publish(&b, &p, &g, None, &[]).unwrap();
            for site in sites.sites() {
                kinds.insert(format!("{:?}", site.kind));
                for id in &site.organism_ids {
                    let o = p.objects().iter().find(|o| o.id == *id).unwrap();
                    let geometry = segments(&o.specimen, o.position_meters, o.axes);
                    assert_eq!(geometry.len(), o.parts.len());
                    for (v, physical) in geometry.iter().zip(&o.parts) {
                        assert_eq!(v.start_meters, physical.start_meters);
                        assert_eq!(v.end_meters, physical.end_meters);
                    }
                }
                let b2 = SignatureSites::publish(
                    &b,
                    &Population::publish(&b, &[]).unwrap(),
                    &g,
                    None,
                    &[],
                )
                .unwrap();
                b2.verify_binding(&sites.binding()).unwrap();
            }
        }
        assert!(kinds.contains("GiantOrganism") && kinds.contains("DenseColony"));
        assert!(salimon_world::canonical::NATURAL_SITES.sites().is_empty());
    }
    #[test]
    #[ignore = "exports actual rare source/site/population geometry; set SALIMON_NATURAL_SITES_FILE"]
    fn natural_site_production_export() {
        let mut models = vec![];
        let mut found = std::collections::BTreeSet::new();
        for seed in 0..128 {
            let (b, p, g) = sources(seed);
            let sites = SignatureSites::publish(&b, &p, &g, None, &[]).unwrap();
            for site in sites.sites() {
                if !matches!(site.kind, Kind::GiantOrganism | Kind::DenseColony)
                    || !found.insert(format!("{:?}", site.kind))
                {
                    continue;
                }
                let objects: Vec<_> = site
                    .organism_ids
                    .iter()
                    .map(|id| p.objects().iter().find(|o| o.id == *id).unwrap())
                    .collect();
                let first = objects[0];
                let geometry: Vec<_> = objects
                    .iter()
                    .flat_map(|o| segments(&o.specimen, o.position_meters, o.axes))
                    .collect();
                let data:Vec<_>=geometry.iter().map(|s|serde_json::json!({"start":s.start_meters,"end":s.end_meters,"r0":s.radius_start_meters,"r1":s.radius_end_meters,"color":s.color,"material":s.material,"flex":s.flex_meters,"coating":s.coating})).collect();
                models.push(serde_json::json!({"name":format!("natural-{:?}",site.kind),"id":site.id,"publication":sites.identity(),"source_ids":site.organism_ids,"bounds":site.bounds,"binding":sites.binding(),"biosphere":b.to_bytes(),"population_binding":p.binding(),"origin":first.position_meters,"axes":first.axes,"segments":data}));
            }
            if found.len() == 2 {
                break;
            }
        }
        assert_eq!(found.len(), 2);
        std::fs::write(
            std::env::var("SALIMON_NATURAL_SITES_FILE").unwrap(),
            serde_json::to_vec_pretty(&models).unwrap(),
        )
        .unwrap();
    }
}
