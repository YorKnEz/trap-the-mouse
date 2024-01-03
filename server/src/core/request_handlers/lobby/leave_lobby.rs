use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{request, SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    request_handlers::error_check, types::{UsersVec, UserType},
};

use super::{error::ServerError, Request};

pub struct LeaveLobbyRequest {
    stream: TcpStream,
    user_id: u32,
    users: UsersVec,
    db_pool: Pool<SqliteConnectionManager>,
}

impl LeaveLobbyRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        users: UsersVec,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> LeaveLobbyRequest {
        LeaveLobbyRequest {
            stream,
            user_id,
            users,
            db_pool,
        }
    }

    fn handler(&self) -> Result<(), ServerError> {
        let conn = self.db_pool.get()?;

        let db_user = match conn.is_connected(self.user_id) {
            Ok(Some(db_user)) => db_user,
            Ok(None) => return Err(ServerError::APINotConnected),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(ServerError::API {
                    message: "invalid id".to_string(),
                })
            }
            Err(e) => return Err(ServerError::InternalRusqlite(e)),
        };

        let mut users = self.users.lock().unwrap();

        let addr = db_user.addr.parse()?;

        let index = match users
            .iter()
            .position(|user| (*user).name == db_user.name && (*user).addr == addr)
        {
            Some(index) => index,
            None => {
                return Err(ServerError::API {
                    message: "you are not connected to this lobby".to_string(),
                })
            }
        };

        let user = users.remove(index);
        let mut find_host = if user.user_type == UserType::Host {
            true
        } else {
            false
        };

        for user in users.iter_mut() {
            if find_host {
                user.user_type = UserType::Host;
                request(user.addr, Type::BecameHost, &())?;
                find_host = false;
            }

            request(user.addr, Type::PlayerLeft, &db_user.id)?;
        }

        Ok(())
    }
}

impl Request for LeaveLobbyRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
