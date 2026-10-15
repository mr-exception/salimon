use salimon_world::resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId};
use salimon_world::spherical_mining::*;
use salimon_world::{CelestialBodyId, WorldPosition};
use std::time::Duration;

fn deposit(local: u64, mass: f64) -> ResourceDeposit {
    ResourceDeposit::new(
        DepositId {
            body: CelestialBodyId::Earth,
            local,
        },
        RawMaterial::new(ResourceId::IronOre, mass).unwrap(),
        WorldPosition::new(0.0, 0.0, 0.0),
        mass,
    )
    .unwrap()
}

#[test]
fn fixed_tick_conserves_cap_cycles_and_final_partial_sphere() {
    let mut source = deposit(1, 23.5);
    let mut mining = SphericalMining::default();
    let mut released = Vec::new();
    for _ in 0..1000 {
        let result = mining.tick(
            true,
            Some(&mut source),
            SPHERE_TICK,
            MiningConfig::default(),
        );
        if let Some(release) = result.released {
            released.push(release);
        }
        let total = released
            .iter()
            .map(|s| s.sphere.material.mass_kg())
            .sum::<f64>()
            + mining.attached().map_or(0.0, |s| s.material.mass_kg());
        assert!((23.5 - source.remaining_mass_kg() - total).abs() < 1e-9);
        assert!(
            mining
                .attached()
                .is_none_or(|s| s.material.mass_kg() <= 10.0)
        );
    }
    assert_eq!(source.remaining_mass_kg(), 0.0);
    assert_eq!(released.len(), 3);
    assert!((released[2].sphere.material.mass_kg() - 3.5).abs() < 1e-9);
    assert_eq!(released[2].reason, ReleaseReason::Depleted);
}

#[test]
fn release_precedence_switching_interrupt_and_zero_time() {
    let mut a = deposit(1, 20.0);
    let mut b = deposit(2, 20.0);
    let mut mining = SphericalMining::default();
    mining.tick(true, Some(&mut a), SPHERE_TICK, MiningConfig::default());
    let mass = mining.attached().unwrap().material.mass_kg();
    assert_eq!(
        mining.tick(true, Some(&mut a), Duration::ZERO, MiningConfig::default()),
        MiningStep::default()
    );
    assert_eq!(
        mining
            .tick(true, Some(&mut b), SPHERE_TICK, MiningConfig::default())
            .released
            .unwrap()
            .reason,
        ReleaseReason::TargetChanged
    );
    assert_eq!(b.remaining_mass_kg(), 20.0);
    mining.tick(true, Some(&mut b), SPHERE_TICK, MiningConfig::default());
    assert_eq!(
        mining
            .tick(false, None, SPHERE_TICK, MiningConfig::default())
            .released
            .unwrap()
            .reason,
        ReleaseReason::InputReleased
    );
    mining.tick(true, Some(&mut a), SPHERE_TICK, MiningConfig::default());
    let result = mining.tick(true, None, SPHERE_TICK, MiningConfig::default());
    assert_eq!(result.released.unwrap().sphere.material.mass_kg(), mass);
    assert_eq!(result.released.unwrap().reason, ReleaseReason::AimLost);
    mining.tick(true, Some(&mut a), SPHERE_TICK, MiningConfig::default());
    assert_eq!(
        mining.interrupt().released.unwrap().reason,
        ReleaseReason::Interrupted
    );
    assert!(mining.interrupt().released.is_none());
}

#[test]
fn bounded_work_tiny_mass_radius_and_validated_tool_upgrades() {
    for (cap, rate) in [(f64::NAN, 1.0), (0.0, 1.0), (10.0, f64::INFINITY)] {
        assert!(MiningConfig::new(cap, rate).is_err());
    }
    let mut source = deposit(1, 0.00000000000001);
    let mut mining = SphericalMining::default();
    let release = mining
        .tick(
            true,
            Some(&mut source),
            SPHERE_TICK,
            MiningConfig::default(),
        )
        .released
        .unwrap();
    assert_eq!(source.remaining_mass_kg(), 0.0);
    assert!(release.sphere.material.mass_kg() > 0.0);
    assert_eq!(release.sphere.radius_meters(), MIN_SPHERE_RADIUS_METERS);
    for resource in [
        ResourceId::IronOre,
        ResourceId::SilicateRock,
        ResourceId::WaterIce,
    ] {
        let material = RawMaterial::new(resource, 10.0).unwrap();
        let sphere = GrowingSphere {
            source: source.id(),
            material,
        };
        let volume = 4.0 / 3.0 * std::f64::consts::PI * sphere.radius_meters().powi(3);
        assert!((volume - material.volume_m3()).abs() < 1e-12);
    }
    let config = MiningConfig::new(0.001, 2.0).unwrap();
    let mut source = deposit(3, 100.0);
    let result = mining.tick(true, Some(&mut source), Duration::MAX, config);
    assert!(result.added_kg <= config.cap_kg());
    assert!(result.released.is_some());
    assert!(mining.attached().is_none());
}
