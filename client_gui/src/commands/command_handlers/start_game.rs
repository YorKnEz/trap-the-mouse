use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{Lobby, UserId},
};

pub fn start_game_cmd(user_id: &UserId, active_lobby: &Option<Lobby>) -> Result<(), CommandError> {
    if let None = active_lobby {
        return Err(CommandError::CommandError {
            message: "you are not connected to a lobby".to_string(),
        });
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::StartGame,
        user_id,
    )?;

    println!("game starting");

    Ok(())
}
