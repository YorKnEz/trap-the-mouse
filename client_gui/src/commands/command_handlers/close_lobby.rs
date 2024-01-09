use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{Lobby, UserId},
};

pub fn close_lobby_cmd(
    user_id: &UserId,
    active_lobby: &mut Option<Lobby>,
) -> Result<u16, CommandError> {
    if active_lobby.is_none() {
        return Err(CommandError::NotConnected);
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::CloseLobby,
        user_id,
    )?;

    let id = active_lobby.as_ref().unwrap().id;
    *active_lobby = None;

    println!("closed lobby");

    Ok(id)
}
