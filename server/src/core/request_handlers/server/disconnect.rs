use std::net::TcpStream;

use anyhow::{anyhow, Result};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use network::SendRecv;

use crate::core::{db::UserOps, request_handlers::error_check};

use super::{error::ServerError, Request};

pub struct DisconnectRequest {
    stream: TcpStream,
    user_id: u32,
    db_pool: Pool<SqliteConnectionManager>,
}

impl DisconnectRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> DisconnectRequest {
        DisconnectRequest {
            stream,
            user_id,
            db_pool,
        }
    }

    fn handler(&self) -> Result<(), ServerError> {
        let conn = self.db_pool.get()?;

        let db_user = match conn.is_connected(self.user_id) {
            Ok(Some(db_user)) => db_user,
            Ok(None) => return Err(ServerError::ApiNotConnected),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(ServerError::Api {
                    message: "invalid id".to_string(),
                })
            }
            Err(e) => return Err(ServerError::InternalRusqlite(e)),
        };

        conn.toggle_connected(db_user.id)?;

        Ok(())
    }
}

impl Request for DisconnectRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
