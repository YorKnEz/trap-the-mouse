use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    request_handlers::{dispatch, error_check},
    types::{BoolMutex, UsersVec},
};

use super::{error::ServerError, Request};

pub struct SendMessageRequest {
    stream: TcpStream,
    user_id: u32,
    message: String,
    users: UsersVec,
    running: BoolMutex,
    db_pool: Pool<SqliteConnectionManager>,
}

impl SendMessageRequest {
    pub fn new(
        stream: TcpStream,
        data: (u32, String),
        users: UsersVec,
        running: BoolMutex,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> SendMessageRequest {
        SendMessageRequest {
            stream,
            user_id: data.0,
            message: data.1,
            users,
            running,
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

        if self.message.is_empty() || self.message.len() > 256 {
            return Err(ServerError::Api {
                message: "message length should be between 1 and 256 characters".to_string(),
            });
        }

        let mut users = self.users.lock().unwrap();

        if let Err(ServerError::InternalShutDown) = dispatch(
            &mut users,
            vec![(Type::Message, &(db_user.name, self.message.clone()))],
            |_| {},
        ) {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        Ok(())
    }
}

impl Request for SendMessageRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
