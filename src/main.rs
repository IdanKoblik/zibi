use niri_ipc::{Action, Request, socket};
use std::io::{self, BufRead};
use std::time::{Duration, Instant};
use zibi_core::direction::Direction;
use zibi_core::direction::detect_direction;
use zibi_core::point::Point;
use zibi_core::point::PointRecord;

const MOVE_THRESHOLD: i16 = 150;
const WINDOW_DURATION: Duration = Duration::from_millis(300);
const COOLDOWN_DURATION: Duration = Duration::from_millis(500);

fn main() {
    let mut niri_socket = socket::Socket::connect().expect("cannot connect to niri socket");

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
                eprintln!("Error: {e}");
                continue;
            }
        }

        if buffer.is_empty() {
            continue;
        }

        let Some(p) = Point::parse(&buffer) else {
            eprintln!("Could not parse point from: {buffer:?}");
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
            println!("Direction: {:?}", dir);
            let action = match dir {
                Direction::Up => Request::Action(Action::FocusWorkspaceUp {}),
                Direction::Down => Request::Action(Action::FocusWorkspaceDown {}),
                Direction::Left => Request::Action(Action::FocusColumnLeft {}),
                Direction::Right => Request::Action(Action::FocusColumnRight {}),
            };

            if let Err(err) = dbg!(niri_socket.send(action)) {
                eprintln!("error when sending request to niri socket, {err}");
            }

            cooldown_until = now + COOLDOWN_DURATION;
            points.clear();
        }
    }
}
