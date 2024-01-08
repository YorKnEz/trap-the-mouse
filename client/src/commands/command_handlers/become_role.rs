use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{Lobby, UserId, UserType},
};

pub fn become_role_cmd(
    user_id: &UserId,
    user_type: UserType,
    active_lobby: &Option<Lobby>,
) -> Result<(), CommandError> {
    if let None = active_lobby {
        return Err(CommandError::CommandError {
            message: "you are not connected to a lobby".to_string(),
        });
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::BecomeRole,
        &(user_id, user_type),
    )?;

    println!("will become {:?}", user_type);

    Ok(())
}
