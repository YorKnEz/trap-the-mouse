use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{request, SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    request_handlers::error_check,
    types::{BoolMutex, UserInfoShort, UserType, UsersVec},
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

        let index = match users.iter().position(|user| user.id == self.user_id) {
            Some(index) => index,
            None => {
                return Err(ServerError::API {
                    message: "you are not connected to this lobby".to_string(),
                })
            }
        };

        let user = users.remove(index);

        let find_host = user.user_type == UserType::Host;
        let mut new_host = None;
        let mut old_type;

        for user in users.iter_mut() {
            if find_host {
                if new_host.is_none() {
                    // try making current user host
                    old_type = user.user_type;
                    user.user_type = UserType::Host;
                    new_host = Some(UserInfoShort::from(user));

                    match request(user.addr, Type::PlayerUpdated, new_host.as_ref().unwrap()) {
                        Ok(()) => {}
                        Err(_) => {
                            new_host = None;
                            user.user_type = old_type;
                            continue;
                        }
                    }
                }

                request(user.addr, Type::PlayerUpdated, new_host.as_ref().unwrap())?;
            }

            request(user.addr, Type::PlayerLeft, &db_user.id)?;
        }

        // close lobby if no new host has been found (i.e. lobby is empty)
        if find_host && new_host.is_none() {
            let mut running = self.running.lock().unwrap();
            *running = false;
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
