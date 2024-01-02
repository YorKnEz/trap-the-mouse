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
    notify_addr: String,
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
            notify_addr: data.1.to_string(),
            db_pool,
        }
    }

    fn handler(&self) -> Result<(), ServerError> {
        let conn = self.db_pool.get()?;

        match conn.add_user(
            &self.name,
            &self.stream.local_addr()?.to_string(),
            &self.notify_addr,
        ) {
            Ok(_) => {}
            // user is already connected, this can happen only if the user abruptly disconnected
            Err(rusqlite::Error::SqliteFailure(e, m)) => {
                if let Some(m) = m {
                    if e.code == rusqlite::ErrorCode::ConstraintViolation && m.starts_with("UNIQUE")
                    {
                        return Ok(());
                        // return Err(ServerError::API {
                        //     message: "you are already connected".to_string(),
                        // });
                    }
                }
            }
            Err(e) => return Err(ServerError::InternalRusqlite(e)),
        }

        Ok(())
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
