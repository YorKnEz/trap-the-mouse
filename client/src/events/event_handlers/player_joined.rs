use crate::types::{ActiveLobby, Player, UserType};

use super::{Event, EventError};

pub struct PlayerJoinedEvent {
    player: Player,
    active_lobby: ActiveLobby,
}

impl PlayerJoinedEvent {
    pub fn new(data: (UserType, u32, String), active_lobby: ActiveLobby) -> PlayerJoinedEvent {
        PlayerJoinedEvent {
            player: Player {
                user_type: data.0,
                id: data.1,
                name: data.2,
            },
            active_lobby,
        }
    }
}

impl Event for PlayerJoinedEvent {
    fn execute(&self) -> Result<(), EventError> {
        {
            let mut active_lobby = self.active_lobby.lock().unwrap();

            if active_lobby.1 {
                active_lobby.0.players.push(self.player.clone());
                println!("player {:?} joined", self.player);
            }
        }

        Ok(())
    }
}
