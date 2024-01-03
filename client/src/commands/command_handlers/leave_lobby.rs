use std::net::SocketAddr;

use network::{request, Type};

use crate::{
    commands::{Command, CommandError},
    UserId, ActiveLobby,
};

pub struct LeaveLobbyCmd {
    user_id: UserId,
    lobby_addr: SocketAddr,
    active_lobby: ActiveLobby,
}

impl LeaveLobbyCmd {
    pub fn new(user_id: UserId, lobby_addr: SocketAddr, active_lobby: ActiveLobby) -> LeaveLobbyCmd {
        LeaveLobbyCmd {
            user_id,
            lobby_addr,
            active_lobby,
        }
    }
}

impl Command for LeaveLobbyCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        request(self.lobby_addr, Type::LeaveLobby, &*self.user_id.borrow())?;

        {
            let mut active_lobby = self.active_lobby.lock().unwrap();

            active_lobby.1 = false;
        }

        println!("left lobby");

        Ok(())
    }
}
