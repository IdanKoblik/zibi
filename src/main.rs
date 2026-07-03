use std::io::{self, BufRead};
use std::time::{Duration, Instant};

use niri_ipc::{Action, Request, socket};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Point {
    x: i16,
    y: i16,
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

fn detect_direction(first: &Point, last: &Point, threshold: i16) -> Option<Direction> {
    let delta_x = first.x - last.x;
    let delta_y = first.y - last.y;

    let abs_dx = delta_x.abs();
    let abs_dy = delta_y.abs();

    if abs_dx > threshold || abs_dy > threshold {
        let dir = if abs_dx > abs_dy {
            if delta_x < 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else {
            if delta_y < 0 {
                Direction::Down
            } else {
                Direction::Up
            }
        };
        Some(dir)
    } else {
        None
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        assert_eq!(Point::parse("3,4"), Some(Point { x: 3, y: 4 }));
    }

    #[test]
    fn parse_negative_values() {
        assert_eq!(Point::parse("-12,-7"), Some(Point { x: -12, y: -7 }));
    }

    #[test]
    fn parse_trims_surrounding_and_inner_whitespace() {
        assert_eq!(Point::parse("  10 , 20  "), Some(Point { x: 10, y: 20 }));
        assert_eq!(Point::parse("\t5,\t6\n"), Some(Point { x: 5, y: 6 }));
    }

    #[test]
    fn parse_zero() {
        assert_eq!(Point::parse("0,0"), Some(Point { x: 0, y: 0 }));
    }

    #[test]
    fn parse_rejects_missing_comma() {
        assert_eq!(Point::parse("42"), None);
    }

    #[test]
    fn parse_rejects_empty() {
        assert_eq!(Point::parse(""), None);
        assert_eq!(Point::parse("   "), None);
    }

    #[test]
    fn parse_rejects_non_numeric() {
        assert_eq!(Point::parse("a,b"), None);
        assert_eq!(Point::parse("1,b"), None);
        assert_eq!(Point::parse("a,2"), None);
    }

    #[test]
    fn parse_rejects_out_of_range() {
        // i16 max is 32767.
        assert_eq!(Point::parse("40000,0"), None);
    }

    #[test]
    fn parse_rejects_extra_component() {
        assert_eq!(Point::parse("1,2,3"), None);
    }

    const T: i16 = 150;

    #[test]
    fn no_direction_when_below_threshold() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 100, y: 100 };
        assert_eq!(detect_direction(&a, &b, T), None);
    }

    #[test]
    fn no_direction_when_exactly_at_threshold() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 150, y: 0 };
        assert_eq!(detect_direction(&a, &b, T), None);
    }

    #[test]
    fn moving_left_to_right_is_right() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 200, y: 0 };
        assert_eq!(detect_direction(&a, &b, T), Some(Direction::Right));
    }

    #[test]
    fn moving_right_to_left_is_left() {
        let a = Point { x: 200, y: 0 };
        let b = Point { x: 0, y: 0 };
        assert_eq!(detect_direction(&a, &b, T), Some(Direction::Left));
    }

    #[test]
    fn moving_top_to_bottom_is_down() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 0, y: 200 };
        assert_eq!(detect_direction(&a, &b, T), Some(Direction::Down));
    }

    #[test]
    fn moving_bottom_to_top_is_up() {
        let a = Point { x: 0, y: 200 };
        let b = Point { x: 0, y: 0 };
        assert_eq!(detect_direction(&a, &b, T), Some(Direction::Up));
    }

    #[test]
    fn dominant_axis_wins_when_both_exceed_threshold() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 300, y: 200 };
        assert_eq!(detect_direction(&a, &b, T), Some(Direction::Right));

        // Vertical movement is larger.
        let c = Point { x: 0, y: 0 };
        let d = Point { x: 200, y: 300 };
        assert_eq!(detect_direction(&c, &d, T), Some(Direction::Down));
    }

    #[test]
    fn ties_favor_vertical_axis() {
        let a = Point { x: 0, y: 0 };
        let b = Point { x: 200, y: 200 };
        assert_eq!(detect_direction(&a, &b, T), Some(Direction::Down));
    }
}
