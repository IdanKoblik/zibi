use serde::Deserialize;
use serde::Serialize;

use gtk::DropDown;

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum Hand {
    Left,
    Right,
}

impl From<Hand> for DropDown {
    fn from(hand: Hand) -> Self {
        match hand {
            Hand::Left => DropDown::from_strings(&["Left", "Right"]),
            Hand::Right => DropDown::from_strings(&["Left", "Right"]),
        }
    }
}
