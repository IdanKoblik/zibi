use serde::{Deserialize, Serialize};

use crate::hand::Hand;
use crate::point::Point;

#[derive(Serialize, Deserialize, Debug)]
pub struct Landmark {
    pub hand: Hand,
    pub points: Vec<Point>,
}

impl Landmark {
    pub fn parse(input: &str) -> Option<Landmark> {
        serde_json::from_str::<Landmark>(input).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let landmark =
            Landmark::parse(r#"{"hand":"Right","points":[{"x":100,"y":200},{"x":10,"y":20}]}"#)
                .expect("valid landmark should parse");

        assert!(matches!(landmark.hand, Hand::Right));
        assert_eq!(
            landmark.points,
            vec![Point { x: 100, y: 200 }, Point { x: 10, y: 20 }]
        );
    }

    #[test]
    fn parse_left_hand() {
        let landmark = Landmark::parse(r#"{"hand":"Left","points":[{"x":-5,"y":-10}]}"#)
            .expect("valid landmark should parse");

        assert!(matches!(landmark.hand, Hand::Left));
        assert_eq!(landmark.points, vec![Point { x: -5, y: -10 }]);
    }

    #[test]
    fn parse_empty_points() {
        let landmark = Landmark::parse(r#"{"hand":"Right","points":[]}"#)
            .expect("valid landmark should parse");

        assert!(matches!(landmark.hand, Hand::Right));
        assert!(landmark.points.is_empty());
    }

    #[test]
    fn parse_ignores_field_order_and_whitespace() {
        let landmark =
            Landmark::parse(r#" { "points" : [ { "y" : 2 , "x" : 1 } ] , "hand" : "Left" } "#)
                .expect("valid landmark should parse");

        assert!(matches!(landmark.hand, Hand::Left));
        assert_eq!(landmark.points, vec![Point { x: 1, y: 2 }]);
    }

    #[test]
    fn parse_rejects_invalid_json() {
        assert!(Landmark::parse("not json").is_none());
        assert!(Landmark::parse("").is_none());
        assert!(Landmark::parse("{").is_none());
    }

    #[test]
    fn parse_rejects_missing_fields() {
        assert!(Landmark::parse(r#"{"hand":"Right"}"#).is_none());
        assert!(Landmark::parse(r#"{"points":[{"x":1,"y":2}]}"#).is_none());
    }

    #[test]
    fn parse_rejects_unknown_hand() {
        assert!(Landmark::parse(r#"{"hand":"Middle","points":[{"x":1,"y":2}]}"#).is_none());
    }

    #[test]
    fn parse_rejects_malformed_point() {
        assert!(Landmark::parse(r#"{"hand":"Right","points":[{"x":1}]}"#).is_none());
        assert!(Landmark::parse(r#"{"hand":"Right","points":"1,2"}"#).is_none());
        assert!(Landmark::parse(r#"{"hand":"Right","points":[{"x":40000,"y":0}]}"#).is_none());
    }
}
