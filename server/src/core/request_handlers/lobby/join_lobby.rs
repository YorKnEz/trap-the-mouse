use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{request, SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    request_handlers::error_check,
    types::{UserInfo, UserInfoShort, UserType, UsersVec},
};

use super::{error::ServerError, Request};

pub struct JoinLobbyRequest {
    stream: TcpStream,
    user_id: u32,
    users: UsersVec,
    db_pool: Pool<SqliteConnectionManager>,
}

impl JoinLobbyRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        users: UsersVec,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> JoinLobbyRequest {
        JoinLobbyRequest {
            stream,
            user_id,
            users,
            db_pool,
        }
    }

    fn handler(&self) -> Result<Vec<UserInfoShort>, ServerError> {
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

        match users
            .iter()
            .position(|user| (*user).name == db_user.name && (*user).addr == addr)
        {
            Some(_) => {
                return Err(ServerError::API {
                    message: "you are already connected to this lobby".to_string(),
                })
            }
            None => {}
        };

        let user: UserInfo = UserInfo {
            id: db_user.id,
            user_type: match users.len() {
                0 => UserType::Host,
                1 => UserType::Player,
                _ => UserType::Spectator,
            },
            name: db_user.name.clone(),
            addr,
        };

        for user in users.iter() {
            request(
                user.addr,
                Type::PlayerJoined,
                &UserInfoShort::from(user),
            )?;
        }

        users.push(user);

        Ok(users
            .iter()
            .map(|i| UserInfoShort::from(i))
            .collect())
    }
}

impl Request for JoinLobbyRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
