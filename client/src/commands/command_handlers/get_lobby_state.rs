use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{LobbyAddr, LobbyShort, LobbyStateShort},
};

pub fn get_lobby_state(lobby_addr: LobbyAddr) -> Result<LobbyShort, CommandError> {
    let res: LobbyStateShort = request(lobby_addr.addr, Type::GetLobbyState, &())?;

    println!("state: {res:?}");

    Ok(LobbyShort {
        id: lobby_addr.id,
        addr: lobby_addr.addr,
        name: res.name,
        players: res.players,
    })
}
