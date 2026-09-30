use salimon_world::mining::{MiningRay, aimed_deposit, extract};
use salimon_world::resource_generation::SurfaceDeposit;
use salimon_world::resources::{DepositId, DepositState, RawMaterial, ResourceDeposit, ResourceId};
use salimon_world::{CELESTIAL_BODIES, CelestialBodyId, WorldPosition};
use std::time::Duration;

fn entry(center: WorldPosition, id: u64) -> SurfaceDeposit {
    SurfaceDeposit {
        deposit: ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: id,
            },
            RawMaterial::new(ResourceId::IronOre, 10.0).unwrap(),
            center,
            10.0,
        )
        .unwrap(),
        body_local_position_meters: [0.0; 3],
        bounds_radius_meters: 0.2,
    }
}

#[test]
fn targeting_rejects_misses_range_obstruction_and_depletion() {
    let origin = WorldPosition::new(0.0, 0.0, 0.0);
    let ray = MiningRay::new(origin, origin.translated([1.0, 0.0, 0.0])).unwrap();
    let near = entry(origin.translated([3.0, 0.0, 0.0]), 1);
    let far = entry(origin.translated([4.3, 0.0, 0.0]), 2);
    let miss = entry(origin.translated([3.0, 1.0, 0.0]), 3);
    let behind = entry(origin.translated([-1.0, 0.0, 0.0]), 4);
    assert_eq!(
        aimed_deposit(ray, &[far, miss, behind, near], None)
            .unwrap()
            .id,
        near.deposit.id()
    );
    assert!(aimed_deposit(ray, &[far, miss, behind], None).is_none());
    assert!(aimed_deposit(ray, &[near], Some(1.0)).is_none());
    assert!(aimed_deposit(ray, &[near], Some(4.0)).is_some());
    let mut depleted = near;
    depleted.deposit.set_remaining_mass_kg(0.0).unwrap();
    assert!(aimed_deposit(ray, &[depleted], None).is_none());
    assert!(MiningRay::new(origin, origin).is_none());
    assert!(MiningRay::new(origin, WorldPosition::new(f64::NAN, 0.0, 0.0)).is_none());
}

#[test]
fn planet_surface_blocks_buried_target_at_large_coordinates() {
    let earth = CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Earth)
        .unwrap();
    let anchor = earth.center.translated([0.0, 0.0, earth.radius_meters]);
    let eye = anchor.translated([0.0, 0.0, 1.75]);
    let ray = MiningRay::new(eye, anchor).unwrap();
    assert!(aimed_deposit(ray, &[entry(anchor, 1)], None).is_some());
    assert!(aimed_deposit(ray, &[entry(anchor.translated([0.0, 0.0, -1.0]), 2)], None).is_none());
}

#[test]
fn extraction_is_time_based_conservative_and_cannot_overdraw() {
    let original = entry(WorldPosition::new(0.0, 0.0, 0.0), 1).deposit;
    let mut a = original;
    let mut b = original;
    let mut c = original;
    extract(&mut a, Duration::from_secs(1));
    for _ in 0..50 {
        extract(&mut b, Duration::from_millis(20));
    }
    for _ in 0..100 {
        extract(&mut c, Duration::from_millis(10));
    }
    assert!((a.remaining_mass_kg() - b.remaining_mass_kg()).abs() < 1e-10);
    assert!((a.remaining_mass_kg() - c.remaining_mass_kg()).abs() < 1e-10);
    assert_eq!(a.remaining_mass_kg(), 8.0);
    assert_eq!(extract(&mut a, Duration::ZERO), 0.0);
    assert_eq!(extract(&mut a, Duration::from_secs(100)), 8.0);
    assert_eq!(a.state(), DepositState::Depleted);
    assert_eq!(extract(&mut a, Duration::from_secs(100)), 0.0);
    assert_eq!(a.id(), original.id());
    assert_eq!(a.material(), original.material());
}

#[test]
fn session_deltas_restore_the_same_deposit_without_inventory_credit() {
    let original = entry(WorldPosition::new(0.0, 0.0, 0.0), 1);
    let mut changed = original;
    let mut session = salimon_world::mining::MiningSession::default();
    assert_eq!(
        session.extract(&mut changed.deposit, Duration::from_secs(1)),
        2.0
    );
    let mut rematerialized = [original];
    session.apply_to(&mut rematerialized);
    assert_eq!(rematerialized[0].deposit.remaining_mass_kg(), 8.0);
    assert_eq!(session.extracted_mass_kg(), 2.0);
}
