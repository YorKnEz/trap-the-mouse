use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::SendRecv;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{db::UserOps, request_handlers::error_check, LobbyVec};

use super::{error::ServerError, Request};

pub struct DeleteLobbyRequest {
    stream: TcpStream,
    name: String,
    id: u16,
    lobbies: LobbyVec,
    db_pool: Pool<SqliteConnectionManager>,
}

impl DeleteLobbyRequest {
    pub fn new(
        stream: TcpStream,
        data: (String, u16),
        lobbies: LobbyVec,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> DeleteLobbyRequest {
        DeleteLobbyRequest {
            stream,
            name: data.0,
            id: data.1,
            lobbies,
            db_pool,
        }
    }

    fn handler(&self) -> Result<(), ServerError> {
        let conn = self.db_pool.get()?;

        let _ = match conn.get_user_by_key(&self.name, &self.stream.local_addr()?.to_string()) {
            Ok(db_user) => db_user,
            Err(_) => return Err(ServerError::APINotConnected),
        };

        let mut lobbies = self.lobbies.lock().unwrap();

        let index = match lobbies.iter().position(|(id, _, _, _)| *id == self.id) {
            Some(index) => index,
            None => {
                return Err(ServerError::API {
                    message: "lobby not found".to_string(),
                })
            }
        };

        let (id, _, running, handle) = lobbies.remove(index);

        {
            let mut running = running.lock().unwrap();
            *running = false;
        }

        match handle.join() {
            Ok(_) => println!("lobby {id} shut down"),
            Err(e) => println!("lobby thread panicked: {e:?}"),
        }

        Ok(())
    }
}

impl Request for DeleteLobbyRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
