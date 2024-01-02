use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::SendRecv;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{db::UserOps, request_handlers::error_check, LobbyVec};

use super::{error::ServerError, Request};

pub struct DeleteLobbyRequest {
    stream: TcpStream,
    user_id: u32,
    lobby_id: u16,
    lobbies: LobbyVec,
    db_pool: Pool<SqliteConnectionManager>,
}

impl DeleteLobbyRequest {
    pub fn new(
        stream: TcpStream,
        data: (u32, u16),
        lobbies: LobbyVec,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> DeleteLobbyRequest {
        DeleteLobbyRequest {
            stream,
            user_id: data.0,
            lobby_id: data.1,
            lobbies,
            db_pool,
        }
    }

    fn handler(&self) -> Result<(), ServerError> {
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

        let mut lobbies = self.lobbies.lock().unwrap();

        let index = match lobbies
            .iter()
            .position(|(id, _, _, _)| *id == self.lobby_id)
        {
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
