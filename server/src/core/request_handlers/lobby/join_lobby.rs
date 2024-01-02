use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{request, SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    lobby::{UserInfo, UserType, UsersVec}, request_handlers::error_check,
};

use super::{error::ServerError, Request};

pub struct JoinLobbyRequest {
    stream: TcpStream,
    name: String,
    users: UsersVec,
    db_pool: Pool<SqliteConnectionManager>,
}

impl JoinLobbyRequest {
    pub fn new(
        stream: TcpStream,
        name: String,
        users: UsersVec,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> JoinLobbyRequest {
        JoinLobbyRequest {
            stream,
            name,
            users,
            db_pool,
        }
    }

    fn handler(&self) -> Result<(), ServerError> {
        let conn = self.db_pool.get()?;

        let db_user = match conn.get_user_by_key(&self.name, &self.stream.local_addr()?.to_string())
        {
            Ok(db_user) => db_user,
            Err(_) => return Err(ServerError::APINotConnected),
        };

        let mut users = self.users.lock().unwrap();

        let user: UserInfo = match users.len() {
            0 => (
                UserType::Host,
                db_user.addr.parse()?,
                db_user.notify_addr.parse()?,
            ),
            1 => (
                UserType::Player,
                db_user.addr.parse()?,
                db_user.notify_addr.parse()?,
            ),
            _ => (
                UserType::Spectator,
                db_user.addr.parse()?,
                db_user.notify_addr.parse()?,
            ),
        };

        for (_, _, notify_addr) in users.iter() {
            request(notify_addr, Type::PlayerJoined, &self.name)?;
        }

        users.push(user);

        Ok(())
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
