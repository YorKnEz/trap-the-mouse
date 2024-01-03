use network::{request, Type};

use crate::{commands::{Command, CommandError}, LobbyVec, UserId, SERVER_ADDR};

pub struct DeleteLobbyCmd {
    user_id: UserId,
    lobby_id: u16,
    lobbies: LobbyVec,
}

impl DeleteLobbyCmd {
    pub fn new(user_id: UserId, lobby_id: u16, lobbies: LobbyVec) -> DeleteLobbyCmd {
        DeleteLobbyCmd {
            user_id,
            lobby_id,
            lobbies,
        }
    }
}

impl Command for DeleteLobbyCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        request(
            SERVER_ADDR,
            Type::DeleteLobby,
            &(*self.user_id.borrow(), self.lobby_id),
        )?;

        let mut lobbies = self.lobbies.lock().unwrap();

        if let Some(index) = lobbies.iter().position(|i| i.id == self.lobby_id) {
            lobbies.remove(index);

            println!("deleted lobby")
        }

        Ok(())
    }
}
