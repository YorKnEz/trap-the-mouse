use serde_derive::Deserialize;

use crate::types::GRID_SIZE;

#[derive(Clone, Debug, Deserialize)]
pub struct GameStartedEvent {
    pub angel: u32, // id of the user that is the angel, if 0 it's the computer
    pub devil: u32, // id of the user that is the devil
    pub devil_pos: (i32, i32),
    pub turn: bool,                           // true - angel, false - devil
    pub grid: [[bool; GRID_SIZE]; GRID_SIZE], // whether the tile is blocked or not
}

impl GameStartedEvent {
    pub fn new(state: GameStartedEvent) -> GameStartedEvent {
        state
    }
}
