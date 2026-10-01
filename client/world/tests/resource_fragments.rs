use salimon_world::mining::MiningSession;
use salimon_world::resource_fragments::{FRAGMENT_MAX_MASS_KG, side_meters};
use salimon_world::resources::{
    DepositId, DepositState, RESOURCE_CATALOG, RawMaterial, ResourceDeposit,
};
use salimon_world::{CELESTIAL_BODIES, CelestialBodyId};
use std::time::Duration;

#[test]
fn physical_output_conserves_every_material_for_small_and_large_steps() {
    for material in RESOURCE_CATALOG {
        for frames in [1, 50, 100] {
            let body = CELESTIAL_BODIES
                .iter()
                .find(|body| body.id == CelestialBodyId::Earth)
                .unwrap();
            let mut deposit = ResourceDeposit::new(
                DepositId {
                    body: body.id,
                    local: 1,
                },
                RawMaterial::new(material.id, 5.25).unwrap(),
                body.center.translated([0.0, 0.0, body.radius_meters]),
                5.25,
            )
            .unwrap();
            let mut session = MiningSession::default();
            assert_eq!(session.extract(&mut deposit, Duration::ZERO), 0.0);
            assert!(session.fragments().is_empty());
            for _ in 0..frames {
                session.extract(
                    &mut deposit,
                    Duration::from_secs_f64(1.0 / f64::from(frames)),
                );
                let sum: f64 = session
                    .fragments()
                    .iter()
                    .map(|piece| piece.material().mass_kg())
                    .sum();
                assert!((sum + deposit.remaining_mass_kg() - 5.25).abs() < 1e-12);
                assert!((sum - session.extracted_mass_kg()).abs() < 1e-12);
            }
            let first = session.fragments()[0];
            session.extract(&mut deposit, Duration::from_secs(100));
            assert_eq!(deposit.state(), DepositState::Depleted);
            assert_eq!(session.fragments().len(), 3);
            assert_eq!(session.fragments()[0].id(), first.id());
            assert_eq!(session.fragments()[0].transform(), first.transform());
            let sum: f64 = session
                .fragments()
                .iter()
                .map(|piece| piece.material().mass_kg())
                .sum();
            assert!((sum - 5.25).abs() < 1e-12);
            for (index, piece) in session.fragments().iter().enumerate() {
                assert_eq!(piece.id().0, index as u64 + 1);
                assert_eq!(piece.source(), deposit.id());
                assert_eq!(piece.material().resource(), material.id);
                assert!(
                    piece.material().mass_kg() > 0.0
                        && piece.material().mass_kg() <= FRAGMENT_MAX_MASS_KG
                );
                assert!((side_meters(*piece).powi(3) - piece.material().volume_m3()).abs() < 1e-15);
                let altitude = piece
                    .transform()
                    .position()
                    .offset_from(body.center)
                    .iter()
                    .map(|v| v * v)
                    .sum::<f64>()
                    .sqrt()
                    - body.radius_meters;
                assert!(
                    altitude > side_meters(*piece) * 0.5,
                    "fragment must be above the surface"
                );
            }
            let before = session.fragments().to_vec();
            assert_eq!(session.extract(&mut deposit, Duration::from_secs(100)), 0.0);
            assert_eq!(session.fragments(), before);
        }
    }
}

#[test]
fn separate_deposits_have_unique_ids_and_requery_does_not_duplicate_output() {
    let body = CELESTIAL_BODIES[3];
    let make = |id| {
        ResourceDeposit::new(
            DepositId {
                body: body.id,
                local: id,
            },
            RawMaterial::new(RESOURCE_CATALOG[0].id, 5.0).unwrap(),
            body.center.translated([0.0, body.radius_meters, 0.0]),
            5.0,
        )
        .unwrap()
    };
    let mut a = make(1);
    let mut b = make(2);
    let mut session = MiningSession::default();
    session.extract(&mut a, Duration::from_millis(250));
    session.extract(&mut b, Duration::from_millis(250));
    session.extract(&mut a, Duration::from_millis(250));
    assert_eq!(session.fragments().len(), 2);
    assert_ne!(session.fragments()[0].id(), session.fragments()[1].id());
    assert_eq!(session.fragments()[0].material().mass_kg(), 1.0);
    assert_eq!(session.fragments()[1].material().mass_kg(), 0.5);
    let before = session.fragments().to_vec();
    let mut reloaded = [salimon_world::resource_generation::SurfaceDeposit {
        deposit: make(1),
        body_local_position_meters: [0.0, body.radius_meters, 0.0],
        bounds_radius_meters: 0.2,
    }];
    session.apply_to(&mut reloaded);
    assert_eq!(reloaded[0].deposit.remaining_mass_kg(), 4.0);
    assert_eq!(session.fragments(), before);
}

#[test]
fn collected_piece_preserves_identity_mass_and_pose_during_further_extraction() {
    use salimon_world::resources::{FragmentId, ResourceTransform};
    let body = CELESTIAL_BODIES[3];
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: body.id,
            local: 1,
        },
        RawMaterial::new(RESOURCE_CATALOG[0].id, 5.0).unwrap(),
        body.center.translated([0.0, body.radius_meters, 0.0]),
        5.0,
    )
    .unwrap();
    let mut session = MiningSession::default();
    session.extract(&mut deposit, Duration::from_millis(250));
    let original = session.fragments()[0];
    assert!(!session.pick_up(FragmentId(999)));
    assert!(session.pick_up(original.id()));
    let held = ResourceTransform::new(
        original.transform().position().translated([1.0, 2.0, 3.0]),
        original.transform().orientation_xyzw(),
    )
    .unwrap();
    session.move_carried(held);
    session.extract(&mut deposit, Duration::from_millis(250));
    assert_eq!(session.fragments().len(), 2);
    assert!(!session.pick_up(session.fragments()[1].id()));
    assert_eq!(session.carried_id(), Some(original.id()));
    assert_eq!(session.fragments()[0].material(), original.material());
    assert_eq!(session.fragments()[0].source(), original.source());
    assert_eq!(session.fragments()[0].transform(), held);
    assert!(session.drop_carried(original.transform()));
    assert!(!session.drop_carried(held));
    session.extract(&mut deposit, Duration::from_secs(100));
    assert_eq!(session.fragments()[0], original);
    let sum: f64 = session
        .fragments()
        .iter()
        .map(|p| p.material().mass_kg())
        .sum();
    assert_eq!(sum, 5.0);
    assert!(session.pick_up(session.fragments()[1].id()));
}
