use crate::types::{ActiveLobby, Player};

use super::{Event, EventError};

pub struct PlayerJoinedEvent {
    player: Player,
    active_lobby: ActiveLobby,
}

impl PlayerJoinedEvent {
    pub fn new(player: Player, active_lobby: ActiveLobby) -> PlayerJoinedEvent {
        PlayerJoinedEvent {
            player,
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
                active_lobby.0.players.dedup_by(|a, b| a.id == b.id);
                println!("player {} joined", self.player);
            }
        }

        Ok(())
    }
}
