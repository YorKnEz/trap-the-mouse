use network::{request, Type};

use crate::{
    commands::{Command, CommandError}, types::{UserId, ActiveLobby},
};

pub struct LeaveLobbyCmd {
    user_id: UserId,
    active_lobby: ActiveLobby,
}

impl LeaveLobbyCmd {
    pub fn new(user_id: UserId, active_lobby: ActiveLobby) -> LeaveLobbyCmd {
        LeaveLobbyCmd {
            user_id,
            active_lobby,
        }
    }
}

impl Command for LeaveLobbyCmd {
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
                Type::LeaveLobby,
                &*self.user_id.borrow(),
            )?;

            active_lobby.1 = false;
        }

        println!("left lobby");

        Ok(())
    }
}
