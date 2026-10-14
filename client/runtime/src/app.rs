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
    camera_body_distances_with_terrain, camera_domain_metrics, format_metric_speed,
    gameplay_domain_metrics,
};
pub(crate) use frames::{character_ship_frame, surface_frame_for_ship};
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
    run_config(config, None)
}

pub(crate) fn run_benchmark(config: crate::benchmark::Config) -> Result<(), RunError> {
    run_config(None, Some(config))
}

fn run_config(
    config: Option<e2e::Config>,
    benchmark: Option<crate::benchmark::Config>,
) -> Result<(), RunError> {
    let event_loop = EventLoop::new()
        .map_err(|error| RunError::new("failed to create native event loop", error))?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut application = ClientApplication::default();
    if config.is_none() && benchmark.is_none() {
        let earth = *salimon_world::CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
            .expect("Earth catalog");
        application
            .enable_terrain(earth, [0.0, 1.0, 0.0])
            .map_err(|e| RunError::new("terrain startup failed", e))?;
    }
    if let Some(config) = config {
        e2e::initialize(&mut application, config)
            .map_err(|error| RunError::new("E2E setup failed", error))?;
    }
    if let Some(config) = benchmark {
        crate::benchmark::initialize(&mut application, config)
            .map_err(|error| RunError::new("benchmark fixture failed", error))?;
        application.benchmark = Some(crate::benchmark::Recorder::new(config, &application));
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
    pub(crate) benchmark: Option<crate::benchmark::Recorder>,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_clock: FrameClock,
    pub(crate) update_clock: UpdateClock,
    camera_prototype: CameraPrototype,
    pub(crate) character: CharacterController,
    pub(crate) ship: ShipController,
    pub(crate) e2e_config: Option<e2e::Config>,
    pub(crate) automation_aim_delta: Option<[f64; 2]>,
    e2e_ready_sent: bool,
    automation: Option<Receiver<Request>>,
    pub(crate) mining: crate::mining::MiningTool,
    pub(crate) terrain_stream: crate::terrain_patches::TerrainStream,
    pub(crate) observations: crate::terrain_observations::Observations,
    pub(crate) capture_status: String,
    capture_path: Option<std::path::PathBuf>,
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
            benchmark: None,
            window: None,
            renderer: None,
            frame_clock: FrameClock::default(),
            update_clock: UpdateClock::default(),
            camera_prototype: CameraPrototype::default(),
            character: CharacterController::default(),
            ship: ShipController::default(),
            e2e_config: None,
            automation_aim_delta: None,
            e2e_ready_sent: false,
            automation: None,
            mining: crate::mining::MiningTool::default(),
            terrain_stream: crate::terrain_patches::TerrainStream::default(),
            observations: crate::terrain_observations::Observations::default(),
            capture_status: "idle".into(),
            capture_path: None,
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
    ship.door_state == DoorState::Open && ship.door_open_fraction >= 1.0
}

impl ClientApplication {
    pub(crate) fn terrain_presentation_ready(&self) -> bool {
        (!self.terrain_stream.has_target || !self.terrain_stream.active.is_empty())
            && self
                .renderer
                .as_ref()
                .is_some_and(|r| r.terrain_presentation_ready(&self.terrain_stream.active))
    }
    pub(crate) fn terrain_uploads(&self) -> (usize, usize) {
        self.renderer
            .as_ref()
            .map_or((0, 0), Renderer::terrain_uploads)
    }
    pub(crate) fn terrain_gpu_bytes(&self) -> usize {
        self.renderer
            .as_ref()
            .map_or(0, Renderer::terrain_gpu_bytes)
    }
    pub(crate) fn terrain_gpu_ready(&self) -> bool {
        self.renderer
            .as_ref()
            .is_some_and(|r| r.terrain_ready(&self.terrain_stream.active))
    }
    pub(crate) fn request_capture(&mut self, path: &str) -> Result<(), String> {
        let path = std::path::PathBuf::from(path);
        if !path.is_absolute() || path.extension().is_none_or(|e| e != "png") || path.exists() {
            return Err("capture needs a new absolute .png path".into());
        }
        self.renderer
            .as_mut()
            .ok_or("renderer unavailable")?
            .request_capture()
            .map_err(|e| e.to_string())?;
        self.capture_path = Some(path);
        self.capture_status = "pending".into();
        Ok(())
    }
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), RunError> {
        if self.window.is_some() {
            return Ok(());
        }

        let mut attributes = window_attributes(self.e2e_config.is_some());
        if self.mining.terrain == salimon_world::terrain::TerrainMode::Representative
            || std::env::var_os("SALIMON_REFERENCE_SIZE").is_some()
        {
            attributes = attributes.with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080));
        }
        if let Some(benchmark) = &self.benchmark {
            attributes = attributes.with_title(benchmark.window_title());
        }
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
        let physical_size = window.inner_size();
        let scale_factor = window.scale_factor();
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
        if let Some(benchmark) = &mut self.benchmark {
            benchmark.set_renderer(
                info,
                physical_size.width,
                physical_size.height,
                scale_factor,
            );
        }
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
        self.reset_update_clock();
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
            self.reset_update_clock();
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
        let surface = self.terrain_frame(ship_snapshot);
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
                        let clear = self.character.location() != CharacterLocation::DoorwayBlend
                            && !salimon_character::boarding_ramp_occupied(local_eye);
                        self.ship.toggle_door_with_clearance(clear);
                        if let Some(message) = self.ship.snapshot().cockpit_message {
                            if matches!(
                                message,
                                CockpitMessage::DoorLockedWhileInFlight
                                    | CockpitMessage::DoorObstructed
                            ) {
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

    pub(crate) fn held_item(
        &self,
        player: salimon_character::CharacterSnapshot,
        valid_target: bool,
    ) -> Option<salimon_renderer::HeldItemInstance> {
        (self.view_mode == ViewMode::Gameplay)
            .then(|| self.mining.held_item(&self.equipment, player, valid_target))
            .flatten()
    }

    fn world_seed(&self) -> u64 {
        self.benchmark.as_ref().map_or_else(
            || self.e2e_config.map_or(0, |config| config.seed),
            |b| b.config.seed,
        )
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

    fn toggle_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Gameplay => ViewMode::PrecisionTour,
            ViewMode::PrecisionTour => ViewMode::Gameplay,
        };
        self.mining.clear_input();
        log::info!("view mode changed to {:?} (F2 toggles)", self.view_mode);
    }
    pub(crate) fn view_name(&self) -> &'static str {
        match self.view_mode {
            ViewMode::Gameplay => "Gameplay",
            ViewMode::PrecisionTour => "PrecisionTour",
        }
    }
    pub(crate) fn automation_window(
        &self,
        action: &str,
        size: Option<[u32; 2]>,
    ) -> Result<(), String> {
        let window = self.window.as_ref().ok_or("native window unavailable")?;
        match action {
            "resize" => {
                let [width, height] = size.ok_or("resize dimensions required")?;
                if !(640..=4096).contains(&width) || !(360..=4096).contains(&height) {
                    return Err("resize dimensions out of range".into());
                }
                let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(width, height));
            }
            "minimize" => window.set_minimized(true),
            "restore" => {
                window.set_minimized(false);
                window.focus_window();
                window.request_redraw();
            }
            _ => return Err("expected resize, minimize, or restore".into()),
        }
        Ok(())
    }
    pub(crate) fn window_state(&self) -> serde_json::Value {
        self.window.as_ref().map_or(serde_json::Value::Null,|w|serde_json::json!({
            "minimized":w.is_minimized(), "inner_size":[w.inner_size().width,w.inner_size().height]
        }))
    }
    pub(crate) fn automation_key(&mut self, key: PhysicalKey, pressed: bool) {
        if key == PhysicalKey::Code(KeyCode::F2) {
            if pressed {
                self.toggle_view();
            }
            return;
        }
        if key == PhysicalKey::Code(KeyCode::F3) {
            if pressed {
                self.diagnostics.toggle();
            }
            return;
        }
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
                let player = self.character.snapshot(frame, self.terrain_frame(ship));
                let pickup = self.mining.session.carried_id().is_none();
                if !pickup
                    || crate::carrying::target(&self.mining, player, frame, door_passable(ship))
                        .is_some()
                {
                    self.carry_action(pickup);
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

    /// Shared native/automation left-button route; only equipment enables mining.
    pub(crate) fn mining_mouse_input(&mut self, pressed: bool) {
        self.mining.set_mouse_held(
            pressed
                && self.view_mode == ViewMode::Gameplay
                && self.equipment.mining_equipped()
                && self.mining.session.carried_id().is_none(),
        );
    }

    fn carry_action(&mut self, pickup: bool) {
        let ship = self.ship.snapshot();
        let frame = character_ship_frame(ship.pose);
        let player = self.character.snapshot(frame, self.terrain_frame(ship));
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
        } else if let Some(pose) = crate::carrying::release_pose(&self.mining, player, frame) {
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
        let player = self.character.snapshot(frame, self.terrain_frame(ship));
        crate::carrying::follow(&mut self.mining, player);
    }

    fn reset_update_clock(&mut self) {
        self.update_clock.reset();
        self.mining.fragment_previous.clear();
    }

    /// Admit deterministic E2E elapsed time to the same fixed-tick accumulator.
    /// Explicit commands drain all admitted ticks without a wall-clock budget;
    /// the fractional remainder stays queued for the next command.
    pub(crate) fn advance_automation(&mut self, elapsed: Duration) {
        self.update_clock.accumulate(elapsed);
        while let Some(tick) = self.update_clock.take_tick() {
            self.advance_game(tick);
        }
    }

    /// One completed gameplay tick, shared by native and automation scheduling.
    pub(crate) fn advance_game(&mut self, delta: Duration) {
        if self.e2e_config.is_none() {
            let ship = self.ship.snapshot();
            let frame = character_ship_frame(ship.pose);
            let player = self
                .character
                .snapshot(frame, self.terrain_frame(ship))
                .eye_position_meters;
            crate::fragment_physics::capture_previous(&mut self.mining, frame, player);
        }
        self.camera_prototype.advance(delta);
        self.ship
            .set_steering_input(self.ship_control_input.steering());
        self.ship.advance(delta);
        let ship = self.ship.snapshot();
        if self.view_mode == ViewMode::Gameplay {
            let eye = self
                .character
                .snapshot(character_ship_frame(ship.pose), self.terrain_frame(ship))
                .eye_position_meters;
            self.character.set_nearby_surface(self.nearby_terrain(eye));
            self.character
                .set_ramp_moving(ship.door_open_fraction > 0.0 && ship.door_open_fraction < 1.0);
            self.character.advance_with_motion(
                delta,
                self.movement_input,
                (
                    character_ship_frame(ship.pose),
                    ship.velocity_meters_per_second,
                ),
                self.terrain_frame(ship),
                door_passable(ship),
                matches!(ship.flight_state, FlightState::Landed { .. }),
            );
            let eye = self
                .character
                .snapshot(character_ship_frame(ship.pose), self.terrain_frame(ship))
                .eye_position_meters;
            self.character.set_nearby_surface(self.nearby_terrain(eye));
        }
        if self.view_mode == ViewMode::Gameplay {
            let frame = character_ship_frame(ship.pose);
            self.mining.advance(
                &self.equipment,
                delta,
                self.character.snapshot(frame, self.terrain_frame(ship)),
                frame,
                door_passable(ship),
                self.world_seed(),
            );
            let player = self
                .character
                .snapshot(frame, self.terrain_frame(ship))
                .eye_position_meters;
            let measurements = if self.benchmark.is_some() {
                crate::fragment_physics::advance_measured(
                    &mut self.mining,
                    frame,
                    delta,
                    player,
                    true,
                )
            } else {
                crate::fragment_physics::advance(&mut self.mining, frame, delta, player);
                Default::default()
            };
            if let Some(benchmark) = &mut self.benchmark {
                benchmark.latest_physics.accumulate(measurements);
            }
        }
        self.sync_carried();
        self.action_bar.advance(delta);
    }

    fn advance_native(&mut self, elapsed: Duration) -> (Duration, usize) {
        let started = Instant::now();
        self.update_clock.accumulate(elapsed);
        if let Some(benchmark) = &mut self.benchmark {
            benchmark.latest_physics.clear_work();
        }
        let mut update_delta = Duration::ZERO;
        let mut ticks = 0;
        for _ in 0..crate::update_clock::MAX_TICKS_PER_RENDER {
            // Always allow one tick; subsequent catch-up is allowed only while
            // within the budget. An individual dense contact solve is not preemptible.
            if ticks > 0 && started.elapsed() >= crate::update_clock::CATCH_UP_BUDGET {
                break;
            }
            let Some(tick) = self.update_clock.take_tick() else {
                break;
            };
            self.advance_game(tick);
            update_delta += tick;
            ticks += 1;
        }
        if let Some(benchmark) = &mut self.benchmark {
            benchmark.latest_scheduler = crate::benchmark::SchedulerSample {
                ticks,
                backlog: self.update_clock.backlog(),
                lifecycle_excluded: self.update_clock.lifecycle_excluded(),
            };
        }
        (update_delta, ticks)
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop, window: Arc<Window>) {
        if self.occluded {
            self.frame_clock.reset_interval();
            self.reset_update_clock();
            self.diagnostics.reset_frame_window();
            return;
        }

        self.render_attempts = self.render_attempts.saturating_add(1);
        let render_started_at = Instant::now();
        let update_started_at = Instant::now();
        let elapsed = if self.automation.is_some() {
            Duration::ZERO
        } else {
            self.update_clock.step(update_started_at)
        };
        let (update_delta, ticks) = self.advance_native(elapsed);
        let simulation_time = update_started_at.elapsed();
        self.sync_carried();
        let ship_snapshot = self.ship.snapshot();
        let ship_frame = character_ship_frame(ship_snapshot.pose);
        let surface_frame = self.terrain_frame(ship_snapshot);
        let world_snapshot = self.camera_prototype.snapshot();
        let (mut camera, scene_instances, mut spheres, light) =
            map_world_to_renderer(world_snapshot);
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
            self.world_seed(),
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
            self.world_seed(),
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
        window.set_title(&self.benchmark.as_ref().map_or_else(
            || gameplay_window_title(monitor_message, interaction),
            crate::benchmark::Recorder::window_title,
        ));
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
            match self
                .mining
                .nearby(camera.position_meters, self.world_seed())
            {
                Ok(deposits) => {
                    resource_meshes.extend(deposits.iter().copied().filter_map(|entry| {
                        crate::resource_presentation::deposit_mesh_with_terrain(
                            entry,
                            self.mining.terrain,
                        )
                    }));
                }
                Err(error) => log::error!("deposit presentation query failed: {error:?}"),
            }
        }
        if self.view_mode == ViewMode::Gameplay {
            for fragment in self.mining.nearby_fragments(camera.position_meters) {
                resource_meshes.push(if self.e2e_config.is_some() {
                    crate::resource_presentation::fragment_mesh(fragment)
                } else {
                    crate::fragment_physics::interpolated_mesh(
                        &self.mining,
                        fragment,
                        ship_frame,
                        self.update_clock.interpolation(),
                    )
                });
            }
        }
        let render_terrain = if self.view_mode == ViewMode::Gameplay {
            self.mining.terrain
        } else {
            salimon_world::terrain::TerrainMode::Flat
        };
        if let Err(error) = self
            .terrain_stream
            .update(camera.position_meters, render_terrain)
        {
            self.fail(event_loop, error);
            return;
        }
        for sphere in &mut spheres {
            let body = crate::terrain::body_at(sphere.center_meters, sphere.radius_meters);
            sphere.radius_meters =
                crate::terrain::far_radius(body, camera.position_meters, render_terrain);
        }
        let body_distances = camera_body_distances_with_terrain(
            world_snapshot,
            camera.position_meters,
            render_terrain,
        );
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
        let held_item = self.held_item(character_snapshot, mining_target.is_some());
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
                    terrain: &self.terrain_stream.active,
                    terrain_mask: self.terrain_stream.mask,
                    light: Some(light),
                    ship: ship_mesh,
                    held_item,
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

        if let Some(capture) = self.renderer.as_mut().and_then(Renderer::take_capture) {
            let result = capture.map_err(|e| e.to_string()).and_then(|bytes| {
                let path = self.capture_path.take().ok_or("capture path missing")?;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map_err(|e| format!("capture {}: {e}", path.display()))?;
                file.write_all(&bytes)
                    .map_err(|e| format!("capture {}: {e}", path.display()))
            });
            self.capture_status =
                result.map_or_else(|e| format!("failed: {e}"), |()| "captured".into());
        }
        match outcome {
            RenderOutcome::Presented(render_stats) => {
                self.retry_at = None;
                event_loop.set_control_flow(ControlFlow::Wait);
                let timing = self
                    .frame_clock
                    .record_presented(render_started_at, Instant::now());
                self.observations.record(
                    timing.frame_interval,
                    cpu_update_time,
                    render_stats,
                    [window.inner_size().width, window.inner_size().height],
                );
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
                        "frame={} interval_ms={:.3} present_wall_ms={:.3} cpu_update_ms={:.3} cpu_render_ms={:.3} gpu_time={:?} ticks={} backlog_ms={:.3}",
                        timing.frame_number,
                        timing.frame_interval.as_secs_f64() * 1_000.0,
                        timing.submission_wall_time.as_secs_f64() * 1_000.0,
                        cpu_update_time.as_secs_f64() * 1_000.0,
                        render_stats.cpu_render_time.as_secs_f64() * 1_000.0,
                        render_stats.gpu_frame_time,
                        ticks,
                        self.update_clock.backlog().as_secs_f64() * 1_000.0
                    );
                }
                let done = self.benchmark.as_mut().is_some_and(|benchmark| {
                    benchmark.record(
                        update_delta,
                        cpu_update_time,
                        simulation_time,
                        Some(render_stats),
                    )
                });
                if done {
                    let accounting = crate::benchmark::accounting(self);
                    let result = self
                        .benchmark
                        .as_mut()
                        .expect("active benchmark")
                        .finish(accounting);
                    match result {
                        Err(error) => self.fail(event_loop, error),
                        Ok(true) => event_loop.exit(),
                        Ok(false) => {
                            if self.benchmark.as_ref().is_some_and(|b| b.switch_camera()) {
                                self.character
                                    .apply_mouse_delta(std::f64::consts::PI / 0.0022, 0.0);
                            }
                            window.request_redraw();
                        }
                    }
                    return;
                }
                window.request_redraw();
            }
            RenderOutcome::Retry => self.schedule_retry(event_loop, RENDER_RETRY_DELAY),
            RenderOutcome::Idle => {
                self.frame_clock.reset_interval();
                self.reset_update_clock();
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
        if self.benchmark.is_none()
            && self.view_mode == ViewMode::Gameplay
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
        self.reset_update_clock();
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
        // Benchmark cameras and population are deterministic initial conditions.
        // Keep native timing/rendering, while desktop input cannot mutate them.
        if self.benchmark.is_some()
            && matches!(
                event,
                WindowEvent::KeyboardInput { .. } | WindowEvent::MouseInput { .. }
            )
        {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                if self.benchmark.is_some() {
                    self.fail(event_loop, "benchmark window closed before completion");
                } else {
                    event_loop.exit();
                }
            }
            WindowEvent::Destroyed => {
                self.renderer = None;
                self.window = None;
                self.retry_at = None;
                self.frame_clock.reset_interval();
                self.reset_update_clock();
                self.diagnostics.reset_frame_window();
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                let size = window.inner_size();
                if self.benchmark.as_ref().is_some_and(|b| {
                    b.surface_changed(size.width, size.height, window.scale_factor())
                }) {
                    self.fail(event_loop, "benchmark surface configuration changed");
                    return;
                }
                self.resize_renderer(event_loop, &window);
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let size = window.inner_size();
                if self.benchmark.as_ref().is_some_and(|b| {
                    b.surface_changed(size.width, size.height, window.scale_factor())
                }) {
                    self.fail(event_loop, "benchmark surface configuration changed");
                    return;
                }
                self.diagnostics.set_scale_factor(window.scale_factor());
                self.action_bar.set_scale_factor(window.scale_factor());
                self.resize_renderer(event_loop, &window);
            }
            WindowEvent::Occluded(is_occluded) => {
                if is_occluded
                    && self
                        .benchmark
                        .as_mut()
                        .is_some_and(|b| !b.restart_before_sampling())
                {
                    self.fail(event_loop, "benchmark window occluded during collection");
                    return;
                }
                log::debug!("native window occluded: {is_occluded}");
                self.occluded = is_occluded;
                self.retry_at = None;
                self.frame_clock.reset_interval();
                self.reset_update_clock();
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
                self.mining_mouse_input(self.cursor_captured && state == ElementState::Pressed);
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } if self.cursor_captured
                && tool_key_event(event.repeat, is_synthetic, event.physical_key) =>
            {
                self.automation_key(event.physical_key, event.state == ElementState::Pressed);
                window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } if self.view_mode == ViewMode::Gameplay
                && self.cursor_captured
                && toolbar_selection_pressed(
                    event.state,
                    event.repeat,
                    event.physical_key,
                    is_synthetic,
                ) =>
            {
                self.automation_key(event.physical_key, true);
                window.request_redraw();
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
                self.toggle_view();
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
    fn automation_partitions_preserve_elapsed_and_match_native_ticks() {
        let mut whole = ClientApplication::default();
        let mut partitioned = ClientApplication::default();
        let pose = salimon_ship::ShipPose {
            position_meters: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
        };
        whole.ship = salimon_ship::ShipController::flying(pose, 10);
        partitioned.ship = whole.ship.clone();
        whole.advance_automation(Duration::from_millis(1003));
        for ms in [7, 201, 1, 394, 400] {
            partitioned.advance_automation(Duration::from_millis(ms));
        }
        assert_eq!(whole.ship.snapshot(), partitioned.ship.snapshot());
        assert_eq!(whole.update_clock.backlog(), Duration::from_millis(11));
        assert_eq!(
            whole.update_clock.backlog(),
            partitioned.update_clock.backlog()
        );
    }

    #[test]
    fn stalled_native_scheduler_matches_shared_fixed_gameplay_ticks() {
        let mut app = ClientApplication::default();
        let mut reference = ClientApplication::default();
        for instance in [&mut app, &mut reference] {
            crate::benchmark_fixture::initialize(
                instance,
                6,
                crate::benchmark_fixture::Layout::Dense,
                159,
                true,
            )
            .unwrap();
        }
        let mut input = Duration::from_secs(2);
        let mut advanced = Duration::ZERO;
        loop {
            let (delta, ticks) = app.advance_native(input);
            input = Duration::ZERO;
            assert!((1..=crate::update_clock::MAX_TICKS_PER_RENDER).contains(&ticks));
            assert_eq!(delta, crate::update_clock::GAME_TICK * ticks as u32);
            for _ in 0..ticks {
                reference.advance_game(crate::update_clock::GAME_TICK);
            }
            advanced += delta;
            assert_eq!(
                advanced + app.update_clock.backlog(),
                Duration::from_secs(2)
            );
            assert_eq!(app.ship.snapshot().pose, reference.ship.snapshot().pose);
            assert_eq!(
                app.mining.session.fragments(),
                reference.mining.session.fragments()
            );
            if app.update_clock.backlog() < crate::update_clock::GAME_TICK {
                break;
            }
        }
        assert_eq!(advanced, Duration::from_secs(2));
    }

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
    fn failed_pickup_preserves_selected_tool_and_mining_input() {
        for slot in [
            ToolbarSlot::One,
            ToolbarSlot::Two,
            ToolbarSlot::Three,
            ToolbarSlot::Four,
            ToolbarSlot::Five,
        ] {
            let mut app = ClientApplication::default();
            app.equipment.select(slot, false);
            app.mining.set_mouse_held(true);
            app.carry_action(true);
            assert!(app.mining.session.carried_id().is_none());
            assert_eq!(app.equipment.selected(), Some(slot));
            assert!(
                app.mining.held,
                "failed pickup must not stow or cancel mining"
            );
            assert_eq!(
                app.mining.carry_feedback,
                Some("Aim at a fragment within 3 m")
            );
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
