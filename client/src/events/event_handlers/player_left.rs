use crate::ActiveLobby;

use super::{Event, EventError};

pub struct PlayerLeftEvent {
    user_id: u32,
    active_lobby: ActiveLobby,
}

impl PlayerLeftEvent {
    pub fn new(user_id: u32, active_lobby: ActiveLobby) -> PlayerLeftEvent {
        PlayerLeftEvent {
            user_id,
            active_lobby,
        }
    }
}

impl Event for PlayerLeftEvent {
    fn execute(&self) -> Result<(), EventError> {
        {
            let mut active_lobby = self.active_lobby.lock().unwrap();

            if active_lobby.1 {
                if let Some(index) = active_lobby
                    .0
                    .players
                    .iter()
                    .position(|p| p.id == self.user_id)
                {
                    active_lobby.0.players.remove(index);

                    println!("player with id {} left", self.user_id);
                }
            }
        }
        Ok(())
    }
}
