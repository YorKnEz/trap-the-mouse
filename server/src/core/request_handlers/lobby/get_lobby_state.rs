use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::SendRecv;

use crate::core::{
    request_handlers::error_check,
    types::{Game, LobbyName, LobbyStateShort, UsersVec},
};

use super::{error::ServerError, Request};

pub struct GetLobbyStateRequest {
    stream: TcpStream,
    name: LobbyName,
    users: UsersVec,
    game: Game,
}

impl GetLobbyStateRequest {
    pub fn new(
        stream: TcpStream,
        _: (),
        name: LobbyName,
        users: UsersVec,
        game: Game,
    ) -> GetLobbyStateRequest {
        GetLobbyStateRequest {
            stream,
            name,
            users,
            game,
        }
    }

    fn handler(&self) -> Result<LobbyStateShort, ServerError> {
        Ok(LobbyStateShort {
            name: { self.name.lock().unwrap().clone() },
            users: { self.users.lock().unwrap().len() as u32 },
            game_going: { self.game.lock().unwrap().is_some() },
        })
    }
}

impl Request for GetLobbyStateRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
