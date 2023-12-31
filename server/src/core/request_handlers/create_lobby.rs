use std::{
    net::{SocketAddr, TcpStream},
    sync::Arc,
    thread,
};

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};

use crate::core::{
    lobby::{Lobby, LobbyId},
    LobbyVec,
};

use super::{Request, error::ServerError};

pub struct CreateLobbyRequest {
    stream: TcpStream,
    id: LobbyId,
    lobbies: LobbyVec,
}

impl CreateLobbyRequest {
    pub fn new(stream: TcpStream, id: LobbyId, lobbies: LobbyVec) -> CreateLobbyRequest {
        CreateLobbyRequest {
            stream,
            id,
            lobbies,
        }
    }

    fn handler(&self) -> Result<(u16, SocketAddr), ServerError> {
        let id = {
            let mut id = self.id.lock().unwrap();
            let ret = *id;
            *id += 1;
            ret
        };

        let lobby = Lobby::new("127.0.0.1:0", id).unwrap();
        let (addr, running) = (lobby.get_addr()?, Arc::clone(&lobby.server.running));

        let handle = thread::spawn(move || {
            lobby.start().unwrap();
        });

        println!("lobby {} started", id);

        {
            let mut lobbies = self.lobbies.lock().unwrap();
            lobbies.push((id, addr, running, handle));
        }

        Ok((id, addr))
    }
}

impl Request for CreateLobbyRequest {
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
