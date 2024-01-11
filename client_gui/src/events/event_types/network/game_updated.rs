use serde_derive::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct GameUpdatedEvent {
    pub win: (bool, bool), // (devil won, angel won)
    pub turn: bool,
    pub user_move: (i32, i32),
}

impl GameUpdatedEvent {
    pub fn new(update: GameUpdatedEvent) -> GameUpdatedEvent {
        update
    }
}
