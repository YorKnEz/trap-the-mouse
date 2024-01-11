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
    if active_lobby.is_none() {
        return Err(CommandError::NotConnected);
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::MakeMove,
        &(*user_id, user_move),
    )?;

    println!("made move");

    Ok(())
}
