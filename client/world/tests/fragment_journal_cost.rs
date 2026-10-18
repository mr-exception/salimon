//! Isolated no-op pose/journal attribution, separate from solver and mass adapters.
use salimon_world::{
    CelestialBodyId, WorldPosition,
    mining::MiningSession,
    resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform},
    spherical_mining::{MiningConfig, SPHERE_TICK},
};
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "release CPU no-op pose/journal attribution; never native FPS"]
fn fragment_journal_cost() {
    let quiet = std::env::var_os("SALIMON_EXPECT_QUIET_JOURNAL").is_some();
    for active in [500, 1000] {
        for inactive in [0, 10_000] {
            let mut session = MiningSession::default();
            for i in 0..active + inactive {
                let pose = ResourceTransform::new(
                    WorldPosition::new(i as f64, 0.0, 0.0),
                    [0.0, 0.0, 0.0, 1.0],
                )
                .unwrap();
                let mut deposit = ResourceDeposit::new(
                    DepositId {
                        body: CelestialBodyId::Earth,
                        local: i as u64,
                    },
                    RawMaterial::new(ResourceId::IronOre, 0.032).unwrap(),
                    pose.position(),
                    0.032,
                )
                .unwrap();
                let id = session
                    .tick_spherical(
                        true,
                        Some(&mut deposit),
                        SPHERE_TICK,
                        MiningConfig::default(),
                        pose,
                    )
                    .unwrap();
                let piece = *session.fragments().last().unwrap();
                assert_eq!(piece.id(), id);
                assert_eq!(piece.material().mass_kg(), 0.032);
                let radius = (3.0 * 0.032 / (4.0 * std::f64::consts::PI * 3500.0)).cbrt();
                assert!(
                    (salimon_world::resource_size::fragment_contact_radius_meters(
                        piece.material()
                    ) - radius)
                        .abs()
                        < 1e-14
                );
            }
            assert_eq!(session.fragments().len(), active + inactive);
            session.drain_fragment_changes().for_each(drop);
            let query = session.fragments()[inactive..].to_vec();
            for sample in 0..20 {
                let now = Instant::now();
                for p in &query {
                    assert!(session.move_loose(black_box(p.id()), black_box(p.transform())));
                }
                let write_ns = now.elapsed().as_nanos();
                let now = Instant::now();
                let events = session.drain_fragment_changes().count();
                let drain_ns = now.elapsed().as_nanos();
                assert_eq!(events, if quiet { 0 } else { active });
                println!(
                    "JOURNAL_COST active={active} inactive={inactive} sample={sample} write_ns={write_ns} drain_ns={drain_ns} events={events}"
                );
            }
        }
    }
}
