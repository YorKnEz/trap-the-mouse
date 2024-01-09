use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{Lobby, UserId},
};

pub fn make_move_cmd(
    user_id: &UserId,
    user_move: (i32, i32),
    active_lobby: &Option<Lobby>,
) -> Result<(), CommandError> {
    if let None = active_lobby {
        return Err(CommandError::CommandError {
            message: "you are not connected to a lobby".to_string(),
        });
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::MakeMove,
        &(*user_id, user_move),
    )?;

    println!("made move");

    Ok(())
}
