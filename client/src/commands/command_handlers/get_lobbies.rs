use network::{request, Type};

use crate::{commands::{Command, CommandError}, LobbyVec, UserId, SERVER_ADDR, types::LobbyAddr};

pub struct GetLobbiesCmd {
    user_id: UserId,
    start: u32,
    offset: u32,
    lobbies: LobbyVec,
}

impl GetLobbiesCmd {
    pub fn new(user_id: UserId, start: u32, offset: u32, lobbies: LobbyVec) -> GetLobbiesCmd {
        GetLobbiesCmd {
            user_id,
            start,
            offset,
            lobbies,
        }
    }
}

impl Command for GetLobbiesCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        let mut new_lobbies: Vec<LobbyAddr> =
            request(SERVER_ADDR, Type::GetLobbies, &(*self.user_id.borrow(), self.start, self.offset))?;

        println!("received: {new_lobbies:?}");

        let mut lobbies = self.lobbies.lock().unwrap();
        lobbies.append(&mut new_lobbies);
        lobbies.sort_by_key(|a| a.id);
        lobbies.dedup_by(|a, b| a.id == b.id);

        Ok(())
    }
}
