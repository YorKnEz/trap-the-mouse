use network::{request, Type};

use crate::{commands::{Command, CommandError}, LobbyVec, UserId, SERVER_ADDR, types::LobbyAddr};

pub struct CreateLobbyCmd {
    user_id: UserId,
    lobbies: LobbyVec,
}

impl CreateLobbyCmd {
    pub fn new(user_id: UserId, lobbies: LobbyVec) -> CreateLobbyCmd {
        CreateLobbyCmd { user_id, lobbies }
    }
}

impl Command for CreateLobbyCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        let lobby: LobbyAddr = request(SERVER_ADDR, Type::CreateLobby, &*self.user_id.borrow())?;

        println!("received: {lobby:?}");

        let mut lobbies = self.lobbies.lock().unwrap();
        lobbies.push(lobby);

        Ok(())
    }
}
