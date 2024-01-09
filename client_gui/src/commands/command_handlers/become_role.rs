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
    if active_lobby.is_none() {
        return Err(CommandError::NotConnected);
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::BecomeRole,
        &(*user_id, user_type),
    )?;

    println!("will become {:?}", user_type);

    Ok(())
}
