//! Physical Core interface and saved direct-material transaction composition.
use super::{GameplaySession, frames::character_ship_frame, interaction::InteractionTarget};
use salimon_ship::core_upgrades::{InputMode, Tier, UpgradeResult};
#[cfg(test)]
mod tests;
pub(crate) fn catalog() -> &'static [Tier] {
    static C: std::sync::LazyLock<Vec<Tier>> = std::sync::LazyLock::new(|| {
        let tiers: Vec<Tier> =
            serde_json::from_str(include_str!("../../catalogs/core-upgrades.json"))
                .expect("Core upgrade catalog");
        salimon_ship::core_upgrades::validate_catalog(&tiers)
            .expect("validated Core upgrade balance");
        tiers
    });
    &C
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum CoreUpgradeAction {
    ToggleInput,
    Apply,
    Recover,
}
impl GameplaySession {
    fn at_core(&self) -> bool {
        let s = self.snapshot();
        let f = character_ship_frame(s.ship.pose);
        self.available_target(
            s.character.location,
            f.world_to_local(s.character.eye_position_meters),
            f.world_to_local(s.character.look_target_meters),
        ) == Some(InteractionTarget::Core)
    }
    pub(crate) fn core_upgrade_action(&mut self, action: CoreUpgradeAction) {
        if self.building.active
            || !self.at_core()
            || !self.admit_survival_action(salimon_survival::restrictions::Action::Menu)
        {
            return;
        }
        let result = match action {
            CoreUpgradeAction::ToggleInput => {
                self.core_upgrades.mode = match self.core_upgrades.mode {
                    InputMode::Fuel => InputMode::Upgrade,
                    InputMode::Upgrade => InputMode::Fuel,
                };
                self.feedback = Some(if self.core_upgrades.mode == InputMode::Upgrade {
                    "CORE UPGRADE MATERIALS selected - E contribute; Y apply; N recover"
                } else {
                    "CORE FUEL INTAKE selected - E burns carried fragment"
                });
                return;
            }
            CoreUpgradeAction::Apply => {
                self.core_upgrades
                    .apply(&mut self.ship, &self.mining.session, catalog())
            }
            CoreUpgradeAction::Recover => {
                let original = self.core_upgrades.pending.last().copied();
                let mut result = self.core_upgrades.recover(&mut self.mining.session);
                if result == UpgradeResult::Recovered {
                    let s = self.snapshot();
                    let frame = character_ship_frame(s.ship.pose);
                    crate::carrying::follow(&mut self.mining, s.character, frame);
                    if self.mining.carry_blocked {
                        let f = original.expect("recovered receipt existed");
                        self.mining.session.remove_fragment(f.id());
                        self.core_upgrades.pending.push(f);
                        self.mining.carry_blocked = false;
                        result = UpgradeResult::Occupied;
                    } else {
                        self.equipment.stow();
                        self.mining.clear_input();
                    }
                }
                result
            }
        };
        self.core_upgrade_feedback(result);
        self.sync_carried();
    }
    pub(super) fn contribute_core_upgrade(&mut self) {
        let result = self
            .core_upgrades
            .contribute(&mut self.mining.session, catalog());
        self.core_upgrade_feedback(result);
        self.sync_carried();
    }
    fn core_upgrade_feedback(&mut self, r: UpgradeResult) {
        self.feedback = Some(match r {
            UpgradeResult::Accepted => {
                "UPGRADE material reserved; not fuel - Y apply when ready, N recover"
            }
            UpgradeResult::Applied => {
                "CORE UPGRADED INSTANTLY - stored joules preserved; no charge or thrust granted"
            }
            UpgradeResult::Recovered => "Original upgrade fragment returned to your hand",
            UpgradeResult::NoItem => "No carried material or pending receipt to recover",
            UpgradeResult::WrongMaterial => {
                "UPGRADE requires iron ore or silicate rock; material was not burned"
            }
            UpgradeResult::Oversized => {
                "Fragment exceeds remaining upgrade cost; bring a smaller piece (nothing consumed)"
            }
            UpgradeResult::NotReady => {
                "UPGRADE incomplete - supply all listed material before activation"
            }
            UpgradeResult::Maximum => "Maximum Core tier reached; no material consumed",
            UpgradeResult::Occupied => "Recovery requires one empty hand and a clear physical exit",
            UpgradeResult::Invalid => {
                "Upgrade declined: check configuration or whole-milligram material quantity"
            }
            UpgradeResult::StorageFull => {
                "Core material tray receipt limit reached; recover partial contributions"
            }
        });
    }
    pub(crate) fn core_upgrade_hint(&self) -> Option<String> {
        if !self.at_core() {
            return None;
        }
        if self.core_upgrades.mode == InputMode::Fuel {
            return Some(format!(
                "CORE FUEL INTAKE - E burns carried fragment\nI upgrades / recover {} reserved fragments | Installed T{}",
                self.core_upgrades.pending.len(),
                self.core_upgrades.applied.len()
            ));
        }
        let Some(t) = self.core_upgrades.next(catalog()) else {
            return Some("CORE UPGRADE MATERIALS - maximum tier\nI select fuel intake".into());
        };
        let left = self.core_upgrades.remaining(catalog()).unwrap_or([0, 0]);
        Some(format!(
            "CORE UPGRADE MATERIALS | Installed T{}: {} GJ\nNext T{}: {} GJ | Need Iron {:.3} / Silicate {:.3} kg\nE contribute | Y instant apply | N recover | I fuel",
            self.core_upgrades.applied.len(),
            self.ship.energy_config().capacity_joules / 1_000_000_000,
            t.level,
            t.capacity_joules / 1_000_000_000,
            left[0] as f64 / 1e6,
            left[1] as f64 / 1e6
        ))
    }
}

/// Explicit native evidence seed: two paid iron receipts and one physical silicate
/// sphere in hand. No claim of a walked mining/acquisition route.
pub(crate) fn evidence_initial_conditions(
    s: &mut GameplaySession,
    powered: bool,
) -> Result<(), String> {
    let eye = [-1., 1.9973, -2.2];
    let target = salimon_character::CORE_INTERACTION_POSITION_METERS;
    let d = salimon_math::sub(target, eye);
    let mut v = serde_json::to_value(s.character.save_state()).map_err(|e| e.to_string())?;
    v["position"] = serde_json::json!({"Inside":{"local":eye}});
    v["yaw_radians"] = d[2].atan2(d[0]).rem_euclid(std::f64::consts::TAU).into();
    v["pitch_radians"] = (d[1] / salimon_math::length(d)).asin().into();
    s.character = salimon_character::CharacterController::from_save(
        serde_json::from_value(v).map_err(|e| e.to_string())?,
        None,
    )
    .map_err(|e| e.to_string())?;
    s.core_upgrades = Default::default();
    s.core_upgrades.mode = InputMode::Upgrade;
    use salimon_world::{
        WorldPosition,
        resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform},
        spherical_mining::{MiningConfig, SPHERE_TICK},
    };
    let frame = character_ship_frame(s.ship.snapshot().pose);
    let p = frame.local_to_world([-1., 1.3, -1.5]);
    let pose = ResourceTransform::new(WorldPosition::new(p[0], p[1], p[2]), [0., 0., 0., 1.])
        .map_err(|e| e.to_string())?;
    for (i, material) in [
        ResourceId::IronOre,
        ResourceId::IronOre,
        ResourceId::SilicateRock,
    ]
    .into_iter()
    .enumerate()
    {
        let mut source = ResourceDeposit::new(
            DepositId {
                body: salimon_world::CelestialBodyId::Earth,
                local: 548000 + i as u64,
            },
            RawMaterial::new(material, 10.).map_err(|e| e.to_string())?,
            pose.position(),
            10.,
        )
        .map_err(|e| e.to_string())?;
        let id = (0..400)
            .find_map(|_| {
                s.mining.session.tick_spherical(
                    true,
                    Some(&mut source),
                    SPHERE_TICK,
                    MiningConfig::default(),
                    pose,
                )
            })
            .ok_or("evidence sphere not released")?;
        if !s.mining.session.pick_up(id) {
            return Err("evidence carry occupied".into());
        }
        s.equipment.stow();
        s.sync_carried();
        if i < 2
            && s.core_upgrades.contribute(&mut s.mining.session, catalog())
                != UpgradeResult::Accepted
        {
            return Err("evidence material not accepted".into());
        }
        s.sync_carried();
    }
    s.core_upgrades.mode = InputMode::Fuel;
    s.ship
        .configure_energy_core(
            s.ship.energy_config(),
            if powered { 12_345_678.456 } else { 0. },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
