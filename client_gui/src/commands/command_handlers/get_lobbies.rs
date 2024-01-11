use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{LobbyAddrVec, UserId},
    SERVER_ADDR,
};

pub fn get_lobbies_cmd(
    user_id: &UserId,
    start: u32,
    offset: u32,
) -> Result<LobbyAddrVec, CommandError> {
    let new_lobbies: LobbyAddrVec =
        request(SERVER_ADDR.with(|&a| a), Type::GetLobbies, &(*user_id, start, offset))?;

    println!("received: {new_lobbies:?}");

    Ok(new_lobbies)
}
