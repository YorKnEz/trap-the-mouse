use network::{request, Type};

use crate::{
    commands::{Command, CommandError},
    types::{ActiveLobby, LobbyVec, UserId},
};

pub struct CloseLobbyCmd {
    user_id: UserId,
    active_lobby: ActiveLobby,
    lobbies: LobbyVec,
}

impl CloseLobbyCmd {
    pub fn new(user_id: UserId, lobby: ActiveLobby, lobbies: LobbyVec) -> CloseLobbyCmd {
        CloseLobbyCmd {
            user_id,
            active_lobby: lobby,
            lobbies,
        }
    }
}

impl Command for CloseLobbyCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        {
            let mut active_lobby = self.active_lobby.lock().unwrap();

            if !active_lobby.1 {
                return Err(CommandError::CommandError {
                    message: "you are not connected to a lobby".to_string(),
                });
            }

            request(
                active_lobby.0.addr,
                Type::CloseLobby,
                &*self.user_id.borrow(),
            )?;

            active_lobby.1 = false;

            let mut lobbies = self.lobbies.lock().unwrap();

            if let Some(index) = lobbies.iter().position(|i| i.id == active_lobby.0.id) {
                lobbies.remove(index);
            }
        }

        println!("closed lobby");

        Ok(())
    }
}
