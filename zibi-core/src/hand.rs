use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub enum Hand {
    Left,
    Right,
}
