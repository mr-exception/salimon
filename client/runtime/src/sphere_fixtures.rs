//! Checked test-only construction through the default production sphere lifecycle.
use salimon_world::{
    CelestialBodyId,
    mining::MiningSession,
    resources::{
        DepositId, FragmentId, RawMaterial, ResourceDeposit, ResourceFragment, ResourceId,
        ResourceTransform,
    },
    spherical_mining::{MiningConfig, SPHERE_TICK},
};

pub(crate) fn assert_sphere(piece: ResourceFragment, resource: ResourceId, mass: f64) {
    assert_eq!(piece.material().resource(), resource);
    // Production debits representable deposit differences (canonical 1e-9 kg budget).
    assert!((piece.material().mass_kg() - mass).abs() < 1e-9);
    let expected = (3.0 * mass
        / (4.0 * std::f64::consts::PI * resource.definition().density_kg_per_m3))
        .cbrt()
        .clamp(0.005, 2.0);
    assert!((crate::fragment_physics::sphere_radius(piece) - expected).abs() < 1e-12);
}

pub(crate) fn release_one(
    session: &mut MiningSession,
    deposit: &mut ResourceDeposit,
    pose: ResourceTransform,
) -> FragmentId {
    let count = session.fragments().len();
    let expected_mass = deposit.remaining_mass_kg().min(10.0);
    assert!(expected_mass > 0.0);
    let mut attached = None;
    for _ in 0..400 {
        let released = session.tick_spherical(
            true,
            Some(deposit),
            SPHERE_TICK,
            MiningConfig::default(),
            pose,
        );
        let id = released
            .or(session.attached_sphere_id())
            .expect("physical reserved sphere");
        assert_eq!(
            *attached.get_or_insert(id),
            id,
            "growth retains its identity"
        );
        assert_eq!(session.fragments().len(), count + 1);
        if released.is_some() {
            let piece = *session.fragments().last().unwrap();
            assert_eq!(piece.id(), id);
            assert_eq!(piece.source(), deposit.id());
            assert_sphere(piece, deposit.material().resource(), expected_mass);
            assert_eq!(session.attached_sphere_id(), None);
            return id;
        }
    }
    panic!("default spherical fixture did not release within 400 ticks");
}

/// Append-only fixture builder: use a fresh session or keep prior fragments.
pub(crate) fn sphere(
    session: &mut MiningSession,
    resource: ResourceId,
    mass: f64,
    pose: ResourceTransform,
) -> FragmentId {
    assert!(mass > 0.0 && mass <= 10.0);
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: CelestialBodyId::Earth,
            local: 303_000 + session.fragments().len() as u64,
        },
        RawMaterial::new(resource, mass).unwrap(),
        pose.position(),
        mass,
    )
    .unwrap();
    let id = release_one(session, &mut deposit, pose);
    assert!(deposit.remaining_mass_kg() < 1e-9);
    id
}

#[test]
fn checked_fixtures_cover_residuals_materials_multiple_releases_and_one_object_carrying() {
    use salimon_world::WorldPosition;
    let pose =
        ResourceTransform::new(WorldPosition::new(0.0, 0.0, 0.0), [0.0, 0.0, 0.0, 1.0]).unwrap();
    for resource in [
        ResourceId::IronOre,
        ResourceId::SilicateRock,
        ResourceId::WaterIce,
    ] {
        for mass in [1e-9, 0.000_003, 0.032, 2.0, 10.0] {
            let mut session = MiningSession::default();
            let id = sphere(&mut session, resource, mass, pose);
            assert_eq!(session.fragments().len(), 1);
            assert!(session.pick_up(id));
            assert_eq!(session.carried_id(), Some(id));
        }
        let mut session = MiningSession::default();
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 303,
            },
            RawMaterial::new(resource, 20.25).unwrap(),
            pose.position(),
            20.25,
        )
        .unwrap();
        let mut ids = Vec::new();
        for mass in [10.0, 10.0, 0.25] {
            let id = release_one(&mut session, &mut deposit, pose);
            assert_sphere(*session.fragments().last().unwrap(), resource, mass);
            assert!(!ids.contains(&id));
            ids.push(id);
        }
        assert_eq!(session.fragments().len(), 3);
        assert!(deposit.remaining_mass_kg() < 1e-9);
        assert!(session.pick_up(ids[0]));
        assert!(!session.pick_up(ids[1]));
        assert_eq!(session.carried_id(), Some(ids[0]));
        assert!(session.drop_carried(pose));
        assert!(session.pick_up(ids[1]));
    }
}
