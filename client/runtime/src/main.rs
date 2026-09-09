//! Native client entry point for the Salimon Phase 0 showcase.

mod app;
mod frame_clock;
mod update_clock;

use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("starting Salimon native client");

    match app::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("Salimon client stopped: {error}");
            ExitCode::FAILURE
        }
    }
}
