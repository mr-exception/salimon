//! Tool input and session extraction composition; no general resource inventory.
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
use salimon_renderer::HeldItemInstance;
use salimon_world::WorldPosition;
use salimon_world::mining::{MiningRay, MiningSession, MiningTarget, aimed_deposit_with_terrain};
use salimon_world::resource_generation::{GenerationError, SurfaceDeposit};

#[derive(Default)]
struct NearbyCache {
    key: Option<([u64; 3], u64, salimon_world::terrain::TerrainMode)>,
    raw: Vec<SurfaceDeposit>,
    revision: Option<u64>,
    reconciled: Rc<Vec<SurfaceDeposit>>,
    queries: u64,
    reconciliations: u64,
    candidate_micros: u128,
    reconciliation_micros: u128,
}

#[derive(Default)]
pub(crate) struct MiningTool {
    pub(crate) ship_gravity_disabled: bool,
    pub(crate) ship_mass: salimon_ship::ShipMass,
    pub(crate) mass_frame: Option<ShipFrame>,
    pub(crate) terrain: salimon_world::terrain::TerrainMode,
    nearby_cache: RefCell<NearbyCache>,
    placements: std::cell::RefCell<crate::resource_presentation::PlacementCache>,
    pub(crate) fragment_previous: std::collections::HashMap<
        salimon_world::resources::FragmentId,
        salimon_physics::ObjectState,
    >,
    pub(crate) fragment_scratch: crate::fragment_physics::FragmentScratch,
    pub(crate) fragment_sleep: salimon_physics::SleepTracker<salimon_world::resources::FragmentId>,
    pub(crate) ship_fragments:
        std::collections::HashMap<salimon_world::resources::FragmentId, [f64; 3]>,
    pub(crate) fragment_motion: std::collections::HashMap<
        salimon_world::resources::FragmentId,
        crate::fragment_physics::FragmentMotion,
    >,
    pub(crate) carry_feedback: Option<&'static str>,
    pub(crate) held: bool,
    pub(crate) config: salimon_world::spherical_mining::MiningConfig,
    f_down: bool,
    pub(crate) session: MiningSession,
}

impl MiningTool {
    /// Latch the initial F edge; a carrying action consumes this entire press.
    pub(crate) fn press_f(&mut self) -> bool {
        if self.f_down {
            return false;
        }
        self.f_down = true;
        true
    }

    pub(crate) fn release_f(&mut self) {
        self.f_down = false;
    }

    pub(crate) fn set_mouse_held(&mut self, pressed: bool) {
        self.held = pressed;
        if !pressed {
            self.session.interrupt_spherical();
        }
    }

    /// Cancel extraction while preserving a contextual F press consumed by pickup.
    pub(crate) fn stop_mining(&mut self) {
        self.held = false;
        self.session.interrupt_spherical();
    }

    pub(crate) fn clear_input(&mut self) {
        self.f_down = false;
        self.stop_mining();
    }

    pub(crate) fn nearby(
        &self,
        position: [f64; 3],
        seed: u64,
    ) -> Result<Rc<Vec<SurfaceDeposit>>, GenerationError> {
        // Exact position bits preserve radius boundaries; look-only redraws reuse
        // candidates but always recompute the ray/obstruction against current aim.
        let key = (position.map(f64::to_bits), seed, self.terrain);
        let mut cache = self.nearby_cache.borrow_mut();
        if cache.key != Some(key) {
            let start = Instant::now();
            let entries = crate::resource_presentation::nearby_deposits_cached(
                position,
                seed,
                self.terrain,
                &self.placements,
            )?;
            cache.candidate_micros += start.elapsed().as_micros();
            cache.queries += 1;
            cache.raw = entries;
            cache.key = Some(key);
            cache.revision = None;
        }
        if cache.revision != Some(self.session.deposit_revision()) {
            let start = Instant::now();
            let mut entries = cache.raw.clone();
            self.session.apply_to(&mut entries);
            cache.reconciled = Rc::new(entries);
            cache.revision = Some(self.session.deposit_revision());
            cache.reconciliations += 1;
            cache.reconciliation_micros += start.elapsed().as_micros();
        }
        Ok(Rc::clone(&cache.reconciled))
    }

    pub(crate) fn query_metrics(&self) -> serde_json::Value {
        let cache = self.nearby_cache.borrow();
        let placements = self.placements.borrow();
        serde_json::json!({"candidate_queries": cache.queries,
            "candidate_and_placement_micros": cache.candidate_micros,
            "candidate_micros": cache.candidate_micros.saturating_sub(placements.micros),
            "reconciliations": cache.reconciliations,
            "reconciliation_micros": cache.reconciliation_micros,
            "placement_misses": placements.misses,
            "placement_micros": placements.micros})
    }

    pub(crate) fn target(
        &self,
        equipment: &crate::equipment::EquipmentToolbar,
        player: CharacterSnapshot,
        frame: ShipFrame,
        door_open: bool,
        seed: u64,
    ) -> Option<MiningTarget> {
        equipment
            .mining_equipped()
            .then(|| self.inspect_target(player, frame, door_open, seed))
            .flatten()
    }

    /// Inspect a reachable deposit even when the handheld tool is stowed.
    pub(crate) fn inspect_target(
        &self,
        player: CharacterSnapshot,
        frame: ShipFrame,
        door_open: bool,
        seed: u64,
    ) -> Option<MiningTarget> {
        if player.location != CharacterLocation::Surface {
            return None;
        }
        let origin = position(player.eye_position_meters);
        let ray = MiningRay::new(origin, position(player.look_target_meters))?;
        let obstruction = salimon_character::ship_sight_obstruction(
            frame.world_to_local(player.eye_position_meters),
            frame.world_to_local(player.look_target_meters),
            door_open,
        );
        aimed_deposit_with_terrain(
            ray,
            &self.nearby(player.eye_position_meters, seed).ok()?,
            obstruction,
            self.terrain,
        )
    }

    pub(crate) fn nearby_fragments(
        &self,
        player: [f64; 3],
    ) -> impl Iterator<Item = salimon_world::resources::ResourceFragment> + '_ {
        self.session
            .fragments()
            .iter()
            .copied()
            .filter(move |piece| {
                let offset = piece.transform().position().offset_from(position(player));
                offset.iter().map(|v| v * v).sum::<f64>()
                    <= crate::resource_presentation::ACTIVE_RADIUS_METERS.powi(2)
            })
    }

    pub(crate) fn advance(
        &mut self,
        equipment: &crate::equipment::EquipmentToolbar,
        delta: Duration,
        player: CharacterSnapshot,
        frame: ShipFrame,
        door_open: bool,
        seed: u64,
    ) {
        let Some(pose) = attachment_pose(player) else {
            self.session.interrupt_spherical();
            return;
        };
        let Some(target) = self
            .held
            .then(|| self.target(equipment, player, frame, door_open, seed))
            .flatten()
        else {
            self.session
                .tick_spherical(self.held, None, delta, self.config, pose);
            return;
        };
        let Ok(deposits) = self.nearby(player.eye_position_meters, seed) else {
            self.session.interrupt_spherical();
            return;
        };
        let mut target = deposits
            .iter()
            .find(|entry| entry.deposit.id() == target.id)
            .map(|entry| entry.deposit);
        self.session
            .tick_spherical(self.held, target.as_mut(), delta, self.config, pose);
    }

    /// Presentation of reserved mass, not an additional physical entity. Recompute
    /// from this frame's camera so looking without a simulation tick never lags.
    pub(crate) fn growth_preview(
        &self,
        equipment: &crate::equipment::EquipmentToolbar,
        player: CharacterSnapshot,
    ) -> Option<salimon_renderer::ResourceMeshInstance> {
        if player.location != CharacterLocation::Surface || !equipment.mining_equipped() {
            return None;
        }
        let id = self.session.attached_sphere_id()?;
        let mut fragment = self
            .session
            .fragments()
            .iter()
            .find(|p| p.id() == id)
            .copied()?;
        fragment.set_transform(preview_pose(player)?);
        Some(crate::resource_presentation::fragment_mesh(fragment))
    }

    /// Equip/location gates and active feedback for the authored visual.
    /// The caller supplies gameplay visibility and the existing target result.
    pub(crate) fn held_item(
        &self,
        equipment: &crate::equipment::EquipmentToolbar,
        player: CharacterSnapshot,
        valid_target: bool,
    ) -> Option<HeldItemInstance> {
        (player.location == CharacterLocation::Surface && equipment.mining_equipped()).then_some(
            HeldItemInstance {
                active: self.held && valid_target,
            },
        )
    }
}

fn attachment_pose(
    player: CharacterSnapshot,
) -> Option<salimon_world::resources::ResourceTransform> {
    preview_pose(player)
}

/// Current camera/tool front-left in absolute f64 meters. Renderer grip is
/// camera-right 0.20, down 0.20, forward 0.55; this point is left of its muzzle.
fn preview_pose(player: CharacterSnapshot) -> Option<salimon_world::resources::ResourceTransform> {
    use salimon_math::{cross, length, sub};
    let direction = sub(player.look_target_meters, player.eye_position_meters);
    let magnitude = length(direction);
    if !magnitude.is_finite() || magnitude <= 1e-12 {
        return None;
    }
    let forward = direction.map(|v| v / magnitude);
    let right = cross(forward, player.up.map(f64::from));
    let magnitude = length(right);
    if !magnitude.is_finite() || magnitude <= 1e-12 {
        return None;
    }
    let right = right.map(|v| v / magnitude);
    let up = cross(right, forward);
    let point = std::array::from_fn(|i| {
        player.eye_position_meters[i] + forward[i] * 0.85 - right[i] * 0.15 - up[i] * 0.20
    });
    salimon_world::resources::ResourceTransform::new(
        position(point),
        salimon_physics::orientation_from_axes([forward, up, right]),
    )
    .ok()
}

fn position(p: [f64; 3]) -> WorldPosition {
    WorldPosition::new(p[0], p[1], p[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carrying_press_is_latched_and_mouse_mining_is_independent() {
        let mut tool = MiningTool::default();
        assert!(tool.press_f());
        assert!(
            !tool.held,
            "carrying consumes F without starting extraction"
        );
        assert!(!tool.press_f(), "repeated press cannot change the action");
        tool.set_mouse_held(true);
        tool.release_f();
        assert!(tool.held, "F release must preserve left mouse mining");
        assert!(tool.press_f());
        tool.set_mouse_held(false);
        assert!(
            !tool.held,
            "held F cannot sustain mining after mouse release"
        );
        tool.set_mouse_held(true);
        tool.stop_mining();
        assert!(!tool.held);
        assert!(!tool.press_f(), "pickup cancellation preserves the F latch");
        tool.clear_input();
        assert!(tool.press_f(), "input reset clears the F latch");
        assert!(!tool.held, "F after reset cannot start mining");
    }

    #[test]
    fn authored_tool_visibility_and_feedback_follow_existing_gates() {
        let mut tool = MiningTool::default();
        let mut equipment = crate::equipment::EquipmentToolbar::default();
        let mut player = CharacterSnapshot {
            location: CharacterLocation::Surface,
            eye_position_meters: [0.0; 3],
            look_target_meters: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            local_ship_position_meters: None,
            doorway_blend_fraction: None,
        };
        assert!(tool.held_item(&equipment, player, true).is_none());
        equipment.select(crate::equipment::ToolbarSlot::One, false);
        assert_eq!(
            tool.held_item(&equipment, player, true),
            Some(HeldItemInstance { active: false })
        );
        tool.held = true;
        assert_eq!(
            tool.held_item(&equipment, player, false),
            Some(HeldItemInstance { active: false })
        );
        assert_eq!(
            tool.held_item(&equipment, player, true),
            Some(HeldItemInstance { active: true })
        );
        player.location = CharacterLocation::InsideShip;
        assert!(tool.held_item(&equipment, player, true).is_none());
        player.location = CharacterLocation::Surface;
        equipment.sync_carrying(true);
        tool.clear_input();
        assert!(tool.held_item(&equipment, player, true).is_none());
        assert!(!tool.held);
    }
}

#[cfg(test)]
mod spherical_tests {
    use super::*;
    use salimon_world::resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId};

    fn player() -> CharacterSnapshot {
        CharacterSnapshot {
            location: CharacterLocation::Surface,
            eye_position_meters: [0.0, 2.0, 0.0],
            look_target_meters: [1.0, 2.0, 0.0],
            up: [0.0, 1.0, 0.0],
            local_ship_position_meters: None,
            doorway_blend_fraction: None,
        }
    }

    #[test]
    fn attached_piece_follows_tool_and_is_excluded_from_loose_physics() {
        let mut tool = MiningTool::default();
        let pose = attachment_pose(player()).unwrap();
        let mut source = ResourceDeposit::new(
            DepositId {
                body: salimon_world::CelestialBodyId::Earth,
                local: 99,
            },
            RawMaterial::new(ResourceId::IronOre, 25.0).unwrap(),
            WorldPosition::new(2.0, 0.0, 0.0),
            25.0,
        )
        .unwrap();
        tool.session.tick_spherical(
            true,
            Some(&mut source),
            Duration::from_millis(16),
            tool.config,
            pose,
        );
        let id = tool.session.attached_sphere_id().unwrap();
        let mut equipment = crate::equipment::EquipmentToolbar::default();
        assert!(tool.growth_preview(&equipment, player()).is_none());
        equipment.select(crate::equipment::ToolbarSlot::One, false);
        let preview = tool.growth_preview(&equipment, player()).unwrap();
        assert_eq!(preview.mesh, salimon_renderer::ResourceMesh::Sphere);
        let mut rotated = player();
        rotated.look_target_meters = [0.0, 2.0, 1.0];
        let turned = tool.growth_preview(&equipment, rotated).unwrap();
        assert_eq!(turned.center_meters, [0.15, 1.8, 0.85]);
        assert_eq!(turned.side_meters, preview.side_meters);
        assert_ne!(turned.orientation_xyzw, preview.orientation_xyzw);
        assert_eq!(
            tool.session.fragments().len(),
            1,
            "preview must not duplicate reserved mass"
        );
        let mut hidden = player();
        hidden.location = CharacterLocation::Cockpit;
        assert!(tool.growth_preview(&equipment, hidden).is_none());

        let frame = ShipFrame {
            origin_meters: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        };
        assert!(crate::fragment_physics::snapshots(&tool, frame, [0.0; 3]).is_empty());
        assert!(!tool.session.pick_up(id));
        tool.set_mouse_held(false);
        assert_eq!(tool.session.attached_sphere_id(), None);
        assert!(tool.growth_preview(&equipment, player()).is_none());
        assert_eq!(
            crate::fragment_physics::snapshots(&tool, frame, [0.0; 3]).len(),
            1
        );
        assert_eq!(tool.session.fragments()[0].id(), id);
        assert!(tool.session.pick_up(id));
    }

    #[test]
    fn default_cap_preview_stays_clear_of_aim_ray_for_every_resource() {
        for resource in [
            ResourceId::WaterIce,
            ResourceId::SilicateRock,
            ResourceId::IronOre,
        ] {
            let radius = salimon_world::spherical_mining::GrowingSphere {
                source: DepositId {
                    body: salimon_world::CelestialBodyId::Earth,
                    local: 0,
                },
                material: RawMaterial::new(
                    resource,
                    salimon_world::spherical_mining::DEFAULT_SPHERE_CAP_KG,
                )
                .unwrap(),
            }
            .radius_meters();
            let center = attachment_pose(player()).unwrap().position().meters();
            let distance_from_aim = (center[1] - 2.0).hypot(center[2]);
            assert!(
                distance_from_aim > radius + 0.03,
                "sphere must leave the crosshair clear"
            );
            assert!(
                center[0] - radius > 0.1,
                "sphere must stay ahead of near plane/player"
            );
        }
    }

    #[test]
    fn tool_attachment_uses_current_camera_basis_and_absolute_f64_pose() {
        let original = attachment_pose(player()).unwrap().position().meters();
        assert_eq!(original, [0.85, 1.8, -0.15]);
        let mut moved = player();
        moved.eye_position_meters = [1e12, -1e12, 1e12];
        moved.look_target_meters = [1e12, -1e12, 1e12 + 1.0];
        let offset = attachment_pose(moved)
            .unwrap()
            .position()
            .offset_from(position(moved.eye_position_meters));
        for (actual, expected) in offset.into_iter().zip([0.15, -0.20, 0.85]) {
            assert!((actual - expected).abs() < 0.0002);
        }
    }
}

#[cfg(test)]
mod query_cache_tests {
    use super::*;
    use salimon_world::{CELESTIAL_BODIES, CelestialBodyId, terrain::TerrainMode};

    #[test]
    fn nearby_cache_reuses_results_and_reconciles_depletion_without_regenerating() {
        let mut tool = MiningTool::default();
        let body = CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == CelestialBodyId::Earth)
            .unwrap();
        let point = body
            .center
            .translated([0.0, body.radius_meters + 2.0, 0.0])
            .meters();
        let first = tool.nearby(point, 42).unwrap();
        assert!(!first.is_empty());
        assert!(Rc::ptr_eq(&first, &tool.nearby(point, 42).unwrap()));
        let mut source = first[0].deposit;
        tool.session
            .extract(&mut source, Duration::from_secs(100_000));
        let restored = tool.nearby(point, 42).unwrap();
        assert_eq!(tool.nearby_cache.borrow().queries, 1);
        assert_eq!(restored[0].deposit.remaining_mass_kg(), 0.0);
        assert!(
            first[0].deposit.remaining_mass_kg() > 0.0,
            "old frame remains immutable"
        );
        tool.nearby(salimon_math::add(point, [256.0, 0.0, 0.0]), 42)
            .unwrap();
        assert_eq!(tool.nearby(point, 42).unwrap(), restored);
        tool.nearby(point, 43).unwrap();
        tool.terrain = TerrainMode::Representative;
        tool.nearby(point, 43).unwrap();
        let moon = CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == CelestialBodyId::Moon)
            .unwrap();
        tool.nearby(
            moon.center
                .translated([0.0, moon.radius_meters, 0.0])
                .meters(),
            43,
        )
        .unwrap();
        assert_eq!(tool.nearby_cache.borrow().queries, 6);
    }

    #[test]
    fn look_only_updates_recompute_aim_and_zero_time_aim_loss_releases() {
        let mut tool = MiningTool::default();
        let body = CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == CelestialBodyId::Earth)
            .unwrap();
        let nearby = tool
            .nearby(
                body.center
                    .translated([0.0, body.radius_meters, 0.0])
                    .meters(),
                42,
            )
            .unwrap();
        let source = nearby[0];
        let center = source.deposit.position().meters();
        let mut player = CharacterSnapshot {
            location: CharacterLocation::Surface,
            eye_position_meters: salimon_math::add(
                center,
                [0.0, source.bounds_radius_meters + 1.0, 0.0],
            ),
            look_target_meters: center,
            up: [0.0, 0.0, 1.0],
            local_ship_position_meters: None,
            doorway_blend_fraction: None,
        };
        let frame = ShipFrame {
            origin_meters: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        };
        let mut equipment = crate::equipment::EquipmentToolbar::default();
        equipment.select(crate::equipment::ToolbarSlot::One, false);
        tool.held = true;
        assert!(tool.target(&equipment, player, frame, false, 42).is_some());
        tool.advance(
            &equipment,
            Duration::from_millis(16),
            player,
            frame,
            false,
            42,
        );
        assert!(tool.session.attached_sphere_id().is_some());
        let queries = tool.nearby_cache.borrow().queries;
        player.look_target_meters = salimon_math::add(player.eye_position_meters, [0.0, 10.0, 0.0]);
        assert!(tool.target(&equipment, player, frame, false, 42).is_none());
        tool.advance(&equipment, Duration::ZERO, player, frame, false, 42);
        assert!(tool.session.attached_sphere_id().is_none());
        assert_eq!(tool.nearby_cache.borrow().queries, queries);
    }

    #[test]
    #[ignore = "opt-in CPU timing; does not certify native performance"]
    fn preparation_profile() {
        let tool = MiningTool {
            terrain: TerrainMode::Representative,
            ..Default::default()
        };
        let mut stream = crate::terrain_patches::TerrainStream::default();
        for body in CELESTIAL_BODIES.iter().filter(|b| {
            [
                CelestialBodyId::Earth,
                CelestialBodyId::Moon,
                CelestialBodyId::Mars,
            ]
            .contains(&b.id)
        }) {
            for offset in [0.0, 256.0, 1200.0, 0.0] {
                let radial = [offset, body.radius_meters, 0.0];
                let direction = salimon_math::scale(radial, 1.0 / salimon_math::length(radial));
                let ground = TerrainMode::Representative.sample(*body, direction);
                let point = body
                    .center
                    .translated(salimon_math::add(
                        ground.body_local_position_meters,
                        salimon_math::scale(ground.outward_normal, 2.0),
                    ))
                    .meters();
                let start = Instant::now();
                let entries = tool.nearby(point, 42).unwrap();
                let cold = start.elapsed();
                let start = Instant::now();
                for _ in 0..100 {
                    tool.nearby(point, 42).unwrap();
                }
                let reuse = start.elapsed();
                stream.update(point, TerrainMode::Representative).unwrap();
                eprintln!(
                    "{} {offset}: {} deposits cold={cold:?} reuse100={reuse:?} selection={}us preparation={}us",
                    body.name,
                    entries.len(),
                    stream.selection_micros,
                    stream.preparation_micros
                );
                for _ in 0..600 {
                    stream.update(point, TerrainMode::Representative).unwrap();
                    if stream.pending == 0 {
                        break;
                    }
                }
                assert_eq!(stream.pending, 0);
                for _ in 0..20 {
                    stream.update(point, TerrainMode::Representative).unwrap();
                }
                eprintln!(
                    "terrain steady preparation={}us peak={}us overruns={} metrics={}",
                    stream.preparation_micros,
                    stream.preparation_peak_micros,
                    stream.preparation_overruns,
                    tool.query_metrics()
                );
            }
        }
    }
}
