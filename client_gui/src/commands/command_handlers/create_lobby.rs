use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{LobbyAddr, UserId},
    SERVER_ADDR,
};

pub fn create_lobby_cmd(user_id: &UserId, name: String) -> Result<LobbyAddr, CommandError> {
    let lobby: LobbyAddr = request(SERVER_ADDR.with(|&a| a), Type::CreateLobby, &(*user_id, name))?;

    println!("received: {lobby:?}");

    Ok(lobby)
}
