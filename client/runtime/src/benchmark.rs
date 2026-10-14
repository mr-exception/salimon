//! Opt-in benchmark orchestration. JSON is emitted after sampling, not per frame.
use crate::{
    app::{ClientApplication, character_ship_frame, surface_frame_for_ship},
    benchmark_fixture::{self, Layout},
    fragment_physics::Measurements,
};
use salimon_renderer::{GpuFrameTime, RenderStats};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Config {
    pub(crate) count: usize,
    pub(crate) layout: Layout,
    pub(crate) seed: u64,
    pub(crate) active: bool,
    pub(crate) away: bool,
    pub(crate) both_cameras: bool,
    pub(crate) headless: bool,
    pub(crate) warmup_seconds: u64,
    pub(crate) sample_seconds: u64,
    pub(crate) minimum_samples: usize,
    pub(crate) settle_steps: u64,
    pub(crate) repeats: usize,
    pub(crate) defer_start: bool,
    pub(crate) baseline_samples: usize,
}

impl Config {
    fn windows(self) -> usize {
        self.repeats * if self.both_cameras { 2 } else { 1 }
    }
}

pub(crate) fn parse(args: &[String]) -> Result<Config, String> {
    let mut config = Config {
        count: 500,
        layout: Layout::Dense,
        seed: 155,
        active: false,
        away: false,
        both_cameras: false,
        headless: false,
        warmup_seconds: 10,
        sample_seconds: 30,
        minimum_samples: 0,
        settle_steps: 0,
        repeats: 1,
        defer_start: false,
        baseline_samples: 0,
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut args = args.iter();
    if args.next().map(String::as_str) != Some("--benchmark") {
        return Err("expected --benchmark".into());
    }
    while let Some(key) = args.next() {
        if !seen.insert(key) {
            return Err(format!("{key} specified twice"));
        }
        let value = args
            .next()
            .ok_or_else(|| format!("{key} requires a value"))?;
        let integer = || {
            value
                .parse::<u64>()
                .map_err(|_| format!("{key} requires a decimal u64"))
        };
        match key.as_str() {
            "--count" => {
                let n = integer()?;
                if n > 1000 {
                    return Err("count must be 0..1000".into());
                }
                config.count = n as usize;
            }
            "--layout" => config.layout = Layout::parse(value)?,
            "--seed" => config.seed = integer()?,
            "--state" => {
                config.active = match value.as_str() {
                    "active" => true,
                    "settled" => false,
                    _ => return Err("state must be active or settled".into()),
                }
            }
            "--camera" => {
                config.both_cameras = value == "both";
                config.away = match value.as_str() {
                    "facing" | "both" => false,
                    "away" => true,
                    _ => return Err("camera must be facing, away or both".into()),
                }
            }
            "--mode" => {
                config.headless = match value.as_str() {
                    "physics" => true,
                    "native" => false,
                    _ => return Err("mode must be native or physics".into()),
                }
            }
            "--warmup-seconds" => {
                config.warmup_seconds = integer()?;
                if config.warmup_seconds > 600 {
                    return Err("warmup must be 0..600 seconds".into());
                }
            }
            "--sample-seconds" => {
                config.sample_seconds = integer()?;
                if !(1..=600).contains(&config.sample_seconds) {
                    return Err("sample must be 1..600 seconds".into());
                }
            }
            "--settle-steps" => {
                config.settle_steps = integer()?;
                if config.settle_steps > 37500 {
                    return Err("settle steps must be 0..37500 (600 simulation seconds)".into());
                }
            }
            "--minimum-samples" => {
                let samples = integer()?;
                if samples > 1000 {
                    return Err("minimum samples must be 0..1000".into());
                }
                config.minimum_samples = samples as usize;
            }
            "--repeats" => {
                let repeats = integer()?;
                if !(1..=10).contains(&repeats) {
                    return Err("repeats must be 1..10".into());
                }
                config.repeats = repeats as usize;
            }
            "--defer-start" => {
                config.defer_start = match value.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err("defer start must be true or false".into()),
                }
            }
            "--baseline-samples" => {
                let samples = integer()?;
                if samples > 1000 {
                    return Err("baseline samples must be 0..1000".into());
                }
                config.baseline_samples = samples as usize;
            }
            _ => return Err(format!("unknown benchmark option {key}")),
        }
    }
    if config.active && config.settle_steps != 0 {
        return Err("active initial conditions cannot be pre-settled".into());
    }
    if config.active && config.repeats != 1 {
        return Err("active initial conditions require separate fresh-process repeats".into());
    }
    if config.both_cameras && (config.active || config.headless) {
        return Err("both cameras requires a settled native population".into());
    }
    if config.baseline_samples > 0
        && (config.active || config.settle_steps == 0 || config.count == 0)
    {
        return Err("solver baseline requires a naturally pre-settled nonempty population".into());
    }
    Ok(config)
}

pub(crate) fn initialize(app: &mut ClientApplication, config: Config) -> Result<(), String> {
    benchmark_fixture::initialize(app, config.count, config.layout, config.seed, config.active)?;
    benchmark_fixture::settle(app, config.count, config.settle_steps)?;
    // Camera remains a production character camera, facing the aft population.
    if !config.away {
        app.character
            .apply_mouse_delta(std::f64::consts::PI / 0.0022, 0.0);
    }
    app.character.apply_mouse_delta(0.0, 220.0);
    if config.defer_start {
        // Explicit runner barrier before creating a native window or starting
        // clocks. Other fixtures can prepare concurrently, then all CPU work
        // stops while each real native sampling window runs alone.
        eprintln!("SALIMON_BENCHMARK_PREPARED");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("benchmark start: {e}"))?;
        if input.trim_end() != "SALIMON_BENCHMARK_START" {
            return Err("expected SALIMON_BENCHMARK_START on stdin".into());
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SchedulerSample {
    pub(crate) ticks: usize,
    pub(crate) backlog: Duration,
    pub(crate) lifecycle_excluded: Duration,
}

#[derive(Clone, Copy)]
struct Sample {
    frame_ms: Option<f64>,
    simulation_delta_ms: f64,
    update_ms: f64,
    simulation_ms: f64,
    physics: Measurements,
    scheduler: SchedulerSample,
    render: Option<RenderStats>,
}

pub(crate) struct Recorder {
    pub(crate) config: Config,
    started: Option<Instant>,
    last_presented: Option<Instant>,
    samples: Vec<Sample>,
    pub(crate) latest_physics: Measurements,
    pub(crate) latest_scheduler: SchedulerSample,
    info: Value,
    initial: Value,
    finished: bool,
    observed: usize,
    stride: usize,
    completed: Vec<Value>,
    baseline: Option<Value>,
}

impl Recorder {
    pub(crate) fn restart_before_sampling(&mut self) -> bool {
        if self
            .samples
            .iter()
            .any(|sample| self.config.headless || sample.frame_ms.is_some())
        {
            return false;
        }
        // macOS can report initial occlusion after the first drawable. No
        // retained frame interval may span that gap; restart the full warm-up.
        self.started = None;
        self.last_presented = None;
        self.samples.clear();
        self.observed = 0;
        self.stride = 1;
        true
    }
    pub(crate) fn surface_changed(&self, width: u32, height: u32, scale: f64) -> bool {
        self.info.get("width").is_some()
            && !(self.info["width"].as_u64() == Some(u64::from(width))
                && self.info["height"].as_u64() == Some(u64::from(height))
                && self.info["scale_factor"].as_f64() == Some(scale))
    }
    pub(crate) fn window_title(&self) -> String {
        let away = self.config.away
            || (self.config.both_cameras && self.completed.len() >= self.config.repeats);
        format!(
            "Salimon — benchmark: {} {} {} {} {}/{}",
            self.config.count,
            self.config.layout.name(),
            if self.config.active {
                "active initial"
            } else {
                "settled requested"
            },
            if away { "away" } else { "facing" },
            self.completed.len() % self.config.repeats + 1,
            self.config.repeats,
        )
    }
    pub(crate) fn switch_camera(&self) -> bool {
        self.config.both_cameras && self.completed.len() == self.config.repeats
    }
    pub(crate) fn new(config: Config, app: &ClientApplication) -> Self {
        Self {
            config,
            started: None,
            last_presented: None,
            samples: Vec::with_capacity(4096),
            latest_physics: Measurements::default(),
            latest_scheduler: SchedulerSample::default(),
            info: Value::Null,
            initial: accounting(app),
            finished: false,
            observed: 0,
            stride: 1,
            completed: Vec::new(),
            baseline: solver_baseline(config, app),
        }
    }
    pub(crate) fn set_renderer(
        &mut self,
        info: &salimon_renderer::RendererInfo,
        width: u32,
        height: u32,
        scale_factor: f64,
    ) {
        self.info = json!({"adapter": info.adapter_name, "backend": info.backend,
            "device_type": info.device_type, "gpu_timestamps_supported": info.timestamp_queries_supported,
            "width": width, "height": height, "scale_factor": scale_factor, "present_mode": "AutoVsync"});
    }
    pub(crate) fn record(
        &mut self,
        delta: Duration,
        update: Duration,
        simulation: Duration,
        render: Option<RenderStats>,
    ) -> bool {
        // The first redraw can have zero simulation delta and no completed
        // physics observation. Do not report default zero counters as a wake.
        if delta.is_zero() && self.started.is_none() {
            return false;
        }
        let now = Instant::now();
        let start = *self.started.get_or_insert(now);
        let interval = self
            .last_presented
            .replace(now)
            .map(|last| now.duration_since(last));
        if now.duration_since(start) >= Duration::from_secs(self.config.warmup_seconds) {
            self.observed += 1;
            if self.samples.len() == 4096 {
                self.samples = self.samples.iter().step_by(2).copied().collect();
                self.stride *= 2;
            }
            if self.observed.is_multiple_of(self.stride) {
                self.samples.push(Sample {
                    // Raw monotonic intervals, never the diagnostics' 250ms clamp.
                    frame_ms: render.and(interval).map(ms),
                    simulation_delta_ms: ms(delta),
                    update_ms: ms(update),
                    simulation_ms: ms(simulation),
                    physics: self.latest_physics,
                    scheduler: self.latest_scheduler,
                    render,
                });
            }
        }
        now.duration_since(start)
            >= Duration::from_secs(self.config.warmup_seconds + self.config.sample_seconds)
            && self
                .samples
                .iter()
                .filter(|s| self.config.headless || s.frame_ms.is_some())
                .count()
                >= self.config.minimum_samples
    }
    pub(crate) fn finish(&mut self, app_accounting: Value) -> Result<bool, String> {
        if self.finished {
            return Err("benchmark already finished".into());
        }
        if self.samples.is_empty() {
            return Err("benchmark recorded no samples".into());
        }
        if app_accounting != self.initial {
            return Err("benchmark lost fragments or changed mass/identity".into());
        }
        let samples: Vec<Value> = self.samples.iter().map(|s| {
            let p = s.physics.solver;
            let r = s.render;
            let batch = r.map(|r| r.resource_batch);
            json!({"frame_ms": s.frame_ms, "simulation_delta_ms": s.simulation_delta_ms,
                "game_ticks": (!self.config.headless).then_some(s.scheduler.ticks), "game_backlog_ms": (!self.config.headless).then_some(ms(s.scheduler.backlog)), "lifecycle_excluded_ms": (!self.config.headless).then_some(ms(s.scheduler.lifecycle_excluded)), "cpu_update_ms": s.update_ms, "cpu_simulation_ms": s.simulation_ms, "cpu_scene_prepare_ms": (s.update_ms - s.simulation_ms).max(0.0), "physics_total_ms": ms(s.physics.total_time),
                "physics_adapter_ms": ms(s.physics.adapter_time), "physics_integration_ms": ms(p.integration_time),
                "physics_contact_ms": ms(p.contact_time), "physics_activation_ms": ms(p.activation_time), "objects_simulated": p.objects,
                "moving_objects": s.physics.moving_objects, "awake_objects": p.awake_objects, "sleeping_objects": p.sleeping_objects, "quiet_objects": p.quiet_objects, "integrated_objects": p.integrated_objects,
                "substeps": p.substeps, "solver_passes": p.solver_passes, "pair_visits": p.pair_visits,
                "radius_candidates": p.radius_candidates, "narrow_phase_tests": p.narrow_phase_tests,
                "contacts": p.contacts, "projection_builds": p.projection_builds,
                "matrix_allocations": p.matrix_allocations, "projection_storage_bytes": p.projection_storage_bytes,
                "cpu_render_ms": r.map(|r| ms(r.cpu_render_time)),
                "resource_prepare_ms": batch.map(|b| ms(b.prepare_time)), "resource_upload_ms": batch.map(|b| ms(b.upload_time)),
                "resource_upload_bytes": batch.map(|b| b.upload_bytes), "resource_triangles": batch.map(|b| b.triangles),
                "resource_scratch_capacity_bytes": batch.map(|b| b.scratch_capacity_bytes),
                "draw_calls": r.map(|r| r.total_draw_calls), "rendered_objects": r.map(|r| r.rendered_objects),
                "gpu_latest_ms": r.and_then(|r| if let GpuFrameTime::Measured(d) = r.gpu_frame_time { Some(ms(d)) } else { None }),
                "gpu_status": r.map(|r| match r.gpu_frame_time { GpuFrameTime::Measured(_) => "measured_latest", GpuFrameTime::Pending => "pending", GpuFrameTime::Unsupported => "unsupported" })})
        }).collect();
        let output = json!({"schema": 1, "mode": if self.config.headless { "physics_wall_clock" } else { "native_wall_clock" },
            "config": {"count": self.config.count, "layout": self.config.layout.name(), "seed": self.config.seed,
                "initial_state": if self.config.active { "active" } else { "settled_requested" },
                "camera": if self.config.away || (self.config.both_cameras && self.completed.len() >= self.config.repeats) { "away" } else { "facing" },
                "warmup_seconds": self.config.warmup_seconds, "sample_seconds": self.config.sample_seconds,
                "minimum_samples": self.config.minimum_samples,
                "settle_steps_limit": self.config.settle_steps},
            "sample_wall_seconds": self.last_presented.zip(self.started).map(|(end, start)|
                end.duration_since(start).as_secs_f64() - self.config.warmup_seconds as f64),
            "renderer": self.info, "accounting": self.initial, "accounting_preserved": true,
            "solver_baseline": self.baseline,
            "updates_observed": self.observed, "sampling_stride": self.stride,
            "elapsed_seconds": self.started.map(|s| s.elapsed().as_secs_f64()),
            "samples": samples});
        self.completed.push(output);
        self.finished = self.completed.len() == self.config.windows();
        if self.finished {
            if self.config.windows() == 1 {
                println!("{}", self.completed[0]);
            } else {
                println!("{}", json!({"schema": 2, "runs": self.completed}));
            }
        } else {
            self.started = None;
            self.last_presented = None;
            self.samples.clear();
            self.observed = 0;
            self.stride = 1;
        }
        Ok(self.finished)
    }
}

fn solver_baseline(config: Config, app: &ClientApplication) -> Option<Value> {
    if config.baseline_samples == 0 {
        return None;
    }
    let frame = character_ship_frame(app.ship.snapshot().pose);
    let eye = app
        .character
        .snapshot(frame, surface_frame_for_ship(app.ship.snapshot()))
        .eye_position_meters;
    let snapshots = crate::fragment_physics::snapshots(&app.mining, frame, eye);
    assert_eq!(snapshots.len(), config.count);
    let ids: Vec<_> = snapshots.iter().map(|(p, _)| p.id()).collect();
    assert!(
        ids.iter()
            .all(|id| app.mining.fragment_sleep.is_sleeping(*id))
    );
    let initial: Vec<_> = snapshots.iter().map(|(_, p)| p.clone()).collect();
    let contains =
        |position, radius| salimon_character::ship_floor_placement(position, radius).is_some();
    let mut samples = Vec::with_capacity(config.baseline_samples);
    for _ in 0..config.baseline_samples {
        // Identical naturally settled authored snapshot on both paths. Cloning
        // replay inputs/cache is outside both timers and never mutates the game.
        let mut baseline = initial.clone();
        let mut sleeping = initial.clone();
        let mut tracker = app.mining.fragment_sleep.clone();
        let started = Instant::now();
        let before =
            salimon_physics::advance_profiled(&mut baseline, Duration::from_millis(16), contains);
        let before_ms = ms(started.elapsed());
        let started = Instant::now();
        let after = salimon_physics::advance_with_sleep(
            &mut tracker,
            &ids,
            &mut sleeping,
            Duration::from_millis(16),
            contains,
            true,
        );
        let after_ms = ms(started.elapsed());
        assert_eq!(sleeping, initial);
        assert_eq!(after.sleeping_objects, config.count);
        samples.push(json!({"stateless_ms": before_ms, "sleep_ms": after_ms,
            "stateless_integrated_objects": before.integrated_objects, "sleep_integrated_objects": after.integrated_objects,
            "stateless_pair_visits": before.pair_visits, "sleep_pair_visits": after.pair_visits,
            "stateless_matrix_allocations": before.matrix_allocations, "sleep_matrix_allocations": after.matrix_allocations,
            "sleeping_objects": after.sleeping_objects}));
    }
    eprintln!(
        "solver baseline: {} identical-snapshot samples, 16 ms simulation delta",
        samples.len()
    );
    Some(
        json!({"mode": "cpu_solver_replay", "simulation_delta_ms": 16,
        "cloning_timed": false, "samples": samples,
        "initial_snapshot": snapshots.iter().map(|(p, s)| json!({"id":p.id().0,
            "position":s.position,"orientation":s.orientation,"velocity":s.velocity,
            "angular_velocity":s.angular_velocity,"side_meters":s.side_meters,"mass_kg":s.mass_kg})).collect::<Vec<_>>()}),
    )
}

pub(crate) fn accounting(app: &ClientApplication) -> Value {
    // Stable session order; poses/velocity are deliberately absent.
    json!({"extracted_mass_kg": app.mining.session.extracted_mass_kg(),
        "fragments": app.mining.session.fragments().iter().map(|p|
            json!({"id": p.id().0, "source": p.source().local,
                "resource": p.material().resource().key(), "mass_kg": p.material().mass_kg()})).collect::<Vec<_>>()})
}
fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

pub(crate) fn run_headless(config: Config) -> Result<(), String> {
    let mut app = ClientApplication::default();
    initialize(&mut app, config)?;
    let mut recorder = Recorder::new(config, &app);
    let delta = Duration::from_millis(16);
    loop {
        let started = Instant::now();
        let ship = app.ship.snapshot();
        let frame = character_ship_frame(ship.pose);
        let player = app
            .character
            .snapshot(frame, surface_frame_for_ship(ship))
            .eye_position_meters;
        recorder.latest_physics =
            crate::fragment_physics::advance_measured(&mut app.mining, frame, delta, player, true);
        let update_time = started.elapsed();
        if recorder.record(delta, update_time, update_time, None)
            && recorder.finish(accounting(&app))?
        {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn surface_metadata_cannot_change_during_startup_or_warmup() {
        let config = parse(&["--benchmark".into()]).unwrap();
        let mut recorder = Recorder::new(config, &ClientApplication::default());
        assert!(!recorder.surface_changed(1920, 1080, 2.0));
        recorder.info = json!({"width":1920,"height":1080,"scale_factor":2.0});
        for started in [None, Some(Instant::now())] {
            recorder.started = started;
            assert!(!recorder.surface_changed(1920, 1080, 2.0));
            assert!(recorder.surface_changed(1280, 1080, 2.0));
            assert!(recorder.surface_changed(1920, 800, 2.0));
            assert!(recorder.surface_changed(1920, 1080, 1.0));
        }
    }
    #[test]
    fn startup_occlusion_restarts_warmup_but_retained_intervals_fail() {
        let config = parse(&["--benchmark".into()]).unwrap();
        let mut recorder = Recorder::new(config, &ClientApplication::default());
        recorder.started = Some(Instant::now());
        recorder.last_presented = recorder.started;
        recorder.observed = 1;
        assert!(recorder.restart_before_sampling());
        assert!(recorder.started.is_none() && recorder.last_presented.is_none());
        assert_eq!(recorder.observed, 0);
        let sample = Sample {
            scheduler: SchedulerSample::default(),
            frame_ms: Some(16.0),
            simulation_delta_ms: 16.0,
            update_ms: 1.0,
            simulation_ms: 1.0,
            physics: Default::default(),
            render: None,
        };
        recorder.samples.push(sample);
        assert!(!recorder.restart_before_sampling());
        assert_eq!(recorder.samples.len(), 1);
        // A preliminary redraw without a completed presentation interval is
        // discarded when native visibility arrives after that redraw.
        recorder.samples[0].frame_ms = None;
        assert!(recorder.restart_before_sampling());
        assert!(recorder.samples.is_empty());
    }
    #[test]
    fn minimum_samples_requires_elapsed_time_and_completed_native_intervals() {
        let config = parse(&[
            "--benchmark".into(),
            "--warmup-seconds".into(),
            "0".into(),
            "--sample-seconds".into(),
            "1".into(),
            "--minimum-samples".into(),
            "2".into(),
        ])
        .unwrap();
        let mut recorder = Recorder::new(config, &ClientApplication::default());
        recorder.started = Some(Instant::now() - Duration::from_secs(2));
        let render = Some(RenderStats {
            resource_batch: Default::default(),
            cpu_render_time: Duration::ZERO,
            gpu_frame_time: GpuFrameTime::Pending,
            visible_objects: 0,
            rendered_objects: 0,
            scene_draw_calls: 0,
            total_draw_calls: 0,
            gpu_memory: None,
        });
        let delta = Duration::from_millis(16);
        assert!(!recorder.record(delta, Duration::ZERO, Duration::ZERO, render));
        assert!(!recorder.record(delta, Duration::ZERO, Duration::ZERO, None));
        assert!(!recorder.record(delta, Duration::ZERO, Duration::ZERO, render));
        assert!(recorder.record(delta, Duration::ZERO, Duration::ZERO, render));
        // Even a sufficient count cannot end a window before its duration.
        recorder.started = Some(Instant::now());
        assert!(!recorder.record(delta, Duration::ZERO, Duration::ZERO, render));
        let mut cpu = Recorder::new(
            Config {
                headless: true,
                ..config
            },
            &ClientApplication::default(),
        );
        cpu.started = Some(Instant::now() - Duration::from_secs(2));
        assert!(!cpu.record(delta, Duration::ZERO, Duration::ZERO, None));
        assert!(cpu.record(delta, Duration::ZERO, Duration::ZERO, None));
    }
    #[test]
    fn repeat_windows_keep_accounting_and_fail_without_samples() {
        let config = parse(&[
            "--benchmark".into(),
            "--count".into(),
            "0".into(),
            "--repeats".into(),
            "3".into(),
            "--warmup-seconds".into(),
            "0".into(),
        ])
        .unwrap();
        let app = ClientApplication::default();
        let mut recorder = Recorder::new(config, &app);
        assert!(recorder.finish(accounting(&app)).is_err());
        for index in 0..3 {
            recorder.record(
                Duration::from_millis(16),
                Duration::ZERO,
                Duration::ZERO,
                None,
            );
            assert_eq!(recorder.finish(accounting(&app)).unwrap(), index == 2);
            assert_eq!(recorder.completed.len(), index + 1);
            assert_eq!(
                recorder.completed[index]["samples"]
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
        }
        assert!(recorder.finished);
    }
    #[test]
    fn zero_delta_redraw_does_not_report_default_physics_counters() {
        let config = parse(&[
            "--benchmark".into(),
            "--count".into(),
            "0".into(),
            "--warmup-seconds".into(),
            "0".into(),
        ])
        .unwrap();
        let mut recorder = Recorder::new(config, &ClientApplication::default());
        assert!(!recorder.record(Duration::ZERO, Duration::ZERO, Duration::ZERO, None));
        assert!(recorder.samples.is_empty());
        assert!(recorder.started.is_none());
    }

    #[test]
    fn both_cameras_get_separate_windows_and_one_transition() {
        let config = parse(&[
            "--benchmark".into(),
            "--count".into(),
            "0".into(),
            "--repeats".into(),
            "2".into(),
            "--camera".into(),
            "both".into(),
            "--warmup-seconds".into(),
            "0".into(),
        ])
        .unwrap();
        let app = ClientApplication::default();
        let mut recorder = Recorder::new(config, &app);
        for index in 0..4 {
            assert!(
                recorder
                    .window_title()
                    .contains(if index < 2 { "facing" } else { "away" })
            );
            assert!(
                recorder
                    .window_title()
                    .ends_with(&format!("{}/2", index % 2 + 1))
            );
            recorder.record(
                Duration::from_millis(16),
                Duration::ZERO,
                Duration::ZERO,
                None,
            );
            assert_eq!(recorder.finish(accounting(&app)).unwrap(), index == 3);
            assert_eq!(recorder.switch_camera(), index == 1);
            assert_eq!(
                recorder.completed[index]["config"]["camera"],
                if index < 2 { "facing" } else { "away" }
            );
        }
    }
    #[test]
    fn authored_dense_preparation_requires_natural_sleep_and_preserves_accounting() {
        let mut config = parse(&[
            "--benchmark".into(),
            "--count".into(),
            "10".into(),
            "--layout".into(),
            "dense".into(),
        ])
        .unwrap();
        let mut app = ClientApplication::default();
        config.settle_steps = 1;
        assert!(
            initialize(&mut app, config)
                .unwrap_err()
                .contains("did not sleep")
        );
        let expected = accounting(&app);
        let mut app = ClientApplication::default();
        config.settle_steps = 1200;
        initialize(&mut app, config).unwrap();
        assert_eq!(accounting(&app), expected);
        config.baseline_samples = 2;
        let baseline = solver_baseline(config, &app).unwrap();
        assert_eq!(baseline["samples"].as_array().unwrap().len(), 2);
        for sample in baseline["samples"].as_array().unwrap() {
            assert_eq!(sample["sleeping_objects"], 10);
            assert_eq!(sample["sleep_integrated_objects"], 0);
            assert_eq!(sample["sleep_pair_visits"], 0);
            assert!(sample["stateless_integrated_objects"].as_u64().unwrap() > 0);
            assert!(sample["stateless_pair_visits"].as_u64().unwrap() > 0);
        }
        let frame = character_ship_frame(app.ship.snapshot().pose);
        let eye = app
            .character
            .snapshot(frame, surface_frame_for_ship(app.ship.snapshot()))
            .eye_position_meters;
        let before = app.mining.session.fragments().to_vec();
        for _ in 0..100 {
            let stats = crate::fragment_physics::advance_measured(
                &mut app.mining,
                frame,
                Duration::from_millis(16),
                eye,
                true,
            );
            assert_eq!(stats.solver.sleeping_objects, 10);
            assert_eq!(stats.solver.integrated_objects, 0);
            assert_eq!(stats.solver.pair_visits, 0);
            assert_eq!(app.mining.session.fragments(), before);
        }
    }

    #[test]
    fn full_game_high_speed_flight_keeps_sleep_and_ship_local_poses() {
        use winit::keyboard::{KeyCode, PhysicalKey};
        let mut app = ClientApplication::default();
        crate::e2e::initialize(
            &mut app,
            crate::e2e::Config {
                scenario: crate::e2e::Scenario::OrbitEarth,
                seed: 155,
                step: Duration::from_millis(16),
                fragment_load: Some((10, Layout::Dense, 1200)),
            },
        )
        .unwrap();
        let initial = accounting(&app);
        let anchors = app.mining.ship_fragments.clone();
        app.ship = salimon_ship::ShipController::flying_with_velocity(
            app.ship.snapshot().pose,
            0,
            [25_000.0, 0.0, 0.0],
        )
        .unwrap();
        app.automation_key(PhysicalKey::Code(KeyCode::KeyA), true);
        for _ in 0..100 {
            app.advance_game(Duration::from_millis(16));
            assert_eq!(app.mining.ship_fragments, anchors);
            assert!(
                app.mining
                    .session
                    .fragments()
                    .iter()
                    .all(|piece| app.mining.fragment_sleep.is_sleeping(piece.id()))
            );
            assert_eq!(accounting(&app), initial);
        }
        assert!((app.ship.snapshot().speed_meters_per_second - 25_000.0).abs() < 1e-6);
    }
    #[test]
    fn rejects_invalid_and_duplicate_options() {
        for args in [
            vec!["--benchmark", "--count", "1001"],
            vec!["--benchmark", "--sample-seconds", "0"],
            vec!["--benchmark", "--state", "sleeping"],
            vec!["--benchmark", "--count", "5", "--count", "5"],
            vec!["--benchmark", "--settle-steps", "37501"],
            vec!["--benchmark", "--settle-steps", "10", "--state", "active"],
            vec!["--benchmark", "--repeats", "0"],
            vec!["--benchmark", "--repeats", "11"],
            vec!["--benchmark", "--repeats", "2", "--state", "active"],
            vec!["--benchmark", "--camera", "both", "--state", "active"],
            vec!["--benchmark", "--camera", "both", "--mode", "physics"],
            vec!["--benchmark", "--defer-start", "yes"],
        ] {
            assert!(parse(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err());
        }
    }
    #[test]
    fn native_large_load_scenario_uses_production_input_and_takeoff() {
        let mut app = ClientApplication::default();
        crate::e2e::initialize(
            &mut app,
            crate::e2e::Config {
                scenario: crate::e2e::Scenario::LandedEarth,
                seed: 155,
                step: Duration::from_millis(16),
                fragment_load: Some((500, Layout::Scattered, 0)),
            },
        )
        .unwrap();
        let scenario: Value =
            serde_json::from_str(include_str!("../../../scenarios/fragment-load.json")).unwrap();
        for step in scenario["steps"].as_array().unwrap() {
            let mut command = step
                .get("action")
                .cloned()
                .unwrap_or(json!({"op": "inspect"}));
            command["protocol"] = json!(1);
            command["id"] = json!(1);
            let response = crate::automation::execute(&mut app, &command.to_string());
            assert_eq!(response["ok"], true, "{response}");
            if let Some(assertion) = step.get("assert") {
                let path = format!("/{}", assertion["path"].as_str().unwrap().replace('.', "/"));
                assert_eq!(
                    response["result"].pointer(&path).unwrap(),
                    &assertion["equals"],
                    "{path}"
                );
            }
        }
    }

    #[test]
    fn primary_and_stress_loads_use_production_updates_without_loss() {
        for count in [500, 1000] {
            let config = parse(&[
                "--benchmark".into(),
                "--count".into(),
                count.to_string(),
                "--layout".into(),
                "scattered".into(),
            ])
            .unwrap();
            let mut app = ClientApplication::default();
            initialize(&mut app, config).unwrap();
            let original = accounting(&app);
            app.advance_game(Duration::from_millis(16));
            assert_eq!(accounting(&app), original);
            assert!(app.mining.session.fragments().iter().all(|p| {
                p.transform()
                    .position()
                    .meters()
                    .iter()
                    .all(|v| v.is_finite())
            }));
            // All anchors remain in the same reference frame on rotated ship writeback.
            app.ship = salimon_ship::ShipController::flying(
                salimon_ship::ShipPose {
                    position_meters: [100_000_000.0, 20_000_000.0, -40_000_000.0],
                    orientation: [
                        0.0,
                        std::f64::consts::FRAC_1_SQRT_2,
                        0.0,
                        std::f64::consts::FRAC_1_SQRT_2,
                    ],
                },
                0,
            );
            app.sync_carried();
            let frame = character_ship_frame(app.ship.snapshot().pose);
            for piece in app.mining.session.fragments() {
                assert_eq!(
                    piece.transform().position().meters(),
                    frame.local_to_world(app.mining.ship_fragments[&piece.id()])
                );
            }
            assert_eq!(accounting(&app), original);
        }
    }

    #[test]
    fn loaded_fixture_accepts_real_f_pickup_drop_and_carry_lockout() {
        let mut app = ClientApplication::default();
        crate::e2e::initialize(
            &mut app,
            crate::e2e::Config {
                fragment_load: None,
                scenario: crate::e2e::Scenario::LandedEarth,
                seed: 155,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        benchmark_fixture::initialize(&mut app, 500, Layout::Scattered, 155, false).unwrap();
        let original = accounting(&app);
        let frame = character_ship_frame(app.ship.snapshot().pose);
        let eye = app
            .character
            .snapshot(frame, surface_frame_for_ship(app.ship.snapshot()))
            .eye_position_meters;
        let ids: Vec<_> = app
            .mining
            .session
            .fragments()
            .iter()
            .filter(|p| {
                salimon_math::length(salimon_math::sub(p.transform().position().meters(), eye))
                    < 3.0
            })
            .map(|p| p.id().0)
            .collect();
        let command = |app: &mut ClientApplication, op: Value| {
            let mut op = op;
            op["protocol"] = json!(1);
            op["id"] = json!(1);
            let response = crate::automation::execute(app, &op.to_string());
            assert_eq!(response["ok"], true, "{response}");
            response
        };
        for id in ids {
            command(&mut app, json!({"op": "aim_fragment", "fragment_id": id}));
            command(
                &mut app,
                json!({"op": "key", "key": "grab_drop", "pressed": true}),
            );
            command(
                &mut app,
                json!({"op": "key", "key": "grab_drop", "pressed": false}),
            );
            if app.mining.session.carried_id().is_some() {
                break;
            }
        }
        assert!(
            app.mining.session.carried_id().is_some(),
            "fixture has a reachable fragment"
        );
        command(
            &mut app,
            json!({"op": "key", "key": "slot_1", "pressed": true}),
        );
        assert!(!app.equipment.mining_equipped());
        app.advance_game(Duration::from_millis(16));
        command(
            &mut app,
            json!({"op": "key", "key": "grab_drop", "pressed": true}),
        );
        command(
            &mut app,
            json!({"op": "key", "key": "grab_drop", "pressed": false}),
        );
        assert!(
            app.mining.session.carried_id().is_none(),
            "production placement permits release"
        );
        app.advance_game(Duration::from_millis(16));
        assert_eq!(accounting(&app), original);
    }

    #[test]
    fn canonical_adapter_retains_uninterrupted_solver_states_between_updates() {
        let mut app = ClientApplication::default();
        benchmark_fixture::initialize(&mut app, 100, Layout::Dense, 155, false).unwrap();
        let frame = character_ship_frame(app.ship.snapshot().pose);
        let eye = app
            .character
            .snapshot(frame, surface_frame_for_ship(app.ship.snapshot()))
            .eye_position_meters;
        let initial = crate::fragment_physics::snapshots(&app.mining, frame, eye);
        let ids: Vec<_> = initial.iter().map(|(fragment, _)| fragment.id()).collect();
        let mut direct: Vec<_> = initial.into_iter().map(|(_, state)| state).collect();
        let mut tracker = salimon_physics::SleepTracker::default();
        for step in 0..10 {
            salimon_physics::advance_with_sleep(
                &mut tracker,
                &ids,
                &mut direct,
                Duration::from_millis(16),
                |position, radius| {
                    salimon_character::ship_floor_placement(position, radius).is_some()
                },
                false,
            );
            crate::fragment_physics::advance(
                &mut app.mining,
                frame,
                Duration::from_millis(16),
                eye,
            );
            let rebuilt = crate::fragment_physics::snapshots(&app.mining, frame, eye);
            for (expected, (fragment, actual)) in direct.iter().zip(rebuilt) {
                let mut expected = expected.clone();
                expected.ground_support_meters = 0.0;
                assert_eq!(actual, expected, "step={step} id={:?}", fragment.id());
                assert_eq!(
                    app.mining.fragment_sleep.is_sleeping(fragment.id()),
                    tracker.is_sleeping(fragment.id())
                );
            }
        }
    }

    #[test]
    fn instrumentation_does_not_change_production_poses_or_accounting() {
        let config = parse(&["--benchmark".into(), "--count".into(), "12".into()]).unwrap();
        let mut a = ClientApplication::default();
        let mut b = ClientApplication::default();
        initialize(&mut a, config).unwrap();
        initialize(&mut b, config).unwrap();
        let original = accounting(&a);
        let frame = character_ship_frame(a.ship.snapshot().pose);
        let eye = a
            .character
            .snapshot(frame, surface_frame_for_ship(a.ship.snapshot()))
            .eye_position_meters;
        for _ in 0..3 {
            crate::fragment_physics::advance(&mut a.mining, frame, Duration::from_millis(16), eye);
            let stats = crate::fragment_physics::advance_measured(
                &mut b.mining,
                frame,
                Duration::from_millis(16),
                eye,
                true,
            );
            assert_eq!(stats.solver.objects, 12);
            assert!(stats.solver.pair_visits < stats.solver.solver_passes * 66);
            assert!(stats.solver.radius_candidates <= stats.solver.pair_visits);
            assert!(stats.solver.narrow_phase_tests <= stats.solver.radius_candidates);
            if stats.solver.projection_builds > 0 {
                assert!(stats.solver.projection_storage_bytes > 0);
            }
            assert_eq!(a.mining.session.fragments(), b.mining.session.fragments());
        }
        assert_eq!(a.mining.session.fragments(), b.mining.session.fragments());
        assert_eq!(original, accounting(&b));
    }
}
