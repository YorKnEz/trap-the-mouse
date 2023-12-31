use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};

use crate::core::LobbyVec;

use super::Request;

pub struct DeleteLobbyRequest {
    stream: TcpStream,
    id: u16,
    lobbies: LobbyVec,
}

impl DeleteLobbyRequest {
    pub fn new(stream: TcpStream, id: u16, lobbies: LobbyVec) -> DeleteLobbyRequest {
        DeleteLobbyRequest {
            stream,
            id,
            lobbies,
        }
    }

    fn handler(&self) -> Result<()> {
        let mut lobbies = self.lobbies.lock().unwrap();
        let index = match lobbies.iter().position(|(id, _, _, _)| *id == self.id) {
            Some(index) => index,
            None => return Err(anyhow!(format!("lobby not found"))),
        };

        let (_, _, running, handle) = lobbies.remove(index);

        {
            let mut running = running.lock().unwrap();
            *running = false;
        }

        match handle.join() {
            Ok(_) => {}
            Err(e) => println!("lobby thread panicked: {e:?}"),
        }

        Ok(())
    }
}

impl Request for DeleteLobbyRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = match self.handler() {
            Ok(res) => (Type::Success, bincode::serialize(&res)?),
            Err(e) => {
                println!("couldn't delete lobby: {e:?}");
                (Type::Error, bincode::serialize("internal error")?)
            }
        };

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
