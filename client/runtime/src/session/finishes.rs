//! Local physical appearance controls; cosmetic source never edits part age/damage.
use super::{GameplaySession, frames::character_ship_frame, interaction::InteractionTarget};
use salimon_ship::finishes::{CABIN_ROOM, CARGO_ROOM, Group};
#[derive(Clone, Copy, Debug)]
pub(crate) enum FinishAction {
    SelectGroup,
    Next,
    Reset,
}
impl GameplaySession {
    fn finish_target(&self) -> Option<InteractionTarget> {
        let s = self.snapshot();
        let f = character_ship_frame(s.ship.pose);
        self.available_target(
            s.character.location,
            f.world_to_local(s.character.eye_position_meters),
            f.world_to_local(s.character.look_target_meters),
        )
    }
    fn finish_room(&self) -> u64 {
        if self
            .character
            .local_ship_position()
            .is_some_and(|p| p[0] < -4.65)
        {
            CARGO_ROOM
        } else {
            CABIN_ROOM
        }
    }
    pub(super) fn validate_finishes(&self) -> Result<(), salimon_persistence::SaveError> {
        if !self.finishes.rooms.contains_key(&CABIN_ROOM)
            || !self.finishes.rooms.contains_key(&CARGO_ROOM)
            || self.finishes.items.iter().any(|(id, i)| {
                i.room != CABIN_ROOM
                    || !matches!(*id, 1001 | 3001 | 3002 | 3020)
                    || (*id != 1001 && self.habitation.cabin.is_none())
            })
        {
            return Err(salimon_persistence::invalid(
                "finish association has no installed room/furniture",
            ));
        }
        Ok(())
    }
    pub(crate) fn finish_action(&mut self, action: FinishAction) {
        if self.building.active
            || !self.admit_survival_action(salimon_survival::restrictions::Action::Menu)
        {
            return;
        }
        let target = self.finish_target();
        if let Some((id, _)) = target.and_then(furniture) {
            match action {
                FinishAction::Next => {
                    self.finishes.cycle_item(CABIN_ROOM, id);
                }
                FinishAction::Reset => self.finishes.clear_item(id),
                FinishAction::SelectGroup => {}
            }
            self.feedback = Some("Furniture finish changed; physical part and wear retained");
        } else if matches!(
            target,
            Some(InteractionTarget::Galley | InteractionTarget::CargoDoor)
        ) {
            let room = self.finish_room();
            match action {
                FinishAction::SelectGroup => self.finish_group = (self.finish_group + 1) % 5,
                FinishAction::Next => {
                    self.finishes
                        .cycle_group(room, Group::ALL[self.finish_group]);
                }
                FinishAction::Reset => {
                    self.finishes.rooms.get_mut(&room).unwrap().groups[self.finish_group] = 0
                }
            }
            self.feedback = Some("Room finish selected; V group, O color, Z original");
        } else {
            self.feedback = Some("Aim at galley/door finish controls or individual furniture");
        }
    }
    pub(crate) fn finish_hint(&self) -> Option<String> {
        let target = self.finish_target();
        if let Some((id, group)) = target.and_then(furniture) {
            let style = self
                .finishes
                .items
                .get(&id)
                .map(|i| salimon_ship::finishes::style_name(i.style))
                .unwrap_or("Room");
            return Some(format!(
                "{} finish: {} | O color | Z inherit | E use",
                group.name(),
                style
            ));
        }
        if matches!(
            target,
            Some(InteractionTarget::Galley | InteractionTarget::CargoDoor)
        ) {
            let room = self.finish_room();
            let g = Group::ALL[self.finish_group];
            let style = self.finishes.rooms.get(&room)?.groups[self.finish_group];
            return Some(format!(
                "Room {} {}: {} | V group | O color | Z reset | E use",
                room,
                g.name(),
                salimon_ship::finishes::style_name(style)
            ));
        }
        None
    }
    pub(crate) fn finish_palette(&self) -> [[[f32; 3]; 5]; 2] {
        [CABIN_ROOM, CARGO_ROOM].map(|r| Group::ALL.map(|g| self.finishes.group_tint(r, g)))
    }
}
fn furniture(t: InteractionTarget) -> Option<(u64, Group)> {
    Some(match t {
        InteractionTarget::CabinSeat => (3002, Group::Upholstery),
        InteractionTarget::Bunk => (1001, Group::Upholstery),
        InteractionTarget::CabinTable => (3001, Group::Cabinetry),
        InteractionTarget::CabinLocker => (3020, Group::Cabinetry),
        _ => return None,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use salimon_persistence::Validate;
    #[test]
    fn physical_controls_keep_costs_energy_and_part_identity_unchanged_and_restore() {
        let mut s = GameplaySession::new_run();
        crate::session::cabin::evidence_initial_conditions(&mut s, "scout-cabin").unwrap();
        let ship = s.ship.save_state();
        let before = serde_json::to_vec(&ship).unwrap();
        let physical = s.habitation.clone();
        s.finish_action(FinishAction::Next);
        assert_eq!(s.finishes.items[&3002].style, 1);
        assert_eq!(serde_json::to_vec(&s.ship.save_state()).unwrap(), before);
        assert_eq!(
            serde_json::to_vec(&s.habitation).unwrap(),
            serde_json::to_vec(&physical).unwrap()
        );
        s.finishes.validate().unwrap();
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert_eq!(r.save_bytes(100).unwrap(), bytes);
        r.finish_action(FinishAction::Reset);
        assert!(!r.finishes.items.contains_key(&3002));
        s.command(super::super::Command::TogglePause);
        let frozen = s.finishes.clone();
        s.command(super::super::Command::Finish(FinishAction::Next));
        assert_eq!(s.finishes, frozen);
    }
}

#[cfg(test)]
mod integrity_tests {
    use super::*;
    #[test]
    fn five_groups_survive_reload_and_unknown_furniture_restore_is_atomic() {
        let mut s = GameplaySession::new_run();
        super::super::cabin::evidence_initial_conditions(&mut s, "scout-finishes").unwrap();
        let before = s.ship.snapshot();
        for _ in 0..5 {
            s.finish_action(FinishAction::Next);
            s.finish_action(FinishAction::SelectGroup);
        }
        assert_eq!(s.finishes.rooms[&CABIN_ROOM].groups, [1; 5]);
        assert_eq!(s.finishes.rooms[&CARGO_ROOM].groups, [0; 5]);
        assert_eq!(s.ship.snapshot(), before);
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert_eq!(r.save_bytes(100).unwrap(), bytes);
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        v["state"]["finishes"]["items"]["999999"] = serde_json::json!({"room":1,"style":1});
        assert!(r.restore_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
        assert_eq!(r.save_bytes(100).unwrap(), bytes);
    }
}
