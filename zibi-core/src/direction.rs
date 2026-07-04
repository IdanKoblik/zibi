use crate::point::Point;

#[derive(Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub fn detect_direction(first: &Point, last: &Point, threshold: i16) -> Option<Direction> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
