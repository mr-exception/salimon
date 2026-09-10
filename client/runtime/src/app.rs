use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use salimon_diagnostics::{
    BodyDistance, Diagnostics, DomainMetrics, FrameSample, GpuMemory, GpuTime,
};
use salimon_renderer::{
    CameraFrame, GpuFrameTime, OverlayImage as RendererOverlayImage, PointLight, RenderOutcome,
    Renderer, SceneFrame, SceneInstance, SphereInstance, SurfaceMaterial, SurfaceSize,
};
use salimon_world::{
    CameraCommand, CameraPrototype, CelestialBodyId, TransitionPhase, WorldSnapshot,
};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

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

#[derive(Default)]
struct ClientApplication {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_clock: FrameClock,
    update_clock: UpdateClock,
    camera_prototype: CameraPrototype,
    diagnostics: Diagnostics,
    render_attempts: u64,
    retry_at: Option<Instant>,
    occluded: bool,
    failure: Option<String>,
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
        let world_snapshot = self.camera_prototype.snapshot();
        let (camera, scene_instances, spheres, light) = map_world_to_renderer(world_snapshot);
        let body_distances = camera_body_distances(world_snapshot);
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
                    camera_domain_metrics(world_snapshot, &body_distances),
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

fn camera_body_distances(snapshot: WorldSnapshot<'_>) -> Vec<BodyDistance<'static>> {
    snapshot
        .celestial_bodies
        .iter()
        .map(|body| BodyDistance {
            name: body.name,
            distance_meters: body.surface_distance_from(snapshot.camera.position),
        })
        .collect()
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
        INITIAL_HEIGHT, INITIAL_WIDTH, camera_body_distances, camera_command,
        camera_domain_metrics, is_diagnostics_toggle, map_world_to_renderer,
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
