use crate::types::Player;

#[derive(Clone, Debug)]
pub struct PlayerUpdatedEvent {
    pub player: Player,
}

impl PlayerUpdatedEvent {
    pub fn new(player: Player) -> PlayerUpdatedEvent {
        PlayerUpdatedEvent { player }
    }
}
