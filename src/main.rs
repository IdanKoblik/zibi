use std::io::{self, BufRead};
use std::time::{Duration, Instant};

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
    Unknown
}

const MOVE_THRESHOLD: i16 = 150;

fn main() {
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut buffer = String::new();

    let mut points = Vec::new();
    let mut last_tick = Instant::now();

    let mut dir = Direction::Unknown;
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

        points.push(p);

        if last_tick.elapsed() >= Duration::from_millis(500) {
            println!("Collected {} points", points.len());
            for p in &points {
                println!("{:?}", p);
            }

            let first = points.first().cloned().unwrap_or_default();
            let second = points.last().cloned().unwrap_or_default();

            let delta_x = first.x - second.x;
            let delta_y = first.y - second.y;
            if delta_x == delta_y {
                continue;
            }

            if delta_x.abs() > delta_y.abs() {
                match delta_x {
                    x if x < -MOVE_THRESHOLD => dir = Direction::Right,
                    x if x > MOVE_THRESHOLD => dir = Direction::Left,
                    _ => {}
                }
            } else {
                match delta_y {
                    y if y < -MOVE_THRESHOLD => dir = Direction::Down,
                    y if y > MOVE_THRESHOLD => dir = Direction::Up,
                    _ => {}
                }
            }

            if dir == Direction::Unknown {
                continue;
            }

            println!("Direction: {:?}", dir);
            dir = Direction::Unknown;

            points.clear();
            last_tick = Instant::now();
        }
    }
}
