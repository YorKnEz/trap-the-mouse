use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};

use super::request_handlers::{
    BecomeRoleRequest, CloseLobbyRequest, GetLobbyStateRequest, InvalidRequest, JoinLobbyRequest,
    LeaveLobbyRequest, MakeHostRequest, PingRequest,
};
use super::types::{LobbyId, UsersVec, LobbyName};
use super::{RequestHandler, RequestQueueItem, ServerCore};
use network::{request, SendRecv, Type};

pub struct Lobby {
    pub server: ServerCore,
    pub id: LobbyId,
    pub name: LobbyName,
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
                    Arc::clone(&self.name),
                    Arc::clone(&self.users),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::JoinLobby => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(JoinLobbyRequest::new(
                    stream,
                    buf,
                    Arc::clone(&self.name),
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
                    Arc::clone(&self.server.running),
                    self.server.db_pool.clone(),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::CloseLobby => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(CloseLobbyRequest::new(
                    stream,
                    buf,
                    Arc::clone(&self.users),
                    Arc::clone(&self.server.running),
                    self.server.db_pool.clone(),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::MakeHost => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(MakeHostRequest::new(
                    stream,
                    buf,
                    Arc::clone(&self.users),
                    self.server.db_pool.clone(),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::BecomeRole => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(BecomeRoleRequest::new(
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
    pub fn new(addr: &str, id: u16, name: String) -> Result<Lobby> {
        let server = ServerCore::new(addr)?;

        Ok(Lobby {
            server,
            id: Arc::new(Mutex::new(id)),
            name: Arc::new(Mutex::new(name)),
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
