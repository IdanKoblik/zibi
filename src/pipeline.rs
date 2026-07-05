use std::io::BufRead;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use niri_ipc::socket;
use tracing::error;
use tracing::warn;

use zibi_core::direction::detect_direction;
use zibi_core::landmark::Landmark;
use zibi_core::point::PointRecord;

use crate::config::Config;
use crate::niri;

const WINDOW_DURATION: Duration = Duration::from_millis(300);
const COOLDOWN_DURATION: Duration = Duration::from_millis(500);

pub fn run<R: BufRead>(
    mut reader: R,
    socket: &mut socket::Socket,
    cfg: &Config,
    points_tx: &Sender<Landmark>,
) {
    let mut buffer = String::new();

    let mut points: Vec<PointRecord> = Vec::new();
    let mut cooldown_until = Instant::now();

    loop {
        buffer.clear();

        match reader.read_line(&mut buffer) {
            Ok(0) => break, // EOF: upstream closed, stop looping
            Ok(_) => buffer = buffer.trim().to_string(),
            Err(e) => {
                error!("Error: {e}");
                continue;
            }
        }

        if buffer.is_empty() {
            continue;
        }

        let Some(lm) = Landmark::parse(&buffer) else {
            warn!("Could not parse point from: {buffer:?}");
            continue;
        };

        if lm.hand != cfg.core.dominant_hand {
            continue;
        }

        let _ = points_tx.send(lm.clone());
        // 8 = Tip finger index
        let Some(p) = lm.points.get(8) else {
            warn!("Cannot get tip finger index");
            continue;
        };

        let now = Instant::now();
        if now < cooldown_until {
            points.clear();
            continue;
        }

        points.push(PointRecord {
            point: p.clone(),
            time: now,
        });

        points.retain(|rec| now.duration_since(rec.time) <= WINDOW_DURATION);

        if points.len() < 2 {
            continue;
        }

        let first = &points.first().unwrap().point;
        let last = &points.last().unwrap().point;

        if let Some(dir) = detect_direction(first, last, cfg.core.move_threshold) {
            niri::send_direction(socket, dir);
            cooldown_until = now + COOLDOWN_DURATION;
            points.clear();
        }
    }
}
