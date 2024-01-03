use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::SendRecv;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    request_handlers::error_check,
    types::{BoolMutex, UserType, UsersVec},
};

use super::{error::ServerError, Request};

pub struct CloseLobbyRequest {
    stream: TcpStream,
    user_id: u32,
    users: UsersVec,
    running: BoolMutex,
    db_pool: Pool<SqliteConnectionManager>,
}

impl CloseLobbyRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        users: UsersVec,
        running: BoolMutex,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> CloseLobbyRequest {
        CloseLobbyRequest {
            stream,
            user_id,
            users,
            running,
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

        match self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.id == self.user_id)
        {
            Some(user) => {
                if user.user_type != UserType::Host {
                    return Err(ServerError::API {
                        message: "you are not the host".to_string(),
                    });
                }

                let mut running = self.running.lock().unwrap();
                *running = false;
            }
            None => {
                return Err(ServerError::API {
                    message: "you are not connected to this lobby".to_string(),
                })
            }
        }

        Ok(())
    }
}

impl Request for CloseLobbyRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
