use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::SendRecv;

use crate::core::{
    request_handlers::error_check,
    types::{LobbyId, LobbyState, UserInfoShort, UsersVec},
};

use super::{error::ServerError, Request};

pub struct GetLobbyStateRequest {
    stream: TcpStream,
    id: LobbyId,
    users: UsersVec,
}

impl GetLobbyStateRequest {
    pub fn new(stream: TcpStream, _: (), id: LobbyId, users: UsersVec) -> GetLobbyStateRequest {
        GetLobbyStateRequest { stream, id, users }
    }

    fn handler(&self) -> Result<LobbyState, ServerError> {
        Ok(LobbyState {
            id: { *self.id.lock().unwrap() },
            users: {
                self.users
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|u| UserInfoShort::from(u))
                    .collect()
            },
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
