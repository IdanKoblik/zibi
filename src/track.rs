use std::env;
use std::io;
use std::process::{Child, Command, Stdio};

use tracing::{error, info};

use crate::config::Config;

pub fn spawn(cfg: &Config) -> Child {
    match spawn_track(cfg) {
        Ok(child) => {
            info!("Spawned track process");
            child
        }
        Err(e) => {
            error!(error = ?e, "Cannot spawn track process");
            panic!("Cannot spawn track process");
        }
    }
}

fn spawn_track(cfg: &Config) -> io::Result<Child> {
    let mut cmd = match env::var("ZIBI_TRACK_CMD") {
        Ok(spec) => {
            let mut parts = spec.split_whitespace();
            let program = parts.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "ZIBI_TRACK_CMD is empty")
            })?;
            let mut cmd = Command::new(program);

            cmd.args(parts);
            cmd
        }
        Err(_) => {
            let track_path = env::current_exe()?
                .parent()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "exe has no parent directory")
                })?
                .join("track");
            Command::new(track_path)
        }
    };

    cmd.args(["--camera", &cfg.core.camera.to_string_lossy()]);
    cmd.stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn()
}
