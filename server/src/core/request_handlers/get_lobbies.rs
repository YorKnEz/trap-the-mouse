use std::net::{SocketAddr, TcpStream};

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};

use crate::core::LobbyVec;

use super::error::ServerError;
use super::Request;

pub struct GetLobbiesRequest {
    stream: TcpStream,
    start: u32,
    offset: u32,
    lobbies: LobbyVec,
}

impl GetLobbiesRequest {
    pub fn new(stream: TcpStream, range: (u32, u32), lobbies: LobbyVec) -> GetLobbiesRequest {
        GetLobbiesRequest {
            stream,
            start: range.0,
            offset: range.1,
            lobbies,
        }
    }

    fn handler(&self) -> Result<Vec<(u16, SocketAddr)>, ServerError> {
        if self.offset > 10 {
            return Err(ServerError::API {
                message: "offset can be at most 10".to_string(),
            });
        }

        let lobbies = { self.lobbies.lock().unwrap() };

        if lobbies.len() < (self.start + self.offset) as usize {
            return Err(ServerError::API {
                message: "invalid start and offset".to_string(),
            });
        }

        Ok(
            (&lobbies[self.start as usize..(self.start + self.offset) as usize])
                .iter()
                .map(|item| (item.0, item.1))
                .collect(),
        )
    }
}

impl Request for GetLobbiesRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = match self.handler() {
            Ok(res) => (Type::Success, bincode::serialize(&res)?),
            Err(e) => {
                println!("couldn't create lobby: {e:?}");
                match e {
                    ServerError::Internal(_) => (Type::Error, bincode::serialize("internal error")?),
                    ServerError::API { message } => (Type::Error, bincode::serialize(&message)?)
                }
            }
        };

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
