//! Applicable resource text and absolute object anchors; no projection or GPU policy.
use crate::mining::MiningTool;
use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
use salimon_world::resources::DepositState;

pub(crate) struct Prompt {
    pub text: String,
    pub placement: salimon_renderer::OverlayPlacement,
}

pub(crate) fn context(
    tool: &MiningTool,
    equipment: &crate::equipment::EquipmentToolbar,
    player: CharacterSnapshot,
    frame: ShipFrame,
    door_open: bool,
    seed: u64,
) -> Option<Prompt> {
    if !matches!(
        player.location,
        CharacterLocation::Surface | CharacterLocation::InsideShip
    ) {
        return None;
    }
    let target = crate::carrying::target(tool, player, frame, door_open);
    if let Some(id) = tool.session.carried_id().or(target) {
        let piece = tool
            .session
            .fragments()
            .iter()
            .find(|piece| piece.id() == id)?;
        let prompt = crate::carrying::context(tool, target, player)?;
        return Some(Prompt {
            placement: salimon_renderer::OverlayPlacement::World {
                anchor_meters: piece.transform().position().meters(),
                radius_meters: salimon_world::resource_fragments::side_meters(*piece)
                    * 0.5
                    * 3.0_f64.sqrt(),
            },
            text: format!(
                "{} fragment - approx {:.2} kg\n{}",
                piece.material().resource().definition().name,
                piece.material().mass_kg(),
                prompt,
            ),
        });
    }
    if player.location != CharacterLocation::Surface {
        return None;
    }
    if let Some(target) = tool.inspect_target(player, frame, door_open, seed) {
        let entries = tool.nearby(player.eye_position_meters, seed).ok()?;
        let deposit = entries
            .iter()
            .find(|entry| entry.deposit.id() == target.id)?;
        let radius = deposit.bounds_radius_meters;
        let deposit = deposit.deposit;
        let state = match deposit.state() {
            DepositState::Untouched => "untouched",
            DepositState::PartiallyMined => "partly mined",
            DepositState::Depleted => return None,
        };
        let prompt = if !equipment.mining_equipped() {
            "1 to equip mining tool"
        } else if tool.held {
            "Mining - release F or left mouse to stop"
        } else {
            "Hold F or left mouse to mine - 2-5 to stow"
        };
        return Some(Prompt {
            placement: salimon_renderer::OverlayPlacement::World {
                anchor_meters: deposit.position().meters(),
                radius_meters: radius,
            },
            text: format!(
                "{} - approx {:.1} kg left - {}\n{}",
                deposit.material().resource().definition().name,
                deposit.remaining_mass_kg(),
                state,
                prompt,
            ),
        });
    }
    equipment.mining_equipped().then(|| Prompt {
        text: "Aim at a deposit within 4 m - 2-5 to stow".to_owned(),
        placement: salimon_renderer::OverlayPlacement::BottomCenter,
    })
}
