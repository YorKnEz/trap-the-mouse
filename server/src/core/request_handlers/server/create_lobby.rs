use std::{net::TcpStream, sync::Arc, thread};

use anyhow::{anyhow, Result};
use network::SendRecv;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    lobby::Lobby,
    request_handlers::error_check,
    types::{LobbyAddr, LobbyId, LobbyVec},
};

use super::{error::ServerError, Request};

pub struct CreateLobbyRequest {
    stream: TcpStream,
    user_id: u32,
    name: String,
    lobby_id: LobbyId,
    lobbies: LobbyVec,
    db_pool: Pool<SqliteConnectionManager>,
}

impl CreateLobbyRequest {
    pub fn new(
        stream: TcpStream,
        data: (u32, String),
        lobby_id: LobbyId,
        lobbies: LobbyVec,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> CreateLobbyRequest {
        CreateLobbyRequest {
            stream,
            user_id: data.0,
            name: data.1,
            lobby_id,
            lobbies,
            db_pool,
        }
    }

    fn handler(&self) -> Result<LobbyAddr, ServerError> {
        let conn = self.db_pool.get()?;

        let _ = match conn.is_connected(self.user_id) {
            Ok(Some(db_user)) => db_user,
            Ok(None) => return Err(ServerError::ApiNotConnected),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(ServerError::Api {
                    message: "invalid id".to_string(),
                })
            }
            Err(e) => return Err(ServerError::InternalRusqlite(e)),
        };

        let id = {
            let mut id = self.lobby_id.lock().unwrap();
            let ret = *id;
            *id += 1;
            ret
        };

        // auto assign a name if none is provided
        let lobby_name = if !self.name.is_empty() {
            self.name.clone()
        } else {
            format!("Lobby {id}")
        };

        let lobby = Lobby::new("127.0.0.1:0", id, lobby_name).unwrap();
        let (addr, running) = (lobby.get_addr()?, Arc::clone(&lobby.server.running));

        let handle = thread::spawn(move || {
            lobby.start().unwrap();
        });

        println!("lobby {} started", id);

        {
            let mut lobbies = self.lobbies.lock().unwrap();
            lobbies.push((id, addr, running, handle));
        }

        Ok(LobbyAddr { id, addr })
    }
}

impl Request for CreateLobbyRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
