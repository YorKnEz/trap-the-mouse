use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};

use super::request_handlers::{InvalidRequest, PingRequest};
use super::{RequestHandler, RequestQueueItem, ServerCore};
use network::{SendRecv, Type};

pub type LobbyId = Arc<Mutex<u16>>;

pub struct Lobby {
    pub server: ServerCore,
    pub id: LobbyId,
}

impl RequestHandler for Lobby {
    fn handle(&self, stream: TcpStream) -> Result<RequestQueueItem> {
        let mut stream = stream;

        let (buf, req_type) = match stream.recv() {
            Ok(res) => res,
            Err(e) => return Err(anyhow!(format!("couldn't recv: {e:?}"))),
        };

        Ok(match req_type {
            Type::Ping => Box::new(PingRequest::new(stream, bincode::deserialize(&buf)?)),
            _ => Box::new(InvalidRequest::new(stream)),
        })
    }
}

impl Lobby {
    pub fn new(addr: &str, id: u16) -> Result<Lobby> {
        let server = ServerCore::new(addr)?;

        Ok(Lobby {
            server,
            id: Arc::new(Mutex::new(id)),
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
