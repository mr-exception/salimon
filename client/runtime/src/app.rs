mod atmosphere;
mod diagnostics;
pub(crate) mod input;
#[cfg(test)]
mod input_tests;
mod interaction;
mod native;
mod organisms;
mod presentation;
mod scene;
mod sky;
mod weather;

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

use crate::action_bar::ActionBar;
use crate::automation::{self, Request};
use crate::e2e;
use crate::frame_clock::FrameClock;
use crate::session::ViewMode;
use crate::session::interaction::InteractionTarget;

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
    if let Some(path) = std::env::var_os("SALIMON_UNIVERSE_PACK") {
        application.universe_stream = Some(
            salimon_universe::streaming::Stream::open(
                std::path::Path::new(&path),
                Default::default(),
            )
            .map_err(|e| RunError::new("open universe stream", e))?,
        );
    }
    if config.is_none() && benchmark.is_none() {
        application.condition_feedback = crate::condition_feedback::Presenter::open();
        application.session = crate::session::GameplaySession::new_canonical_run()
            .map_err(|e| RunError::new("canonical journey startup failed", e))?;
    }
    // Isolated automation stays off personal preferences unless its caller supplies
    // an explicit test-owned file. Exercise real settings IO without user saves.
    if config.is_some() && std::env::var_os("SALIMON_SETTINGS_FILE").is_some() {
        application.condition_feedback = crate::condition_feedback::Presenter::open();
    }
    if config.is_none() && benchmark.is_none() {
        application.session.wait_for_verified_launch();
        application.launch = Some(
            crate::launch::Launch::start(
                crate::checkpoints::default_directory()
                    .map_err(|e| RunError::new("save directory", e))?,
            )
            .map_err(|e| RunError::new("start journey recovery", e))?,
        );
    }
    application.preserve_startup_backlog = application.checkpoints.is_some();
    if let Some(config) = config {
        application.e2e_input_isolated =
            std::env::var_os("SALIMON_E2E_ISOLATE_INPUT").is_some_and(|v| v == "1");
        if application.e2e_input_isolated {
            log::info!(
                "E2E desktop input isolated; protocol controls retained across focus resets"
            );
        }
        if config.scenario == e2e::Scenario::CanonicalTerrain {
            application.scout_reference = crate::scout_reference::find("universe-orbit");
        }
        if config.scenario == e2e::Scenario::HallPrototype {
            application.hall_prototype = Some(Default::default());
        }
        e2e::initialize(&mut application.session, config)
            .map(|()| application.e2e_config = Some(config))
            .map_err(|error| RunError::new("E2E setup failed", error))?;
        if let Some(directory) = std::env::var_os("SALIMON_E2E_LAUNCH_DIR") {
            if config.scenario != e2e::Scenario::CanonicalJourney {
                return Err(RunError::new(
                    "offline launch probe",
                    "requires canonical-journey fixture",
                ));
            }
            application.session.wait_for_verified_launch();
            application.launch = Some(
                crate::launch::Launch::start(directory.into())
                    .map_err(|e| RunError::new("offline launch probe", e))?,
            );
        }
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
    launch: Option<crate::launch::Launch>,
    launch_region_requested: bool,
    launch_exit_requested: bool,
    preserve_startup_backlog: bool,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_clock: FrameClock,
    pub(crate) e2e_config: Option<e2e::Config>,
    e2e_ready_sent: bool,
    pub(crate) e2e_input_isolated: bool,
    automation: Option<Receiver<Request>>,
    pub(crate) universe_stream: Option<salimon_universe::streaming::Stream>,
    pub(crate) terrain_stream: crate::terrain_patches::TerrainStream,
    pub(crate) observations: crate::terrain_observations::Observations,
    pub(crate) capture_status: String,
    pub(crate) scout_reference: Option<crate::scout_reference::View>,
    pub(crate) hall_prototype: Option<crate::hall_prototype::Prototype>,
    capture_path: Option<std::path::PathBuf>,
    cursor_captured: bool,
    wheel_raster: salimon_renderer::ConsumableWheelRaster,
    native_focused: bool,
    diagnostics: Diagnostics,
    action_bar: ActionBar,
    render_attempts: u64,
    retry_at: Option<Instant>,
    pub(crate) survival_hud: crate::survival_hud::Hud,
    journey_ended: crate::journey_ended::Screen,
    condition_feedback: crate::condition_feedback::Presenter,
    scout_audio: crate::scout_audio::Presenter,
    occluded: bool,
    failure: Option<String>,
}

impl Default for ClientApplication {
    fn default() -> Self {
        Self {
            session: crate::session::GameplaySession::default(),
            benchmark: None,
            checkpoints: None,
            launch: None,
            launch_region_requested: false,
            launch_exit_requested: false,
            preserve_startup_backlog: false,
            window: None,
            renderer: None,
            frame_clock: FrameClock::default(),
            e2e_config: None,
            e2e_ready_sent: false,
            e2e_input_isolated: false,
            automation: None,
            universe_stream: None,
            terrain_stream: crate::terrain_patches::TerrainStream::default(),
            observations: crate::terrain_observations::Observations::default(),
            capture_status: "idle".into(),
            scout_reference: None,
            hall_prototype: None,
            capture_path: None,
            cursor_captured: false,
            wheel_raster: Default::default(),
            native_focused: false,
            diagnostics: Diagnostics::default(),
            action_bar: ActionBar::default(),
            render_attempts: 0,
            retry_at: None,
            survival_hud: Default::default(),
            journey_ended: Default::default(),
            condition_feedback: Default::default(),
            scout_audio: Default::default(),
            occluded: false,
            failure: None,
        }
    }
}

impl ClientApplication {
    pub(crate) fn fixture_launch(
        &mut self,
        seconds: u64,
        hold: bool,
        backward_clock: bool,
    ) -> Result<(), String> {
        use salimon_persistence::storage::{NativeStore, SaveManager};
        if self.e2e_config.is_none()
            || self.session.motion.is_none()
            || self.launch.is_some()
            || seconds > 86400
        {
            return Err(
                "requires a canonical E2E session, no pending launch, and 0..86400 seconds".into(),
            );
        }
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let serial = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "salimon-e2e-launch-{}-{serial}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).map_err(|e| e.to_string())?;
        let saved = crate::checkpoints::now()
            .map_err(|e| e.to_string())?
            .max(self.session.offline.watermark);
        let bytes = self.session.save_bytes(saved).map_err(|e| e.to_string())?;
        let mut store =
            SaveManager::new(NativeStore::open(directory.clone()).map_err(|e| e.to_string())?);
        store
            .checkpoint(&bytes, |b| {
                crate::session::GameplaySession::default().restore_bytes(b)
            })
            .map_err(|e| e.to_string())?;
        drop(store);
        let utc = if backward_clock {
            saved.saturating_sub(1)
        } else {
            saved.checked_add(seconds).ok_or("fixture clock overflow")?
        };
        let job =
            crate::launch::Launch::start_at(directory, utc, hold).map_err(|e| e.to_string())?;
        self.checkpoints.take();
        self.session.wait_for_verified_launch();
        self.scout_audio.stop();
        self.launch = Some(job);
        self.launch_region_requested = false;
        if let Some(window) = &self.window {
            release_cursor(window);
        }
        self.cursor_captured = false;
        Ok(())
    }
    pub(crate) fn release_launch_fixture(&self) -> bool {
        self.launch
            .as_ref()
            .is_some_and(crate::launch::Launch::release_fixture)
    }
    fn poll_launch(&mut self) {
        if let Some((checkpoints, mut session)) =
            self.launch.as_mut().and_then(crate::launch::Launch::poll)
        {
            session.policy = self.session.policy;
            session.wait_for_verified_launch();
            self.session = session;
            self.checkpoints = Some(checkpoints);
            self.terrain_stream = Default::default();
            self.launch_region_requested = false;
            self.preserve_startup_backlog = true;
            self.frame_clock.reset_interval();
        }
    }
    fn launch_ready(&self) -> bool {
        self.launch
            .as_ref()
            .is_some_and(crate::launch::Launch::is_ready)
            && self.launch_region_requested
            && self.terrain_presentation_ready()
            && self.universe_stream.as_ref().is_none_or(|s| s.ready())
    }
    pub(crate) fn launch_observation(&self) -> serde_json::Value {
        self.launch.as_ref().map_or(serde_json::Value::Null, |job| {
            let mut v = job.observation();
            if self.e2e_config.is_some() {
                v["fixture_directory"] = job.directory().to_string_lossy().to_string().into();
            }
            v["source_ready"] = job.is_ready().into();
            v["ready"] = self.launch_ready().into();
            if job.is_ready() && !self.launch_ready() {
                v["progress"] = serde_json::to_value(crate::launch_progress::Progress::at(
                    crate::launch_progress::Stage::Region,
                ))
                .expect("bounded progress");
                v["percentage"] = serde_json::Value::Null;
            }
            v
        })
    }
    pub(crate) fn audio_telemetry(&self) -> serde_json::Value {
        let mut v = self.scout_audio.telemetry();
        v["settings"] =
            serde_json::to_value(self.condition_feedback.settings).expect("bounded settings");
        v
    }
    pub(crate) fn ship_resource_stats(&self) -> Option<[u64; 4]> {
        self.renderer.as_ref().map(Renderer::ship_resource_stats)
    }
    pub(crate) fn fixture_batch_stats(&self) -> Option<salimon_renderer::FixtureBatchStats> {
        self.renderer.as_ref().map(Renderer::fixture_batch_stats)
    }

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
            || (self.e2e_config.is_none()
                && self.session.mining.terrain == salimon_world::terrain::TerrainMode::Canonical)
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
        self.cursor_captured =
            !self.session.survival.snapshot().terminal && capture_cursor(&window);
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
        self.journey_ended.clear_pointer();
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
            self.release_input_capture();
            self.session.reset_presentation();
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
            "minimized":w.is_minimized(), "focused":self.native_focused,"occluded":self.occluded, "inner_size":[w.inner_size().width,w.inner_size().height]
        }))
    }
    /// The only host restart path: publish archive and both fresh generations
    /// before exposing gameplay. Native and automation callers share it.
    pub(crate) fn restart_terminal(&mut self) -> bool {
        if !self.session.survival.snapshot().terminal {
            return false;
        }
        let result = match &mut self.checkpoints {
            Some(checkpoints) => checkpoints.restart(&mut self.session),
            None => Err(salimon_persistence::invalid(
                "verified restart storage unavailable",
            )),
        };
        match result {
            Ok(()) => {
                self.reset_update_clock();
                self.terrain_stream = Default::default();
                self.action_bar = Default::default();
                self.survival_hud = Default::default();
                self.journey_ended = Default::default();
                self.cursor_captured = false;
                true
            }
            Err(error) => {
                log::error!("restart failed: {error}");
                self.journey_ended.restart_failed();
                if let Some(checkpoints) = &mut self.checkpoints {
                    checkpoints.warning =
                        Some("RESTART FAILED - terminal save retained; retry Restart Game");
                }
                false
            }
        }
    }

    fn input_policy(&self) -> InputPolicy {
        InputPolicy {
            gameplay: self.session.view_mode == ViewMode::Gameplay,
            captured: self.cursor_captured || self.session.wheel.open,
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
        log::trace!(
            "input key={key:?} pressed={pressed} source={source:?} ship_controls={:?} orientation={:?}",
            self.session.input.steering(),
            self.session.ship.snapshot().pose.orientation,
        );
        if self.e2e_input_isolated && matches!(source, InputSource::Native) {
            return;
        }
        if self.launch.is_some() {
            if pressed && !repeat && !synthetic && self.input_policy().admits_press(source, true) {
                if matches!(
                    key,
                    PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter)
                ) && self.launch_ready()
                {
                    self.launch.take();
                    self.session.resume_verified_launch();
                    log::info!("verified journey resumed");
                    self.frame_clock.reset_interval();
                    if let Some(window) = &self.window {
                        self.cursor_captured = capture_cursor(window);
                    }
                } else if key == PhysicalKey::Code(KeyCode::KeyR)
                    && self
                        .launch
                        .as_ref()
                        .is_some_and(crate::launch::Launch::failed)
                {
                    let directory = self.launch.as_ref().expect("launch present").directory();
                    match crate::launch::Launch::start(directory) {
                        Ok(job) => self.launch = Some(job),
                        Err(error) => {
                            self.session.feedback = Some(if error.to_string().is_empty() {
                                "Journey recovery could not start"
                            } else {
                                "Journey recovery could not start - check storage and clock"
                            })
                        }
                    }
                } else if key == PhysicalKey::Code(KeyCode::KeyX) {
                    self.launch_exit_requested = true;
                }
            }
            return;
        }
        if self.session.survival.snapshot().terminal {
            if self.input_policy().admits_press(source, true)
                && input::terminal_restart_pressed(
                    if pressed {
                        ElementState::Pressed
                    } else {
                        ElementState::Released
                    },
                    repeat,
                    synthetic,
                    key,
                )
            {
                self.restart_terminal();
            }
            return;
        }
        if self.session.is_paused()
            && pressed
            && !repeat
            && !synthetic
            && self.input_policy().admits_press(source, true)
            && !self.condition_feedback.settings_open
        {
            use crate::checkpoints::control::Request;
            let request = match key {
                PhysicalKey::Code(KeyCode::KeyN) => Some(Request::SaveNow),
                PhysicalKey::Code(KeyCode::KeyX) => Some(Request::SaveAndExit),
                PhysicalKey::Code(KeyCode::KeyR) => Some(Request::Retry),
                PhysicalKey::Code(KeyCode::KeyC) => Some(Request::CancelExit),
                PhysicalKey::Code(KeyCode::KeyQ) => Some(Request::ExitWithRisk),
                _ => None,
            };
            if let Some(request) = request {
                if let Some(checkpoints) = &mut self.checkpoints {
                    checkpoints.request(request, &self.session);
                }
                return;
            }
        }
        let pause_key = self.session.view_mode == ViewMode::Gameplay
            && (key == PhysicalKey::Code(KeyCode::Escape)
                || (self.session.is_paused()
                    && matches!(
                        key,
                        PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter)
                    )));
        if self.session.is_paused()
            && pressed
            && !repeat
            && !synthetic
            && self.input_policy().admits_press(source, true)
        {
            match key {
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.condition_feedback.settings_open = !self.condition_feedback.settings_open
                }
                PhysicalKey::Code(KeyCode::KeyC) if self.condition_feedback.settings_open => {
                    self.condition_feedback.toggle_setting(0)
                }
                PhysicalKey::Code(KeyCode::KeyA) if self.condition_feedback.settings_open => {
                    self.condition_feedback.toggle_setting(1)
                }
                PhysicalKey::Code(KeyCode::KeyV) if self.condition_feedback.settings_open => {
                    self.condition_feedback.toggle_setting(2)
                }
                PhysicalKey::Code(KeyCode::KeyM) if self.condition_feedback.settings_open => {
                    self.condition_feedback.toggle_setting(3)
                }
                PhysicalKey::Code(KeyCode::KeyB) if self.condition_feedback.settings_open => {
                    self.condition_feedback.toggle_setting(4)
                }
                PhysicalKey::Code(KeyCode::KeyE) if self.condition_feedback.settings_open => {
                    self.condition_feedback.toggle_setting(6)
                }
                PhysicalKey::Code(KeyCode::KeyF) if self.condition_feedback.settings_open => {
                    self.condition_feedback.toggle_setting(7)
                }
                PhysicalKey::Code(KeyCode::KeyT) if self.condition_feedback.settings_open => {
                    self.condition_feedback.toggle_setting(8)
                }
                PhysicalKey::Code(KeyCode::KeyG) if self.condition_feedback.settings_open => {
                    self.condition_feedback.toggle_setting(5)
                }
                _ => {}
            }
        }
        if pause_key && self.session.surface_chart.open && !self.session.is_paused() {
            if pressed && !repeat && !synthetic && self.input_policy().admits_press(source, true) {
                self.session
                    .surface_key(crate::session::input::Key::Cancel, true);
            }
            return;
        }
        if pause_key && self.session.building.active && !self.session.is_paused() {
            if pressed && !repeat && !synthetic && self.input_policy().admits_press(source, true) {
                self.session.command(crate::session::Command::Build(
                    crate::session::construction::BuildAction::Toggle,
                ));
            }
            return;
        }
        if pause_key {
            if pressed && !repeat && !synthetic && self.input_policy().admits_press(source, true) {
                if self.session.is_paused()
                    && let Some(checkpoints) = &mut self.checkpoints
                {
                    checkpoints.request(
                        crate::checkpoints::control::Request::CancelExit,
                        &self.session,
                    );
                }
                self.session.command(crate::session::Command::TogglePause);
                self.scout_audio.stop();
                self.condition_feedback.settings_open = false;
                self.survival_hud.close_details();
                self.cursor_captured = false;
                if let Some(window) = &self.window {
                    if self.session.is_paused() {
                        release_cursor(window);
                    } else {
                        self.cursor_captured = capture_cursor(window);
                    }
                }
            }
            return;
        }
        if self.session.is_paused() {
            return;
        }
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
        if pressed
            && self.session.view_mode == ViewMode::Gameplay
            && !self.session.crafting_ui.open
            && !self.session.archive_reader.open
            && !self.session.wheel.open
            && !self.session.survival.snapshot().terminal
        {
            match key {
                PhysicalKey::Code(KeyCode::KeyH) => {
                    self.survival_hud
                        .observe(crate::survival_hud::Observation::capture(&self.session));
                    self.survival_hud.toggle_details();
                    if self.survival_hud.details_open {
                        self.session.command(crate::session::Command::InspectVitals);
                    }
                    return;
                }
                PhysicalKey::Code(KeyCode::BracketLeft | KeyCode::BracketRight)
                    if self.survival_hud.details_open =>
                {
                    self.survival_hud
                        .next_page(key == PhysicalKey::Code(KeyCode::BracketRight));
                    return;
                }
                _ => {}
            }
        }
        if let Some(hall) = self.hall_prototype.as_mut() {
            if pressed && !repeat && !synthetic {
                let delta = match key {
                    PhysicalKey::Code(KeyCode::KeyW) => Some([0., -0.5]),
                    PhysicalKey::Code(KeyCode::KeyS) => Some([0., 0.5]),
                    PhysicalKey::Code(KeyCode::KeyA) => Some([-0.5, 0.]),
                    PhysicalKey::Code(KeyCode::KeyD) => Some([0.5, 0.]),
                    _ => None,
                };
                if let Some(delta) = delta {
                    let _ = hall.walk(delta);
                }
            }
            return;
        }
        let wheel_was_open = self.session.wheel.open;
        input::dispatch_session_key(&mut self.session, key, pressed);
        self.sync_wheel_cursor(wheel_was_open);
        self.show_session_feedback();
    }

    fn sync_wheel_cursor(&mut self, was_open: bool) {
        if was_open == self.session.wheel.open {
            return;
        }
        if let Some(window) = &self.window {
            if self.session.wheel.open {
                release_cursor(window);
                self.cursor_captured = false;
            } else if self.native_focused
                && !self.occluded
                && !self.session.is_paused()
                && !self.session.survival.snapshot().terminal
            {
                self.cursor_captured = capture_cursor(window);
            }
        }
    }

    fn set_input_focus(&mut self, focused: bool) {
        log::debug!(
            "native input focus={focused} ship_controls={:?} orientation={:?}",
            self.session.input.steering(),
            self.session.ship.snapshot().pose.orientation,
        );
        self.native_focused = focused;
        if !focused {
            self.release_input_capture();
        }
    }

    fn release_input_capture(&mut self) {
        log::trace!(
            "releasing native input: ship_controls={:?}",
            self.session.input.steering()
        );
        self.scout_audio.stop();
        self.cursor_captured = false;
        self.survival_hud.close_details();
        self.journey_ended.clear_pointer();
        if !self.e2e_input_isolated {
            self.session.clear_held_controls();
        }
    }

    fn reset_update_clock(&mut self) {
        if self.launch.is_some() {
            self.session.clear_held_controls();
            self.session.reset_presentation();
            return;
        }
        // Fixed-step protocol time/holds must not depend on desktop focus or resize.
        if self.e2e_input_isolated {
            return;
        }
        self.journey_ended.clear_pointer();
        self.survival_hud.close_details();
        self.session.update_clock.reset();
        if !self.e2e_input_isolated {
            self.session.clear_held_controls();
        }
        self.session.reset_presentation();
    }

    fn advance_native(&mut self, elapsed: Duration) -> (Duration, usize) {
        if self.launch.is_some() {
            return (Duration::ZERO, 0);
        }
        let started = Instant::now();
        if self.session.is_paused() || self.session.survival.snapshot().terminal {
            self.session.update_clock.reset();
        } else {
            self.session.update_clock.accumulate(elapsed);
        }
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
        self.condition_feedback
            .update(crate::condition_feedback::Observation::capture(
                &self.session,
                self.native_focused,
                self.occluded,
            ));
        let audio = crate::scout_audio::Observation::capture(
            &self.session,
            self.native_focused
                && !self.occluded
                && self.e2e_config.is_none()
                && self.benchmark.is_none(),
        );
        let condition_cue = self.condition_feedback.take_audio_cue();
        if self.hall_prototype.is_some() {
            self.scout_audio.stop();
        } else if self.e2e_config.is_some() {
            self.scout_audio.preview(
                crate::scout_audio::Observation::capture(&self.session, true),
                self.condition_feedback.settings,
            );
        } else {
            self.scout_audio
                .update(audio, self.condition_feedback.settings, condition_cue);
        }
        if self.scout_audio.unavailable() {
            self.condition_feedback.message =
                "Ship audio unavailable - check audio device/output".into();
        }
        if let Some(checkpoints) = &mut self.checkpoints {
            checkpoints.poll(&self.session);
        }
        if self.session.survival.snapshot().terminal && self.cursor_captured {
            if let Some(window) = &self.window {
                release_cursor(window);
            }
            self.release_input_capture();
        }
        (update_delta, ticks)
    }

    /// Observe host IO without stepping the paused session.
    fn pause_menu_text(&self) -> Option<String> {
        if let Some(job) = &self.launch {
            return Some(if job.is_ready() && !self.launch_ready() {
                crate::launch_progress::Progress::at(crate::launch_progress::Stage::Region)
                    .menu(None)
            } else {
                job.menu()
            });
        }
        if !self.session.is_paused() {
            return None;
        }
        let mut text = String::from(
            "SALIMON / PAUSED\n\nENTER / ESC   RESUME\nN   SAVE NOW\nS   SETTINGS\nX   SAVE & EXIT\n\n",
        );
        if let Some(error) = &self.session.motion_error {
            return Some(format!(
                "SALIMON / MOTION UNVERIFIED\n\nResume unavailable\n{error}\n\nClose and restore the last verified checkpoint."
            ));
        }
        use crate::checkpoints::control::Status;
        match &self.checkpoints {
            None => text.push_str("Saving unavailable in this session"),
            Some(checkpoints) => {
                let (status, verified, _) = checkpoints.status();
                text.push_str(match status {
                    Status::Idle => "No checkpoint verified this session",
                    Status::Saving => "Saving...",
                    Status::Saved { .. } => "Saved",
                    Status::Failed => "SAVE FAILED - check storage/permissions",
                });
                if let Some(utc) = verified {
                    text.push_str(&format!(
                        "\nLast verified: {}",
                        crate::checkpoints::control::timestamp(utc)
                    ));
                }
                if status == Status::Failed {
                    text.push_str("\nR   RETRY");
                }
                if checkpoints.exit_requested() {
                    text.push_str("\nC   CANCEL EXIT");
                    if status == Status::Failed {
                        text.push_str("\nQ   EXIT WITHOUT SAVING");
                    }
                }
            }
        }
        Some(text)
    }

    /// Background presentation stops, but the running, living session continues
    /// through the same bounded physical ticks and checkpoint scheduler.
    fn advance_background(&mut self, now: Instant) {
        let elapsed = self.session.update_clock.step(now);
        self.advance_native(elapsed);
    }
}

impl ApplicationHandler for ClientApplication {
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = &event {
            log::trace!(
                "native mouse delta={delta:?} captured={} focused={} occluded={} isolated={} view={:?}",
                self.cursor_captured,
                self.native_focused,
                self.occluded,
                self.e2e_input_isolated,
                self.session.view_mode,
            );
        }
        if self.benchmark.is_none()
            && !self.e2e_input_isolated
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
            self.cursor_captured = if self.launch.is_some() || self.session.is_paused() {
                release_cursor(window);
                false
            } else {
                capture_cursor(window)
            };
            window.request_redraw();
        }
        log::info!("application resumed");
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.scout_audio.stop();
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
        if (self.benchmark.is_some() || self.e2e_input_isolated)
            && matches!(
                event,
                WindowEvent::KeyboardInput { .. }
                    | WindowEvent::MouseInput { .. }
                    | WindowEvent::CursorMoved { .. }
                    | WindowEvent::MouseWheel { .. }
            )
        {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                if self.benchmark.is_some() {
                    self.fail(event_loop, "benchmark window closed before completion");
                } else if self.checkpoints.is_some()
                    && self.session.view_mode == ViewMode::Gameplay
                    && !self.session.survival.snapshot().terminal
                {
                    if !self.session.is_paused() {
                        self.session.command(crate::session::Command::TogglePause);
                    }
                    self.condition_feedback.settings_open = false;
                    release_cursor(&window);
                    self.release_input_capture();
                    if let Some(checkpoints) = &mut self.checkpoints {
                        checkpoints.request(
                            crate::checkpoints::control::Request::SaveAndExit,
                            &self.session,
                        );
                    }
                    window.request_redraw();
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
                self.release_input_capture();
                self.session.reset_presentation();
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
                    if self.session.is_paused() || self.session.survival.snapshot().terminal {
                        // Focus may have been lost before death. The capture
                        // flag is already clear then, but the OS cursor still
                        // needs explicit restoration for the restart button.
                        release_cursor(&window);
                        self.release_input_capture();
                    } else {
                        self.cursor_captured = capture_cursor(&window);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } if self.session.wheel.open => {
                let hovered = salimon_renderer::consumable_wheel_hit_test(
                    [window.inner_size().width, window.inner_size().height],
                    [position.x, position.y],
                )
                .and_then(|i| salimon_fabrication::ConsumableSlot::ALL.get(i).copied());
                self.session.command(crate::session::Command::Wheel(
                    crate::session::consumable_wheel::WheelAction::Hover(hovered),
                ));
            }
            WindowEvent::CursorLeft { .. } if self.session.wheel.open => {
                self.session.command(crate::session::Command::Wheel(
                    crate::session::consumable_wheel::WheelAction::Hover(None),
                ));
            }
            WindowEvent::MouseInput { state, button, .. } if self.session.wheel.open => {
                if button == MouseButton::Left && self.native_focused && !self.occluded {
                    let was_open = self.session.wheel.open;
                    self.session.command(crate::session::Command::Wheel(
                        crate::session::consumable_wheel::WheelAction::Primary(
                            state == ElementState::Pressed,
                        ),
                    ));
                    self.sync_wheel_cursor(was_open);
                    self.show_session_feedback();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.session.survival.snapshot().terminal
                    && self.native_focused
                    && !self.occluded
                {
                    self.journey_ended.pointer_moved([position.x, position.y]);
                    window.request_redraw();
                } else {
                    self.journey_ended.clear_pointer();
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.journey_ended.clear_pointer();
                if self.session.survival.snapshot().terminal {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. }
                if self.session.survival.snapshot().terminal =>
            {
                if state == ElementState::Pressed
                    && button == MouseButton::Left
                    && self.input_policy().admits_press(InputSource::Native, true)
                    && self.journey_ended.clicked_restart(
                        [window.inner_size().width, window.inner_size().height],
                        window.scale_factor(),
                    )
                {
                    self.restart_terminal();
                }
                window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } if !self.cursor_captured
                && self.native_focused
                && !self.occluded
                && !self.session.is_paused()
                && !self.session.survival.snapshot().terminal =>
            {
                self.cursor_captured = capture_cursor(&window);
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } if !is_synthetic
                && release_cursor_pressed(event.state, event.repeat, event.physical_key)
                && (event.physical_key == PhysicalKey::Code(KeyCode::F1)
                    || self.session.view_mode != ViewMode::Gameplay) =>
            {
                release_cursor(&window);
                self.release_input_capture();
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } if self.session.building.active => {
                if self.input_policy().admits_press(InputSource::Native, false) {
                    self.session.command(crate::session::Command::Build(
                        crate::session::construction::BuildAction::Cancel,
                    ));
                }
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
                if self.session.surface_chart.editing
                    && event.state == ElementState::Pressed
                    && !is_synthetic
                    && self.input_policy().admits_press(InputSource::Native, false)
                    && let Some(text) = &event.text
                    && !text.is_empty()
                    && !text.chars().any(char::is_control)
                {
                    let _ = self.session.surface_text(text);
                    window.request_redraw();
                    return;
                }
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
        self.poll_launch();
        if self.launch_exit_requested {
            event_loop.exit();
            return;
        }
        if self.launch.is_none()
            && let Some(checkpoints) = &mut self.checkpoints
        {
            checkpoints.poll(&self.session);
            if checkpoints.status().2 {
                event_loop.exit();
                return;
            }
        }
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
        if self.launch.is_some() {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(16),
            ));
            return;
        }
        if self.renderer.is_some()
            && self
                .window
                .as_ref()
                .is_some_and(|window| self.occluded || !surface_size(window).is_drawable())
        {
            self.advance_background(Instant::now());
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + crate::update_clock::GAME_TICK,
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
            && !checkpoints.status().2
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
    fn status_panel_input_uses_focus_capture_repeat_and_modal_guards() {
        let mut app = ClientApplication::default();
        let key = PhysicalKey::Code(KeyCode::KeyH);
        app.dispatch_key(key, true, false, false, InputSource::Native);
        assert!(!app.survival_hud.details_open);
        app.native_focused = true;
        app.cursor_captured = true;
        let before = app.session.save_bytes(1).unwrap();
        app.dispatch_key(key, true, false, false, InputSource::Native);
        assert!(app.survival_hud.details_open);
        assert_eq!(app.session.save_bytes(1).unwrap(), before);
        for (repeat, synthetic) in [(true, false), (false, true)] {
            app.dispatch_key(key, true, repeat, synthetic, InputSource::Native);
            assert!(app.survival_hud.details_open);
        }
        app.session.advance_game(Duration::from_millis(16));
        assert!(app.session.survival.snapshot().active_time > Duration::ZERO);
        app.set_input_focus(false);
        assert!(!app.survival_hud.details_open);
        app.native_focused = true;
        app.cursor_captured = true;
        app.dispatch_key(key, true, false, false, InputSource::Native);
        assert!(app.survival_hud.details_open);
        app.reset_update_clock();
        assert!(!app.survival_hud.details_open);
        app.session.crafting_ui.open = true;
        app.dispatch_key(key, true, false, false, InputSource::Native);
        assert!(!app.survival_hud.details_open);
        app.session.crafting_ui.open = false;
        app.session.view_mode = ViewMode::PrecisionTour;
        app.dispatch_key(key, true, false, false, InputSource::Native);
        assert!(!app.survival_hud.details_open);
        assert_ne!(app.session.save_bytes(1).unwrap(), before); // only the tick changed authority
    }

    #[test]
    fn background_ticks_preserve_active_time_travel_and_resume_without_double_counting() {
        let mut app = ClientApplication::default();
        app.session.ship = salimon_ship::ShipController::flying_with_velocity(
            salimon_ship::ShipPose {
                position_meters: [0.; 3],
                orientation: [0., 0., 0., 1.],
            },
            0,
            [100., 0., 0.],
        )
        .unwrap();
        let start = Instant::now();
        app.session.update_clock.step(start);
        app.set_input_focus(false);
        app.occluded = true;
        for n in 1..=10 {
            app.advance_background(start + Duration::from_millis(16 * n));
        }
        assert_eq!(
            app.session.survival.snapshot().active_time,
            Duration::from_millis(160)
        );
        assert!((app.session.run.journey.distance_meters - 16.).abs() < 1e-9);
        app.occluded = false;
        let elapsed = app
            .session
            .update_clock
            .step(start + Duration::from_millis(176));
        app.advance_native(elapsed);
        assert_eq!(
            app.session.survival.snapshot().active_time,
            Duration::from_millis(176)
        );
        assert!((app.session.run.journey.distance_meters - 17.6).abs() < 1e-9);
    }

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

#[cfg(test)]
mod launch_admission_tests {
    use super::*;
    #[test]
    fn pending_launch_rejects_resume_and_preserves_saved_work_during_ui_frames() {
        let mut app = ClientApplication::default();
        let config = e2e::Config {
            scenario: e2e::Scenario::CanonicalJourney,
            seed: 0,
            step: Duration::from_millis(16),
            fragment_load: None,
        };
        e2e::initialize(&mut app.session, config).unwrap();
        app.e2e_config = Some(config);
        app.session
            .update_clock
            .accumulate(Duration::from_millis(50));
        app.fixture_launch(60, true, false).unwrap();
        let directory = app.launch.as_ref().unwrap().directory();
        let deadline = Instant::now() + Duration::from_secs(10);
        while app.launch.as_ref().unwrap().progress().completed.is_zero() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        let before = app.session.save_bytes(1).unwrap();
        app.dispatch_key(
            PhysicalKey::Code(KeyCode::Enter),
            true,
            false,
            false,
            InputSource::Automation,
        );
        assert!(!app.launch_ready());
        assert!(app.session.is_paused());
        assert_eq!(
            app.advance_native(Duration::from_secs(60)),
            (Duration::ZERO, 0)
        );
        app.reset_update_clock();
        assert_eq!(
            app.session.update_clock.backlog(),
            Duration::from_millis(50)
        );
        assert_eq!(app.session.save_bytes(1).unwrap(), before);
        assert!(
            app.pause_menu_text()
                .unwrap()
                .contains("Resume unavailable")
        );
        drop(app);
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(test)]
mod launch_region_admission_tests {
    use super::*;
    #[test]
    fn durable_source_alone_cannot_bypass_current_region_readiness() {
        let mut app = ClientApplication::default();
        let config = e2e::Config {
            scenario: e2e::Scenario::CanonicalJourney,
            seed: 0,
            step: Duration::from_millis(16),
            fragment_load: None,
        };
        e2e::initialize(&mut app.session, config).unwrap();
        app.e2e_config = Some(config);
        app.fixture_launch(0, false, false).unwrap();
        let directory = app.launch.as_ref().unwrap().directory();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !app.launch.as_ref().unwrap().is_ready() {
            app.poll_launch();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(app.launch_observation()["source_ready"].as_bool().unwrap());
        assert!(!app.launch_ready());
        assert!(
            app.pause_menu_text()
                .unwrap()
                .contains("Preparing the current region")
        );
        app.dispatch_key(
            PhysicalKey::Code(KeyCode::Enter),
            true,
            false,
            false,
            InputSource::Automation,
        );
        assert!(app.session.is_paused());
        drop(app);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
