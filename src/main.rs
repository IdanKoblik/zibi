use tracing::{error, info};

pub mod config;
mod logging;
mod niri;
mod pipeline;
mod track;

fn main() {
    logging::init();

    let mut niri_socket = niri::connect();

    info!("Loading zibi config");
    let cfg = config::Config::load();

    let mut child = track::spawn(&cfg);
    let stdout = child.stdout.take().unwrap_or_else(|| {
        error!("track child has no stdout handle");
        panic!("track child has no stdout handle");
    });

    pipeline::run(std::io::BufReader::new(stdout), &mut niri_socket, &cfg);

    info!("track stream ended, shutting down tracker");
    let _ = child.kill();
    let _ = child.wait();
}
