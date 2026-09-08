use super::*;
use crate::{
    WorldPosition, mining::MiningSession, resources::ResourceTransform,
    spherical_mining::MiningConfig,
};
fn pose() -> ResourceTransform {
    ResourceTransform::new(WorldPosition::new(0., 0., 0.), [0., 0., 0., 1.]).unwrap()
}
#[test]
fn detachment_transfers_one_same_part_atomically_and_stale_authority_cannot_duplicate() {
    let f = super::super::harvest_fixture::publish();
    let c = &f.catalog;
    let s = c
        .sources()
        .iter()
        .find(|s| s.method == Method::Detach)
        .unwrap();
    assert!(
        s.carriable(),
        "radius {} mass {}",
        s.shape.radius(),
        s.initial_mass_kg
    );
    let mut j = Journal::new(c);
    let mut world = MiningSession::default();
    let before = format!("{:?}", world.save_state());
    assert!(
        world
            .detach_biological(c, &mut j, &s.key, 1, pose())
            .is_err()
    );
    assert_eq!(before, format!("{:?}", world.save_state()));
    assert!(j.deltas().is_empty());
    let id = world
        .detach_biological(c, &mut j, &s.key, 0, pose())
        .unwrap();
    let object = *world.fragment(id).unwrap();
    assert_eq!(object.natural_shape(), Some(s.shape));
    assert_eq!(object.material().mass_kg(), s.initial_mass_kg);
    assert_eq!(world.carried_id(), Some(id));
    assert!(
        world
            .detach_biological(c, &mut j, &s.key, 0, pose())
            .is_err()
    );
    world.drop_carried(pose());
    assert!(
        world
            .detach_biological(c, &mut j, &s.key, 1, pose())
            .is_err()
    );
    assert_eq!(world.fragments().len(), 1);
    assert_eq!(j.remaining(s), 0.);
    let copy = j.clone();
    copy.validate(c).unwrap();
    let reopened = MiningSession::from_save(world.save_state()).unwrap();
    assert_eq!(reopened.fragment(id), Some(&object));
    let mut corrupt = j.clone();
    corrupt.deltas.push(corrupt.deltas[0].clone());
    assert!(corrupt.validate(c).is_err());
}
#[test]
fn embedded_material_reuses_sphere_rate_caps_interrupt_and_conserves_across_reopen() {
    let f = super::super::harvest_fixture::publish();
    let c = &f.catalog;
    let s = c
        .sources()
        .iter()
        .find(|s| s.method == Method::Embedded)
        .unwrap();
    let mut j = Journal::new(c);
    let mut w = MiningSession::default();
    let config = MiningConfig::new(0.05, 2.).unwrap();
    assert_eq!(
        w.mine_biological(
            c,
            &mut j,
            (&s.key, 0),
            std::time::Duration::ZERO,
            config,
            pose()
        )
        .unwrap(),
        0.
    );
    assert!(j.deltas().is_empty());
    for _ in 0..10000 {
        if j.remaining(s) == 0. {
            break;
        }
        let rev = j.revision(&s.key);
        w.mine_biological(
            c,
            &mut j,
            (&s.key, rev),
            std::time::Duration::from_millis(16),
            config,
            pose(),
        )
        .unwrap();
        let total: f64 = w.fragments().iter().map(|f| f.material().mass_kg()).sum();
        assert!((s.initial_mass_kg - j.remaining(s) - total).abs() < 1e-9);
        assert!(
            w.fragments()
                .iter()
                .all(|f| f.material().mass_kg() <= 0.05 + 1e-12)
        );
        if rev == 2 {
            w.interrupt_spherical();
            w = MiningSession::from_save(w.save_state()).unwrap();
        }
    }
    assert_eq!(j.remaining(s), 0.);
    assert!(w.attached_sphere_id().is_none());
    let count = w.fragments().len();
    let rev = j.revision(&s.key);
    assert!(
        w.mine_biological(
            c,
            &mut j,
            (&s.key, rev),
            std::time::Duration::from_millis(16),
            config,
            pose()
        )
        .is_err()
    );
    assert_eq!(count, w.fragments().len());
    assert!(
        w.fragments()
            .iter()
            .all(|f| f.material().resource() == ResourceId::MedicinalCompounds)
    );
}
#[test]
fn yield_is_explicit_bound_to_source_and_too_large_parts_reject_without_debit() {
    let f = super::super::harvest_fixture::publish();
    let empty = Catalog::publish(&f.population, CelestialBodyId::Moon, &[]).unwrap();
    assert!(empty.sources().is_empty());
    let giant = f
        .population
        .objects()
        .iter()
        .find(|o| o.specimen.form == super::super::organisms::Form::Giant)
        .unwrap();
    let part = giant.specimen.parts.last().unwrap();
    let c = Catalog::publish(
        &f.population,
        CelestialBodyId::Moon,
        &[Designation {
            key: Key {
                organism: giant.id.clone(),
                part: part.id,
            },
            method: Method::Detach,
            material: ResourceId::OrganicBiomass,
        }],
    )
    .unwrap();
    let s = &c.sources()[0];
    assert!(!s.carriable());
    let mut j = Journal::new(&c);
    let mut w = MiningSession::default();
    assert_eq!(
        w.detach_biological(&c, &mut j, &s.key, 0, pose()),
        Err(crate::mining::HarvestError::TooLarge)
    );
    assert!(j.deltas().is_empty() && w.fragments().is_empty());
    assert!(j.validate(&empty).is_err());
}
