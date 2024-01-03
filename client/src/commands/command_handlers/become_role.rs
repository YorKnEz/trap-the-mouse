use network::{request, Type};

use crate::{
    commands::{Command, CommandError},
    types::{ActiveLobby, UserId, UserType},
};

pub struct BecomeRoleCmd {
    user_id: UserId,
    user_type: UserType,
    active_lobby: ActiveLobby,
}

impl BecomeRoleCmd {
    pub fn new(
        user_id: UserId,
        user_type: UserType,
        active_lobby: ActiveLobby,
    ) -> BecomeRoleCmd {
        BecomeRoleCmd {
            user_id,
            user_type,
            active_lobby,
        }
    }
}

impl Command for BecomeRoleCmd {
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
                Type::BecomeRole,
                &(*self.user_id.borrow(), self.user_type),
            )?;
        }

        println!("will become {:?}", self.user_type);

        Ok(())
    }
}
