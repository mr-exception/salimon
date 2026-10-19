use salimon_world::mining::MiningSession;
use salimon_world::resource_distribution::{
    BodyResourceDistribution, default_resource_distribution,
};
use salimon_world::resource_generation::materialize_nearby_deposits;
use salimon_world::resources::*;
use salimon_world::spherical_mining::MiningConfig;
use salimon_world::{CELESTIAL_BODIES, CelestialBodyId};
use std::time::Duration;

#[test]
fn feedstocks_preserve_mineral_generation_and_have_real_session_lifecycles() {
    let body = *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Earth)
        .unwrap();
    let profile = default_resource_distribution(body.id);
    let minerals = BodyResourceDistribution::new(
        body.id,
        profile.seed(),
        profile
            .generation_inputs(0, None)
            .iter()
            .filter(|i| !i.distribution.resource().is_consumable_feedstock())
            .map(|i| i.distribution)
            .collect(),
        vec![],
    )
    .unwrap();
    let player = body.center.translated([0.0, 0.0, body.radius_meters]);
    let query = |p: &BodyResourceDistribution| {
        materialize_nearby_deposits(body, p, 123, None, player, 150.0).unwrap()
    };
    let all = query(&profile);
    assert_eq!(all, query(&profile));
    assert_eq!(
        all.iter()
            .filter(|d| !d.deposit.material().resource().is_consumable_feedstock())
            .copied()
            .collect::<Vec<_>>(),
        query(&minerals)
    );
    for id in [ResourceId::OrganicBiomass, ResourceId::MedicinalCompounds] {
        let original = all
            .iter()
            .find(|d| d.deposit.material().resource() == id)
            .unwrap()
            .deposit;
        let mut deposit = original;
        let mut session = MiningSession::default();
        let pose = ResourceTransform::new(player, [0.0, 0.0, 0.0, 1.0]).unwrap();
        for _ in 0..500 {
            session.tick_spherical(
                true,
                Some(&mut deposit),
                Duration::from_millis(100),
                MiningConfig::default(),
                pose,
            );
        }
        session.interrupt_spherical();
        assert_eq!(deposit.remaining_mass_kg(), 0.0);
        assert!(
            (session
                .fragments()
                .iter()
                .map(|f| f.material().mass_kg())
                .sum::<f64>()
                - original.material().mass_kg())
            .abs()
                < 1e-9
        );
        assert!(
            session
                .fragments()
                .iter()
                .all(|f| f.material().resource() == id)
        );
        let piece = session.fragments()[0];
        assert!(session.pick_up(piece.id()));
        assert_eq!(session.carried_id(), Some(piece.id()));
        let mut reloaded = original;
        session.tick_spherical(
            false,
            Some(&mut reloaded),
            Duration::ZERO,
            MiningConfig::default(),
            pose,
        );
        assert_eq!(reloaded.remaining_mass_kg(), 0.0);
        assert_eq!(session.fragment(piece.id()), Some(&piece));
        assert!(session.remove_fragment(piece.id()));
        assert_eq!(session.carried_id(), None);
        assert!(!session.remove_fragment(piece.id()));
    }
}
