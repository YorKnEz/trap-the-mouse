use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{Lobby, UserId},
    SERVER_ADDR,
};

pub fn change_name_cmd(
    user_id: &UserId,
    name: String,
    active_lobby: &Option<Lobby>,
) -> Result<(), CommandError> {
    if name.is_empty() {
        return Err(CommandError::EmptyString);
    }

    request(SERVER_ADDR.with(|&a| a), Type::ChangeName, &(*user_id, name))?;

    if let Some(active_lobby) = active_lobby {
        request(active_lobby.addr, Type::ChangedName, user_id)?;
    }

    Ok(())
}
