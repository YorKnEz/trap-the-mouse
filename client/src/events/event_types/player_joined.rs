use crate::types::Player;

#[derive(Clone, Debug)]
pub struct PlayerJoinedEvent {
    pub player: Player,
}

impl PlayerJoinedEvent {
    pub fn new(player: Player) -> PlayerJoinedEvent {
        PlayerJoinedEvent { player }
    }
}
