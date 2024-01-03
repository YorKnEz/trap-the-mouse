use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{request, SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    request_handlers::error_check,
    types::{UserType, UsersVec},
};

use super::{error::ServerError, Request};

pub struct MakeHostRequest {
    stream: TcpStream,
    user_id: u32,
    new_host_id: u32,
    users: UsersVec,
    db_pool: Pool<SqliteConnectionManager>,
}

impl MakeHostRequest {
    pub fn new(
        stream: TcpStream,
        data: (u32, u32),
        users: UsersVec,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> MakeHostRequest {
        MakeHostRequest {
            stream,
            user_id: data.0,
            new_host_id: data.1,
            users,
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

        let mut users = self.users.lock().unwrap();

        let new_host_role = {
            let host = match users.iter().find(|user| user.id == self.user_id) {
                Some(index) => index,
                None => {
                    return Err(ServerError::API {
                        message: "you are not connected to this lobby".to_string(),
                    })
                }
            };

            if host.user_type != UserType::Host {
                return Err(ServerError::API {
                    message: "you are not the host".to_string(),
                });
            }

            let user = match users.iter().find(|user| user.id == self.new_host_id) {
                Some(index) => index,
                None => {
                    return Err(ServerError::API {
                        message: "user is not connected to this lobby".to_string(),
                    })
                }
            };

            user.user_type
        };

        for user in users.iter_mut() {
            if user.id == self.user_id {
                user.user_type = new_host_role;
            }

            if user.id == self.new_host_id {
                user.user_type = UserType::Host;
            }

            request(
                user.addr,
                Type::BecameRole,
                &(self.new_host_id, UserType::Host),
            )?;
            request(user.addr, Type::BecameRole, &(self.user_id, new_host_role))?;
        }

        Ok(())
    }
}

impl Request for MakeHostRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
