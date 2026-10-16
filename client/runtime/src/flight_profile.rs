//! CPU-only, deterministic workloads over production runtime/domain adapters.
use crate::{
    app::{ClientApplication, character_ship_frame},
    e2e::{self, Scenario},
    flight_regression,
};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let mut samples = 20;
    let mut stacked = false;
    let mut options = args.iter().skip(1);
    while let Some(option) = options.next() {
        match option.as_str() {
            "--samples" => {
                samples = options
                    .next()
                    .ok_or("--samples needs a value")?
                    .parse::<usize>()
                    .map_err(|_| "samples must be 20..1000")?
            }
            "--stacked" => stacked = true,
            _ => return Err("expected --flight-profile [--samples 20..1000] [--stacked]".into()),
        }
    }
    if !(20..=1000).contains(&samples) {
        return Err("samples must be 20..1000".into());
    }
    if cfg!(debug_assertions) {
        return Err("flight profiles require a release build".into());
    }
    let mut rows = Vec::new();
    for count in [0, 500] {
        for workload in [
            "mass-idle",
            "mass-crossing",
            "moving-powered",
            "moving-zero-g",
            "power-transition",
            "dense-contacts",
        ] {
            eprintln!("Profiling {count} fragments: {workload}");
            let row = profile(count, workload, samples, stacked)?;
            eprintln!("SALIMON_FLIGHT_PROFILE_ROW {row}");
            rows.push(row);
        }
    }
    println!(
        "{}",
        json!({"schema":1,"kind":"flight-cpu-profile","fixture":if stacked {"stacked"} else {"deck"},"step_seconds":0.016,"samples":samples,"rows":rows})
    );
    Ok(())
}

fn app(count: usize) -> Result<ClientApplication, String> {
    let mut app = ClientApplication::default();
    e2e::initialize(
        &mut app,
        e2e::Config {
            scenario: Scenario::FlightRegression,
            seed: 196,
            step: Duration::from_millis(16),
            fragment_load: None,
        },
    )?;
    flight_regression::initialize_cargo(&mut app, count)?;
    Ok(app)
}

fn distribution(raw: Vec<f64>) -> Value {
    let mut values = raw.clone();
    values.sort_by(f64::total_cmp);
    let percentile = |p: f64| {
        values[((values.len() as f64 * p).ceil() as usize)
            .saturating_sub(1)
            .min(values.len() - 1)]
    };
    json!({"p50_ms":percentile(0.50),"p95_ms":percentile(0.95),"p99_ms":percentile(0.99),"max_ms":values.last(),"samples":values.len(),"raw_ms":raw})
}

fn profile(count: usize, workload: &str, samples: usize, stacked: bool) -> Result<Value, String> {
    let mut app = app(count)?;
    if workload == "moving-zero-g" {
        app.ship.set_stored_core_energy_joules(0);
    }
    if workload == "dense-contacts" {
        // Dense single-layer deck contacts: 500 real authored hulls without
        // imported penetration. Multilayer pile stress uses the existing dense
        // benchmark and is deliberately not called a settled/60 FPS result here.
        for (i, piece) in app.mining.session.fragments().iter().enumerate() {
            let support = crate::fragment_physics::sphere_radius(*piece);
            let local = if stacked {
                [
                    -5.5 + (i % 10) as f64 * 0.12,
                    0.4 + (i / 100) as f64 * 0.12,
                    -1.0 + ((i / 10) % 10) as f64 * 0.12,
                ]
            } else {
                [
                    -6.4 + (i % 20) as f64 * 0.12,
                    salimon_character::SHIP_FLOOR_HEIGHT_METERS + support,
                    -2.6 + (i / 20) as f64 * 0.12,
                ]
            };
            app.mining.ship_fragments.insert(piece.id(), local);
        }
        app.sync_carried();
    }
    let identities: Vec<_> = app
        .mining
        .session
        .fragments()
        .iter()
        .map(|p| p.id())
        .collect();
    let expected_mass = app.ship.snapshot().total_mass_kg;
    let mut times = Vec::with_capacity(samples);
    let mut moving_counts = Vec::with_capacity(samples);
    // Fixed sample count, fixed steps, fresh initial population for each row.
    // No warmup silently turns an active-contact row into a settled claim.
    for index in 0..samples {
        let start = Instant::now();
        match workload {
            "mass-idle" => {
                let frame = character_ship_frame(app.ship.snapshot().pose);
                crate::carrying::sync_ship_mass(&mut app.mining, frame, &mut app.ship);
                std::hint::black_box(app.ship.snapshot().total_mass_kg);
            }
            "mass-crossing" => {
                if let Some(piece) = app
                    .mining
                    .session
                    .fragments()
                    .get(index % count.max(1))
                    .copied()
                {
                    let frame = character_ship_frame(app.ship.snapshot().pose);
                    let original = piece.transform();
                    let outside = frame.local_to_world([-8.0, 1.0, 0.0]);
                    let pose = salimon_world::resources::ResourceTransform::new(
                        salimon_world::WorldPosition::new(outside[0], outside[1], outside[2]),
                        original.orientation_xyzw(),
                    )
                    .map_err(|e| format!("boundary pose: {e:?}"))?;
                    app.mining.session.move_loose(piece.id(), pose);
                    crate::carrying::sync_ship_mass(&mut app.mining, frame, &mut app.ship);
                    if (app.ship.snapshot().total_mass_kg
                        - (expected_mass - piece.material().mass_kg()))
                    .abs()
                        > 1e-6
                    {
                        return Err("boundary exit mass mismatch".into());
                    }
                    app.mining.session.move_loose(piece.id(), original);
                    crate::carrying::sync_ship_mass(&mut app.mining, frame, &mut app.ship);
                }
            }
            "moving-powered" | "moving-zero-g" | "dense-contacts" | "power-transition" => {
                if workload == "power-transition" {
                    app.ship.set_stored_core_energy_joules(if index % 2 == 0 {
                        0
                    } else {
                        45_000_000_000
                    });
                }
                app.advance_game(Duration::from_millis(16));
            }
            _ => return Err("unknown workload".into()),
        }
        times.push(start.elapsed().as_secs_f64() * 1000.0);
        // Validation/counters outside the timed window; observation never drives simulation.
        if app
            .mining
            .session
            .fragments()
            .iter()
            .map(|p| p.id())
            .ne(identities.iter().copied())
        {
            return Err("population identity changed".into());
        }
        if (app.ship.snapshot().total_mass_kg - expected_mass).abs() > 1e-6 {
            return Err(format!("{workload}: onboard mass changed"));
        }
        moving_counts.push(
            app.mining
                .fragment_motion
                .values()
                .filter(|m| {
                    salimon_math::length(m.velocity) > 1e-5
                        || salimon_math::length(m.angular_velocity) > 1e-5
                })
                .count(),
        );
    }
    Ok(
        json!({"count":count,"workload":workload,"cargo_mass_kg":expected_mass-100_000.0,"moving_objects":moving_counts,"cpu_update":distribution(times),"hardware_fps_validated":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn percentiles_retain_spikes_and_sample_count() {
        let d = distribution((1..=100).map(f64::from).collect());
        assert_eq!(d["p50_ms"], 50.0);
        assert_eq!(d["p95_ms"], 95.0);
        assert_eq!(d["p99_ms"], 99.0);
        assert_eq!(d["max_ms"], 100.0);
    }
    #[test]
    fn workload_population_and_event_mass_are_checked() {
        assert!(profile(5, "dense-contacts", 2, true).is_ok());
        for name in [
            "mass-idle",
            "mass-crossing",
            "moving-powered",
            "moving-zero-g",
            "power-transition",
            "dense-contacts",
        ] {
            assert!(profile(5, name, 2, false).is_ok(), "{name}");
        }
    }
}
