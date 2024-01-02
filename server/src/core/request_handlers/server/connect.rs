use std::net::{SocketAddr, TcpStream};

use anyhow::{anyhow, Result};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use network::SendRecv;

use crate::core::{db::UserOps, request_handlers::error_check};

use super::{error::ServerError, Request};

pub struct ConnectRequest {
    stream: TcpStream,
    name: String,
    addr: SocketAddr,
    db_pool: Pool<SqliteConnectionManager>,
}

impl ConnectRequest {
    pub fn new(
        stream: TcpStream,
        data: (String, SocketAddr),
        db_pool: Pool<SqliteConnectionManager>,
    ) -> ConnectRequest {
        ConnectRequest {
            stream,
            name: data.0,
            addr: data.1,
            db_pool,
        }
    }

    fn handler(&self) -> Result<u32, ServerError> {
        let conn = self.db_pool.get()?;

        match conn.add_user(&self.name, &self.addr.to_string()) {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(e, Some(m))) => {
                if e.code != rusqlite::ErrorCode::ConstraintViolation {
                    return Err(ServerError::InternalRusqlite(
                        rusqlite::Error::SqliteFailure(e, Some(m)),
                    ));
                }

                // user is already added to db, just continue the handler
            }
            Err(e) => return Err(ServerError::InternalRusqlite(e)),
        }

        let db_user = match conn.get_user_by_key(&self.name, &self.addr.to_string()) {
            Ok(db_user) => db_user,
            Err(e) => return Err(ServerError::InternalRusqlite(e)),
        };

        if db_user.connected == 0 {
            conn.toggle_connected(db_user.id)?;
        }

        Ok(db_user.id)
    }
}

impl Request for ConnectRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
