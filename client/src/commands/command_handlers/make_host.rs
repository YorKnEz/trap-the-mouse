use network::{request, Type};

use crate::{
    commands::{Command, CommandError},
    types::{ActiveLobby, UserId},
};

pub struct MakeHostCmd {
    user_id: UserId,
    new_host_id: u32,
    active_lobby: ActiveLobby,
}

impl MakeHostCmd {
    pub fn new(
        user_id: UserId,
        new_host_id: u32,
        active_lobby: ActiveLobby,
    ) -> MakeHostCmd {
        MakeHostCmd {
            user_id,
            new_host_id,
            active_lobby,
        }
    }
}

impl Command for MakeHostCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        {
            let active_lobby = self.active_lobby.lock().unwrap();

            if !active_lobby.1 {
                return Err(CommandError::CommandError {
                    message: "you are not connected to a lobby".to_string(),
                });
            }

            request(
                active_lobby.0.addr,
                Type::MakeHost,
                &(*self.user_id.borrow(), self.new_host_id),
            )?;
        }

        println!("user {} will be host", self.new_host_id);

        Ok(())
    }
}
