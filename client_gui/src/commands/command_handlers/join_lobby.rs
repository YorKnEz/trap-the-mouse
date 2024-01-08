use std::net::SocketAddr;

use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{Lobby, LobbyState, UserId},
};

pub fn join_lobby_cmd(
    user_id: &UserId,
    lobby_addr: SocketAddr,
    active_lobby: &Option<Lobby>,
) -> Result<LobbyState, CommandError> {
    if let Some(_) = active_lobby {
        return Err(CommandError::CommandError {
            message: "you are already in a lobby".to_string(),
        });
    }

    let res: LobbyState = request(lobby_addr, Type::JoinLobby, user_id)?;

    println!("joined lobby");

    Ok(res)
}
