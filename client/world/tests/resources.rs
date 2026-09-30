use salimon_world::resources::*;
use salimon_world::{CelestialBodyId, WorldPosition};
use std::collections::HashSet;

const ORIGIN: WorldPosition = WorldPosition::new(0.0, 0.0, 0.0);
const DEPOSIT: DepositId = DepositId {
    body: CelestialBodyId::Moon,
    local: 42,
};

#[test]
fn catalog_keys_are_stable_unique_and_round_trip() {
    assert_eq!(ResourceId::IronOre.key(), "iron-ore");
    assert_eq!(ResourceId::SilicateRock.key(), "silicate-rock");
    assert_eq!(ResourceId::WaterIce.key(), "water-ice");
    let keys: HashSet<_> = RESOURCE_CATALOG
        .iter()
        .map(|entry| entry.id.key())
        .collect();
    let ids: HashSet<_> = RESOURCE_CATALOG.iter().map(|entry| entry.id).collect();
    assert_eq!(keys.len(), 3);
    assert_eq!(ids.len(), 3);
    for entry in RESOURCE_CATALOG {
        assert_eq!(ResourceId::from_key(entry.id.key()), Some(entry.id));
        assert_eq!(entry.id.definition(), entry);
        assert!(!entry.name.is_empty());
        assert!(entry.density_kg_per_m3.is_finite() && entry.density_kg_per_m3 > 0.0);
        let material = RawMaterial::new(entry.id, entry.density_kg_per_m3).unwrap();
        assert_eq!(material.volume_m3(), 1.0);
    }
    assert_eq!(ResourceId::from_key("unknown"), None);
    assert_eq!(ResourceId::from_key("Iron ore"), None);
}

#[test]
fn invalid_quantities_never_enter_material_or_deposit_state() {
    for mass in [-1.0, 0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            RawMaterial::new(ResourceId::IronOre, mass),
            Err(ResourceError::InvalidMass)
        );
    }
    assert_eq!(
        RawMaterial::new(ResourceId::IronOre, f64::from_bits(1)),
        Err(ResourceError::InvalidVolume)
    );
    assert!(RawMaterial::new(ResourceId::IronOre, f64::MAX).is_ok());
    let material = RawMaterial::new(ResourceId::IronOre, 10.0).unwrap();
    for remaining in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 10.001] {
        assert!(ResourceDeposit::new(DEPOSIT, material, ORIGIN, remaining).is_err());
    }
}

#[test]
fn deposit_transitions_are_bounded_and_preserve_identity() {
    let material = RawMaterial::new(ResourceId::WaterIce, 10.0).unwrap();
    let mut deposit = ResourceDeposit::new(DEPOSIT, material, ORIGIN, 10.0).unwrap();
    assert_eq!(deposit.state(), DepositState::Untouched);
    deposit.set_remaining_mass_kg(4.0).unwrap();
    assert_eq!(deposit.state(), DepositState::PartiallyMined);
    assert_eq!(deposit.remaining_volume_m3(), 4.0 / 920.0);
    let previous = deposit;
    for remaining in [-1.0, f64::NAN, f64::INFINITY, 5.0, 11.0] {
        assert!(deposit.set_remaining_mass_kg(remaining).is_err());
        assert_eq!(deposit, previous);
    }
    deposit.set_remaining_mass_kg(0.0).unwrap();
    assert_eq!(deposit.state(), DepositState::Depleted);
    assert_eq!(deposit.remaining_volume_m3(), 0.0);
    assert_eq!(deposit.material(), material);
    assert_eq!(deposit.id(), DEPOSIT);
    assert_eq!(deposit.position(), ORIGIN);
    assert!(deposit.set_remaining_mass_kg(1.0).is_err());
    deposit.set_remaining_mass_kg(0.0).unwrap();
    assert_eq!(
        ResourceDeposit::new(DEPOSIT, material, ORIGIN, 4.0)
            .unwrap()
            .state(),
        DepositState::PartiallyMined
    );
    assert_eq!(
        ResourceDeposit::new(DEPOSIT, material, ORIGIN, 0.0)
            .unwrap()
            .state(),
        DepositState::Depleted
    );
}

#[test]
fn transforms_reject_nonfinite_positions_and_invalid_rotations() {
    let material = RawMaterial::new(ResourceId::IronOre, 10.0).unwrap();
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let position = WorldPosition::new(value, 0.0, 0.0);
        assert!(ResourceTransform::new(position, [0.0, 0.0, 0.0, 1.0]).is_err());
        assert!(ResourceDeposit::new(DEPOSIT, material, position, 10.0).is_err());
    }
    for rotation in [
        [0.0; 4],
        [1.0; 4],
        [f64::NAN; 4],
        [f64::INFINITY; 4],
        [f64::MAX; 4],
    ] {
        assert_eq!(
            ResourceTransform::new(ORIGIN, rotation),
            Err(ResourceError::InvalidTransform)
        );
    }
    assert!(ResourceTransform::new(ORIGIN, [0.5; 4]).is_ok());
}

#[test]
fn moving_a_fragment_preserves_mass_volume_material_and_provenance() {
    let material = RawMaterial::new(ResourceId::SilicateRock, 2.7).unwrap();
    let pose = ResourceTransform::new(ORIGIN, [0.0, 0.0, 0.0, 1.0]).unwrap();
    let mut fragment = ResourceFragment::new(FragmentId(7), DEPOSIT, material, pose);
    let new_pose = ResourceTransform::new(WorldPosition::new(1e12, 2.0, 3.0), [0.5; 4]).unwrap();
    fragment.set_transform(new_pose);
    assert_eq!(fragment.id(), FragmentId(7));
    assert_eq!(fragment.source(), DEPOSIT);
    assert_eq!(fragment.material(), material);
    assert_eq!(fragment.transform(), new_pose);
    assert_eq!(fragment.transform().orientation_xyzw(), [0.5; 4]);
    assert_eq!(
        fragment.transform().position(),
        WorldPosition::new(1e12, 2.0, 3.0)
    );
    assert_eq!(material.resource(), ResourceId::SilicateRock);
    assert_eq!(material.mass_kg(), 2.7);
    assert_eq!(material.volume_m3(), 0.001);
    assert_ne!(
        DEPOSIT,
        DepositId {
            body: CelestialBodyId::Earth,
            local: 42
        }
    );
}
