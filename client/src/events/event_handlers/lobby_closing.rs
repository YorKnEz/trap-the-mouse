use crate::{ActiveLobby, LobbyVec};

use super::{Event, EventError};

pub struct LobbyClosingEvent {
    lobbies: LobbyVec,
    active_lobby: ActiveLobby,
}

impl LobbyClosingEvent {
    pub fn new(_: (), lobbies: LobbyVec, active_lobby: ActiveLobby) -> LobbyClosingEvent {
        LobbyClosingEvent {
            lobbies,
            active_lobby,
        }
    }
}

impl Event for LobbyClosingEvent {
    fn execute(&self) -> Result<(), EventError> {
        {
            let mut active_lobby = self.active_lobby.lock().unwrap();

            if active_lobby.1 {
                let mut lobbies = self.lobbies.lock().unwrap();

                if let Some(index) = lobbies.iter().position(|l| l.id == active_lobby.0.id) {
                    lobbies.remove(index);
                }

                active_lobby.1 = false;

                println!("lobby closing");
            }
        }
        Ok(())
    }
}
