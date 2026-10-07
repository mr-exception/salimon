mod diagnostics;
mod frames;
mod input;
mod interaction;
mod native;
mod scene;

use std::error::Error;
use std::fmt;
use std::io::Write;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use salimon_character::{CharacterController, CharacterLocation, MovementInput};
use salimon_diagnostics::Diagnostics;
use salimon_renderer::{
    CameraFrame, OverlayImage as RendererOverlayImage, OverlayPlacement, RenderOutcome, Renderer,
    SceneFrame,
};
use salimon_ship::{CockpitMessage, DoorState, FlightState, ShipController, ShipSnapshot};
use salimon_world::{CameraPrototype, TransitionPhase};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use diagnostics::{
    camera_body_distances_from, camera_domain_metrics, format_metric_speed, gameplay_domain_metrics,
};
pub(crate) use frames::{character_ship_frame, nearby_surface_at, surface_frame_for_ship};
use input::{
    ShipControlInput, camera_command, interaction_pressed, is_diagnostics_toggle, is_movement_key,
    is_ship_control_key, landing_action_pressed, release_cursor_pressed, thruster_step,
    tool_key_event, toolbar_selection_pressed, toolbar_slot, update_movement_input,
    update_ship_control_input, view_toggle_pressed,
};
use interaction::{
    InteractionTarget, action_bar_context, available_interaction_target, gameplay_window_title,
};
use native::{capture_cursor, release_cursor, surface_size, window_attributes};
use scene::{map_ship_to_renderer, map_world_to_renderer};

use crate::action_bar::ActionBar;
use crate::automation::{self, Request};
use crate::e2e;
use crate::frame_clock::FrameClock;
use crate::update_clock::UpdateClock;

const TIMING_LOG_INTERVAL: u64 = 300;
const RENDER_RETRY_DELAY: Duration = Duration::from_millis(50);
const IDLE_RETRY_DELAY: Duration = Duration::from_millis(250);

pub(crate) fn run(config: Option<e2e::Config>) -> Result<(), RunError> {
    let event_loop = EventLoop::new()
        .map_err(|error| RunError::new("failed to create native event loop", error))?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut application = ClientApplication::default();
    if let Some(config) = config {
        e2e::initialize(&mut application, config)
            .map_err(|error| RunError::new("E2E setup failed", error))?;
    }
    event_loop
        .run_app(&mut application)
        .map_err(|error| RunError::new("native event loop failed", error))?;

    match application.failure {
        Some(failure) => Err(RunError::new("application lifecycle failed", failure)),
        None => Ok(()),
    }
}

#[derive(Debug)]
pub(crate) struct RunError {
    context: &'static str,
    detail: String,
}

impl RunError {
    fn new(context: &'static str, detail: impl fmt::Display) -> Self {
        Self {
            context,
            detail: detail.to_string(),
        }
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for RunError {}

pub(crate) struct ClientApplication {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_clock: FrameClock,
    pub(crate) update_clock: UpdateClock,
    camera_prototype: CameraPrototype,
    pub(crate) character: CharacterController,
    pub(crate) ship: ShipController,
    pub(crate) e2e_config: Option<e2e::Config>,
    e2e_ready_sent: bool,
    automation: Option<Receiver<Request>>,
    pub(crate) mining: crate::mining::MiningTool,
    pub(crate) equipment: crate::equipment::EquipmentToolbar,
    movement_input: MovementInput,
    ship_control_input: ShipControlInput,
    view_mode: ViewMode,
    cursor_captured: bool,
    diagnostics: Diagnostics,
    action_bar: ActionBar,
    render_attempts: u64,
    retry_at: Option<Instant>,
    occluded: bool,
    failure: Option<String>,
}

impl Default for ClientApplication {
    fn default() -> Self {
        Self {
            window: None,
            renderer: None,
            frame_clock: FrameClock::default(),
            update_clock: UpdateClock::default(),
            camera_prototype: CameraPrototype::default(),
            character: CharacterController::default(),
            ship: ShipController::default(),
            e2e_config: None,
            e2e_ready_sent: false,
            automation: None,
            mining: crate::mining::MiningTool::default(),
            equipment: crate::equipment::EquipmentToolbar::default(),
            movement_input: MovementInput::default(),
            ship_control_input: ShipControlInput::default(),
            view_mode: ViewMode::Gameplay,
            cursor_captured: false,
            diagnostics: Diagnostics::default(),
            action_bar: ActionBar::default(),
            render_attempts: 0,
            retry_at: None,
            occluded: false,
            failure: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ViewMode {
    #[default]
    Gameplay,
    PrecisionTour,
}

fn door_passable(ship: ShipSnapshot) -> bool {
    ship.door_state == DoorState::Open && ship.door_open_fraction >= 0.95
}

impl ClientApplication {
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), RunError> {
        if self.window.is_some() {
            return Ok(());
        }

        let attributes = window_attributes(self.e2e_config.is_some());
        let window = event_loop
            .create_window(attributes)
            .map_err(|error| RunError::new("failed to create native window", error))?;
        window.set_visible(true);
        window.focus_window();
        self.cursor_captured = capture_cursor(&window);
        let size = window.inner_size();
        log::info!(
            "native window created: physical_size={}x{} scale_factor={:.2}",
            size.width,
            size.height,
            window.scale_factor()
        );
        self.diagnostics.set_scale_factor(window.scale_factor());
        self.action_bar.set_scale_factor(window.scale_factor());
        self.window = Some(Arc::new(window));
        Ok(())
    }

    fn initialize_renderer(&mut self) -> Result<(), RunError> {
        let window =
            self.window.as_ref().cloned().ok_or_else(|| {
                RunError::new("failed to initialize renderer", "window is missing")
            })?;
        let size = surface_size(&window);
        let renderer = pollster::block_on(Renderer::new(window, size))
            .map_err(|error| RunError::new("failed to initialize renderer", error))?;
        let info = renderer.info();
        log::info!(
            "renderer ready: adapter=\"{}\" backend={} device_type={} gpu_timestamps={}",
            info.adapter_name,
            info.backend,
            info.device_type,
            info.timestamp_queries_supported
        );
        self.renderer = Some(renderer);
        if let Some(config) = self.e2e_config.filter(|_| !self.e2e_ready_sent) {
            println!(
                "SALIMON_E2E_READY scenario={} seed={} step_ms={}",
                config.scenario.name(),
                config.seed,
                config.step.as_millis()
            );
            std::io::stdout()
                .flush()
                .map_err(|error| RunError::new("failed to emit E2E ready signal", error))?;
            self.e2e_ready_sent = true;
            automation::ready(config.scenario.name(), config.seed, config.step.as_millis());
            self.automation = Some(automation::start());
        }
        self.render_attempts = 0;
        self.retry_at = None;
        self.frame_clock.reset_interval();
        self.update_clock.reset();
        self.diagnostics.reset_frame_window();
        Ok(())
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl fmt::Display) {
        let message = error.to_string();
        log::error!("{message}");
        if self.failure.is_none() {
            self.failure = Some(message);
        }
        event_loop.exit();
    }

    fn resize_renderer(&mut self, event_loop: &ActiveEventLoop, window: &Window) {
        let size = surface_size(window);
        log::debug!("native drawable resized: {size:?}");
        self.retry_at = None;
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize(size);
        }
        if size.is_drawable() && !self.occluded {
            event_loop.set_control_flow(ControlFlow::Wait);
            window.request_redraw();
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
            self.frame_clock.reset_interval();
            self.update_clock.reset();
            self.diagnostics.reset_frame_window();
        }
    }

    fn schedule_retry(&mut self, event_loop: &ActiveEventLoop, delay: Duration) {
        let retry_at = Instant::now() + delay;
        self.retry_at = Some(retry_at);
        event_loop.set_control_flow(ControlFlow::WaitUntil(retry_at));
    }

    pub(crate) fn interact(&mut self) {
        let ship_snapshot = self.ship.snapshot();
        let ship_frame = character_ship_frame(ship_snapshot.pose);
        let surface = surface_frame_for_ship(ship_snapshot);
        match self.character.location() {
            CharacterLocation::Cockpit => {
                self.character.leave_cockpit();
                self.ship.set_cockpit_control(false);
                self.ship_control_input = ShipControlInput::default();
                log::info!("left cockpit control; ship motion remains autonomous");
            }
            CharacterLocation::InsideShip
            | CharacterLocation::DoorwayBlend
            | CharacterLocation::Surface
            | CharacterLocation::Space
            | CharacterLocation::NearbyBody => {
                let character = self.character.snapshot(ship_frame, surface);
                let local_eye = ship_frame.world_to_local(character.eye_position_meters);
                let local_look = ship_frame.world_to_local(character.look_target_meters);
                let interaction =
                    available_interaction_target(self.character.location(), local_eye, local_look);
                match interaction {
                    Some(InteractionTarget::Cockpit) => {
                        self.character.enter_cockpit();
                        if self.character.location() == CharacterLocation::Cockpit {
                            self.ship.set_cockpit_control(true);
                            log::info!("entered cockpit control");
                        }
                    }
                    Some(InteractionTarget::ExitDoor) => {
                        self.ship.toggle_door();
                        if let Some(message) = self.ship.snapshot().cockpit_message {
                            if message == CockpitMessage::DoorLockedWhileInFlight {
                                self.action_bar.show_transient(message.text());
                            }
                            log::info!("cockpit monitor: {}", message.text());
                        } else {
                            log::info!("exit door is now {:?}", self.ship.snapshot().door_state);
                        }
                    }
                    None => {}
                }
            }
        }
    }

    pub(crate) fn e2e_step(&self) -> Option<Duration> {
        self.e2e_config.map(|config| config.step)
    }

    pub(crate) fn camera_phase(&self) -> TransitionPhase {
        self.camera_prototype.snapshot().transition_phase
    }

    pub(crate) fn automation_interaction(
        &self,
        local_eye: [f64; 3],
        local_look: [f64; 3],
    ) -> Option<&'static str> {
        match available_interaction_target(self.character.location(), local_eye, local_look) {
            Some(InteractionTarget::Cockpit) => Some("cockpit"),
            Some(InteractionTarget::ExitDoor) => Some("exit_door"),
            None => None,
        }
    }

    pub(crate) fn automation_key(&mut self, key: PhysicalKey, pressed: bool) {
        if self.view_mode != ViewMode::Gameplay {
            if let Some(command) = camera_command(
                if pressed {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                },
                false,
                key,
                true,
            ) {
                self.camera_prototype.apply_command(command);
            }
            return;
        }
        if let Some(slot) = toolbar_slot(key) {
            if pressed {
                let previous = self.equipment.selected();
                self.equipment
                    .select(slot, self.mining.session.carried_id().is_some());
                if previous != self.equipment.selected() {
                    self.mining.clear_input();
                }
            }
            return;
        }
        if key == PhysicalKey::Code(KeyCode::KeyF) {
            if !pressed {
                self.mining.release_f();
            } else if self.mining.press_f() {
                let ship = self.ship.snapshot();
                let frame = character_ship_frame(ship.pose);
                let player = self.character.snapshot(frame, surface_frame_for_ship(ship));
                let pickup = self.mining.session.carried_id().is_none();
                if !pickup
                    || crate::carrying::target(&self.mining, player, frame, door_passable(ship))
                        .is_some()
                {
                    self.carry_action(pickup);
                } else if self.equipment.mining_equipped() {
                    self.mining.start_f_mining();
                }
            }
            return;
        }
        if self.character.location() == CharacterLocation::Cockpit && is_ship_control_key(key) {
            update_ship_control_input(&mut self.ship_control_input, key, pressed);
        } else {
            update_movement_input(&mut self.movement_input, key, pressed);
        }
    }

    fn carry_action(&mut self, pickup: bool) {
        let ship = self.ship.snapshot();
        let frame = character_ship_frame(ship.pose);
        let player = self.character.snapshot(frame, surface_frame_for_ship(ship));
        let door_open = door_passable(ship);
        let message = if pickup {
            match crate::carrying::target(&self.mining, player, frame, door_open) {
                Some(id) if self.mining.session.pick_up(id) => {
                    self.equipment.sync_carrying(true);
                    self.mining.stop_mining();
                    self.mining.fragment_motion.remove(&id);
                    self.mining.ship_fragments.remove(&id);
                    "Fragment picked up - F to drop"
                }
                Some(_) => "Only one world object - F to drop first",
                None => "Aim at a fragment within 3 m",
            }
        } else if self.mining.session.carried_id().is_none() {
            "No world object carried"
        } else if let Some(pose) = crate::carrying::release_pose(&self.mining, player) {
            let id = self
                .mining
                .session
                .carried_id()
                .expect("drop has carried identity");
            self.mining.session.drop_carried(pose);
            if player.location == CharacterLocation::InsideShip {
                self.mining
                    .ship_fragments
                    .insert(id, frame.world_to_local(pose.position().meters()));
            } else {
                self.mining.ship_fragments.remove(&id);
            }
            let forward = std::array::from_fn(|i| {
                player.look_target_meters[i] - player.eye_position_meters[i]
            });
            let length = forward.iter().map(|v| v * v).sum::<f64>().sqrt();
            let forward = forward.map(|v| v / length);
            crate::fragment_physics::release(
                &mut self.mining,
                id,
                forward,
                frame,
                player.location == CharacterLocation::InsideShip,
            );
            "Fragment dropped"
        } else {
            "Cannot drop fragment here"
        };
        crate::carrying::sync_ship_fragments(&mut self.mining, frame);
        crate::carrying::follow(&mut self.mining, player);
        self.mining.carry_feedback = Some(message);
        self.action_bar.show_transient(message);
    }

    pub(crate) fn sync_carried(&mut self) {
        let ship = self.ship.snapshot();
        let frame = character_ship_frame(ship.pose);
        crate::carrying::sync_ship_fragments(&mut self.mining, frame);
        crate::carrying::follow(
            &mut self.mining,
            self.character.snapshot(frame, surface_frame_for_ship(ship)),
        );
    }

    /// The same portable gameplay update runs on redraw and on explicit E2E steps.
    pub(crate) fn advance_game(&mut self, delta: Duration) {
        self.camera_prototype.advance(delta);
        self.ship
            .set_steering_input(self.ship_control_input.steering());
        self.ship.advance(delta);
        let ship = self.ship.snapshot();
        if self.view_mode == ViewMode::Gameplay {
            let eye = self
                .character
                .snapshot(
                    character_ship_frame(ship.pose),
                    surface_frame_for_ship(ship),
                )
                .eye_position_meters;
            self.character.set_nearby_surface(nearby_surface_at(eye));
            self.character.advance_with_motion(
                delta,
                self.movement_input,
                (
                    character_ship_frame(ship.pose),
                    ship.velocity_meters_per_second,
                ),
                surface_frame_for_ship(ship),
                door_passable(ship),
                matches!(ship.flight_state, FlightState::Landed { .. }),
            );
            let eye = self
                .character
                .snapshot(
                    character_ship_frame(ship.pose),
                    surface_frame_for_ship(ship),
                )
                .eye_position_meters;
            self.character.set_nearby_surface(nearby_surface_at(eye));
        }
        if self.view_mode == ViewMode::Gameplay {
            let frame = character_ship_frame(ship.pose);
            self.mining.advance(
                &self.equipment,
                delta,
                self.character.snapshot(frame, surface_frame_for_ship(ship)),
                frame,
                door_passable(ship),
                self.e2e_config.map_or(0, |config| config.seed),
            );
            crate::fragment_physics::advance(
                &mut self.mining,
                frame,
                delta,
                self.character
                    .snapshot(frame, surface_frame_for_ship(ship))
                    .eye_position_meters,
            );
        }
        self.sync_carried();
        self.action_bar.advance(delta);
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop, window: Arc<Window>) {
        if self.occluded {
            self.frame_clock.reset_interval();
            self.update_clock.reset();
            self.diagnostics.reset_frame_window();
            return;
        }

        self.render_attempts = self.render_attempts.saturating_add(1);
        let render_started_at = Instant::now();
        let update_started_at = Instant::now();
        let update_delta = if self.automation.is_some() {
            Duration::ZERO
        } else {
            self.update_clock.step(update_started_at)
        };
        if !update_delta.is_zero() {
            self.advance_game(update_delta);
        }
        self.sync_carried();
        let ship_snapshot = self.ship.snapshot();
        let ship_frame = character_ship_frame(ship_snapshot.pose);
        let surface_frame = surface_frame_for_ship(ship_snapshot);
        let world_snapshot = self.camera_prototype.snapshot();
        let (mut camera, scene_instances, spheres, light) = map_world_to_renderer(world_snapshot);
        let mut resource_meshes = Vec::new();
        let character_snapshot = self.character.snapshot(ship_frame, surface_frame);
        let monitor_message = self.ship.contextual_cockpit_message();
        let interaction = if self.view_mode == ViewMode::Gameplay
            && character_snapshot.location != CharacterLocation::Cockpit
        {
            available_interaction_target(
                character_snapshot.location,
                ship_frame.world_to_local(character_snapshot.eye_position_meters),
                ship_frame.world_to_local(character_snapshot.look_target_meters),
            )
        } else {
            None
        };
        let mining_target = self.mining.target(
            &self.equipment,
            character_snapshot,
            ship_frame,
            door_passable(ship_snapshot),
            self.e2e_config.map_or(0, |config| config.seed),
        );
        let fragment_interaction = interaction != Some(InteractionTarget::ExitDoor)
            && (self.mining.session.carried_id().is_some()
                || crate::carrying::target(
                    &self.mining,
                    character_snapshot,
                    ship_frame,
                    door_passable(ship_snapshot),
                )
                .is_some());
        let resource_context = crate::resource_context::context(
            &self.mining,
            &self.equipment,
            character_snapshot,
            ship_frame,
            door_passable(ship_snapshot),
            self.e2e_config.map_or(0, |config| config.seed),
        );
        let ship_context = action_bar_context(monitor_message, interaction).map(|text| {
            crate::resource_context::Prompt {
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
                        interaction::prompt_placement(target, ship_frame)
                    })
                },
            }
        });
        let contextual_action = if fragment_interaction {
            resource_context.or(ship_context)
        } else {
            ship_context.or(resource_context)
        };
        let contextual_placement = contextual_action
            .as_ref()
            .map_or(OverlayPlacement::BottomCenter, |prompt| prompt.placement);
        self.action_bar
            .set_contextual(contextual_action.map(|prompt| prompt.text));
        window.set_title(&gameplay_window_title(monitor_message, interaction));
        let ship_mesh = if self.view_mode == ViewMode::Gameplay {
            camera = CameraFrame {
                position_meters: character_snapshot.eye_position_meters,
                target_meters: character_snapshot.look_target_meters,
                up: character_snapshot.up,
                vertical_fov_radians: 70.0_f32.to_radians(),
                near_plane_meters: 0.05,
            };
            Some(map_ship_to_renderer(ship_snapshot))
        } else {
            None
        };
        if self.view_mode == ViewMode::Gameplay {
            match self.mining.nearby(
                camera.position_meters,
                self.e2e_config.map_or(0, |config| config.seed),
            ) {
                Ok(deposits) => {
                    resource_meshes.extend(
                        deposits
                            .iter()
                            .copied()
                            .filter_map(crate::resource_presentation::deposit_mesh),
                    );
                }
                Err(error) => log::error!("deposit presentation query failed: {error:?}"),
            }
        }
        if self.view_mode == ViewMode::Gameplay {
            for fragment in self.mining.nearby_fragments(camera.position_meters) {
                resource_meshes.push(crate::resource_presentation::fragment_mesh(fragment));
            }
        }
        let body_distances = camera_body_distances_from(world_snapshot, camera.position_meters);
        let cpu_update_time = update_started_at.elapsed();
        let overlay_image = self
            .diagnostics
            .overlay()
            .map(|image| RendererOverlayImage {
                width: image.width,
                height: image.height,
                rgba8: image.rgba8,
                revision: image.revision,
                placement: OverlayPlacement::TopLeft,
            });
        let action_bar_image = self
            .action_bar
            .image(self.view_mode == ViewMode::Gameplay)
            .map(|image| RendererOverlayImage {
                width: image.width,
                height: image.height,
                rgba8: image.rgba8,
                revision: image.revision,
                placement: if self.action_bar.has_transient() {
                    OverlayPlacement::BottomCenter
                } else {
                    contextual_placement
                },
            });
        let outcome = match self
            .renderer
            .as_mut()
            .expect("redraws require an initialized renderer")
            .render(
                SceneFrame {
                    camera,
                    instances: &scene_instances,
                    resource_meshes: &resource_meshes,
                    spheres: &spheres,
                    light: Some(light),
                    ship: ship_mesh,
                    held_item: if self.view_mode == ViewMode::Gameplay {
                        self.mining.held_item(
                            &self.equipment,
                            character_snapshot,
                            mining_target.is_some(),
                        )
                    } else {
                        None
                    },
                },
                overlay_image,
                action_bar_image,
                crate::reticle::image(self.view_mode == ViewMode::Gameplay, &self.equipment),
                (self.view_mode == ViewMode::Gameplay).then(|| self.equipment.presentation()),
                window.scale_factor(),
                || window.pre_present_notify(),
            ) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.fail(event_loop, error);
                return;
            }
        };
        if self.render_attempts == 1
            || (!matches!(outcome, RenderOutcome::Presented(_))
                && self.render_attempts.is_multiple_of(100))
        {
            log::debug!(
                "render attempt {} completed with {outcome:?}",
                self.render_attempts
            );
        }

        match outcome {
            RenderOutcome::Presented(render_stats) => {
                self.retry_at = None;
                event_loop.set_control_flow(ControlFlow::Wait);
                let timing = self
                    .frame_clock
                    .record_presented(render_started_at, Instant::now());
                self.diagnostics.record_presented(
                    diagnostics::frame_sample(timing, render_stats, cpu_update_time),
                    if self.view_mode == ViewMode::Gameplay {
                        gameplay_domain_metrics(
                            character_snapshot.eye_position_meters,
                            ship_snapshot,
                            &body_distances,
                        )
                    } else {
                        camera_domain_metrics(world_snapshot, &body_distances)
                    },
                );
                if timing.frame_number == 1
                    || timing.frame_number.is_multiple_of(TIMING_LOG_INTERVAL)
                {
                    log::debug!(
                        target: "salimon_client::frame",
                        "frame={} interval_ms={:.3} present_wall_ms={:.3} cpu_update_ms={:.3} cpu_render_ms={:.3} gpu_time={:?}",
                        timing.frame_number,
                        timing.frame_interval.as_secs_f64() * 1_000.0,
                        timing.submission_wall_time.as_secs_f64() * 1_000.0,
                        cpu_update_time.as_secs_f64() * 1_000.0,
                        render_stats.cpu_render_time.as_secs_f64() * 1_000.0,
                        render_stats.gpu_frame_time
                    );
                }
                window.request_redraw();
            }
            RenderOutcome::Retry => self.schedule_retry(event_loop, RENDER_RETRY_DELAY),
            RenderOutcome::Idle => {
                self.frame_clock.reset_interval();
                self.update_clock.reset();
                self.diagnostics.reset_frame_window();
                if surface_size(&window).is_drawable() {
                    self.schedule_retry(event_loop, IDLE_RETRY_DELAY);
                } else {
                    self.retry_at = None;
                    event_loop.set_control_flow(ControlFlow::Wait);
                }
            }
            RenderOutcome::SurfaceLost => {
                log::warn!("rendering surface was lost; rebuilding renderer");
                self.renderer = None;
                if let Err(error) = self.initialize_renderer() {
                    self.fail(event_loop, error);
                    return;
                }
                self.schedule_retry(event_loop, RENDER_RETRY_DELAY);
            }
        }
    }
}

impl ApplicationHandler for ClientApplication {
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if self.view_mode == ViewMode::Gameplay
            && self.cursor_captured
            && let DeviceEvent::MouseMotion { delta } = event
        {
            self.character.apply_mouse_delta(delta.0, delta.1);
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.occluded = false;
        self.retry_at = None;
        event_loop.set_control_flow(ControlFlow::Wait);
        if let Err(error) = self.create_window(event_loop) {
            self.fail(event_loop, error);
            return;
        }
        if self.renderer.is_none()
            && let Err(error) = self.initialize_renderer()
        {
            self.fail(event_loop, error);
            return;
        }

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        log::info!("application resumed");
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
        self.renderer = None;
        self.retry_at = None;
        self.frame_clock.reset_interval();
        self.update_clock.reset();
        self.diagnostics.reset_frame_window();
        self.mining.clear_input();
        log::info!("application suspended; GPU presentation resources released");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref().cloned() else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Destroyed => {
                self.renderer = None;
                self.window = None;
                self.retry_at = None;
                self.frame_clock.reset_interval();
                self.update_clock.reset();
                self.diagnostics.reset_frame_window();
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                self.resize_renderer(event_loop, &window);
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                self.diagnostics.set_scale_factor(window.scale_factor());
                self.action_bar.set_scale_factor(window.scale_factor());
                self.resize_renderer(event_loop, &window);
            }
            WindowEvent::Occluded(is_occluded) => {
                log::debug!("native window occluded: {is_occluded}");
                self.occluded = is_occluded;
                self.retry_at = None;
                self.frame_clock.reset_interval();
                self.update_clock.reset();
                self.diagnostics.reset_frame_window();
                if is_occluded {
                    event_loop.set_control_flow(ControlFlow::Wait);
                } else {
                    event_loop.set_control_flow(ControlFlow::Wait);
                    window.request_redraw();
                }
            }
            WindowEvent::Focused(focused) => {
                if focused {
                    self.cursor_captured = capture_cursor(&window);
                } else {
                    self.movement_input = MovementInput::default();
                    self.ship_control_input = ShipControlInput::default();
                    self.cursor_captured = false;
                    self.mining.clear_input();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } if !self.cursor_captured => {
                self.cursor_captured = capture_cursor(&window);
            }
            WindowEvent::KeyboardInput { event, .. }
                if release_cursor_pressed(event.state, event.repeat, event.physical_key) =>
            {
                release_cursor(&window);
                self.cursor_captured = false;
                self.movement_input = MovementInput::default();
                self.ship_control_input = ShipControlInput::default();
                self.mining.clear_input();
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mining.set_mouse_held(
                    self.cursor_captured
                        && self.equipment.mining_equipped()
                        && state == ElementState::Pressed,
                );
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } if event.physical_key == PhysicalKey::Code(KeyCode::KeyF) => {
                if self.cursor_captured
                    && tool_key_event(event.repeat, is_synthetic, event.physical_key)
                {
                    self.automation_key(event.physical_key, event.state == ElementState::Pressed);
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } if self.view_mode == ViewMode::Gameplay
                && toolbar_slot(event.physical_key).is_some() =>
            {
                if self.cursor_captured
                    && toolbar_selection_pressed(
                        event.state,
                        event.repeat,
                        event.physical_key,
                        is_synthetic,
                    )
                {
                    self.automation_key(event.physical_key, true);
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if interaction_pressed(event.state, event.repeat, event.physical_key) =>
            {
                self.interact();
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if landing_action_pressed(event.state, event.repeat, event.physical_key) =>
            {
                self.ship.trigger_landing_action();
                if let Some(message) = self.ship.contextual_cockpit_message() {
                    log::info!("cockpit monitor: {}", message.text());
                }
                window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } if view_toggle_pressed(
                event.state,
                event.repeat,
                event.physical_key,
                is_synthetic,
            ) =>
            {
                self.view_mode = match self.view_mode {
                    ViewMode::Gameplay => ViewMode::PrecisionTour,
                    ViewMode::PrecisionTour => ViewMode::Gameplay,
                };
                self.mining.clear_input();
                log::info!("view mode changed to {:?} (F2 toggles)", self.view_mode);
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if self.character.location() == CharacterLocation::Cockpit
                    && is_ship_control_key(event.physical_key) =>
            {
                update_ship_control_input(
                    &mut self.ship_control_input,
                    event.physical_key,
                    event.state == ElementState::Pressed,
                );
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if self.character.location() == CharacterLocation::Cockpit
                    && thruster_step(event.state, event.repeat, event.physical_key).is_some() =>
            {
                self.ship.adjust_thruster(
                    thruster_step(event.state, event.repeat, event.physical_key)
                        .expect("guard accepts only thruster steps"),
                );
                let snapshot = self.ship.snapshot();
                log::info!(
                    "cockpit monitor: {} / {}%",
                    format_metric_speed(snapshot.speed_meters_per_second),
                    snapshot.thruster_percentage
                );
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } if is_movement_key(event.physical_key) => {
                update_movement_input(
                    &mut self.movement_input,
                    event.physical_key,
                    event.state == ElementState::Pressed,
                );
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if is_diagnostics_toggle(event.state, event.repeat, event.physical_key) =>
            {
                let visible = self.diagnostics.toggle();
                log::info!(
                    "engineering diagnostics overlay {} (F3 toggles)",
                    if visible { "visible" } else { "hidden" }
                );
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if camera_command(
                    event.state,
                    event.repeat,
                    event.physical_key,
                    self.view_mode == ViewMode::PrecisionTour,
                )
                .is_some() =>
            {
                let command = camera_command(
                    event.state,
                    event.repeat,
                    event.physical_key,
                    self.view_mode == ViewMode::PrecisionTour,
                )
                .expect("guard accepts only camera commands");
                self.camera_prototype.apply_command(command);
                self.mining.clear_input();
                self.view_mode = ViewMode::PrecisionTour;
                log::info!(
                    "camera tour command: {command:?}; paused={}",
                    self.camera_prototype.is_paused()
                );
                window.request_redraw();
            }
            WindowEvent::RedrawRequested if self.renderer.is_some() => {
                self.redraw(event_loop, window);
            }
            _ => {}
        }
    }

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        self.diagnostics.record_memory_warning();
        log::warn!("the operating system reported memory pressure");
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.automation.is_some() {
            // Drain the queue on the native thread; the stdin thread never touches state.
            while let Some(request) = self
                .automation
                .as_ref()
                .and_then(|receiver| receiver.try_recv().ok())
            {
                if Instant::now() <= request.deadline {
                    let response = automation::execute(self, &request.line);
                    let _ = request.reply.send(response);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            if self.retry_at.is_some_and(|at| Instant::now() >= at) {
                self.retry_at = None;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(10),
            ));
            return;
        }
        let Some(retry_at) = self.retry_at else {
            return;
        };

        if Instant::now() >= retry_at {
            self.retry_at = None;
            event_loop.set_control_flow(ControlFlow::Wait);
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(retry_at));
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.renderer = None;
        self.window = None;
        log::info!("application exiting cleanly");
    }
}

#[cfg(test)]
mod toolbar_tests {
    use super::*;
    use crate::equipment::ToolbarSlot;
    use salimon_world::{CameraCommand, CelestialBodyId};

    #[test]
    fn numeric_inputs_select_gameplay_slots_without_changing_the_tour() {
        let mut app = ClientApplication::default();
        let initial_camera = app.camera_prototype.snapshot().camera;
        for (key, slot) in [
            (KeyCode::Digit1, ToolbarSlot::One),
            (KeyCode::Digit2, ToolbarSlot::Two),
            (KeyCode::Digit3, ToolbarSlot::Three),
            (KeyCode::Digit4, ToolbarSlot::Four),
            (KeyCode::Digit5, ToolbarSlot::Five),
        ] {
            app.automation_key(PhysicalKey::Code(key), true);
            app.automation_key(PhysicalKey::Code(key), false);
            assert_eq!(app.equipment.selected(), Some(slot));
            assert_eq!(app.view_mode, ViewMode::Gameplay);
            assert_eq!(app.camera_prototype.snapshot().camera, initial_camera);
        }
    }

    #[test]
    fn precision_tour_numbers_leave_equipment_selection_intact() {
        let mut app = ClientApplication::default();
        app.automation_key(PhysicalKey::Code(KeyCode::Digit4), true);
        app.view_mode = ViewMode::PrecisionTour;
        for (key, body) in [
            (KeyCode::Digit1, CelestialBodyId::Sun),
            (KeyCode::Digit2, CelestialBodyId::Mercury),
            (KeyCode::Digit3, CelestialBodyId::Venus),
            (KeyCode::Digit4, CelestialBodyId::Earth),
            (KeyCode::Digit5, CelestialBodyId::Moon),
            (KeyCode::Digit6, CelestialBodyId::Mars),
        ] {
            let mut expected = CameraPrototype::default();
            expected.apply_command(CameraCommand::InspectBody(body));
            app.automation_key(PhysicalKey::Code(key), true);
            assert_eq!(
                app.camera_prototype.snapshot().camera,
                expected.snapshot().camera
            );
            assert_eq!(app.equipment.selected(), Some(ToolbarSlot::Four));
        }
    }
}
