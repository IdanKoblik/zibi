use serde::Deserialize;
use serde::Serialize;

use gtk::DropDown;

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub enum Hand {
    Left = 0,
    Right = 1,
}

impl From<Hand> for DropDown {
    fn from(hand: Hand) -> Self {
        let dropdown = DropDown::from_strings(&["Left", "Right"]);
        dropdown.set_selected(hand as u32);
        dropdown
    }
}
