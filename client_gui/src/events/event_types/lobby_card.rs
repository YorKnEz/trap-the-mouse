use crate::{types::LobbyShort, events::Window};

#[derive(Clone, Debug)]
pub struct LobbyCardEventData {
    pub window: Window,
    pub id: u32,
    pub data: LobbyShort,
}

impl PartialEq for LobbyCardEventData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.window == other.window
    }
}
