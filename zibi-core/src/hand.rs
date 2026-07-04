use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug)]
pub enum Hand {
    Left,
    Right,
}
