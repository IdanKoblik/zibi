use serde::{Deserialize, Serialize};

use crate::hand::Hand;
use crate::point::Point;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Landmark {
    pub hand: Hand,
    pub camera_width: i16,
    pub camera_height: i16,
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
        let landmark = Landmark::parse(
            r#"{"hand":"Right","camera_width":640,"camera_height":480,"points":[{"x":100,"y":200},{"x":10,"y":20}]}"#,
        )
        .expect("valid landmark should parse");

        assert!(matches!(landmark.hand, Hand::Right));
        assert_eq!(landmark.camera_width, 640);
        assert_eq!(landmark.camera_height, 480);
        assert_eq!(
            landmark.points,
            vec![Point { x: 100, y: 200 }, Point { x: 10, y: 20 }]
        );
    }

    #[test]
    fn parse_left_hand() {
        let landmark = Landmark::parse(
            r#"{"hand":"Left","camera_width":320,"camera_height":240,"points":[{"x":-5,"y":-10}]}"#,
        )
        .expect("valid landmark should parse");

        assert!(matches!(landmark.hand, Hand::Left));
        assert_eq!(landmark.camera_width, 320);
        assert_eq!(landmark.camera_height, 240);
        assert_eq!(landmark.points, vec![Point { x: -5, y: -10 }]);
    }

    #[test]
    fn parse_empty_points() {
        let landmark = Landmark::parse(
            r#"{"hand":"Right","camera_width":640,"camera_height":480,"points":[]}"#,
        )
        .expect("valid landmark should parse");

        assert!(matches!(landmark.hand, Hand::Right));
        assert!(landmark.points.is_empty());
    }

    #[test]
    fn parse_ignores_field_order_and_whitespace() {
        let landmark = Landmark::parse(
            r#" { "points" : [ { "y" : 2 , "x" : 1 } ] , "camera_height" : 480 , "camera_width" : 640 , "hand" : "Left" } "#,
        )
        .expect("valid landmark should parse");

        assert!(matches!(landmark.hand, Hand::Left));
        assert_eq!(landmark.camera_width, 640);
        assert_eq!(landmark.camera_height, 480);
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
        assert!(Landmark::parse(r#"{"hand":"Right","camera_width":640,"camera_height":480}"#).is_none());
        assert!(Landmark::parse(
            r#"{"camera_width":640,"camera_height":480,"points":[{"x":1,"y":2}]}"#
        )
        .is_none());
    }

    #[test]
    fn parse_rejects_missing_camera_dimensions() {
        assert!(Landmark::parse(r#"{"hand":"Right","points":[{"x":1,"y":2}]}"#).is_none());
        assert!(Landmark::parse(
            r#"{"hand":"Right","camera_width":640,"points":[{"x":1,"y":2}]}"#
        )
        .is_none());
        assert!(Landmark::parse(
            r#"{"hand":"Right","camera_height":480,"points":[{"x":1,"y":2}]}"#
        )
        .is_none());
    }

    #[test]
    fn parse_rejects_unknown_hand() {
        assert!(Landmark::parse(
            r#"{"hand":"Middle","camera_width":640,"camera_height":480,"points":[{"x":1,"y":2}]}"#
        )
        .is_none());
    }

    #[test]
    fn parse_rejects_malformed_point() {
        assert!(Landmark::parse(
            r#"{"hand":"Right","camera_width":640,"camera_height":480,"points":[{"x":1}]}"#
        )
        .is_none());
        assert!(Landmark::parse(
            r#"{"hand":"Right","camera_width":640,"camera_height":480,"points":"1,2"}"#
        )
        .is_none());
        assert!(Landmark::parse(
            r#"{"hand":"Right","camera_width":640,"camera_height":480,"points":[{"x":40000,"y":0}]}"#
        )
        .is_none());
    }

    #[test]
    fn parse_rejects_out_of_range_camera_dimensions() {
        assert!(Landmark::parse(
            r#"{"hand":"Right","camera_width":40000,"camera_height":480,"points":[]}"#
        )
        .is_none());
        assert!(Landmark::parse(
            r#"{"hand":"Right","camera_width":640,"camera_height":40000,"points":[]}"#
        )
        .is_none());
    }
}
