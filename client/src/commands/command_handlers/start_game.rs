use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{Lobby, UserId},
};

pub fn start_game_cmd(user_id: &UserId, active_lobby: &Option<Lobby>) -> Result<(), CommandError> {
    if active_lobby.is_none() {
        return Err(CommandError::NotConnected);
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::StartGame,
        user_id,
    )?;

    println!("game starting");

    Ok(())
}
