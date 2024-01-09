use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{request, SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    request_handlers::error_check,
    types::{LobbyName, LobbyState, UserInfo, UserInfoShort, UserType, UsersVec, Game},
};

use super::{error::ServerError, Request};

pub struct JoinLobbyRequest {
    stream: TcpStream,
    user_id: u32,
    lobby_name: LobbyName,
    users: UsersVec,
    game: Game,
    db_pool: Pool<SqliteConnectionManager>,
}

impl JoinLobbyRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        lobby_name: LobbyName,
        users: UsersVec,
        game: Game,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> JoinLobbyRequest {
        JoinLobbyRequest {
            stream,
            user_id,
            lobby_name,
            users,
            game,
            db_pool,
        }
    }

    fn handler(&self) -> Result<LobbyState, ServerError> {
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

        match users.iter().position(|user| user.id == self.user_id) {
            Some(_) => {
                return Err(ServerError::API {
                    message: "you are already connected to this lobby".to_string(),
                })
            }
            None => {}
        };

        let new_user: UserInfo = UserInfo {
            id: db_user.id,
            user_type: match users.len() {
                0 => UserType::Host,
                1 => UserType::Player,
                _ => UserType::Spectator,
            },
            name: db_user.name.clone(),
            addr: db_user.addr.parse()?,
        };

        let new_user_short = UserInfoShort::from(&new_user);

        for user in users.iter() {
            request(user.addr, Type::PlayerJoined, &new_user_short)?;
        }

        users.push(new_user);

        Ok(LobbyState {
            name: { self.lobby_name.lock().unwrap().clone() },
            users: users.iter().map(|i| UserInfoShort::from(i)).collect(),
            game: { self.game.lock().unwrap().clone() }
        })
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
