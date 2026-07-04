use niri_ipc::{Action, Request, socket};

use std::io::{self, BufRead};
use std::time::{Duration, Instant};
use std::{env, fs::OpenOptions, path::PathBuf};

use zibi_core::direction::Direction;
use zibi_core::direction::detect_direction;
use zibi_core::point::Point;
use zibi_core::point::PointRecord;

use tracing::error;
use tracing::info;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::SystemTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use chrono::Local;

const MOVE_THRESHOLD: i16 = 150;
const WINDOW_DURATION: Duration = Duration::from_millis(300);
const COOLDOWN_DURATION: Duration = Duration::from_millis(500);

fn open_log_file() -> std::fs::File {
    let home = env::var("HOME").expect("HOME not set");

    let suffix = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let path = PathBuf::from(home)
        .join(".local/share/zibi")
        .join(format!("zibi.{suffix}.log"));

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create log directory");
    }

    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .expect("failed to open log file")
}

fn main() {
    let log_file = open_log_file();

    let file_layer = fmt::layer()
        .with_timer(SystemTime::default())
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(log_file);

    let console_layer = fmt::layer()
        .with_timer(SystemTime::default())
        .with_file(true)
        .with_line_number(true)
        .with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();

    let mut niri_socket = match socket::Socket::connect() {
        Ok(socket) => {
            info!("Connected to niri socket");
            socket
        }
        Err(e) => {
            error!(error = ?e, "Cannot connect to niri socket");
            panic!("Cannot connect to niri socket");
        }
    };

    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut buffer = String::new();

    let mut points: Vec<PointRecord> = Vec::new();
    let mut cooldown_until = Instant::now();

    loop {
        buffer.clear();

        match handle.read_line(&mut buffer) {
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

        let Some(p) = Point::parse(&buffer) else {
            error!("Could not parse point from: {buffer:?}");
            continue;
        };

        let now = Instant::now();

        if now < cooldown_until {
            points.clear();
            continue;
        }

        points.push(PointRecord {
            point: p,
            time: now,
        });

        points.retain(|rec| now.duration_since(rec.time) <= WINDOW_DURATION);

        if points.len() < 2 {
            continue;
        }

        let first = &points.first().unwrap().point;
        let last = &points.last().unwrap().point;

        if let Some(dir) = detect_direction(first, last, MOVE_THRESHOLD) {
            info!("Direction: {:?}", dir);
            let action = match dir {
                Direction::Up => Request::Action(Action::FocusWorkspaceUp {}),
                Direction::Down => Request::Action(Action::FocusWorkspaceDown {}),
                Direction::Left => Request::Action(Action::FocusColumnLeft {}),
                Direction::Right => Request::Action(Action::FocusColumnRight {}),
            };

            if let Err(err) = niri_socket.send(action) {
                error!("error when sending request to niri socket, {err}");
            }

            cooldown_until = now + COOLDOWN_DURATION;
            points.clear();
        }
    }
}
