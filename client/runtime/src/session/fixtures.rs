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
        let mut nodes = vec![node];
        let rails = salimon_ship::attachments::HABITATION_RAILS
            .iter()
            .copied()
            .chain([(2010, [-5.8, 0.30, 1.7])]);
        for (id, p) in rails.filter(|_| !self.mounts.mounts.is_empty()) {
            nodes.push(FixtureNode {
                id,
                parent: None,
                mesh: FixtureMesh::Shelf,
                local: FixturePose {
                    position_meters: [p[0], p[1] - 0.05, p[2] + 0.18],
                    scale: 0.16,
                    ..Default::default()
                },
                tint: [0.8, 0.55, 0.25],
                powered: ship.power_available,
            });
        }
        let frame = super::frames::character_ship_frame(ship.pose);
        for m in &self.mounts.mounts {
            let carried = self.mounts.carried.as_ref().filter(|c| c.id == m.id);
            nodes.push(FixtureNode {
                id: m.id,
                parent: None,
                mesh: FixtureMesh::Shelf,
                local: FixturePose {
                    position_meters: carried
                        .map_or_else(|| m.position(), |c| frame.world_to_local(c.world_position)),
                    ..Default::default()
                },
                tint: [1.; 3],
                powered: ship.power_available,
            });
            if carried.is_none() {
                for side in [-1., 1.] {
                    let mut p = m.position();
                    p[0] += side * 0.20;
                    p[1] += 0.04;
                    p[2] += if m.secured.is_some() { 0.02 } else { 0.12 };
                    nodes.push(FixtureNode {
                        id: m.id + if side < 0. { 10000 } else { 20000 },
                        parent: None,
                        mesh: FixtureMesh::Shelf,
                        local: FixturePose {
                            position_meters: p,
                            orientation_xyzw: [
                                0.,
                                0.,
                                std::f64::consts::FRAC_1_SQRT_2,
                                std::f64::consts::FRAC_1_SQRT_2,
                            ],
                            scale: 0.16,
                        },
                        tint: [0.8, 0.55, 0.25],
                        powered: ship.power_available,
                    });
                }
            }
        }
        salimon_renderer::resolve_fixture_hierarchy(
            FixturePose {
                position_meters: ship.pose.position_meters,
                orientation_xyzw: ship.pose.orientation,
                scale: 1.,
            },
            &nodes,
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
        assert_eq!(before.len(), 11);
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
