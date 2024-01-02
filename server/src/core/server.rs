use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use anyhow::{anyhow, Result};

use super::lobby::LobbyId;
use super::request_handlers::{
    ConnectRequest, CreateLobbyRequest, DeleteLobbyRequest, DisconnectRequest, GetLobbiesRequest,
    InvalidRequest, PingRequest,
};
use super::{BoolMutex, RequestHandler, RequestQueueItem, ServerCore};
use network::{SendRecv, Type};

pub type LobbyVec = Arc<Mutex<Vec<(u16, SocketAddr, BoolMutex, JoinHandle<()>)>>>;

pub struct Server {
    server: ServerCore,
    lobby_id: LobbyId,
    lobbies: LobbyVec,
}

impl RequestHandler for Server {
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
            Type::Connect => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(ConnectRequest::new(
                    stream,
                    buf,
                    self.server.db_pool.clone(),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::Disconnect => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(DisconnectRequest::new(
                    stream,
                    buf,
                    self.server.db_pool.clone(),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::CreateLobby => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(CreateLobbyRequest::new(
                    stream,
                    buf,
                    Arc::clone(&self.lobby_id),
                    Arc::clone(&self.lobbies),
                    self.server.db_pool.clone(),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::DeleteLobby => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(DeleteLobbyRequest::new(
                    stream,
                    buf,
                    Arc::clone(&self.lobbies),
                    self.server.db_pool.clone(),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            Type::GetLobbies => match bincode::deserialize(&buf) {
                Ok(buf) => Box::new(GetLobbiesRequest::new(
                    stream,
                    buf,
                    Arc::clone(&self.lobbies),
                    self.server.db_pool.clone(),
                )),
                Err(_) => Box::new(InvalidRequest::new(stream, "invalid data")),
            },
            _ => Box::new(InvalidRequest::new(stream, "invalid request")),
        })
    }
}

impl Server {
    pub fn new(addr: &str) -> Result<Server> {
        let server = ServerCore::new(addr)?;

        // create sighandler that sets it to false to signal that the server should stop
        {
            let running = Arc::clone(&server.running);

            ctrlc::set_handler(move || {
                println!("interrupt received, terminating...");

                let mut running = running.lock().unwrap();

                *running = false;
            })?;
        }

        Ok(Server {
            server,
            lobby_id: Arc::new(Mutex::new(0)),
            lobbies: Arc::new(Mutex::new(vec![])),
        })
    }

    pub fn start(&self) -> Result<()> {
        self.server.start(self)?;

        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let mut lobbies = self.lobbies.lock().unwrap();
        while let Some((id, _, running, handle)) = lobbies.pop() {
            {
                let mut running = running.lock().unwrap();
                *running = false;
            }

            match handle.join() {
                Ok(_) => println!("lobby {id} shut down"),
                Err(e) => println!("lobby thread panicked: {e:?}"),
            }
        }

        println!("server shut down")
    }
}
