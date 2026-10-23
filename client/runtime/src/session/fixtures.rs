//! Version-1 migration from the combined source berth to one stable fixture ID.
use super::GameplaySession;
use salimon_renderer::{FixtureInstance, FixtureMesh, FixtureNode, FixturePose};
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InteriorLayout {
    format: String,
    version: u32,
    starter_bunk_id: u64,
    legacy_nodes: [String; 2],
    physical_source: String,
}
fn layout() -> &'static InteriorLayout {
    static LAYOUT: std::sync::LazyLock<InteriorLayout> = std::sync::LazyLock::new(|| {
        let v: InteriorLayout =
            serde_json::from_str(include_str!("../../../assets/ship/interior-layout-v1.json"))
                .expect("published interior layout");
        assert_eq!(v.format, "salimon.interior-layout");
        assert_eq!(v.version, salimon_renderer::FIXTURE_LAYOUT_VERSION);
        assert_eq!(v.legacy_nodes, ["Starter_BunkFrame", "Starter_BunkCushion"]);
        assert_eq!(v.physical_source, "session.habitation");
        v
    });
    &LAYOUT
}
impl GameplaySession {
    pub(crate) fn fixture_instances(
        &self,
        ship: salimon_ship::ShipSnapshot,
    ) -> Vec<FixtureInstance> {
        let angle = self.habitation.bunk_open_fraction * std::f64::consts::FRAC_PI_2;
        let node = FixtureNode {
            id: layout().starter_bunk_id,
            parent: None,
            mesh: FixtureMesh::Bunk,
            local: FixturePose {
                position_meters: salimon_character::habitation_geometry::BUNK_HINGE_METERS,
                orientation_xyzw: [(angle / 2.).sin(), 0., 0., (angle / 2.).cos()],
                scale: 1.,
            },
            tint: [1.; 3],
            powered: ship.power_available,
        };
        salimon_renderer::resolve_fixture_hierarchy(
            FixturePose {
                position_meters: ship.pose.position_meters,
                orientation_xyzw: ship.pose.orientation,
                scale: 1.,
            },
            &[node],
        )
        .expect("validated ship and habitation source")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_save_source_migrates_to_one_stable_fixture_without_mass_or_gameplay_mutation() {
        let mut s = GameplaySession::new_run();
        s.habitation.bunk_open_fraction = 0.375;
        let bytes = s.save_bytes(100).unwrap();
        let before = s.fixture_instances(s.ship.snapshot());
        assert_eq!(before.len(), 1);
        assert_eq!(before[0].id, 1001);
        assert_eq!(s.save_bytes(100).unwrap(), bytes);
        let mass = s.ship.snapshot().total_mass_kg;
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert_eq!(r.fixture_instances(r.ship.snapshot()), before);
        assert_eq!(r.ship.snapshot().total_mass_kg, mass);
        r.ship.set_stored_core_energy_joules(0);
        assert!(!r.fixture_instances(r.ship.snapshot())[0].powered);
    }
}
