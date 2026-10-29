//! Contextual prompt priority and anchors; observation never grants interaction authority.
use super::super::interaction;
use super::super::{
    CharacterLocation, ClientApplication, InteractionTarget, OverlayPlacement, ViewMode,
    action_bar_context,
};
use crate::session::door_passable;

pub(super) struct PromptFrame {
    pub(super) monitor_message: Option<salimon_ship::CockpitMessage>,
    pub(super) interaction: Option<InteractionTarget>,
    pub(super) mining_target: bool,
    pub(super) contextual_placement: OverlayPlacement,
}

impl ClientApplication {
    pub(super) fn prepare_prompts(
        &mut self,
        character_snapshot: salimon_character::CharacterSnapshot,
        ship_snapshot: salimon_ship::ShipSnapshot,
        ship_frame: salimon_character::ShipFrame,
        render_ship_frame: salimon_character::ShipFrame,
        presentation: &crate::session::Presentation,
    ) -> PromptFrame {
        let monitor_message = self.session.contextual_cockpit_message();
        let interaction = if self.session.view_mode == ViewMode::Gameplay
            && character_snapshot.location != CharacterLocation::Cockpit
        {
            self.session
                .available_target(
                    character_snapshot.location,
                    ship_frame.world_to_local(character_snapshot.eye_position_meters),
                    ship_frame.world_to_local(character_snapshot.look_target_meters),
                )
                .filter(|target| self.session.permitted_interaction(*target))
        } else {
            None
        };
        let mining_target = self.session.mining.target(
            &self.session.equipment,
            character_snapshot,
            ship_frame,
            door_passable(ship_snapshot),
            self.world_seed(),
        );
        let fragment_interaction = !matches!(
            interaction,
            Some(
                InteractionTarget::ExitDoor
                    | InteractionTarget::Core
                    | InteractionTarget::Fabricator
            )
        ) && (self.session.mining.session.carried_object().is_some()
            || crate::carrying::target(
                &self.session.mining,
                character_snapshot,
                ship_frame,
                door_passable(ship_snapshot),
            )
            .is_some());
        let resource_context = crate::resource_context::context(
            &self.session.mining,
            &self.session.equipment,
            character_snapshot,
            ship_frame,
            door_passable(ship_snapshot),
            self.world_seed(),
        );
        let ship_context = action_bar_context(monitor_message, interaction).map(|text| {
            crate::resource_context::Prompt {
                fragment: None,
                text: text.to_owned(),
                placement: if monitor_message
                    .filter(|message| {
                        *message != salimon_ship::CockpitMessage::DoorLockedWhileInFlight
                    })
                    .is_some()
                {
                    OverlayPlacement::BottomCenter
                } else {
                    interaction.map_or(OverlayPlacement::BottomCenter, |target| {
                        self.session.cabin_interaction_anchor(target).map_or_else(
                            || interaction::prompt_placement(target, render_ship_frame),
                            |p| OverlayPlacement::World {
                                anchor_meters: render_ship_frame.local_to_world(p),
                                radius_meters: 0.15,
                            },
                        )
                    })
                },
            }
        });
        let contextual_action = if let Some(text) = self.session.construction_hint() {
            Some(crate::resource_context::Prompt {
                fragment: None,
                text,
                placement: OverlayPlacement::BottomCenter,
            })
        } else if let Some(text) = self.session.cabin_hint() {
            if matches!(
                interaction,
                Some(InteractionTarget::PrivacyScreen | InteractionTarget::ObservationShutter)
            ) {
                ship_context.or(resource_context)
            } else {
                Some(crate::resource_context::Prompt {
                    fragment: None,
                    text: text.to_owned(),
                    placement: OverlayPlacement::BottomCenter,
                })
            }
        } else if let Some(text) = self.session.scanner_hint() {
            Some(crate::resource_context::Prompt {
                fragment: None,
                text,
                placement: if self.session.scanner.chart_mode {
                    OverlayPlacement::BottomCenter
                } else {
                    scanner_readout_placement(
                        self.window.as_ref().map_or(0, |w| w.inner_size().width),
                    )
                },
            })
        } else if let Some(text) = self.session.core_upgrade_hint() {
            Some(crate::resource_context::Prompt {
                fragment: None,
                text,
                placement: OverlayPlacement::BottomCenter,
            })
        } else if let Some(text) = self.session.mount_prompt() {
            Some(crate::resource_context::Prompt {
                fragment: None,
                text: text.to_owned(),
                placement: OverlayPlacement::BottomCenter,
            })
        } else if let Some(text) = self
            .session
            .light_hint()
            .or_else(|| self.session.finish_hint())
        {
            Some(crate::resource_context::Prompt {
                fragment: None,
                text,
                placement: OverlayPlacement::BottomCenter,
            })
        } else if fragment_interaction {
            resource_context.or(ship_context)
        } else {
            ship_context.or(resource_context)
        };
        let mut contextual_placement = contextual_action
            .as_ref()
            .map_or(OverlayPlacement::BottomCenter, |prompt| prompt.placement);
        if let OverlayPlacement::World { radius_meters, .. } = contextual_placement
            && let Some(fragment) = contextual_action
                .as_ref()
                .and_then(|prompt| prompt.fragment)
                .and_then(|id| self.session.mining.session.fragment(id))
        {
            contextual_placement = OverlayPlacement::World {
                anchor_meters: presentation
                    .fragment_mesh(&self.session, *fragment)
                    .center_meters,
                radius_meters,
            };
        }
        if self.session.survival.snapshot().terminal {
            contextual_placement = OverlayPlacement::BottomCenter;
        }
        self.action_bar.set_combined_contextual(
            self.session.building.active || interaction == Some(InteractionTarget::Core),
        );
        self.action_bar
            .set_contextual(if self.session.survival.snapshot().terminal {
                None
            } else {
                contextual_action.map(|prompt| prompt.text).or_else(|| {
                    (!self.survival_hud.details_open && !self.diagnostics.is_visible())
                        .then(|| self.session.tutorial_text())
                        .flatten()
                })
            });
        PromptFrame {
            monitor_message,
            interaction,
            mining_target: mining_target.is_some(),
            contextual_placement,
        }
    }
}

fn scanner_readout_placement(width: u32) -> OverlayPlacement {
    // At 1280+ px even the 39-cell/3x face and maximum compact survival HUD
    // fit side by side. Keep the center clear when the 2x toolbar grows upward.
    if width >= 1280 {
        OverlayPlacement::TopRight
    } else {
        OverlayPlacement::BottomCenter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_readout_reserves_the_center_at_wide_and_high_dpi_viewports() {
        for width in [1280, 1920, 2560] {
            assert_eq!(scanner_readout_placement(width), OverlayPlacement::TopRight);
        }
        for width in [640, 960, 1279] {
            assert_eq!(
                scanner_readout_placement(width),
                OverlayPlacement::BottomCenter
            );
        }
    }
}
