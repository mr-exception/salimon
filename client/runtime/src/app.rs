mod diagnostics;
pub(crate) mod input;
#[cfg(test)]
mod input_tests;
mod interaction;
mod native;
mod scene;

use std::error::Error;
use std::fmt;
use std::io::Write;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use salimon_character::CharacterLocation;
use salimon_diagnostics::Diagnostics;
use salimon_renderer::{
    CameraFrame, OverlayImage as RendererOverlayImage, OverlayPlacement, RenderOutcome, Renderer,
    SceneFrame,
};
#[cfg(test)]
use salimon_ship::ShipController;
#[cfg(test)]
use salimon_world::CameraPrototype;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::session::frames::character_ship_frame;
use diagnostics::{
    camera_body_distances_with_terrain, camera_domain_metrics, gameplay_domain_metrics,
};
use input::{InputPolicy, InputSource, release_cursor_pressed};
use interaction::{action_bar_context, gameplay_window_title};
use native::{capture_cursor, release_cursor, surface_size, window_attributes};
use scene::{map_ship_to_renderer, map_world_to_renderer};

use crate::action_bar::ActionBar;
use crate::automation::{self, Request};
use crate::e2e;
use crate::frame_clock::FrameClock;
use crate::session::interaction::{InteractionTarget, available_interaction_target};
use crate::session::{ViewMode, door_passable};

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
            .session
            .enable_terrain(earth, [0.0, 1.0, 0.0])
            .map_err(|e| RunError::new("terrain startup failed", e))?;
    }
    if config.is_none() && benchmark.is_none() {
        application.checkpoints = Some(
            crate::checkpoints::Checkpoints::open(
                crate::checkpoints::default_directory()
                    .map_err(|e| RunError::new("save directory", e))?,
                &mut application.session,
            )
            .map_err(|e| RunError::new("restore active run", e))?,
        );
    }
    application.preserve_startup_backlog = application.checkpoints.is_some();
    if let Some(config) = config {
        e2e::initialize(&mut application.session, config)
            .map(|()| application.e2e_config = Some(config))
            .map_err(|error| RunError::new("E2E setup failed", error))?;
    }
    if let Some(config) = benchmark {
        crate::benchmark::initialize(&mut application.session, config)
            .map_err(|error| RunError::new("benchmark fixture failed", error))?;
        crate::benchmark::wait_for_start(config)
            .map_err(|error| RunError::new("benchmark start failed", error))?;
        application.benchmark = Some(crate::benchmark::Recorder::new(
            config,
            &application.session,
        ));
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
    pub(crate) session: crate::session::GameplaySession,
    pub(crate) benchmark: Option<crate::benchmark::Recorder>,
    checkpoints: Option<crate::checkpoints::Checkpoints>,
    preserve_startup_backlog: bool,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_clock: FrameClock,
    pub(crate) e2e_config: Option<e2e::Config>,
    e2e_ready_sent: bool,
    automation: Option<Receiver<Request>>,
    pub(crate) terrain_stream: crate::terrain_patches::TerrainStream,
    pub(crate) observations: crate::terrain_observations::Observations,
    pub(crate) capture_status: String,
    capture_path: Option<std::path::PathBuf>,
    cursor_captured: bool,
    native_focused: bool,
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
            session: crate::session::GameplaySession::default(),
            benchmark: None,
            checkpoints: None,
            preserve_startup_backlog: false,
            window: None,
            renderer: None,
            frame_clock: FrameClock::default(),
            e2e_config: None,
            e2e_ready_sent: false,
            automation: None,
            terrain_stream: crate::terrain_patches::TerrainStream::default(),
            observations: crate::terrain_observations::Observations::default(),
            capture_status: "idle".into(),
            capture_path: None,
            cursor_captured: false,
            native_focused: false,
            diagnostics: Diagnostics::default(),
            action_bar: ActionBar::default(),
            render_attempts: 0,
            retry_at: None,
            occluded: false,
            failure: None,
        }
    }
}

impl ClientApplication {
    pub(crate) fn show_session_feedback(&mut self) {
        if let Some(message) = self.session.feedback.take() {
            self.action_bar.show_transient(message);
        }
    }
    pub(crate) fn advance_feedback(&mut self, elapsed: Duration) {
        self.action_bar.advance(elapsed);
    }
    pub(crate) fn terrain_presentation_ready(&self) -> bool {
        (!self.terrain_stream.has_target || !self.terrain_stream.active.is_empty())
            && self
                .renderer
                .as_ref()
                .is_some_and(|r| r.terrain_presentation_ready(&self.terrain_stream.active))
    }
    pub(crate) fn terrain_upload_times(&self) -> Option<(u128, u128)> {
        self.renderer.as_ref().map(Renderer::terrain_upload_times)
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
        if self.session.mining.terrain == salimon_world::terrain::TerrainMode::Representative
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
        self.native_focused = window.has_focus();
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
        if !std::mem::take(&mut self.preserve_startup_backlog) {
            self.reset_update_clock();
        }
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

    fn world_seed(&self) -> u64 {
        self.session.seed
    }

    pub(crate) fn e2e_step(&self) -> Option<Duration> {
        self.e2e_config.map(|config| config.step)
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
    pub(crate) fn visual_context(&self) -> serde_json::Value {
        let Some(renderer) = self.renderer.as_ref() else {
            return serde_json::Value::Null;
        };
        let Some(window) = self.window.as_ref() else {
            return serde_json::Value::Null;
        };
        let info = renderer.info();
        serde_json::json!({"backend": info.backend, "adapter": info.adapter_name,
            "driver": info.driver, "viewport_pixels": [window.inner_size().width, window.inner_size().height],
            "scale_factor": window.scale_factor()})
    }
    pub(crate) fn window_state(&self) -> serde_json::Value {
        self.window.as_ref().map_or(serde_json::Value::Null,|w|serde_json::json!({
            "minimized":w.is_minimized(), "inner_size":[w.inner_size().width,w.inner_size().height]
        }))
    }
    fn input_policy(&self) -> InputPolicy {
        InputPolicy {
            gameplay: self.session.view_mode == ViewMode::Gameplay,
            captured: self.cursor_captured,
            focused: self.native_focused,
            occluded: self.occluded,
        }
    }

    pub(crate) fn automation_key(&mut self, key: PhysicalKey, pressed: bool) {
        self.dispatch_key(key, pressed, false, false, InputSource::Automation);
    }

    fn dispatch_key(
        &mut self,
        key: PhysicalKey,
        pressed: bool,
        repeat: bool,
        synthetic: bool,
        source: InputSource,
    ) {
        if !self
            .input_policy()
            .admits_key(source, key, pressed, repeat, synthetic)
        {
            return;
        }
        if pressed && key == PhysicalKey::Code(KeyCode::F3) {
            self.diagnostics.toggle();
            return;
        }
        input::dispatch_session_key(&mut self.session, key, pressed);
        self.show_session_feedback();
    }

    fn set_input_focus(&mut self, focused: bool) {
        self.native_focused = focused;
        if !focused {
            self.release_input_capture();
        }
    }

    fn release_input_capture(&mut self) {
        self.cursor_captured = false;
        self.session.clear_held_controls();
    }

    fn reset_update_clock(&mut self) {
        self.session.update_clock.reset();
        self.session.clear_held_controls();
        self.session.reset_presentation();
    }

    fn advance_native(&mut self, elapsed: Duration) -> (Duration, usize) {
        let started = Instant::now();
        self.session.update_clock.accumulate(elapsed);
        if let Some(benchmark) = &mut self.benchmark {
            benchmark.latest_physics.clear_work();
            self.session.latest_physics.clear_work();
        }
        let mut update_delta = Duration::ZERO;
        let mut ticks = 0;
        for _ in 0..crate::update_clock::MAX_TICKS_PER_RENDER {
            // Always allow one tick; subsequent catch-up is allowed only while
            // within the budget. An individual dense contact solve is not preemptible.
            if ticks > 0 && started.elapsed() >= crate::update_clock::CATCH_UP_BUDGET {
                break;
            }
            let Some(tick) = self.session.update_clock.take_tick() else {
                break;
            };
            self.session.advance_game(tick);
            self.action_bar.advance(tick);
            update_delta += tick;
            ticks += 1;
        }
        if let Some(benchmark) = &mut self.benchmark {
            benchmark.latest_physics = self.session.latest_physics;
            benchmark.latest_scheduler = crate::benchmark::SchedulerSample {
                ticks,
                backlog: self.session.update_clock.backlog(),
                lifecycle_excluded: self.session.update_clock.lifecycle_excluded(),
            };
        }
        if let Some(checkpoints) = &mut self.checkpoints {
            checkpoints.poll(&self.session);
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
            self.session.update_clock.step(update_started_at)
        };
        let (update_delta, ticks) = self.advance_native(elapsed);
        let simulation_time = update_started_at.elapsed();
        self.session.sync_carried();
        self.show_session_feedback();
        let ship_snapshot = self.session.ship.snapshot();
        let ship_frame = character_ship_frame(ship_snapshot.pose);
        let surface_frame = self.session.terrain_frame(ship_snapshot);
        let world_snapshot = self.session.camera_prototype.snapshot();
        let (mut camera, scene_instances, mut spheres, light) =
            map_world_to_renderer(world_snapshot);
        let mut resource_meshes = Vec::new();
        let character_snapshot = self.session.character.snapshot(ship_frame, surface_frame);
        let presentation = self.session.presentation();
        let render_ship_frame = character_ship_frame(presentation.ship.pose);
        let monitor_message = self.session.ship.contextual_cockpit_message();
        let interaction = if self.session.view_mode == ViewMode::Gameplay
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
        ) && (self.session.mining.session.carried_id().is_some()
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
                        interaction::prompt_placement(target, render_ship_frame)
                    })
                },
            }
        });
        let contextual_action = if fragment_interaction {
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
        self.action_bar
            .set_contextual(contextual_action.map(|prompt| prompt.text));
        window.set_title(&self.benchmark.as_ref().map_or_else(
            || {
                let title = gameplay_window_title(monitor_message, interaction);
                self.checkpoints
                    .as_ref()
                    .and_then(|c| c.warning)
                    .map_or(title.clone(), |warning| format!("{title} | {warning}"))
            },
            crate::benchmark::Recorder::window_title,
        ));
        let ship_mesh = if self.session.view_mode == ViewMode::Gameplay {
            camera = CameraFrame {
                position_meters: presentation.character.eye_position_meters,
                target_meters: presentation.character.look_target_meters,
                up: presentation.character.up,
                vertical_fov_radians: 70.0_f32.to_radians(),
                near_plane_meters: 0.05,
            };
            let mut mesh = map_ship_to_renderer(presentation.ship);
            mesh.fabricator_output_present = self.session.fabricator_output_present();
            mesh.fabricator_output_medical = self.session.fabricator_output_medical();
            mesh.instruments.fabricator = self.session.fabricator_instruments();
            Some(mesh)
        } else {
            None
        };
        if self.session.view_mode == ViewMode::Gameplay {
            match self
                .session
                .mining
                .nearby(camera.position_meters, self.world_seed())
            {
                Ok(deposits) => {
                    resource_meshes.extend(deposits.iter().copied().filter_map(|entry| {
                        crate::resource_presentation::deposit_mesh_with_terrain(
                            entry,
                            self.session.mining.terrain,
                        )
                    }));
                }
                Err(error) => log::error!("deposit presentation query failed: {error:?}"),
            }
        }
        if self.session.view_mode == ViewMode::Gameplay {
            for fragment in self.session.mining.nearby_fragments(camera.position_meters) {
                if self.session.mining.session.attached_sphere_id() == Some(fragment.id()) {
                    continue;
                }
                resource_meshes.push(presentation.fragment_mesh(&self.session, fragment));
            }
        }
        if self.session.view_mode == ViewMode::Gameplay
            && let Some(preview) = self
                .session
                .mining
                .growth_preview(&self.session.equipment, presentation.character)
        {
            resource_meshes.push(preview);
        }
        let render_terrain = if self.session.view_mode == ViewMode::Gameplay {
            self.session.mining.terrain
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
        let crafting_text = self.session.crafting_text();
        let crafting_pixels = crafting_text
            .as_ref()
            .map(|text| crate::action_bar::rasterize_action_bar(text, 3));
        let overlay_image = crafting_pixels
            .as_ref()
            .map(|(width, height, pixels)| RendererOverlayImage {
                width: *width,
                height: *height,
                rgba8: pixels,
                revision: {
                    use std::hash::{Hash, Hasher};
                    let mut hash = std::collections::hash_map::DefaultHasher::new();
                    crafting_text.hash(&mut hash);
                    hash.finish()
                },
                placement: OverlayPlacement::Center,
            })
            .or_else(|| {
                self.diagnostics
                    .overlay()
                    .map(|image| RendererOverlayImage {
                        width: image.width,
                        height: image.height,
                        rgba8: image.rgba8,
                        revision: image.revision,
                        placement: OverlayPlacement::TopLeft,
                    })
            });
        let action_bar_image = self
            .action_bar
            .image(self.session.view_mode == ViewMode::Gameplay)
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
        let held_item = self
            .session
            .held_item(presentation.character, mining_target.is_some());
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
                crate::reticle::image(
                    self.session.view_mode == ViewMode::Gameplay,
                    &self.session.equipment,
                ),
                (self.session.view_mode == ViewMode::Gameplay)
                    .then(|| self.session.equipment_presentation()),
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
                let ready = window.has_focus()
                    && window.is_visible() == Some(true)
                    && window.is_minimized() != Some(true)
                    && !self.occluded;
                if let Some(benchmark) = self.benchmark.as_mut()
                    && benchmark.native_readiness_changed(ready)
                    && let Err(error) = benchmark.observe_native("presented_window_poll", ready)
                {
                    self.fail(event_loop, error);
                    return;
                }
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
                    if self.session.view_mode == ViewMode::Gameplay {
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
                        self.session.update_clock.backlog().as_secs_f64() * 1_000.0
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
                    let accounting = crate::benchmark::accounting(&self.session);
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
                                self.session
                                    .character
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
                if let Some(benchmark) = self.benchmark.as_mut()
                    && let Err(error) = benchmark.observe_native("surface_lost", false)
                {
                    self.fail(event_loop, error);
                    return;
                }
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
            && self.session.view_mode == ViewMode::Gameplay
            && self.cursor_captured
            && let DeviceEvent::MouseMotion { delta } = event
        {
            self.session
                .command(crate::session::Command::Look(delta.0, delta.1));
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
            self.native_focused = window.has_focus();
            self.cursor_captured = capture_cursor(window);
            window.request_redraw();
        }
        log::info!("application resumed");
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(benchmark) = self.benchmark.as_mut()
            && let Err(error) = benchmark.observe_native("suspended", false)
        {
            self.fail(event_loop, error);
            return;
        }
        event_loop.set_control_flow(ControlFlow::Wait);
        self.renderer = None;
        self.native_focused = false;
        self.cursor_captured = false;
        self.retry_at = None;
        self.frame_clock.reset_interval();
        self.reset_update_clock();
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
                    && let Some(benchmark) = self.benchmark.as_mut()
                    && let Err(error) = benchmark.observe_native("occluded", false)
                {
                    self.fail(event_loop, error);
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
                self.set_input_focus(focused);
                if let Some(benchmark) = self.benchmark.as_mut()
                    && let Err(error) = benchmark.observe_native(
                        if focused { "focused" } else { "unfocused" },
                        focused && window.is_visible() == Some(true) && !self.occluded,
                    )
                {
                    self.fail(event_loop, error);
                    return;
                }
                if focused {
                    self.cursor_captured = capture_cursor(&window);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } if !self.cursor_captured && self.native_focused && !self.occluded => {
                self.cursor_captured = capture_cursor(&window);
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } if !is_synthetic
                && release_cursor_pressed(event.state, event.repeat, event.physical_key) =>
            {
                release_cursor(&window);
                self.release_input_capture();
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                if !pressed || self.input_policy().admits_press(InputSource::Native, false) {
                    self.session
                        .command(crate::session::Command::Mining(pressed));
                }
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                self.dispatch_key(
                    event.physical_key,
                    event.state == ElementState::Pressed,
                    event.repeat,
                    is_synthetic,
                    InputSource::Native,
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
        if let Some(mut checkpoints) = self.checkpoints.take()
            && let Err(error) = checkpoints.finish(&self.session)
        {
            log::error!("exit checkpoint failed: {error}");
            self.failure = Some(format!("exit checkpoint failed: {error}"));
        }
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
        whole.session.ship = salimon_ship::ShipController::flying(pose, 10);
        partitioned.session.ship = whole.session.ship.clone();
        whole
            .session
            .advance_automation(Duration::from_millis(1003));
        for ms in [7, 201, 1, 394, 400] {
            partitioned
                .session
                .advance_automation(Duration::from_millis(ms));
        }
        assert_eq!(
            whole.session.ship.snapshot(),
            partitioned.session.ship.snapshot()
        );
        assert_eq!(
            whole.session.update_clock.backlog(),
            Duration::from_millis(11)
        );
        assert_eq!(
            whole.session.update_clock.backlog(),
            partitioned.session.update_clock.backlog()
        );
    }

    #[test]
    fn stalled_native_scheduler_matches_shared_fixed_gameplay_ticks() {
        let mut app = ClientApplication::default();
        let mut reference = ClientApplication::default();
        for instance in [&mut app, &mut reference] {
            crate::benchmark_fixture::initialize(
                &mut instance.session,
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
                reference
                    .session
                    .advance_game(crate::update_clock::GAME_TICK);
            }
            advanced += delta;
            assert_eq!(
                advanced + app.session.update_clock.backlog(),
                Duration::from_secs(2)
            );
            assert_eq!(
                app.session.ship.snapshot().pose,
                reference.session.ship.snapshot().pose
            );
            assert_eq!(
                app.session.mining.session.fragments(),
                reference.session.mining.session.fragments()
            );
            if app.session.update_clock.backlog() < crate::update_clock::GAME_TICK {
                break;
            }
        }
        assert_eq!(advanced, Duration::from_secs(2));
    }

    #[test]
    fn numeric_inputs_select_gameplay_slots_without_changing_the_tour() {
        let mut app = ClientApplication::default();
        let initial_camera = app.session.camera_prototype.snapshot().camera;
        for (key, slot) in [
            (KeyCode::Digit1, ToolbarSlot::One),
            (KeyCode::Digit2, ToolbarSlot::Two),
            (KeyCode::Digit3, ToolbarSlot::Three),
            (KeyCode::Digit4, ToolbarSlot::Four),
            (KeyCode::Digit5, ToolbarSlot::Five),
        ] {
            app.automation_key(PhysicalKey::Code(key), true);
            app.automation_key(PhysicalKey::Code(key), false);
            assert_eq!(app.session.equipment.selected(), Some(slot));
            assert_eq!(app.session.view_mode, ViewMode::Gameplay);
            assert_eq!(
                app.session.camera_prototype.snapshot().camera,
                initial_camera
            );
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
            app.session.equipment.select(slot, false);
            app.session.mining.set_mouse_held(true);
            app.session.carry_action(true);
            assert!(app.session.mining.session.carried_id().is_none());
            assert_eq!(app.session.equipment.selected(), Some(slot));
            assert!(
                app.session.mining.input.held,
                "failed pickup must not stow or cancel mining"
            );
            assert_eq!(
                app.session.mining.carry_feedback,
                Some("Aim at a fragment within 3 m")
            );
        }
    }

    #[test]
    fn precision_tour_numbers_leave_equipment_selection_intact() {
        let mut app = ClientApplication::default();
        app.automation_key(PhysicalKey::Code(KeyCode::Digit4), true);
        app.session.view_mode = ViewMode::PrecisionTour;
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
                app.session.camera_prototype.snapshot().camera,
                expected.snapshot().camera
            );
            assert_eq!(app.session.equipment.selected(), Some(ToolbarSlot::Four));
        }
    }
}
