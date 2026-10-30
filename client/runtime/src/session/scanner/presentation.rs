//! Observational cockpit adapter. No catalog entries become discoveries here.
use super::*;
use salimon_character::CharacterLocation;
impl GameplaySession {
    pub(crate) fn scanner_next_contact(&mut self) {
        if self.scanner_controls_available() {
            self.scanner.selected = self.scanner.selected.wrapping_add(1);
            self.scanner.chart_note = None;
        }
    }
    pub(crate) fn scanner_controls_available(&self) -> bool {
        self.mining.terrain == salimon_world::terrain::TerrainMode::Canonical
            && self.view_mode == super::super::ViewMode::Gameplay
            && self.character.location() == CharacterLocation::Cockpit
            && !self.is_paused()
            && !self.survival.snapshot().terminal
            && !self.archive_reader.open
            && !self.crafting_ui.open
            && !self.wheel.open
    }
    pub(crate) fn scanner_observations(
        &self,
    ) -> Vec<(&salimon_universe::scanner::Encounter, [f64; 3])> {
        let ship = self.ship.snapshot();
        let body = salimon_world::canonical::START.manifest().planet.id.clone();
        let origin = Position::BodyLocal {
            body,
            meters: ship.pose.position_meters,
        };
        self.scanner
            .contacts
            .active(self.scanner.utc_millis)
            .filter_map(|e| {
                e.observation()
                    .position
                    .relative_meters(&origin)
                    .ok()
                    .map(|p| (e, p))
            })
            .collect()
    }
    pub(crate) fn scanner_geometry(
        &self,
        pose: salimon_ship::ShipPose,
    ) -> Vec<salimon_renderer::SceneInstance> {
        if self.surface_chart.open {
            return self.surface_geometry(pose);
        }
        if self.scanner.chart_mode {
            return self.chart_geometry(pose);
        }
        if self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical {
            return Vec::new();
        }
        let observations = self.scanner_observations();
        let index = self.scanner.selected.checked_rem(observations.len());
        let source = self.ship.snapshot().pose.axes();
        let contacts: Vec<_> = observations
            .iter()
            .enumerate()
            .map(|(i, (_, v))| salimon_renderer::HologramContact {
                relative_ship_meters: source.map(|axis| salimon_math::dot(*v, axis)),
                selected: Some(i) == index,
                state: salimon_renderer::HologramContactState::Fresh,
            })
            .collect();
        let frame = super::super::frames::character_ship_frame(pose);
        salimon_renderer::scanner_hologram(
            &contacts,
            self.scanner.powered,
            self.scanner.parameters.range_meters,
        )
        .into_iter()
        .map(|mut i| {
            i.center_meters = frame.local_to_world(i.center_meters);
            i.orientation = pose.orientation.map(|v| v as f32);
            i
        })
        .collect()
    }
    pub(crate) fn scanner_hint(&self) -> Option<String> {
        if !self.scanner_controls_available() {
            return None;
        }
        if self.scanner.chart_mode {
            return Some(self.chart_hint());
        }
        let contacts = self.scanner_observations();
        let index = self.scanner.selected.checked_rem(contacts.len());
        let selected = index.map(|i| contacts[i]);
        let name = if selected.is_some_and(|(e, _)| {
            e.observation().id == salimon_world::canonical::START.manifest().planet.id
        }) {
            "VESPER"
        } else {
            "CONTACT"
        };
        Some(
            salimon_renderer::ScannerReadout {
                powered: self.scanner.powered,
                selected: index.unwrap_or(0),
                contacts: contacts.len(),
                label: name,
                center_meters: selected.map(|(_, v)| v[0].hypot(v[1]).hypot(v[2])),
                age_seconds: selected.map(|(e, _)| {
                    (self.scanner.utc_millis - e.last_seen_utc_millis()) as f64 / 1000.
                }),
            }
            .text(),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_readout_uses_shared_face_bounded_rows_and_observed_state_only() {
        let mut s = GameplaySession::new_canonical_run().unwrap();
        s.character = Default::default();
        s.character.enter_cockpit();
        s.ship.set_cockpit_control(true);
        s.advance_automation(Duration::from_millis(16));
        assert!(s.scanner_hint().unwrap().contains("NO FRESH CONTACTS"));
        s.advance_automation(Duration::from_millis(5008));
        let saved = s.save_bytes(100).unwrap();
        let text = s.scanner_hint().unwrap();
        assert!(text.contains("FRESH VESPER"));
        assert!(text.contains(" km | AGE "));
        assert!(text.contains("[M] NEXT [P] CHART"));
        for scale in [2, 3] {
            let (width, height, pixels) = crate::action_bar::rasterize_action_bar(&text, scale);
            assert!(width <= 720 && height <= 100);
            assert!(
                pixels
                    .as_chunks::<4>()
                    .0
                    .contains(&salimon_renderer::ui_style::IVORY)
            );
        }
        assert_eq!(saved, s.save_bytes(100).unwrap());
        s.scanner.powered = false;
        assert!(s.scanner_hint().unwrap().contains("CORE POWER REQUIRED"));
        assert_eq!(s.scanner.contacts.history().count(), 1);
    }
    #[test]
    fn scanner_hologram_unknown_contact_pose_and_power_are_observational() {
        let mut s = GameplaySession::new_canonical_run().unwrap();
        assert_eq!(s.scanner_geometry(s.ship.snapshot().pose).len(), 2);
        s.advance_automation(Duration::from_millis(5008));
        let saved = s.save_bytes(100).unwrap();
        let pose = s.ship.snapshot().pose;
        assert!(s.scanner_geometry(pose).len() > 99);
        assert_eq!(s.save_bytes(100).unwrap(), saved);
        s.scanner.powered = false;
        assert_eq!(s.scanner_geometry(pose).len(), 2);
        assert!(!s.scanner_controls_available());
        s.scanner_next_contact();
        assert_eq!(s.scanner.selected, 0);
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;
    use crate::session::input::{Key, dispatch};
    #[test]
    fn scanner_selection_uses_cockpit_modal_pause_and_terminal_gates_without_discovery() {
        let mut s = GameplaySession::new_canonical_run().unwrap();
        s.character = Default::default();
        s.character.enter_cockpit();
        s.ship.set_cockpit_control(true);
        dispatch(&mut s, Key::ScannerNext, true);
        assert_eq!(s.scanner.selected, 1);
        assert_eq!(s.scanner.contacts.history().count(), 0);
        dispatch(&mut s, Key::ScannerNext, false);
        assert_eq!(s.scanner.selected, 1);
        s.command(crate::session::Command::TogglePause);
        dispatch(&mut s, Key::ScannerNext, true);
        assert_eq!(s.scanner.selected, 1);
        s.command(crate::session::Command::TogglePause);
        s.archive_reader.open = true;
        dispatch(&mut s, Key::ScannerNext, true);
        assert_eq!(s.scanner.selected, 1);
        s.archive_reader.open = false;
        s.character.leave_cockpit();
        dispatch(&mut s, Key::ScannerNext, true);
        assert_eq!(s.scanner.selected, 1);
        s.character.enter_cockpit();
        s.survival
            .command(salimon_survival::Command::Damage(
                salimon_survival::Points::FULL,
            ))
            .unwrap();
        dispatch(&mut s, Key::ScannerNext, true);
        assert_eq!(s.scanner.selected, 1);
    }
}
