use std::{
    net::{SocketAddr, TcpStream},
    sync::Arc,
    thread,
};

use anyhow::{anyhow, Result};
use network::SendRecv;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    lobby::{Lobby, LobbyId},
    request_handlers::error_check,
    LobbyVec,
};

use super::{error::ServerError, Request};

pub struct CreateLobbyRequest {
    stream: TcpStream,
    user_id: u32,
    lobby_id: LobbyId,
    lobbies: LobbyVec,
    db_pool: Pool<SqliteConnectionManager>,
}

impl CreateLobbyRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        lobby_id: LobbyId,
        lobbies: LobbyVec,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> CreateLobbyRequest {
        CreateLobbyRequest {
            stream,
            user_id,
            lobby_id,
            lobbies,
            db_pool,
        }
    }

    fn handler(&self) -> Result<(u16, SocketAddr), ServerError> {
        let conn = self.db_pool.get()?;

        let _ = match conn.is_connected(self.user_id) {
            Ok(Some(db_user)) => db_user,
            Ok(None) => return Err(ServerError::APINotConnected),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(ServerError::API {
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
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
