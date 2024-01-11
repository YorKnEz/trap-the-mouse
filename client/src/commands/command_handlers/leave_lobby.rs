use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{Lobby, UserId},
};

pub fn leave_lobby_cmd(
    user_id: &UserId,
    active_lobby: &mut Option<Lobby>,
) -> Result<(), CommandError> {
    if active_lobby.is_none() {
        return Err(CommandError::NotConnected);
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::LeaveLobby,
        &user_id,
    )?;

    *active_lobby = None;

    println!("left lobby");

    Ok(())
}
