use crate::{events::Window, types::Player};

#[derive(Clone, Debug)]
pub struct PlayerCardEventData {
    pub window: Window,
    pub id: u32,
    pub data: Player,
}

impl PartialEq for PlayerCardEventData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.window == other.window
    }
}
