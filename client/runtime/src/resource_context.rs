//! Aimed physical-object information, composed into the existing action bar.
use crate::mining::MiningTool;
use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
use salimon_world::resources::DepositState;

pub(crate) fn context(
    tool: &MiningTool,
    player: CharacterSnapshot,
    frame: ShipFrame,
    door_open: bool,
    seed: u64,
) -> Option<String> {
    if !matches!(
        player.location,
        CharacterLocation::Surface | CharacterLocation::InsideShip
    ) {
        return None;
    }
    let target = crate::carrying::target(tool, player, frame, door_open);
    if let Some(id) = target.or(tool.session.carried_id()) {
        let piece = tool
            .session
            .fragments()
            .iter()
            .find(|piece| piece.id() == id)?;
        let prompt = crate::carrying::context(tool, target, player)?;
        return Some(format!(
            "{} fragment - approx {:.2} kg\n{}",
            piece.material().resource().definition().name,
            piece.material().mass_kg(),
            prompt,
        ));
    }
    if player.location != CharacterLocation::Surface {
        return None;
    }
    if let Some(target) = tool.inspect_target(player, frame, door_open, seed) {
        let entries = tool.nearby(player.eye_position_meters, seed).ok()?;
        let deposit = entries
            .iter()
            .find(|entry| entry.deposit.id() == target.id)?
            .deposit;
        let state = match deposit.state() {
            DepositState::Untouched => "untouched",
            DepositState::PartiallyMined => "partly mined",
            DepositState::Depleted => return None,
        };
        let prompt = if !tool.equipped {
            "M to equip mining tool"
        } else if tool.held {
            "Mining - release F or left mouse to stop"
        } else {
            "Hold F or left mouse to mine - M to stow"
        };
        return Some(format!(
            "{} - approx {:.1} kg left - {}\n{}",
            deposit.material().resource().definition().name,
            deposit.remaining_mass_kg(),
            state,
            prompt,
        ));
    }
    tool.equipped
        .then(|| "Aim at a deposit within 4 m - M to stow".to_owned())
}
