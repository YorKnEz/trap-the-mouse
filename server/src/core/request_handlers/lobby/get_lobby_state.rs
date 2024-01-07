use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::SendRecv;

use crate::core::{
    request_handlers::error_check,
    types::{LobbyName, UsersVec, LobbyStateShort},
};

use super::{error::ServerError, Request};

pub struct GetLobbyStateRequest {
    stream: TcpStream,
    name: LobbyName,
    users: UsersVec,
}

impl GetLobbyStateRequest {
    pub fn new(stream: TcpStream, _: (), name: LobbyName, users: UsersVec) -> GetLobbyStateRequest {
        GetLobbyStateRequest {
            stream,
            name,
            users,
        }
    }

    fn handler(&self) -> Result<LobbyStateShort, ServerError> {
        Ok(LobbyStateShort {
            name: { self.name.lock().unwrap().clone() },
            users: { self.users.lock().unwrap().len() as u32 },
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
