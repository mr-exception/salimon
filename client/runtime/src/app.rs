use std::error::Error;
use std::fmt;
use std::io::Write;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use salimon_character::{
    CharacterController, CharacterLocation, MovementInput, ShipFrame, SurfaceFrame,
};
use salimon_diagnostics::{
    BodyDistance, Diagnostics, DomainMetrics, FrameSample, GpuMemory, GpuTime,
};
use salimon_renderer::{
    CameraFrame, CockpitInstruments, GpuFrameTime, NearbyBodyInstruments,
    OverlayImage as RendererOverlayImage, OverlayPlacement, PointLight, RenderOutcome, Renderer,
    SceneFrame, SceneInstance, ShipMeshInstance, SphereInstance, SurfaceMaterial, SurfaceSize,
};
use salimon_ship::{
    CockpitMessage, DoorState, FlightState, ShipController, ShipPose, ShipSnapshot, SteeringInput,
};
use salimon_world::{
    CELESTIAL_BODIES, CameraCommand, CameraPrototype, CelestialBodyId, TransitionPhase,
    WorldPosition, WorldSnapshot,
};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

use crate::action_bar::ActionBar;
use crate::automation::{self, Request};
use crate::e2e;
use crate::frame_clock::FrameClock;
use crate::update_clock::UpdateClock;

const WINDOW_TITLE: &str = "Salimon — Compressed Solar System";
const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;
const MINIMUM_WIDTH: f64 = 640.0;
const MINIMUM_HEIGHT: f64 = 360.0;
const TIMING_LOG_INTERVAL: u64 = 300;
const RENDER_RETRY_DELAY: Duration = Duration::from_millis(50);
const IDLE_RETRY_DELAY: Duration = Duration::from_millis(250);
const COCKPIT_INTERACTION_POSITION_METERS: [f64; 3] = [2.76, 1.799_032_258_064_516, 0.0];
const COCKPIT_INTERACTION_RANGE_METERS: f64 = 4.0;
const COCKPIT_INTERACTION_MINIMUM_AIM_DOT: f64 = 0.866_025_403_784_438_6;
const COCKPIT_INTERACTION_PROMPT: &str = "Press E to use";

const fn initial_window_size() -> PhysicalSize<u32> {
    PhysicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT)
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InteractionTarget {
    Cockpit,
    ExitDoor,
}

fn interaction_target(
    local_eye_position: [f64; 3],
    local_look_target: [f64; 3],
) -> Option<InteractionTarget> {
    let to_cockpit = subtract(COCKPIT_INTERACTION_POSITION_METERS, local_eye_position);
    let look_direction = subtract(local_look_target, local_eye_position);
    let cockpit_distance = vector_length(to_cockpit);
    let cockpit_is_aimed_at = cockpit_distance > f64::EPSILON
        && cockpit_distance <= COCKPIT_INTERACTION_RANGE_METERS
        && dot(to_cockpit, look_direction)
            / (cockpit_distance * vector_length(look_direction)).max(f64::EPSILON)
            >= COCKPIT_INTERACTION_MINIMUM_AIM_DOT;

    if cockpit_is_aimed_at {
        Some(InteractionTarget::Cockpit)
    } else if (local_eye_position[0] + 7.10).abs() < 2.7 && local_eye_position[2].abs() < 2.3 {
        Some(InteractionTarget::ExitDoor)
    } else {
        None
    }
}

fn available_interaction_target(
    location: CharacterLocation,
    local_eye_position: [f64; 3],
    local_look_target: [f64; 3],
) -> Option<InteractionTarget> {
    match interaction_target(local_eye_position, local_look_target) {
        Some(InteractionTarget::Cockpit) if location != CharacterLocation::InsideShip => None,
        target => target,
    }
}

fn subtract(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|index| left[index] - right[index])
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter().zip(right).map(|(a, b)| a * b).sum()
}

fn vector_length(vector: [f64; 3]) -> f64 {
    dot(vector, vector).sqrt()
}

fn gameplay_window_title(
    monitor_message: Option<CockpitMessage>,
    interaction: Option<InteractionTarget>,
) -> String {
    if let Some(message) = monitor_message {
        format!("{WINDOW_TITLE} — {}", message.text())
    } else if interaction == Some(InteractionTarget::Cockpit) {
        format!("{WINDOW_TITLE} — {COCKPIT_INTERACTION_PROMPT}")
    } else {
        WINDOW_TITLE.to_owned()
    }
}

fn action_bar_context(
    monitor_message: Option<CockpitMessage>,
    interaction: Option<InteractionTarget>,
) -> Option<&'static str> {
    monitor_message
        .filter(|message| *message != CockpitMessage::DoorLockedWhileInFlight)
        .map(CockpitMessage::text)
        .or(match interaction {
            Some(InteractionTarget::Cockpit) => Some(COCKPIT_INTERACTION_PROMPT),
            Some(InteractionTarget::ExitDoor) => Some("E to open/close exit door"),
            None => None,
        })
}

fn map_ship_to_renderer(ship: ShipSnapshot) -> ShipMeshInstance {
    ShipMeshInstance {
        position_meters: ship.pose.position_meters,
        orientation: ship.pose.orientation.map(|value| value as f32),
        door_open_fraction: ship.door_open_fraction as f32,
        instruments: CockpitInstruments {
            speed_meters_per_second: ship.speed_meters_per_second,
            thruster_percentage: ship.thruster_percentage,
            core_energy_capacity_joules: ship.energy_core.capacity_joules,
            core_energy_stored_joules: ship.energy_core.stored_joules,
            nearby_body: ship.nearby_body.map(|body| NearbyBodyInstruments {
                name: body.name,
                surface_distance_meters: body.surface_distance_meters,
                radial_speed_meters_per_second: body.radial_speed_meters_per_second,
            }),
        },
    }
}

fn door_passable(ship: ShipSnapshot) -> bool {
    ship.door_state == DoorState::Open && ship.door_open_fraction >= 0.95
}

impl ClientApplication {
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), RunError> {
        if self.window.is_some() {
            return Ok(());
        }

        let attributes = WindowAttributes::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size(if self.e2e_config.is_some() {
                PhysicalSize::new(1280, 800)
            } else {
                initial_window_size()
            })
            .with_min_inner_size(LogicalSize::new(MINIMUM_WIDTH, MINIMUM_HEIGHT))
            .with_resizable(true);
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
                if matches!(
                    character.location,
                    CharacterLocation::Surface | CharacterLocation::InsideShip
                ) && (self.mining.session.carried_id().is_some()
                    || crate::carrying::target(
                        &self.mining,
                        character,
                        ship_frame,
                        door_passable(ship_snapshot),
                    )
                    .is_some())
                    && interaction != Some(InteractionTarget::ExitDoor)
                {
                    self.carry_action(self.mining.session.carried_id().is_none());
                    return;
                }
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
            return;
        }
        if matches!(key, PhysicalKey::Code(KeyCode::KeyQ | KeyCode::KeyG)) {
            if pressed {
                self.carry_action(key == PhysicalKey::Code(KeyCode::KeyQ));
            }
            return;
        }
        if key == PhysicalKey::Code(KeyCode::KeyM) && pressed {
            self.mining.toggle();
            return;
        }
        if key == PhysicalKey::Code(KeyCode::KeyF) {
            self.mining.held = pressed;
            return;
        }
        if self.character.location() == CharacterLocation::Cockpit
            && ship_control_key(key).is_some()
        {
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
                    self.mining.fragment_motion.remove(&id);
                    self.mining.ship_fragments.remove(&id);
                    "Fragment picked up - E to drop"
                }
                Some(_) => "Only one world object - E to drop first",
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
        let (mut camera, mut scene_instances, spheres, light) =
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
            character_snapshot,
            ship_frame,
            door_passable(ship_snapshot),
            self.e2e_config.map_or(0, |config| config.seed),
        );
        let ship_context = action_bar_context(monitor_message, interaction).map(str::to_owned);
        let contextual_action = if fragment_interaction {
            resource_context.or(ship_context)
        } else {
            ship_context.or(resource_context)
        };
        self.action_bar.set_contextual(contextual_action);
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
                Ok(deposits) => scene_instances.extend(
                    deposits
                        .into_iter()
                        .filter_map(crate::resource_presentation::visual),
                ),
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
                placement: OverlayPlacement::BottomCenter,
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
                        self.mining
                            .held_item(character_snapshot, mining_target.is_some())
                    } else {
                        None
                    },
                },
                overlay_image,
                action_bar_image,
                crate::reticle::image(self.view_mode == ViewMode::Gameplay, &self.mining),
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
                    FrameSample {
                        frame_number: timing.frame_number,
                        frame_interval: timing.frame_interval,
                        cpu_render_time: render_stats.cpu_render_time,
                        cpu_update_time: Some(cpu_update_time),
                        gpu_time: match render_stats.gpu_frame_time {
                            GpuFrameTime::Unsupported => GpuTime::Unsupported,
                            GpuFrameTime::Pending => GpuTime::Pending,
                            GpuFrameTime::Measured(duration) => GpuTime::Measured(duration),
                        },
                        visible_objects: render_stats.visible_objects,
                        rendered_objects: render_stats.rendered_objects,
                        scene_draw_calls: render_stats.scene_draw_calls,
                        total_draw_calls: render_stats.total_draw_calls,
                        gpu_memory: render_stats.gpu_memory.map(|memory| GpuMemory {
                            allocated_bytes: memory.allocated_bytes,
                            reserved_bytes: memory.reserved_bytes,
                        }),
                    },
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
        self.mining.held = false;
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
                    self.mining.held = false;
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
                self.mining.held = false;
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mining.held = self.cursor_captured && state == ElementState::Pressed;
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.physical_key == PhysicalKey::Code(KeyCode::KeyM)
                    || event.physical_key == PhysicalKey::Code(KeyCode::KeyF)
                    || event.physical_key == PhysicalKey::Code(KeyCode::KeyQ)
                    || event.physical_key == PhysicalKey::Code(KeyCode::KeyG) =>
            {
                if self.cursor_captured && !event.repeat {
                    self.automation_key(event.physical_key, event.state == ElementState::Pressed);
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
                self.mining.held = false;
                log::info!("view mode changed to {:?} (F2 toggles)", self.view_mode);
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if self.character.location() == CharacterLocation::Cockpit
                    && ship_control_key(event.physical_key).is_some() =>
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
            WindowEvent::KeyboardInput { event, .. }
                if movement_key(event.physical_key).is_some() =>
            {
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
                if camera_command(event.state, event.repeat, event.physical_key).is_some() =>
            {
                let command = camera_command(event.state, event.repeat, event.physical_key)
                    .expect("guard accepts only camera commands");
                self.camera_prototype.apply_command(command);
                self.mining.held = false;
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

fn surface_size(window: &Window) -> SurfaceSize {
    let size = window.inner_size();
    SurfaceSize::new(size.width, size.height)
}

fn capture_cursor(window: &Window) -> bool {
    if let Err(error) = window.set_cursor_grab(CursorGrabMode::Locked) {
        log::warn!("could not lock cursor for mouse look: {error}");
        window.set_cursor_visible(true);
        return false;
    }
    window.set_cursor_visible(false);
    true
}

fn release_cursor(window: &Window) {
    if let Err(error) = window.set_cursor_grab(CursorGrabMode::None) {
        log::warn!("could not release cursor: {error}");
    }
    window.set_cursor_visible(true);
}

fn release_cursor_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::Escape)
}

pub(crate) fn character_ship_frame(pose: ShipPose) -> ShipFrame {
    ShipFrame {
        origin_meters: pose.position_meters,
        axes: pose.axes(),
    }
}

pub(crate) fn surface_frame_for_ship(ship: ShipSnapshot) -> SurfaceFrame {
    let body_id = match ship.flight_state {
        FlightState::Landed { body }
        | FlightState::AssistedLanding { body }
        | FlightState::AssistedTakeoff { body } => body,
        FlightState::Flying => {
            CELESTIAL_BODIES
                .iter()
                .filter(|body| body.role == salimon_world::BodyRole::Solid)
                .min_by(|left, right| {
                    distance_squared(left.center.meters(), ship.pose.position_meters).total_cmp(
                        &distance_squared(right.center.meters(), ship.pose.position_meters),
                    )
                })
                .expect("world catalog always contains a solid body")
                .id
        }
    };
    let body = CELESTIAL_BODIES
        .iter()
        .find(|body| body.id == body_id)
        .expect("ship body identifier belongs to the world catalog");
    SurfaceFrame {
        body_center_meters: body.center.meters(),
        radius_meters: body.radius_meters,
    }
}

pub(crate) fn nearby_surface_at(eye: [f64; 3]) -> Option<SurfaceFrame> {
    salimon_world::nearby_solid_body(WorldPosition::new(eye[0], eye[1], eye[2])).map(|(body, _)| {
        SurfaceFrame {
            body_center_meters: body.center.meters(),
            radius_meters: body.radius_meters,
        }
    })
}

fn distance_squared(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(a, b)| (a - b) * (a - b))
        .sum()
}

fn interaction_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::KeyE)
}

fn landing_action_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::KeyL)
}

fn view_toggle_pressed(
    state: ElementState,
    repeat: bool,
    key: PhysicalKey,
    is_synthetic: bool,
) -> bool {
    // Winit replays held keys on focus changes. A replay must not toggle views.
    state == ElementState::Pressed
        && !repeat
        && !is_synthetic
        && key == PhysicalKey::Code(KeyCode::F2)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MovementKey {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
    Descend,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ShipControlInput {
    pitch_up: bool,
    pitch_down: bool,
    yaw_left: bool,
    yaw_right: bool,
    roll_left: bool,
    roll_right: bool,
}

impl ShipControlInput {
    fn steering(self) -> SteeringInput {
        SteeringInput {
            pitch: f64::from(i8::from(self.pitch_up) - i8::from(self.pitch_down)),
            yaw: f64::from(i8::from(self.yaw_left) - i8::from(self.yaw_right)),
            roll: f64::from(i8::from(self.roll_right) - i8::from(self.roll_left)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShipControlKey {
    PitchUp,
    PitchDown,
    YawLeft,
    YawRight,
    RollLeft,
    RollRight,
}

fn ship_control_key(key: PhysicalKey) -> Option<ShipControlKey> {
    match key {
        PhysicalKey::Code(KeyCode::KeyW) => Some(ShipControlKey::PitchUp),
        PhysicalKey::Code(KeyCode::KeyS) => Some(ShipControlKey::PitchDown),
        PhysicalKey::Code(KeyCode::KeyA) => Some(ShipControlKey::YawLeft),
        PhysicalKey::Code(KeyCode::KeyD) => Some(ShipControlKey::YawRight),
        PhysicalKey::Code(KeyCode::ArrowLeft) => Some(ShipControlKey::RollLeft),
        PhysicalKey::Code(KeyCode::ArrowRight) => Some(ShipControlKey::RollRight),
        _ => None,
    }
}

fn update_ship_control_input(input: &mut ShipControlInput, key: PhysicalKey, pressed: bool) {
    match ship_control_key(key) {
        Some(ShipControlKey::PitchUp) => input.pitch_up = pressed,
        Some(ShipControlKey::PitchDown) => input.pitch_down = pressed,
        Some(ShipControlKey::YawLeft) => input.yaw_left = pressed,
        Some(ShipControlKey::YawRight) => input.yaw_right = pressed,
        Some(ShipControlKey::RollLeft) => input.roll_left = pressed,
        Some(ShipControlKey::RollRight) => input.roll_right = pressed,
        None => {}
    }
}

fn thruster_step(state: ElementState, repeat: bool, key: PhysicalKey) -> Option<i8> {
    if state != ElementState::Pressed || repeat {
        return None;
    }
    match key {
        PhysicalKey::Code(KeyCode::ArrowUp) => Some(1),
        PhysicalKey::Code(KeyCode::ArrowDown) => Some(-1),
        _ => None,
    }
}

fn format_metric_speed(meters_per_second: f64) -> String {
    if meters_per_second >= 1_000_000.0 {
        format!("{:.2} Mm/s", meters_per_second / 1_000_000.0)
    } else if meters_per_second >= 1_000.0 {
        format!("{:.2} km/s", meters_per_second / 1_000.0)
    } else {
        format!("{meters_per_second:.1} m/s")
    }
}

fn movement_key(key: PhysicalKey) -> Option<MovementKey> {
    match key {
        PhysicalKey::Code(KeyCode::KeyW) => Some(MovementKey::Forward),
        PhysicalKey::Code(KeyCode::KeyS) => Some(MovementKey::Backward),
        PhysicalKey::Code(KeyCode::KeyA) => Some(MovementKey::Left),
        PhysicalKey::Code(KeyCode::KeyD) => Some(MovementKey::Right),
        PhysicalKey::Code(KeyCode::Space) => Some(MovementKey::Jump),
        PhysicalKey::Code(KeyCode::ShiftLeft) => Some(MovementKey::Descend),
        _ => None,
    }
}

fn update_movement_input(input: &mut MovementInput, key: PhysicalKey, pressed: bool) {
    match movement_key(key) {
        Some(MovementKey::Forward) => input.forward = pressed,
        Some(MovementKey::Backward) => input.backward = pressed,
        Some(MovementKey::Left) => input.left = pressed,
        Some(MovementKey::Right) => input.right = pressed,
        Some(MovementKey::Jump) => input.jump = pressed,
        Some(MovementKey::Descend) => input.descend = pressed,
        None => {}
    }
}

fn is_diagnostics_toggle(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::F3)
}

fn camera_command(state: ElementState, repeat: bool, key: PhysicalKey) -> Option<CameraCommand> {
    if state != ElementState::Pressed || repeat {
        return None;
    }

    match key {
        PhysicalKey::Code(KeyCode::KeyP) => Some(CameraCommand::TogglePause),
        PhysicalKey::Code(KeyCode::KeyR) => Some(CameraCommand::Restart),
        PhysicalKey::Code(KeyCode::KeyN) => Some(CameraCommand::JumpToNear),
        PhysicalKey::Code(KeyCode::Digit1) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Sun))
        }
        PhysicalKey::Code(KeyCode::Digit2) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Mercury))
        }
        PhysicalKey::Code(KeyCode::Digit3) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Venus))
        }
        PhysicalKey::Code(KeyCode::Digit4) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Earth))
        }
        PhysicalKey::Code(KeyCode::Digit5) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Moon))
        }
        PhysicalKey::Code(KeyCode::Digit6) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Mars))
        }
        _ => None,
    }
}

fn map_world_to_renderer(
    snapshot: WorldSnapshot<'_>,
) -> (
    CameraFrame,
    Vec<SceneInstance>,
    Vec<SphereInstance>,
    PointLight,
) {
    let camera = CameraFrame {
        position_meters: snapshot.camera.position.meters(),
        target_meters: snapshot.camera.target.meters(),
        up: snapshot.camera.up.map(|component| component as f32),
        vertical_fov_radians: snapshot.camera.vertical_field_of_view_radians as f32,
        near_plane_meters: snapshot.camera.physical_near_plane_meters as f32,
    };
    let spheres = snapshot
        .celestial_bodies
        .iter()
        .map(|body| SphereInstance {
            center_meters: body.center.meters(),
            radius_meters: body.radius_meters,
            material: match body.id {
                CelestialBodyId::Sun => SurfaceMaterial::Emissive,
                CelestialBodyId::Mercury => SurfaceMaterial::Stone,
                CelestialBodyId::Venus => SurfaceMaterial::Ochre,
                CelestialBodyId::Earth => SurfaceMaterial::Oceanic,
                CelestialBodyId::Moon => SurfaceMaterial::Slate,
                CelestialBodyId::Mars => SurfaceMaterial::Rust,
            },
        })
        .collect();
    let instances = snapshot
        .precision_markers
        .iter()
        .map(|marker| SceneInstance {
            center_meters: marker.absolute_center.meters(),
            half_extents_meters: marker.half_extents_meters,
            color: marker.color,
        })
        .collect();
    let sun = snapshot
        .celestial_bodies
        .iter()
        .find(|body| body.id == CelestialBodyId::Sun)
        .expect("world catalog always contains the Sun");
    let light = PointLight {
        position_meters: sun.center.meters(),
        color: [1.0, 0.96, 0.88],
    };

    (camera, instances, spheres, light)
}

#[cfg(test)]
fn camera_body_distances(snapshot: WorldSnapshot<'_>) -> Vec<BodyDistance<'static>> {
    camera_body_distances_from(snapshot, snapshot.camera.position.meters())
}

fn camera_body_distances_from(
    snapshot: WorldSnapshot<'_>,
    position_meters: [f64; 3],
) -> Vec<BodyDistance<'static>> {
    let position = WorldPosition::new(position_meters[0], position_meters[1], position_meters[2]);
    snapshot
        .celestial_bodies
        .iter()
        .map(|body| BodyDistance {
            name: body.name,
            distance_meters: body.surface_distance_from(position),
        })
        .collect()
}

fn gameplay_domain_metrics<'a>(
    player_position: [f64; 3],
    ship: ShipSnapshot,
    body_distances: &'a [BodyDistance<'static>],
) -> DomainMetrics<'a> {
    DomainMetrics {
        camera_position: Some(player_position),
        player_position: Some(player_position),
        ship_position: Some(ship.pose.position_meters),
        ship_velocity: Some(ship.velocity_meters_per_second),
        ship_speed_mps: Some(ship.speed_meters_per_second),
        thruster_percent: Some(ship.thruster_percentage),
        nearby_bodies: body_distances,
        ..DomainMetrics::default()
    }
}

fn camera_domain_metrics<'a>(
    snapshot: WorldSnapshot<'_>,
    body_distances: &'a [BodyDistance<'static>],
) -> DomainMetrics<'a> {
    DomainMetrics {
        camera_position: Some(snapshot.camera.position.meters()),
        camera_altitude_meters: Some(snapshot.camera.altitude_meters),
        camera_phase: Some(transition_phase_name(snapshot.transition_phase)),
        camera_paused: Some(snapshot.paused),
        nearby_bodies: body_distances,
        ..DomainMetrics::default()
    }
}

const fn transition_phase_name(phase: TransitionPhase) -> &'static str {
    match phase {
        TransitionPhase::Approach => "approach",
        TransitionPhase::NearDwell => "near dwell",
        TransitionPhase::Retreat => "retreat",
        TransitionPhase::FarDwell => "far dwell",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        COCKPIT_INTERACTION_PROMPT, INITIAL_HEIGHT, INITIAL_WIDTH, InteractionTarget,
        ShipControlInput, action_bar_context, available_interaction_target, camera_body_distances,
        camera_command, camera_domain_metrics, format_metric_speed, gameplay_window_title,
        initial_window_size, interaction_target, is_diagnostics_toggle, landing_action_pressed,
        map_ship_to_renderer, map_world_to_renderer, release_cursor_pressed, ship_control_key,
        thruster_step, update_ship_control_input, view_toggle_pressed,
    };
    use salimon_character::CharacterLocation;
    use salimon_renderer::CockpitInstruments;
    use salimon_ship::{CockpitMessage, FlightState, ShipController};
    use salimon_world::{CELESTIAL_BODIES, CameraCommand, CameraPrototype, CelestialBodyId};
    use winit::event::ElementState;
    use winit::keyboard::{KeyCode, PhysicalKey};

    #[test]
    fn focus_replayed_f2_does_not_toggle_the_view_again() {
        let f2 = PhysicalKey::Code(KeyCode::F2);
        assert!(view_toggle_pressed(ElementState::Pressed, false, f2, false));
        assert!(!view_toggle_pressed(ElementState::Pressed, false, f2, true));
        assert!(!view_toggle_pressed(ElementState::Pressed, true, f2, false));
        assert!(!view_toggle_pressed(
            ElementState::Released,
            false,
            f2,
            false
        ));
        assert!(!view_toggle_pressed(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::F3),
            false,
        ));
    }

    #[test]
    fn f3_toggles_only_on_the_initial_press() {
        let f3 = PhysicalKey::Code(KeyCode::F3);

        assert!(is_diagnostics_toggle(ElementState::Pressed, false, f3));
        assert!(!is_diagnostics_toggle(ElementState::Pressed, true, f3));
        assert!(!is_diagnostics_toggle(ElementState::Released, false, f3));
        assert!(!is_diagnostics_toggle(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::F2),
        ));
    }

    #[test]
    fn escape_releases_cursor_only_on_the_initial_press() {
        let escape = PhysicalKey::Code(KeyCode::Escape);
        assert!(release_cursor_pressed(ElementState::Pressed, false, escape));
        assert!(!release_cursor_pressed(ElementState::Pressed, true, escape));
        assert!(!release_cursor_pressed(
            ElementState::Released,
            false,
            escape
        ));
    }

    #[test]
    fn landing_action_accepts_only_initial_l_press() {
        let landing = PhysicalKey::Code(KeyCode::KeyL);
        assert!(landing_action_pressed(
            ElementState::Pressed,
            false,
            landing
        ));
        assert!(!landing_action_pressed(
            ElementState::Pressed,
            true,
            landing
        ));
        assert!(!landing_action_pressed(
            ElementState::Released,
            false,
            landing
        ));
        assert!(!landing_action_pressed(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::KeyK),
        ));
    }

    #[test]
    fn camera_keys_map_only_initial_physical_presses() {
        for (key, body) in [
            (KeyCode::Digit1, CelestialBodyId::Sun),
            (KeyCode::Digit2, CelestialBodyId::Mercury),
            (KeyCode::Digit3, CelestialBodyId::Venus),
            (KeyCode::Digit4, CelestialBodyId::Earth),
            (KeyCode::Digit5, CelestialBodyId::Moon),
            (KeyCode::Digit6, CelestialBodyId::Mars),
        ] {
            assert_eq!(
                camera_command(ElementState::Pressed, false, PhysicalKey::Code(key)),
                Some(CameraCommand::InspectBody(body))
            );
            assert_eq!(
                camera_command(ElementState::Pressed, true, PhysicalKey::Code(key)),
                None
            );
            assert_eq!(
                camera_command(ElementState::Released, false, PhysicalKey::Code(key)),
                None
            );
        }
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyP),
            ),
            Some(CameraCommand::TogglePause)
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyR),
            ),
            Some(CameraCommand::Restart)
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyN),
            ),
            Some(CameraCommand::JumpToNear)
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                true,
                PhysicalKey::Code(KeyCode::KeyP),
            ),
            None
        );
        assert_eq!(
            camera_command(
                ElementState::Released,
                false,
                PhysicalKey::Code(KeyCode::KeyN),
            ),
            None
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyW),
            ),
            None
        );
    }

    #[test]
    fn cockpit_keys_map_to_independent_held_axes() {
        let mut input = ShipControlInput::default();
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyW), true);
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyA), true);
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::ArrowRight), true);
        assert_eq!(input.steering().pitch, 1.0);
        assert_eq!(input.steering().yaw, 1.0);
        assert_eq!(input.steering().roll, 1.0);
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyW), false);
        assert_eq!(input.steering().pitch, 0.0);
        assert!(ship_control_key(PhysicalKey::Code(KeyCode::ArrowUp)).is_none());
    }

    #[test]
    fn a_and_d_emit_the_corrected_opposite_yaw_directions() {
        let mut input = ShipControlInput::default();
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyA), true);
        assert_eq!(input.steering().yaw, 1.0);

        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyA), false);
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyD), true);
        assert_eq!(input.steering().yaw, -1.0);
    }

    #[test]
    fn thruster_keys_accept_only_initial_arrow_presses() {
        let up = PhysicalKey::Code(KeyCode::ArrowUp);
        let down = PhysicalKey::Code(KeyCode::ArrowDown);
        assert_eq!(thruster_step(ElementState::Pressed, false, up), Some(1));
        assert_eq!(thruster_step(ElementState::Pressed, false, down), Some(-1));
        assert_eq!(thruster_step(ElementState::Pressed, true, up), None);
        assert_eq!(thruster_step(ElementState::Released, false, down), None);
    }

    #[test]
    fn cockpit_speed_uses_practical_metric_units() {
        assert_eq!(format_metric_speed(12.0), "12.0 m/s");
        assert_eq!(format_metric_speed(12_500.0), "12.50 km/s");
        assert_eq!(format_metric_speed(2_500_000.0), "2.50 Mm/s");
    }

    #[test]
    fn cockpit_instruments_follow_live_power_and_landing_after_leaving_the_seat() {
        let pose = ShipController::default().snapshot().pose;
        let mut ship = ShipController::flying(pose, 0);
        let stopped = map_ship_to_renderer(ship.snapshot());
        assert_eq!(stopped.instruments.speed_meters_per_second, 0.0);
        assert_eq!(stopped.instruments.thruster_percentage, 0);
        assert_eq!(
            stopped.instruments.core_energy_capacity_joules,
            salimon_ship::DEFAULT_CORE_ENERGY_CAPACITY_JOULES
        );
        assert_eq!(stopped.instruments.nearby_body.unwrap().name, "Earth");

        ship.set_stored_core_energy_joules(250_000_000_000);
        let changed_energy = map_ship_to_renderer(ship.snapshot());
        assert_eq!(
            changed_energy.instruments.core_energy_stored_joules,
            250_000_000_000
        );

        ship.adjust_thruster(1);
        let minimum_power = map_ship_to_renderer(ship.snapshot());
        assert_eq!(minimum_power.instruments.thruster_percentage, 1);
        assert_eq!(
            minimum_power.instruments.speed_meters_per_second,
            salimon_world::PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND / 100.0
        );

        ship.adjust_thruster(99);
        let full_power = map_ship_to_renderer(ship.snapshot());
        assert_eq!(full_power.instruments.thruster_percentage, 100);
        assert_eq!(
            full_power.instruments.speed_meters_per_second,
            salimon_world::PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND
        );

        ship.trigger_landing_action();
        ship.set_cockpit_control(false);
        let assisting = map_ship_to_renderer(ship.snapshot()).instruments;
        assert_eq!(assisting.core_energy_stored_joules, 250_000_000_000);
        assert_eq!(
            assisting.speed_meters_per_second,
            full_power.instruments.speed_meters_per_second
        );
        // Landing starts with an alignment hold, even when already at the
        // surface. The live monitor must report the actual stationary phase.
        assert_eq!(
            assisting
                .nearby_body
                .unwrap()
                .radial_speed_meters_per_second,
            0.0
        );
        ship.advance(std::time::Duration::from_millis(100));
        assert!(matches!(
            ship.snapshot().flight_state,
            FlightState::AssistedLanding { .. }
        ));
        ship.advance(salimon_ship::LANDING_ASSIST_DURATION - std::time::Duration::from_millis(100));
        assert!(matches!(
            ship.snapshot().flight_state,
            FlightState::Landed { .. }
        ));
        assert_eq!(
            map_ship_to_renderer(ship.snapshot()).instruments,
            CockpitInstruments {
                core_energy_stored_joules: 250_000_000_000,
                ..stopped.instruments
            }
        );

        let out_of_range = ShipController::flying(
            salimon_ship::ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        assert!(
            map_ship_to_renderer(out_of_range.snapshot())
                .instruments
                .nearby_body
                .is_none()
        );
    }

    #[test]
    fn cockpit_proximity_monitor_tracks_eased_assists_without_pilot_authority() {
        let mut ship = ShipController::default();
        ship.set_cockpit_control(true);
        ship.trigger_landing_action();
        ship.set_cockpit_control(false);
        ship.advance(std::time::Duration::from_secs(1));
        let lifting = map_ship_to_renderer(ship.snapshot())
            .instruments
            .nearby_body
            .unwrap();
        assert!((lifting.radial_speed_meters_per_second - 11.25).abs() < 1.0e-9);
        ship.advance(std::time::Duration::from_secs(1));
        let lifted = map_ship_to_renderer(ship.snapshot())
            .instruments
            .nearby_body
            .unwrap();
        assert!(lifted.surface_distance_meters > lifting.surface_distance_meters);
        assert_eq!(lifted.radial_speed_meters_per_second, 0.0);

        let mut pose = ShipController::default().snapshot().pose;
        pose.position_meters[1] += 100.0;
        let mut ship = ShipController::flying(pose, 100);
        ship.trigger_landing_action();
        ship.set_cockpit_control(false);
        ship.advance(std::time::Duration::from_secs(2));
        let aligned = map_ship_to_renderer(ship.snapshot())
            .instruments
            .nearby_body
            .unwrap();
        assert_eq!(aligned.radial_speed_meters_per_second, 0.0);
        ship.advance(std::time::Duration::from_secs(1));
        let descending = map_ship_to_renderer(ship.snapshot())
            .instruments
            .nearby_body
            .unwrap();
        assert!(descending.surface_distance_meters < aligned.surface_distance_meters);
        assert!(descending.radial_speed_meters_per_second < 0.0);
    }

    #[test]
    fn runtime_mapping_preserves_spheres_light_camera_and_separate_markers() {
        let snapshot = CameraPrototype::default().snapshot();
        let (camera, instances, spheres, light) = map_world_to_renderer(snapshot);
        let body_distances = camera_body_distances(snapshot);
        let metrics = camera_domain_metrics(snapshot, &body_distances);

        assert_eq!(camera.position_meters, snapshot.camera.position.meters());
        assert_eq!(camera.target_meters, snapshot.camera.target.meters());
        assert_eq!(instances.len(), 3);
        assert_eq!(spheres.len(), 6);
        assert_eq!(snapshot.celestial_bodies.len(), 6);
        assert_eq!(snapshot.precision_markers.len(), 3);
        for (instance, body) in spheres.iter().zip(snapshot.celestial_bodies) {
            assert_eq!(instance.center_meters, body.center.meters());
            assert_eq!(instance.radius_meters, body.radius_meters);
            assert_eq!(
                instance.material == salimon_renderer::SurfaceMaterial::Emissive,
                body.id == CelestialBodyId::Sun
            );
        }
        assert_eq!(
            light.position_meters,
            snapshot.celestial_bodies[0].center.meters()
        );
        for (instance, marker) in instances.iter().zip(snapshot.precision_markers) {
            assert_eq!(instance.center_meters, marker.absolute_center.meters());
            assert_eq!(instance.half_extents_meters, marker.half_extents_meters);
            assert_eq!(instance.color, marker.color);
        }
        assert_eq!(metrics.camera_position, Some(camera.position_meters));
        assert_eq!(
            metrics.camera_altitude_meters,
            Some(snapshot.camera.altitude_meters)
        );
        assert_eq!(metrics.camera_phase, Some("approach"));
        assert_eq!(metrics.camera_paused, Some(false));
        assert_eq!(metrics.nearby_bodies, body_distances);
    }

    #[test]
    fn interaction_zones_follow_seat_aisle_and_rear_door() {
        let aisle = [0.50, 1.997_311_827_956_989_2, -2.20];
        assert_eq!(
            interaction_target(aisle, [2.76, 1.799_032_258_064_516, 0.0]),
            Some(InteractionTarget::Cockpit)
        );
        assert_eq!(
            interaction_target(aisle, [-1.76, 2.195_591_397_849_462, -4.40]),
            None,
        );
        assert_eq!(
            interaction_target(
                [-7.10, 1.344_086_021_505_376_3, 0.0],
                [-8.10, 1.344_086_021_505_376_3, 0.0],
            ),
            Some(InteractionTarget::ExitDoor)
        );
        assert_eq!(
            interaction_target(
                [-2.0, 1.997_311_827_956_989_2, 3.0],
                [2.76, 1.799_032_258_064_516, 0.0],
            ),
            None
        );
        assert_eq!(
            interaction_target(
                [-7.0, 1.997_311_827_956_989_2, 4.0],
                [-8.0, 1.997_311_827_956_989_2, 4.0],
            ),
            None
        );
    }

    #[test]
    fn cockpit_interaction_requires_ship_interior_but_exterior_door_remains_usable() {
        let aisle = [0.50, 1.997_311_827_956_989_2, -2.20];
        let cockpit = [2.76, 1.799_032_258_064_516, 0.0];
        assert_eq!(
            available_interaction_target(CharacterLocation::InsideShip, aisle, cockpit),
            Some(InteractionTarget::Cockpit)
        );
        for location in [CharacterLocation::DoorwayBlend, CharacterLocation::Surface] {
            assert_eq!(available_interaction_target(location, aisle, cockpit), None);
            assert_eq!(
                available_interaction_target(
                    location,
                    [-7.10, 1.344_086_021_505_376_3, 0.0],
                    [-8.10, 1.344_086_021_505_376_3, 0.0],
                ),
                Some(InteractionTarget::ExitDoor)
            );
        }
    }

    #[test]
    fn cockpit_prompt_is_shown_only_for_the_aimed_nearby_cockpit() {
        let ship = ShipController::default().snapshot();
        assert_eq!(
            ship.flight_state,
            FlightState::Landed {
                body: CelestialBodyId::Earth
            }
        );
        assert_eq!(
            gameplay_window_title(None, Some(InteractionTarget::Cockpit)),
            format!("Salimon — Compressed Solar System — {COCKPIT_INTERACTION_PROMPT}")
        );
        assert_eq!(
            gameplay_window_title(None, None),
            "Salimon — Compressed Solar System"
        );
    }

    #[test]
    fn action_bar_context_tracks_ship_prompt_state_transitions() {
        let mut landed = ShipController::default();
        assert_eq!(
            action_bar_context(landed.contextual_cockpit_message(), None),
            None
        );
        assert_eq!(
            action_bar_context(None, Some(InteractionTarget::ExitDoor)),
            Some("E to open/close exit door")
        );

        landed.set_cockpit_control(true);
        assert_eq!(
            action_bar_context(landed.contextual_cockpit_message(), None),
            Some("Press L to take off")
        );
        landed.toggle_door();
        landed.trigger_landing_action();
        assert_eq!(
            action_bar_context(landed.contextual_cockpit_message(), None),
            Some("Close door before takeoff")
        );
        landed.toggle_door();
        assert_eq!(
            action_bar_context(landed.contextual_cockpit_message(), None),
            Some("Press L to take off")
        );
        landed.set_cockpit_control(false);
        assert_eq!(
            action_bar_context(landed.contextual_cockpit_message(), None),
            None
        );

        let pose = ShipController::default().snapshot().pose;
        let flying = ShipController::flying(pose, 0);
        assert_eq!(
            action_bar_context(flying.contextual_cockpit_message(), None),
            Some("Press L to land")
        );
        assert_eq!(
            action_bar_context(
                Some(CockpitMessage::DoorLockedWhileInFlight),
                Some(super::InteractionTarget::Cockpit),
            ),
            Some(super::COCKPIT_INTERACTION_PROMPT),
            "timed blocked feedback is owned by ActionBar, not stale ship state"
        );
    }

    #[test]
    fn initial_overview_fully_frames_every_sphere() {
        let snapshot = CameraPrototype::default().snapshot();
        let aspect_ratio = INITIAL_WIDTH as f64 / INITIAL_HEIGHT as f64;
        let vertical_tangent = (snapshot.camera.vertical_field_of_view_radians * 0.5).tan();
        let horizontal_tangent = vertical_tangent * aspect_ratio;

        for body in snapshot.celestial_bodies {
            let offset = body.center.offset_from(snapshot.camera.position);
            let forward_depth = -offset[2];
            let nearest_face_depth = forward_depth - body.radius_meters;
            assert!(
                nearest_face_depth > snapshot.camera.physical_near_plane_meters,
                "{}'s nearest face must be in front of the overview camera",
                body.name
            );
            assert!(
                offset[0].abs() + body.radius_meters <= nearest_face_depth * horizontal_tangent,
                "{} must fit horizontally in the initial overview",
                body.name
            );
            assert!(
                offset[1].abs() + body.radius_meters <= nearest_face_depth * vertical_tangent,
                "{} must fit vertically in the initial overview",
                body.name
            );
        }
    }

    #[test]
    fn initial_drawable_requests_the_phase_zero_benchmark_resolution() {
        assert_eq!(initial_window_size().width, 1920);
        assert_eq!(initial_window_size().height, 1080);
    }

    #[test]
    fn diagnostics_reports_every_body_and_near_view_earth_is_closest() {
        let mut prototype = CameraPrototype::default();
        prototype.apply_command(CameraCommand::JumpToNear);
        let snapshot = prototype.snapshot();
        let distances = camera_body_distances(snapshot);

        assert_eq!(distances.len(), CELESTIAL_BODIES.len());
        assert_eq!(
            distances.iter().map(|body| body.name).collect::<Vec<_>>(),
            CELESTIAL_BODIES
                .iter()
                .map(|body| body.name)
                .collect::<Vec<_>>()
        );
        assert!(distances.iter().all(|body| body.distance_meters >= 0.0));
        let nearest = distances
            .iter()
            .min_by(|first, second| first.distance_meters.total_cmp(&second.distance_meters))
            .expect("catalog is nonempty");
        let earth = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .expect("Earth is canonical");

        assert_eq!(nearest.name, earth.name);
        assert!((nearest.distance_meters - 12.0).abs() <= f64::EPSILON);
    }
}
