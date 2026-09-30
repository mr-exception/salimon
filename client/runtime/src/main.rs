//! Native client entry point for the Salimon Phase 0 showcase.

mod action_bar;
mod app;
mod automation;
mod e2e;
mod frame_clock;
mod mining;
mod resource_presentation;
mod update_clock;

use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("starting Salimon native client");

    let config = match e2e::parse_args(std::env::args().skip(1)) {
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
