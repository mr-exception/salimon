use std::time::Duration;

use salimon_world::mining::MiningSession;
use salimon_world::resource_distribution::default_resource_distribution;
use salimon_world::resource_generation::materialize_nearby_deposits;
use salimon_world::resources::DepositState;
use salimon_world::{CELESTIAL_BODIES, WorldPosition};

#[test]
fn generated_untouched_partial_and_depleted_deposits_survive_repeated_streaming() {
    for body in CELESTIAL_BODIES.iter().copied().skip(1) {
        let profile = default_resource_distribution(body.id);
        let query = |player: WorldPosition| {
            materialize_nearby_deposits(body, &profile, 123, None, player, 120.0).unwrap()
        };
        let home = body.center.translated([0.0, body.radius_meters, 0.0]);
        let away = body.center.translated([body.radius_meters, 0.0, 0.0]);
        let mut active = query(home);
        assert!(active.len() >= 3);
        let originals = [active[0], active[1], active[2]];
        let mut session = MiningSession::default();
        session.extract(&mut active[1].deposit, Duration::from_millis(500));
        session.extract(&mut active[2].deposit, Duration::from_secs(1000));
        let expected = [active[0], active[1], active[2]];
        assert_eq!(expected[0].deposit.state(), DepositState::Untouched);
        assert_eq!(expected[1].deposit.state(), DepositState::PartiallyMined);
        assert_eq!(expected[2].deposit.state(), DepositState::Depleted);
        let fragments = session.fragments().to_vec();
        let removed = session.extracted_mass_kg();
        for _ in 0..3 {
            drop(active);
            active = query(away);
            assert!(active.iter().all(|entry| {
                !originals
                    .iter()
                    .any(|old| old.deposit.id() == entry.deposit.id())
            }));
            session.apply_to(&mut active);
            drop(active);
            active = query(home);
            session.apply_to(&mut active);
            session.apply_to(&mut active); // Idempotent on an already restored list.
            for saved in expected {
                assert_eq!(
                    *active
                        .iter()
                        .find(|entry| entry.deposit.id() == saved.deposit.id())
                        .unwrap(),
                    saved
                );
            }
            assert_eq!(session.fragments(), fragments);
            assert_eq!(session.extracted_mass_kg(), removed);
        }
        // A new local world has no journal from the old session.
        let mut fresh = query(home);
        MiningSession::default().apply_to(&mut fresh);
        for original in originals {
            assert_eq!(
                *fresh
                    .iter()
                    .find(|entry| entry.deposit.id() == original.deposit.id())
                    .unwrap(),
                original
            );
        }
    }
}

#[test]
fn extraction_reconciles_fresh_or_stale_copies_before_emitting_mass() {
    let body = CELESTIAL_BODIES[3];
    let original = materialize_nearby_deposits(
        body,
        &default_resource_distribution(body.id),
        0,
        None,
        body.center.translated([0.0, body.radius_meters, 0.0]),
        120.0,
    )
    .unwrap()[0]
        .deposit;
    let mut session = MiningSession::default();
    let mut copy = original;
    assert_eq!(session.extract(&mut copy, Duration::from_millis(500)), 1.0);
    let mut stale = original;
    assert_eq!(session.extract(&mut stale, Duration::from_millis(500)), 1.0);
    assert_eq!(
        stale.remaining_mass_kg(),
        original.remaining_mass_kg() - 2.0
    );
    session.extract(&mut stale, Duration::from_secs(1000));
    let output = session.fragments().to_vec();
    let mut regenerated = original;
    assert_eq!(
        session.extract(&mut regenerated, Duration::from_secs(1000)),
        0.0
    );
    assert_eq!(regenerated.state(), DepositState::Depleted);
    assert_eq!(session.fragments(), output);
    assert!((session.extracted_mass_kg() - original.remaining_mass_kg()).abs() < 1e-10);
}
