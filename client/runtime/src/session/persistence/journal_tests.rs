use salimon_world::{
    CelestialBodyId, WorldPosition,
    mining::MiningSession,
    resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId},
};
use std::time::Duration;
#[test]
fn depletion_survives_streaming_and_consumed_ids_never_reallocate() {
    let make = || {
        ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 123,
            },
            RawMaterial::new(ResourceId::IronOre, 20.).unwrap(),
            WorldPosition::new(0., 0., 0.),
            20.,
        )
        .unwrap()
    };
    let mut world = MiningSession::default();
    let mut deposit = make();
    world.extract(&mut deposit, Duration::from_millis(500));
    let id = world.fragments()[0].id();
    assert!(world.remove_fragment(id));
    let saved = world.save_state();
    let mut restored = MiningSession::from_save(
        serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap(),
    )
    .unwrap();
    let mut streamed = make();
    // Reconcile through the ordinary extraction path, including a zero tick.
    restored.extract(&mut streamed, Duration::ZERO);
    assert_eq!(streamed.remaining_mass_kg(), deposit.remaining_mass_kg());
    restored.extract(&mut streamed, Duration::from_millis(500));
    assert!(restored.fragments()[0].id().0 > id.0);
    assert_eq!(restored.save_state().extracted_mass_kg, 2.);
}
