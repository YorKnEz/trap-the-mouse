use network::{request, Type};

use crate::{
    commands::{Command, CommandError},
    types::{LobbyAddr, UserId, ActiveLobby, Lobby, LobbyState},
};

pub struct JoinLobbyCmd {
    user_id: UserId,
    lobby: LobbyAddr,
    active_lobby: ActiveLobby,
}

impl JoinLobbyCmd {
    pub fn new(user_id: UserId, lobby: LobbyAddr, active_lobby: ActiveLobby) -> JoinLobbyCmd {
        JoinLobbyCmd {
            user_id,
            lobby,
            active_lobby,
        }
    }
}

impl Command for JoinLobbyCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        {
            let mut active_lobby = self.active_lobby.lock().unwrap();

            if active_lobby.1 {
                return Err(CommandError::CommandError {
                    message: "you are already in a lobby".to_string(),
                });
            }

            let state: LobbyState =
                request(self.lobby.addr, Type::JoinLobby, &*self.user_id.borrow())?;

            active_lobby.1 = true;
            active_lobby.0 = Lobby {
                id: state.id,
                name: state.name.clone(),
                addr: self.lobby.addr,
                players: state.users,
            }
        }

        println!("joined lobby");

        Ok(())
    }
}
