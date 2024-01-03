use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::SendRecv;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::db::UserOps;
use crate::core::request_handlers::error_check;
use crate::core::types::{LobbyAddr, LobbyVec};

use super::error::ServerError;
use super::Request;

pub struct GetLobbiesRequest {
    stream: TcpStream,
    user_id: u32,
    start: u32,
    offset: u32,
    lobbies: LobbyVec,
    db_pool: Pool<SqliteConnectionManager>,
}

impl GetLobbiesRequest {
    pub fn new(
        stream: TcpStream,
        data: (u32, u32, u32),
        lobbies: LobbyVec,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> GetLobbiesRequest {
        GetLobbiesRequest {
            stream,
            user_id: data.0,
            start: data.1,
            offset: data.2,
            lobbies,
            db_pool,
        }
    }

    fn handler(&self) -> Result<Vec<LobbyAddr>, ServerError> {
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

        if self.offset > 10 {
            return Err(ServerError::API {
                message: "offset can be at most 10".to_string(),
            });
        }

        if self.start > self.start.wrapping_add(self.offset) {
            return Err(ServerError::API {
                message: "invalid range".to_string(),
            });
        }

        let lobbies = { self.lobbies.lock().unwrap() };

        let start = (self.start as usize).min(lobbies.len());
        let end = (start + self.offset as usize).min(lobbies.len());

        Ok((&lobbies[start..end])
            .iter()
            .map(|item| LobbyAddr {
                id: item.0,
                addr: item.1,
            })
            .collect())
    }
}

impl Request for GetLobbiesRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
