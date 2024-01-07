use network::{request, Type};

use crate::{
    commands::{Command, CommandError},
    types::{LobbyAddr, UserId, LobbyVec},
    SERVER_ADDR,
};

pub struct CreateLobbyCmd {
    user_id: UserId,
    lobby_name: String,
    lobbies: LobbyVec,
}

impl CreateLobbyCmd {
    pub fn new(user_id: UserId, lobby_name: String, lobbies: LobbyVec) -> CreateLobbyCmd {
        CreateLobbyCmd { user_id, lobby_name, lobbies }
    }
}

impl Command for CreateLobbyCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        let lobby: LobbyAddr = request(SERVER_ADDR, Type::CreateLobby, &(*self.user_id.borrow(), self.lobby_name.clone()))?;

        println!("received: {lobby:?}");

        let mut lobbies = self.lobbies.lock().unwrap();
        lobbies.push(lobby);

        Ok(())
    }
}
