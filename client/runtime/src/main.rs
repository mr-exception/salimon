//! Native client entry point for the Salimon Phase 0 showcase.

mod action_bar;
mod app;
mod automation;
mod benchmark;
mod benchmark_fixture;
mod carrying;
mod checkpoints;
mod e2e;
mod equipment;
mod flight_profile;
mod flight_regression;
#[cfg(test)]
mod fragment_costs;
mod fragment_physics;
mod frame_clock;
mod mining;
#[cfg(test)]
mod mining_emission;
mod resource_context;
mod resource_presentation;
mod reticle;
mod session;
#[cfg(test)]
mod sphere_benchmark;
#[cfg(test)]
mod sphere_fixtures;
#[cfg(test)]
mod sphere_interactions;
#[cfg(test)]
mod sphere_workloads;
mod terrain;
mod terrain_observations;
mod terrain_patches;
mod update_clock;

use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("starting Salimon native client");

    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--flight-profile") {
        return match flight_profile::run(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Salimon flight profile failed: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if args.first().map(String::as_str) == Some("--benchmark") {
        let result = benchmark::parse(&args).and_then(|config| {
            if config.headless {
                benchmark::run_headless(config)
            } else {
                app::run_benchmark(config).map_err(|error| error.to_string())
            }
        });
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Salimon benchmark failed: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let config = match e2e::parse_args(args) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Salimon launch error: {error}");
            return ExitCode::FAILURE;
        }
    };
    match app::run(config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("Salimon client stopped: {error}");
            ExitCode::FAILURE
        }
    }
}
