use std::net::TcpStream;

use anyhow::{anyhow, Result};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use network::SendRecv;

use crate::core::{db::UserOps, request_handlers::error_check};

use super::{error::ServerError, Request};

pub struct ChangeNameRequest {
    stream: TcpStream,
    user_id: u32,
    name: String,
    db_pool: Pool<SqliteConnectionManager>,
}

impl ChangeNameRequest {
    pub fn new(
        stream: TcpStream,
        data: (u32, String),
        db_pool: Pool<SqliteConnectionManager>,
    ) -> ChangeNameRequest {
        ChangeNameRequest {
            stream,
            user_id: data.0,
            name: data.1,
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

        if !(2 <= self.name.len() && self.name.len() < 256) {
            return Err(ServerError::API {
                message: "username must be between 2 and 255 characters".to_string(),
            });
        }

        let _ = match conn.change_user_name(self.user_id, &self.name) {
            Ok(_) => {}
            Err(e) => return Err(ServerError::InternalRusqlite(e)),
        };

        Ok(())
    }
}

impl Request for ChangeNameRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
