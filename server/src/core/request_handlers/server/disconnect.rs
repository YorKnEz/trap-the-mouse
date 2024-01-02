use std::net::TcpStream;

use anyhow::{anyhow, Result};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use network::SendRecv;

use crate::core::{db::UserOps, request_handlers::error_check};

use super::{error::ServerError, Request};

pub struct DisconnectRequest {
    stream: TcpStream,
    name: String,
    db_pool: Pool<SqliteConnectionManager>,
}

impl DisconnectRequest {
    pub fn new(
        stream: TcpStream,
        name: String,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> DisconnectRequest {
        DisconnectRequest {
            stream,
            name,
            db_pool,
        }
    }

    fn handler(&self) -> Result<(), ServerError> {
        let conn = self.db_pool.get()?;

        conn.remove_user(&self.name, &self.stream.local_addr()?.to_string())?;

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
