use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};

use super::request_handlers::{
    GetLobbyStateRequest, InvalidRequest, JoinLobbyRequest, LeaveLobbyRequest, PingRequest,
};
use super::types::{LobbyId, UsersVec};
use super::{RequestHandler, RequestQueueItem, ServerCore};
use network::{request, SendRecv, Type};

pub struct Lobby {
    pub server: ServerCore,
    pub id: LobbyId,
    pub users: UsersVec,
}

impl RequestHandler for Lobby {
    fn handle(&self, stream: TcpStream) -> Result<RequestQueueItem> {
        let mut stream = stream;

        let (buf, req_type) = match stream.recv() {
            Ok(res) => res,
            Err(e) => return Err(anyhow!(format!("couldn't recv: {e:?}"))),
        };

        Ok(match req_type {
            Type::Ping => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(PingRequest::new(stream, buf)),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::GetLobbyState => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(GetLobbyStateRequest::new(
                    stream,
                    buf,
                    Arc::clone(&self.id),
                    Arc::clone(&self.users),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::JoinLobby => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(JoinLobbyRequest::new(
                    stream,
                    buf,
                    Arc::clone(&self.users),
                    self.server.db_pool.clone(),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::LeaveLobby => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(LeaveLobbyRequest::new(
                    stream,
                    buf,
                    Arc::clone(&self.users),
                    self.server.db_pool.clone(),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            _ => Box::new(InvalidRequest::new(stream, "invalid request")),
        })
    }
}

impl Lobby {
    pub fn new(addr: &str, id: u16) -> Result<Lobby> {
        let server = ServerCore::new(addr)?;

        Ok(Lobby {
            server,
            id: Arc::new(Mutex::new(id)),
            users: Arc::new(Mutex::new(vec![])),
        })
    }

    pub fn start(&self) -> Result<()> {
        self.server.start(self)?;
        Ok(())
    }

    pub fn get_addr(&self) -> Result<SocketAddr> {
        let addr = self.server.get_addr()?;
        Ok(addr)
    }
}

impl Drop for Lobby {
    fn drop(&mut self) {
        let mut users = self.users.lock().unwrap();

        while let Some(user) = users.pop() {
            request(user.addr, Type::LobbyClosing, &()).unwrap_or(());
        }
    }
}
