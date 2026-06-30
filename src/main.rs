use std::io::{self, BufRead};
use std::time::{Duration, Instant};

use niri_ipc::{Action, Request, socket};

#[derive(Debug, Default, Clone)]
struct Point {
    x: i16,
    y: i16
}

impl Point {
    fn parse(input: &str) -> Option<Self> {
        let (x, y) = input.trim().split_once(',')?;

        Some(Self {
            x: x.trim().parse().ok()?,
            y: y.trim().parse().ok()?,
        })
    }

}

#[derive(Debug, PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone)]
struct PointRecord {
    point: Point,
    time: Instant,
}

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

        let delta_x = first.x - last.x;
        let delta_y = first.y - last.y;

        let abs_dx = delta_x.abs();
        let abs_dy = delta_y.abs();

        if abs_dx > MOVE_THRESHOLD || abs_dy > MOVE_THRESHOLD {
            let dir = if abs_dx > abs_dy {
                if delta_x < 0 { Direction::Right } else { Direction::Left }
            } else {
                if delta_y < 0 { Direction::Down } else { Direction::Up }
            };

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
