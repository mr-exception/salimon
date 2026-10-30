//! Canonical baseline collision residency, independent of rendering or player save deltas.
use super::GameplaySession;
use salimon_physics::static_boxes::{StaticBox, StaticField};
use std::sync::{Arc, LazyLock};
#[cfg(test)]
pub(crate) static FIELD: LazyLock<Arc<StaticField>> = LazyLock::new(|| {
    let boxes = salimon_world::canonical::ORGANISMS
        .objects()
        .iter()
        .flat_map(|o| o.parts.iter())
        .map(|p| StaticBox::new(p.bounds).expect("published physical part bound"))
        .collect();
    Arc::new(StaticField::new(boxes).expect("bounded canonical physical population"))
});
static HULL: LazyLock<StaticBox> = LazyLock::new(|| {
    let bounds = salimon_character::ship_installation_obstacles()
        .into_iter()
        .fold(
            [
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ],
            |a, b| {
                std::array::from_fn(|i| {
                    if i % 2 == 0 {
                        a[i].min(b[i])
                    } else {
                        a[i].max(b[i])
                    }
                })
            },
        );
    StaticBox::new(bounds).expect("authored ship solids")
});
impl GameplaySession {
    pub(super) fn sync_organism_collisions(&mut self) -> bool {
        let field = if self.mining.terrain == salimon_world::terrain::TerrainMode::Canonical
            || self.biology.fixture
        {
            self.organism_field()
        } else {
            Arc::new(StaticField::default())
        };
        self.mining.biological_obstacles = field.clone();
        let mut bounds = HULL.bounds();
        if !field.boxes().is_empty() {
            for part in self.construction_solids() {
                bounds = std::array::from_fn(|i| {
                    if i % 2 == 0 {
                        bounds[i].min(part[i])
                    } else {
                        bounds[i].max(part[i])
                    }
                });
            }
        }
        let hull = StaticBox::new(bounds).expect("validated authored and installed hull solids");
        self.install_organism_field(field, hull)
    }
    fn install_organism_field(&mut self, field: Arc<StaticField>, hull: StaticBox) -> bool {
        self.character.set_surface_obstacles(field.clone());
        if self.ship.set_surface_obstacles(field, hull).is_err() {
            self.ship.hold_for_surface_obstruction();
            return false;
        }
        true
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn unsafe_restored_overlap_holds_pose_without_removing_the_organism_or_spending_energy() {
        let mut session = super::GameplaySession::default();
        let before = session.ship.snapshot();
        let p = before.pose.position_meters;
        let collider = super::StaticBox::new(std::array::from_fn(|i| {
            p[i / 2] + if i % 2 == 0 { -2. } else { 2. }
        }))
        .unwrap();
        let field = std::sync::Arc::new(super::StaticField::new(vec![collider]).unwrap());
        assert!(!session.install_organism_field(field.clone(), *super::HULL));
        let after = session.ship.snapshot();
        assert_eq!(before.pose, after.pose);
        assert_eq!(before.energy_core, after.energy_core);
        assert_eq!(field.boxes().len(), 1);
        assert!(!after.engines_enabled);
    }
    #[test]
    fn canonical_population_and_collision_are_sterile_and_rebuild_identically() {
        let p = &salimon_world::canonical::ORGANISMS;
        assert!(p.objects().is_empty());
        assert!(super::FIELD.boxes().is_empty());
        let binding = p.binding();
        p.verify_binding(&binding).unwrap();
        assert!(super::HULL.bounds()[0] < super::HULL.bounds()[1]);
    }
}

#[cfg(test)]
fn fixture_population() -> salimon_world::environment::organism_placement::Population {
    use salimon_world::{
        environment::{
            climate::{Chemistry, ClimateDefinition, ClimateInputs, Condensable},
            ecology::{BiosphereDecision, BiosphereDefinition, BodyNature},
            organism_placement::Population,
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
    for seed in 17..81 {
        let b = BiosphereDefinition::new(
            salimon_universe::CatalogId::from_source("fixture", "physical-vegetation").unwrap(),
            BodyNature::Fictional,
            BiosphereDecision::Developed,
            climate.clone(),
            1,
            seed,
        )
        .unwrap();
        let population = Population::publish(&b, &[]).unwrap();
        if population
            .objects()
            .iter()
            .any(|o| o.specimen.form == salimon_world::environment::organisms::Form::Giant)
        {
            println!("controlled physical fixture seed={seed}");
            return population;
        }
    }
    panic!("bounded physical fixtures must contain a supported giant")
}
#[cfg(test)]
mod integration {
    use super::*;
    use salimon_character::{CharacterController, MovementInput, ShipFrame, SurfaceFrame};
    #[test]
    fn physical_population_live_character_and_ship_use_same_source_colliders() {
        let population = fixture_population();
        let object = population
            .objects()
            .iter()
            .find(|o| o.specimen.form == salimon_world::environment::organisms::Form::Giant)
            .expect("controlled physical fixture includes a giant form");
        let field = Arc::new(
            StaticField::new(
                object
                    .parts
                    .iter()
                    .map(|p| StaticBox::new(p.bounds).unwrap())
                    .collect(),
            )
            .unwrap(),
        );
        let origin = object.position_meters;
        let axes = object.axes;
        let surface = SurfaceFrame {
            query: None,
            body_center_meters: [0.; 3],
            radius_meters: salimon_math::length(origin),
        };
        let start = salimon_math::add(origin, salimon_math::scale(axes[0], -20.));
        let mut player = CharacterController::on_surface(surface, start).unwrap();
        player.set_surface_obstacles(field.clone());
        let ship_frame = ShipFrame {
            origin_meters: salimon_math::add(origin, salimon_math::scale(axes[0], 1000.)),
            axes,
        };
        player.advance_with_motion(
            std::time::Duration::from_secs(15),
            MovementInput {
                forward: true,
                ..Default::default()
            },
            (ship_frame, [0.; 3]),
            surface,
            false,
            true,
        );
        let eye = player.snapshot(ship_frame, surface).eye_position_meters;
        let progress = salimon_math::dot(salimon_math::sub(eye, origin), axes[0]);
        println!("character source={} stopped tangent={progress}", object.id);
        assert!(progress < 0.);
        let start = salimon_math::add(
            salimon_math::add(origin, salimon_math::scale(axes[0], -60.)),
            salimon_math::scale(axes[1], 10.),
        );
        let pose = salimon_ship::ShipPose {
            position_meters: start,
            orientation: [0., 0., 0., 1.],
        };
        let velocity = salimon_math::scale(axes[0], 120.);
        let mut ship =
            salimon_ship::ShipController::flying_with_velocity(pose, 0, velocity).unwrap();
        ship.set_surface_obstacles(
            field,
            StaticBox::new([-10., 10., -5., 5., -3., 3.]).unwrap(),
        )
        .unwrap();
        ship.advance(std::time::Duration::from_secs(1));
        let snapshot = ship.snapshot();
        let progress = salimon_math::dot(
            salimon_math::sub(snapshot.pose.position_meters, origin),
            axes[0],
        );
        println!("ship source={} stopped tangent={progress}", object.id);
        assert!(progress < 0.);
        assert_eq!(snapshot.velocity_meters_per_second, [0.; 3]);
        assert!(!snapshot.engines_enabled);
    }
    #[test]
    #[ignore = "export physical source identities/parts; set SALIMON_PHYSICAL_ORGANISMS_FILE"]
    fn physical_population_geometry_export() {
        let population = fixture_population();
        assert!(!population.objects().is_empty());
        let mut models = vec![];
        let giant = population
            .objects()
            .iter()
            .find(|o| o.specimen.form == salimon_world::environment::organisms::Form::Giant);
        let selected = giant
            .into_iter()
            .chain(
                population
                    .objects()
                    .iter()
                    .filter(|o| giant.is_none_or(|g| o.id != g.id)),
            )
            .take(12);
        for (index, o) in selected.enumerate() {
            let segments=o.specimen.parts.iter().zip(&o.parts).map(|(source,p)|serde_json::json!({"part_id":p.id,"start":p.start_meters,"end":p.end_meters,"r0":source.radius_start,"r1":source.radius_end,"color":o.specimen.color,"bounds":p.bounds})).collect::<Vec<_>>();
            models.push(serde_json::json!({"name":format!("physical-{index}-{:?}",o.specimen.form),"id":o.id,"population":population.identity(),"site":o.site_id,"origin":o.position_meters,"axes":o.axes,"segments":segments}));
        }
        let path = std::env::var("SALIMON_PHYSICAL_ORGANISMS_FILE").unwrap();
        std::fs::write(&path, serde_json::to_vec_pretty(&models).unwrap()).unwrap();
        std::fs::write(format!("{path}.binding"), population.binding()).unwrap();
        println!(
            "physical population={} objects={} first-capture-count={}",
            population.identity(),
            population.objects().len(),
            models.len()
        );
    }
}
