//! Physical room switches and presentation mapping. No invented lighting wattage.
pub(super) mod save;
use super::{GameplaySession, frames::character_ship_frame, interaction::InteractionTarget};
use salimon_ship::lighting::{Setting, rgb};
#[derive(Clone, Copy, Debug)]
pub(crate) enum LightAction {
    Preset,
    Master,
    Fixture,
    Brightness,
    General,
    Warmth,
}
pub(crate) const POSITIONS: [[f32; 4]; 5] = [
    [-0.8, 2.64, -3.5, 0.9],
    [0.8, 2.64, -2.4, 0.9],
    [0.65, 2.64, 3.4, 0.8],
    [-3., 2.64, -3.4, 0.8],
    [-3., 2.64, 2.5, 0.5],
];
impl GameplaySession {
    fn light_target(&self) -> Option<(u64, usize)> {
        let s = self.snapshot();
        let f = character_ship_frame(s.ship.pose);
        let target = self.available_target(
            s.character.location,
            f.world_to_local(s.character.eye_position_meters),
            f.world_to_local(s.character.look_target_meters),
        )?;
        let room = if f.world_to_local(s.character.eye_position_meters)[0] < -4.65 {
            2
        } else {
            1
        };
        Some((
            room,
            match target {
                InteractionTarget::Bunk => 0,
                InteractionTarget::CabinTable | InteractionTarget::CabinSeat => 1,
                InteractionTarget::Galley => 2,
                InteractionTarget::Hygiene => 3,
                InteractionTarget::Core | InteractionTarget::Fabricator => 4,
                InteractionTarget::CargoDoor => 4,
                _ => return None,
            },
        ))
    }
    pub(crate) fn light_action(&mut self, action: LightAction) {
        if self.building.active
            || !self.admit_survival_action(salimon_survival::restrictions::Action::Menu)
        {
            return;
        }
        let Some((id, index)) = self.light_target() else {
            self.feedback = Some("Aim at a physical room or fixture light control");
            return;
        };
        let r = self
            .lighting
            .rooms
            .get_mut(&id)
            .expect("installed light room");
        match action {
            LightAction::Preset => r.next_preset(),
            LightAction::Master => r.master = !r.master,
            LightAction::Fixture => {
                let mut s = r.fixtures[index].unwrap_or(r.general);
                s.enabled = !s.enabled;
                r.fixtures[index] = Some(s);
            }
            LightAction::Brightness => {
                let mut s = r.fixtures[index].unwrap_or(r.general);
                s.brightness = if s.brightness >= 100 {
                    10
                } else {
                    (s.brightness + 10).min(100)
                };
                r.fixtures[index] = Some(s);
                r.preset = salimon_ship::lighting::Preset::Custom;
            }
            LightAction::General => {
                r.general.brightness = if r.general.brightness >= 100 {
                    10
                } else {
                    (r.general.brightness + 10).min(100)
                };
                r.preset = salimon_ship::lighting::Preset::Custom;
            }
            LightAction::Warmth => {
                r.general.warmth = if r.general.warmth >= 100 {
                    0
                } else {
                    (r.general.warmth + 10).min(100)
                };
                r.preset = salimon_ship::lighting::Preset::Custom;
            }
        }
        self.feedback =
            Some("Lighting preference saved; power and actual fixture condition take precedence");
    }
    pub(crate) fn light_hint(&self) -> Option<String> {
        let (id, index) = self.light_target()?;
        let r = &self.lighting.rooms[&id];
        let s = r.fixtures[index].unwrap_or(r.general);
        Some(format!(
            "Room {id} {:?} | Master {} | Fixture {}: {}%\nF4 preset | F5 master | F6 fixture | F7 brighter | F8 warmth | F9 room brighter",
            r.preset,
            if r.master { "On" } else { "Off" },
            index + 1,
            s.brightness
        ))
    }
    pub(crate) fn lighting_dto(&self) -> salimon_renderer::CabinLighting {
        let powered = self.ship.snapshot().power_available;
        let window = self
            .habitation
            .cabin
            .as_ref()
            .map_or(1., |c| c.motions[3].eased()) as f32;
        let natural = self.exterior_illumination() * window;
        let mut dto = salimon_renderer::CabinLighting::default();
        dto.positions[..5].copy_from_slice(&POSITIONS);
        dto.positions[5] = [0., 1.7, -3.7, 0.35];
        dto.colors[5] = [natural * 0.35, natural * 0.4, natural * 0.48, 0.];
        for room in 0..2 {
            let r = &self.lighting.rooms[&(room as u64 + 1)];
            let fill = rgb(Setting {
                enabled: r.master && powered && r.general.enabled,
                ..r.general
            });
            dto.ambient[room] = [
                0.012 + 0.35 * fill[0],
                0.016 + 0.35 * fill[1],
                0.022 + 0.35 * fill[2],
                1.,
            ];
        }
        for i in 0..5 {
            let color = rgb(self.lighting.rooms[&1].effective(i, powered, true));
            dto.colors[i] = [color[0] * 0.5, color[1] * 0.5, color[2] * 0.5, 0.];
        }
        dto
    }
    fn exterior_illumination(&self) -> f32 {
        let ship = self.ship.snapshot();
        let f = character_ship_frame(ship.pose);
        let origin = f.local_to_world([0., 1.7, -3.7]);
        let sun = salimon_world::CELESTIAL_BODIES[0].center.meters();
        let delta = salimon_math::sub(sun, origin);
        let distance = salimon_math::length(delta);
        let ray = salimon_math::scale(delta, 1. / distance);
        for body in &salimon_world::CELESTIAL_BODIES[1..] {
            let to = salimon_math::sub(body.center.meters(), origin);
            let t = salimon_math::dot(to, ray);
            if t > 0.
                && t < distance
                && salimon_math::length(salimon_math::sub(to, salimon_math::scale(ray, t)))
                    < body.radius_meters
            {
                return 0.;
            }
        }
        let normal = salimon_math::sub(f.local_to_world([0., 0., -1.]), f.local_to_world([0.; 3]));
        (0.15 + 0.85 * salimon_math::dot(ray, normal).max(0.)) as f32
    }
    pub(crate) fn light_inspect(&self) -> serde_json::Value {
        serde_json::json!({"source":self.lighting,"render":self.lighting_dto().ambient,"lamp_colors":self.lighting_dto().colors,"window_fraction":self.habitation.cabin.as_ref().map(|c|c.motions[3].fraction)})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use salimon_persistence::Validate;
    #[test]
    fn physical_controls_outage_save_restore_and_shutter_change_real_mapping() {
        let mut s = GameplaySession::new_run();
        super::super::cabin::evidence_initial_conditions(&mut s, "scout-finishes").unwrap();
        let mass = s.ship.snapshot().total_mass_kg;
        let energy = s.ship.snapshot().energy_core.stored_joules;
        s.light_action(LightAction::Preset);
        s.light_action(LightAction::Brightness);
        let source = s.lighting.clone();
        let lit = s.lighting_dto();
        s.ship.set_stored_core_energy_joules(0);
        let dark = s.lighting_dto();
        assert_ne!(lit.colors, dark.colors);
        assert_eq!(s.lighting, source);
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert_eq!(r.lighting, source);
        assert_eq!(r.ship.snapshot().total_mass_kg, mass);
        s.ship.set_stored_core_energy_joules(energy);
        assert_eq!(s.lighting_dto(), lit);
        s.habitation.cabin.as_mut().unwrap().motions[3].fraction = 0.;
        assert!(lit.colors[5][0] > 0.);
        assert_eq!(s.lighting_dto().colors[5][0], 0.);
        s.lighting.validate().unwrap();
    }
    #[test]
    fn distant_or_cockpit_input_cannot_change_preferences() {
        let mut s = GameplaySession::default();
        let old = s.lighting.clone();
        s.light_action(LightAction::Master);
        assert_eq!(s.lighting, old);
    }
}
