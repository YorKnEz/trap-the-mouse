use network::{request, Type};

use crate::{commands::CommandError, types::{UserId, Lobby}, SERVER_ADDR};

pub fn change_name_cmd(user_id: &UserId, name: String, active_lobby: &Option<Lobby>) -> Result<(), CommandError> {
    request(SERVER_ADDR, Type::ChangeName, &(*user_id, name))?;

    if let Some(active_lobby) = active_lobby {
        request(active_lobby.addr, Type::ChangedName, user_id)?;
    }

    Ok(())
}
