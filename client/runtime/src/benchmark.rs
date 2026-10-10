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
    pub(crate) headless: bool,
    pub(crate) warmup_seconds: u64,
    pub(crate) sample_seconds: u64,
}

pub(crate) fn parse(args: &[String]) -> Result<Config, String> {
    let mut config = Config {
        count: 500,
        layout: Layout::Dense,
        seed: 155,
        active: false,
        away: false,
        headless: false,
        warmup_seconds: 10,
        sample_seconds: 30,
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
                config.away = match value.as_str() {
                    "facing" => false,
                    "away" => true,
                    _ => return Err("camera must be facing or away".into()),
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
            _ => return Err(format!("unknown benchmark option {key}")),
        }
    }
    Ok(config)
}

pub(crate) fn initialize(app: &mut ClientApplication, config: Config) -> Result<(), String> {
    benchmark_fixture::initialize(app, config.count, config.layout, config.seed, config.active)?;
    // Camera remains a production character camera, facing the aft population.
    if !config.away {
        app.character
            .apply_mouse_delta(std::f64::consts::PI / 0.0022, 0.0);
    }
    app.character.apply_mouse_delta(0.0, 220.0);
    Ok(())
}

#[derive(Clone, Copy)]
struct Sample {
    frame_ms: Option<f64>,
    simulation_delta_ms: f64,
    update_ms: f64,
    simulation_ms: f64,
    physics: Measurements,
    render: Option<RenderStats>,
}

pub(crate) struct Recorder {
    pub(crate) config: Config,
    started: Option<Instant>,
    last_presented: Option<Instant>,
    samples: Vec<Sample>,
    pub(crate) latest_physics: Measurements,
    info: Value,
    initial: Value,
    finished: bool,
    observed: usize,
    stride: usize,
}

impl Recorder {
    pub(crate) fn new(config: Config, app: &ClientApplication) -> Self {
        Self {
            config,
            started: None,
            last_presented: None,
            samples: Vec::with_capacity(4096),
            latest_physics: Measurements::default(),
            info: Value::Null,
            initial: accounting(app),
            finished: false,
            observed: 0,
            stride: 1,
        }
    }
    pub(crate) fn set_renderer(
        &mut self,
        info: &salimon_renderer::RendererInfo,
        width: u32,
        height: u32,
    ) {
        self.info = json!({"adapter": info.adapter_name, "backend": info.backend,
            "device_type": info.device_type, "gpu_timestamps_supported": info.timestamp_queries_supported,
            "width": width, "height": height, "present_mode": "AutoVsync"});
    }
    pub(crate) fn record(
        &mut self,
        delta: Duration,
        update: Duration,
        simulation: Duration,
        render: Option<RenderStats>,
    ) -> bool {
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
                    render,
                });
            }
        }
        now.duration_since(start)
            >= Duration::from_secs(self.config.warmup_seconds + self.config.sample_seconds)
    }
    pub(crate) fn finish(&mut self, app_accounting: Value) -> Result<(), String> {
        if self.finished {
            return Err("benchmark already finished".into());
        }
        self.finished = true;
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
                "cpu_update_ms": s.update_ms, "cpu_simulation_ms": s.simulation_ms, "cpu_scene_prepare_ms": (s.update_ms - s.simulation_ms).max(0.0), "physics_total_ms": ms(s.physics.total_time),
                "physics_adapter_ms": ms(s.physics.adapter_time), "physics_integration_ms": ms(p.integration_time),
                "physics_contact_ms": ms(p.contact_time), "physics_activation_ms": ms(p.activation_time), "objects_simulated": p.objects,
                "moving_objects": s.physics.moving_objects, "awake_objects": p.awake_objects, "sleeping_objects": p.sleeping_objects, "integrated_objects": p.integrated_objects,
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
                "camera": if self.config.away { "away" } else { "facing" },
                "warmup_seconds": self.config.warmup_seconds, "sample_seconds": self.config.sample_seconds},
            "renderer": self.info, "accounting": self.initial, "accounting_preserved": true,
            "updates_observed": self.observed, "sampling_stride": self.stride,
            "elapsed_seconds": self.started.map(|s| s.elapsed().as_secs_f64()),
            "samples": samples});
        println!("{output}");
        Ok(())
    }
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
        if recorder.record(delta, update_time, update_time, None) {
            return recorder.finish(accounting(&app));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_and_duplicate_options() {
        for args in [
            vec!["--benchmark", "--count", "1001"],
            vec!["--benchmark", "--sample-seconds", "0"],
            vec!["--benchmark", "--state", "sleeping"],
            vec!["--benchmark", "--count", "5", "--count", "5"],
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
                fragment_load: Some((500, Layout::Scattered)),
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
            assert_eq!(stats.solver.pair_visits, stats.solver.substeps * 16 * 66);
            assert_eq!(stats.solver.matrix_allocations, stats.solver.substeps * 13);
        }
        assert_eq!(a.mining.session.fragments(), b.mining.session.fragments());
        assert_eq!(original, accounting(&b));
    }
}
