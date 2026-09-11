use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use salimon_character::{
    CharacterController, CharacterLocation, MovementInput, ShipFrame, SurfaceFrame,
};
use salimon_diagnostics::{
    BodyDistance, Diagnostics, DomainMetrics, FrameSample, GpuMemory, GpuTime,
};
use salimon_renderer::{
    CameraFrame, GpuFrameTime, OverlayImage as RendererOverlayImage, PointLight, RenderOutcome,
    Renderer, SceneFrame, SceneInstance, ShipMeshInstance, SphereInstance, SurfaceMaterial,
    SurfaceSize,
};
use salimon_ship::{DoorState, FlightState, ShipController, ShipPose, ShipSnapshot};
use salimon_world::{
    CELESTIAL_BODIES, CameraCommand, CameraPrototype, CelestialBodyId, TransitionPhase,
    WorldPosition, WorldSnapshot,
};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

use crate::frame_clock::FrameClock;
use crate::update_clock::UpdateClock;

const WINDOW_TITLE: &str = "Salimon — Compressed Solar System";
const INITIAL_WIDTH: f64 = 1280.0;
const INITIAL_HEIGHT: f64 = 720.0;
const MINIMUM_WIDTH: f64 = 640.0;
const MINIMUM_HEIGHT: f64 = 360.0;
const TIMING_LOG_INTERVAL: u64 = 300;
const RENDER_RETRY_DELAY: Duration = Duration::from_millis(50);
const IDLE_RETRY_DELAY: Duration = Duration::from_millis(250);

pub(crate) fn run() -> Result<(), RunError> {
    let event_loop = EventLoop::new()
        .map_err(|error| RunError::new("failed to create native event loop", error))?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut application = ClientApplication::default();
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

struct ClientApplication {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_clock: FrameClock,
    update_clock: UpdateClock,
    camera_prototype: CameraPrototype,
    character: CharacterController,
    ship: ShipController,
    movement_input: MovementInput,
    view_mode: ViewMode,
    cursor_captured: bool,
    diagnostics: Diagnostics,
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
            movement_input: MovementInput::default(),
            view_mode: ViewMode::Gameplay,
            cursor_captured: false,
            diagnostics: Diagnostics::default(),
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

fn interaction_target(local_eye_position: [f64; 3]) -> Option<InteractionTarget> {
    if (local_eye_position[0] - 2.76).abs() < 4.0 && local_eye_position[2].abs() < 2.2 {
        Some(InteractionTarget::Cockpit)
    } else if (local_eye_position[0] + 7.10).abs() < 2.7 && local_eye_position[2].abs() < 2.3 {
        Some(InteractionTarget::ExitDoor)
    } else {
        None
    }
}

impl ClientApplication {
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), RunError> {
        if self.window.is_some() {
            return Ok(());
        }

        let attributes = WindowAttributes::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT))
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

    fn interact(&mut self) {
        let ship_snapshot = self.ship.snapshot();
        let ship_frame = character_ship_frame(ship_snapshot.pose);
        let surface = earth_surface_frame();
        match self.character.location() {
            CharacterLocation::Cockpit => {
                self.character.leave_cockpit();
                self.ship.set_cockpit_control(false);
                log::info!("left cockpit control; ship motion remains autonomous");
            }
            CharacterLocation::InsideShip
            | CharacterLocation::DoorwayBlend
            | CharacterLocation::Surface => {
                let character = self.character.snapshot(ship_frame, surface);
                let local = ship_frame.world_to_local(character.eye_position_meters);
                match interaction_target(local) {
                    Some(InteractionTarget::Cockpit) => {
                        self.character.enter_cockpit();
                        self.ship.set_cockpit_control(true);
                        log::info!("entered cockpit control");
                    }
                    Some(InteractionTarget::ExitDoor) => {
                        self.ship.toggle_door();
                        if let Some(message) = self.ship.snapshot().cockpit_message {
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
        let update_delta = self.update_clock.step(update_started_at);
        self.camera_prototype.advance(update_delta);
        self.ship.advance(update_delta);
        let ship_snapshot = self.ship.snapshot();
        let ship_frame = character_ship_frame(ship_snapshot.pose);
        let surface_frame = earth_surface_frame();
        if self.view_mode == ViewMode::Gameplay {
            self.character.advance(
                update_delta,
                self.movement_input,
                ship_frame,
                surface_frame,
                ship_snapshot.door_state == DoorState::Open,
                matches!(ship_snapshot.flight_state, FlightState::Landed { .. }),
            );
        }
        let world_snapshot = self.camera_prototype.snapshot();
        let (mut camera, scene_instances, spheres, light) = map_world_to_renderer(world_snapshot);
        let character_snapshot = self.character.snapshot(ship_frame, surface_frame);
        let ship_mesh = if self.view_mode == ViewMode::Gameplay {
            camera = CameraFrame {
                position_meters: character_snapshot.eye_position_meters,
                target_meters: character_snapshot.look_target_meters,
                up: character_snapshot.up,
                vertical_fov_radians: 70.0_f32.to_radians(),
                near_plane_meters: 0.05,
            };
            Some(ShipMeshInstance {
                position_meters: ship_snapshot.pose.position_meters,
                orientation: ship_snapshot.pose.orientation.map(|value| value as f32),
                door_open: ship_snapshot.door_state == DoorState::Open,
            })
        } else {
            None
        };
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
            });
        let outcome = match self
            .renderer
            .as_mut()
            .expect("redraws require an initialized renderer")
            .render(
                SceneFrame {
                    camera,
                    instances: &scene_instances,
                    spheres: &spheres,
                    light: Some(light),
                    ship: ship_mesh,
                },
                overlay_image,
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
                    self.cursor_captured = false;
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
            }
            WindowEvent::KeyboardInput { event, .. }
                if interaction_pressed(event.state, event.repeat, event.physical_key) =>
            {
                self.interact();
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if view_toggle_pressed(event.state, event.repeat, event.physical_key) =>
            {
                self.view_mode = match self.view_mode {
                    ViewMode::Gameplay => ViewMode::PrecisionTour,
                    ViewMode::PrecisionTour => ViewMode::Gameplay,
                };
                log::info!("view mode changed to {:?} (F2 toggles)", self.view_mode);
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

fn character_ship_frame(pose: ShipPose) -> ShipFrame {
    ShipFrame {
        origin_meters: pose.position_meters,
        axes: pose.axes(),
    }
}

fn earth_surface_frame() -> SurfaceFrame {
    let earth = CELESTIAL_BODIES
        .iter()
        .find(|body| body.id == CelestialBodyId::Earth)
        .expect("world catalog always contains Earth");
    SurfaceFrame {
        body_center_meters: earth.center.meters(),
        radius_meters: earth.radius_meters,
    }
}

fn interaction_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::KeyE)
}

fn view_toggle_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::F2)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MovementKey {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
}

fn movement_key(key: PhysicalKey) -> Option<MovementKey> {
    match key {
        PhysicalKey::Code(KeyCode::KeyW) => Some(MovementKey::Forward),
        PhysicalKey::Code(KeyCode::KeyS) => Some(MovementKey::Backward),
        PhysicalKey::Code(KeyCode::KeyA) => Some(MovementKey::Left),
        PhysicalKey::Code(KeyCode::KeyD) => Some(MovementKey::Right),
        PhysicalKey::Code(KeyCode::Space) => Some(MovementKey::Jump),
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
    let forward = ship.pose.axes()[0];
    DomainMetrics {
        camera_position: Some(player_position),
        player_position: Some(player_position),
        ship_position: Some(ship.pose.position_meters),
        ship_velocity: Some(forward.map(|value| value * ship.speed_meters_per_second)),
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
        INITIAL_HEIGHT, INITIAL_WIDTH, InteractionTarget, camera_body_distances, camera_command,
        camera_domain_metrics, interaction_target, is_diagnostics_toggle, map_world_to_renderer,
        release_cursor_pressed,
    };
    use salimon_world::{CELESTIAL_BODIES, CameraCommand, CameraPrototype, CelestialBodyId};
    use winit::event::ElementState;
    use winit::keyboard::{KeyCode, PhysicalKey};

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
    fn task10_interaction_zones_follow_enlarged_asset_markers() {
        assert_eq!(
            interaction_target([2.76, 2.77, 0.0]),
            Some(InteractionTarget::Cockpit)
        );
        assert_eq!(
            interaction_target([-7.10, 2.50, 0.0]),
            Some(InteractionTarget::ExitDoor)
        );
        assert_eq!(interaction_target([-2.0, 2.08, 3.0]), None);
    }

    #[test]
    fn initial_overview_fully_frames_every_sphere() {
        let snapshot = CameraPrototype::default().snapshot();
        let aspect_ratio = INITIAL_WIDTH / INITIAL_HEIGHT;
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
