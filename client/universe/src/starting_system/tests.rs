use super::*;

#[test]
fn starting_system_publication_and_world_golden() {
    let p = candidate_publication().unwrap();
    assert_eq!(p.sources().len(), 8);
    assert_eq!(p.sha256().unwrap(), CANDIDATE_SHA256);
    let start = freeze(&p).unwrap();
    let bytes = start.encode().unwrap();
    assert_eq!(
        bytes,
        include_str!("../../fixtures/starting-world-v1.json")
            .trim_end()
            .as_bytes()
    );
    assert_eq!(synthesis::sha256(&bytes), WORLD_SHA256);
    assert_eq!(FrozenStart::decode(&bytes).unwrap(), start);
    assert_eq!(permanent().unwrap().encode().unwrap(), bytes);
    assert_eq!(start.manifest().system.id.as_str(), SYSTEM_ID);
    assert_eq!(start.manifest().star.id.as_str(), STAR_ID);
    assert_eq!(start.manifest().planet.id.as_str(), PLANET_ID);
    assert_eq!(start.manifest().planet.display_name, "Vesper");
    assert_ne!(start.manifest().source_recipe_sha256, WORLD_SHA256);
    let audits = assess(&p).unwrap();
    assert_eq!(audits.len(), 26);
    assert_eq!(audits.iter().filter(|a| a.eligible).count(), 5);
    assert_eq!(
        audits.iter().find(|a| a.eligible).unwrap().planet.as_str(),
        PLANET_ID
    );
}

#[test]
fn starting_system_distance_independent_split_cell_proof() {
    let start = permanent().unwrap();
    let d = &start.manifest().distance;
    // Independent cell/offset subtraction and sum-of-squares, no generator
    // placement function. Integer difference must precede conversion to meters.
    let mut sum = 0.0;
    for axis in 0..3 {
        let dc = i128::from(d.system_position.cells()[axis])
            - i128::from(d.solar_barycenter.cells()[axis]);
        let delta = dc as f64 * 1_099_511_627_776.0 + d.system_position.offset_meters()[axis]
            - d.solar_barycenter.offset_meters()[axis];
        sum += delta * delta;
    }
    let distance = sum.sqrt();
    assert!((distance / LIGHT_YEAR_METERS - 37.55006266702156).abs() < 1e-12);
    let lower =
        (distance - 2.0 * AU_METERS - start.manifest().physical.semi_major_axis_meters - 1000.0)
            / LIGHT_YEAR_METERS;
    assert_eq!(lower, d.minimum_planet_earth_distance_light_years);
    assert!(lower > 37.55);
    assert!(lower > 30.0);
    assert!(lower < d.system_distance_light_years);
}

#[test]
fn starting_system_physical_limits_and_source_recipe_preservation() {
    let p = candidate_publication().unwrap();
    let start = freeze(&p).unwrap();
    let m = start.manifest();
    let phys = &m.physical;
    let recipe = &p.bodies()[&m.planet.id];
    assert_eq!(phys.mass_kg, recipe.mass_kg);
    assert_eq!(phys.radius_meters, recipe.radius_meters);
    assert_eq!(phys.period_seconds, recipe.orbit.period_seconds);
    assert_eq!(digest(recipe).unwrap(), m.source_recipe_sha256);
    // Independent SI density, Kepler, gravity, Roche and main-sequence screens.
    let r = phys.radius_meters;
    let density = phys.mass_kg / ((4.0 / 3.0) * std::f64::consts::PI * ((r * r) * r));
    assert!((density - 5060.0).abs() < 1e-10);
    assert!((phys.gravity_meters_per_second_squared - 4.223197713467944).abs() < 1e-12);
    let a = phys.semi_major_axis_meters;
    let period =
        std::f64::consts::TAU * (((a * a) * a) / (G * (phys.star_mass_kg + phys.mass_kg))).sqrt();
    assert_eq!(period, phys.period_seconds);
    assert!(a > r + phys.star_radius_meters);
    assert!(a > 2.44 * r * (phys.star_mass_kg / phys.mass_kg).cbrt());
    let stellar_mass = phys.star_mass_kg / SOLAR_MASS_KG;
    let life_years = 1e10 / stellar_mass.powf(2.5);
    assert!(phys.age_seconds / crate::inference::YEAR_SECONDS < life_years);
    assert!(phys.equilibrium_temperature_kelvin > 160.0);
    assert!(phys.equilibrium_temperature_kelvin < 165.0);
    assert_eq!(phys.surface_pressure_pascals, 0.0);
    for domain in &m.resources.outcrops {
        assert!(domain.initial_mass_kg.is_finite());
    }
}

#[test]
fn starting_system_refuses_altered_candidate_without_reselection() {
    let request: Request =
        serde_json::from_slice(include_bytes!("../../fixtures/starting-candidates-v1.json"))
            .unwrap();
    let base = Catalog::new("awakening-empty-v1".into(), vec![], vec![]).unwrap();
    for change in 0..3 {
        let mut altered = request.clone();
        match change {
            0 => altered.seed += 1,
            1 => altered.regions[0].center = solar_barycenter().unwrap(),
            _ => altered.revision.push_str("-changed"),
        }
        let p = synthesis::build(&base, altered).unwrap();
        assert!(freeze(&p).is_err());
        if change == 1 {
            assert!(assess(&p).unwrap().iter().all(|a| !a.eligible));
        }
    }
}

#[test]
fn starting_system_rejects_corrupt_versions_parameters_and_names() {
    let start = permanent().unwrap();
    let bytes = start.encode().unwrap();
    for field in [
        "version",
        "planet",
        "terrain",
        "resources",
        "visual",
        "arrival",
        "distance",
        "physical",
        "provenance",
    ] {
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        v[field] = serde_json::json!(999);
        assert!(
            FrozenStart::decode(&serde_json::to_vec(&v).unwrap()).is_err(),
            "{field}"
        );
    }
    for change in 0..4 {
        let mut manifest = start.manifest().clone();
        match change {
            0 => manifest.planet.display_name = "different".into(),
            1 => manifest.terrain.seed += 1,
            2 => manifest.resources.outcrops[0].initial_mass_kg += 1.0,
            _ => manifest.arrival.ship_tangent_xz_meters[0] = -0.0,
        }
        assert!(FrozenStart::decode(&json(&manifest).unwrap()).is_err());
    }
    assert!(FrozenStart::decode(b"{}").is_err());
    assert!(FrozenStart::decode(&vec![b' '; MAX_MANIFEST_BYTES + 1]).is_err());
    let text = String::from_utf8(bytes).unwrap();
    assert!(
        FrozenStart::decode(
            text.replacen("\"version\":1", "\"version\":1,\"version\":1", 1)
                .as_bytes()
        )
        .is_err()
    );
    assert!(
        FrozenStart::decode(
            text.replacen("\"version\":1", "\"extra\":1,\"version\":1", 1)
                .as_bytes()
        )
        .is_err()
    );
}

#[test]
fn starting_system_arrival_surface_and_all_resource_types_accessible() {
    let start = permanent().unwrap();
    let m = start.manifest();
    let keys: std::collections::BTreeSet<_> = m
        .resources
        .outcrops
        .iter()
        .map(|s| s.material.key())
        .collect();
    assert_eq!(
        keys,
        [
            "iron-ore",
            "silicate-rock",
            "water-ice",
            "organic-biomass",
            "medicinal-compounds"
        ]
        .into_iter()
        .collect()
    );
    let player = start
        .surface_point(m.arrival.traveler_tangent_xz_meters)
        .unwrap();
    let ship = start
        .surface_point(m.arrival.ship_tangent_xz_meters)
        .unwrap();
    assert!((player[0] - ship[0]).abs() > m.arrival.ship_clearance_radius_meters);
    for site in &m.resources.outcrops {
        let point = start.surface_point(site.tangent_xz_meters).unwrap();
        assert_eq!(start.height_meters(point).unwrap(), 0.0);
        let walk = (point[0] - player[0]).hypot(point[2] - player[2]);
        assert!(walk <= m.arrival.resource_walk_limit_meters);
        assert!(site.initial_mass_kg >= 100.0);
        assert!(point[0].hypot(point[2]) > m.arrival.ship_clearance_radius_meters);
    }
    // Sample routes independently from publication's analytic clearance test.
    for site in &m.resources.outcrops {
        let mut from = m.arrival.traveler_tangent_xz_meters;
        let mut length = 0.0;
        for to in site
            .approach_waypoints_xz_meters
            .iter()
            .copied()
            .chain(std::iter::once(site.tangent_xz_meters))
        {
            length += (to[0] - from[0]).hypot(to[1] - from[1]);
            for step in 0..=100 {
                let t = f64::from(step) / 100.0;
                let xz = std::array::from_fn(|i| from[i] * (1.0 - t) + to[i] * t);
                assert!(xz[0].hypot(xz[1]) > m.arrival.ship_clearance_radius_meters);
                assert_eq!(
                    start
                        .height_meters(start.surface_point(xz).unwrap())
                        .unwrap(),
                    0.0
                );
            }
            from = to;
        }
        assert!(length <= m.arrival.resource_walk_limit_meters);
    }
}

#[test]
fn starting_system_rejects_blocked_and_unreachable_arrival_paths() {
    let start = permanent().unwrap();
    for change in 0..4 {
        let mut m = start.manifest().clone();
        match change {
            0 => m
                .resources
                .outcrops
                .iter_mut()
                .find(|s| s.material == Material::IronOre)
                .unwrap()
                .approach_waypoints_xz_meters
                .clear(),
            1 => m.arrival.resource_walk_limit_meters = 20.0,
            2 => m.arrival.traveler_tangent_xz_meters = [0.0, 0.0],
            _ => m.resources.outcrops[0].tangent_xz_meters = [90.0, 0.0],
        }
        assert!(validate_arrival(&m).is_err());
    }
}

#[test]
fn starting_system_single_surface_bounds_poles_and_invalid_coordinates() {
    let start = permanent().unwrap();
    let mut distinct = std::collections::BTreeSet::new();
    for x in -20..=20 {
        for z in -20..=20 {
            let h = start
                .height_meters([f64::from(x), 1.0, f64::from(z)])
                .unwrap();
            assert!((0.0..=1800.0).contains(&h));
            distinct.insert(h.to_bits());
        }
    }
    assert!(distinct.len() > 500);
    assert_eq!(start.height_meters([0.0, 1.0, 0.0]).unwrap(), 0.0);
    assert!(start.height_meters([0.0, -1.0, 0.0]).unwrap() >= 0.0);
    for bad in [[0.0; 3], [f64::NAN, 0.0, 0.0], [0.0, f64::INFINITY, 0.0]] {
        assert!(start.height_meters(bad).is_err());
    }
    assert!(start.surface_point([1001.0, 0.0]).is_err());
    assert!(start.surface_point([f64::NAN, 0.0]).is_err());
    // Overflow-safe and underflow-safe direction normalization.
    assert_eq!(start.height_meters([0.0, f64::MAX, 0.0]).unwrap(), 0.0);
    assert_eq!(
        start.height_meters([0.0, f64::MIN_POSITIVE, 0.0]).unwrap(),
        0.0
    );
}

#[test]
fn starting_system_visual_domain_cannot_change_physical_surface_or_deposits() {
    let start = permanent().unwrap();
    let mut changed = start.manifest().clone();
    changed.visual.seed += 1;
    changed.visual.ground_srgb = [0, 0, 0];
    let old = definition(start.manifest()).unwrap();
    let new = definition(&changed).unwrap();
    assert_ne!(old.sha256, new.sha256);
    for name in ["terrain", "resources"] {
        assert_eq!(
            old.domains.iter().find(|d| d.name == name),
            new.domains.iter().find(|d| d.name == name)
        );
    }
    assert_eq!(
        height(
            &changed.terrain,
            changed.physical.radius_meters,
            [0.2, 1.0, -0.4]
        )
        .unwrap(),
        start.height_meters([0.2, 1.0, -0.4]).unwrap()
    );
    assert_eq!(changed.resources, start.manifest().resources);
    let resource_domain = old.domains.iter().find(|d| d.name == "resources").unwrap();
    assert_eq!(resource_domain.dependencies, ["terrain"]);
}
