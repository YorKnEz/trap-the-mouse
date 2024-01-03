use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{request, SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    request_handlers::error_check,
    types::{BoolMutex, UserType, UsersVec},
};

use super::{error::ServerError, Request};

pub struct LeaveLobbyRequest {
    stream: TcpStream,
    user_id: u32,
    users: UsersVec,
    running: BoolMutex,
    db_pool: Pool<SqliteConnectionManager>,
}

impl LeaveLobbyRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        users: UsersVec,
        running: BoolMutex,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> LeaveLobbyRequest {
        LeaveLobbyRequest {
            stream,
            user_id,
            users,
            running,
            db_pool,
        }
    }

    fn handler(&self) -> Result<bool, ServerError> {
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

        let find_host = user.user_type == UserType::Host;
        let mut host_id = 0;

        for user in users.iter_mut() {
            if find_host {
                if host_id == 0 {
                    user.user_type = UserType::Host;
                    host_id = user.id;
                }

                request(user.addr, Type::BecameHost, &host_id)?;
            }

            request(user.addr, Type::PlayerLeft, &db_user.id)?;
        }

        // close lobby if no new host has been found (i.e. lobby is empty)
        if find_host && host_id == 0 {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        Ok(find_host && host_id == 0)
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
