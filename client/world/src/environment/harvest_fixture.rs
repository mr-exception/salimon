//! Explicit fictional organism/yield publication for controlled integration evidence.
//! Uses the existing representative Moon geometry, never canonical Vesper life.
use super::{
    climate::{Chemistry, ClimateDefinition, ClimateInputs, Condensable},
    ecology::{BiosphereDecision, BiosphereDefinition, BodyNature},
    harvest::{Catalog, Designation, Key, Method},
    organism_placement::Population,
    organisms::Form,
};
use crate::{CelestialBodyId, resources::ResourceId, terrain::TerrainMode};
pub struct Fixture {
    pub biosphere: BiosphereDefinition,
    pub population: Population,
    pub catalog: Catalog,
}
pub fn publish() -> Fixture {
    let terrain = TerrainMode::Representative
        .definition(CelestialBodyId::Moon)
        .expect("fixture terrain")
        .clone();
    let climate = ClimateDefinition::new(
        terrain.geology(),
        1,
        37,
        ClimateInputs {
            chemistry: Chemistry::Nitrogen,
            condensable: Condensable::Water,
            pressure_pascal: 101325.,
            scale_height_meters: 8000.,
            rotation_seconds: 86400.,
            year_seconds: 31557600.,
            axial_tilt_radians: 0.4,
            stellar_flux_w_m2: 1361.,
            albedo: 0.3,
            greenhouse_kelvin: 30.,
            condensable_fraction: 0.8,
        },
    )
    .expect("fixture climate");
    let biosphere = BiosphereDefinition::new(
        salimon_universe::CatalogId::from_source("fictional-fixture", "harvest-world")
            .expect("fixture ID"),
        BodyNature::Fictional,
        BiosphereDecision::Developed,
        climate,
        1,
        17,
    )
    .expect("explicit fictional life");
    let population = Population::publish(&biosphere, &[]).expect("stable physical population");
    // Explicit authoring: this one fixture species has biomass appendages and a
    // medicinal reserve in a separate part. No general organism-to-medicine rule.
    let object = population
        .objects()
        .iter()
        .find(|o| o.specimen.form == Form::Frond)
        .expect("published fixture frond");
    let designations = [
        Designation {
            key: Key {
                organism: object.id.clone(),
                part: 1,
            },
            method: Method::Detach,
            material: ResourceId::OrganicBiomass,
        },
        Designation {
            key: Key {
                organism: object.id.clone(),
                part: 2,
            },
            method: Method::Embedded,
            material: ResourceId::MedicinalCompounds,
        },
    ];
    let catalog = Catalog::publish(&population, CelestialBodyId::Moon, &designations)
        .expect("explicit yields");
    Fixture {
        biosphere,
        population,
        catalog,
    }
}
