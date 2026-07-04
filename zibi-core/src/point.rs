use std::time::Instant;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub fn parse(input: &str) -> Option<Self> {
        let (x, y) = input.trim().split_once(',')?;

        Some(Self {
            x: x.trim().parse().ok()?,
            y: y.trim().parse().ok()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PointRecord {
    pub point: Point,
    pub time: Instant,
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

}
