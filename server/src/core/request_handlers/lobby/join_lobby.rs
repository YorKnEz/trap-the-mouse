use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    request_handlers::{dispatch, error_check},
    types::{BoolMutex, Game, LobbyName, LobbyState, UserInfo, UserInfoShort, UserType, UsersVec},
};

use super::{error::ServerError, Request};

pub struct JoinLobbyRequest {
    stream: TcpStream,
    user_id: u32,
    lobby_name: LobbyName,
    users: UsersVec,
    game: Game,
    running: BoolMutex,
    db_pool: Pool<SqliteConnectionManager>,
}

impl JoinLobbyRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        lobby_name: LobbyName,
        users: UsersVec,
        game: Game,
        running: BoolMutex,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> JoinLobbyRequest {
        JoinLobbyRequest {
            stream,
            user_id,
            lobby_name,
            users,
            game,
            running,
            db_pool,
        }
    }

    fn handler(&self) -> Result<LobbyState, ServerError> {
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

        let mut users = self.users.lock().unwrap();

        if users.iter().any(|user| user.id == self.user_id) {
            return Err(ServerError::Api {
                message: "you are already connected to this lobby".to_string(),
            });
        }

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

        if !users.is_empty() {
            if let Err(ServerError::InternalShutDown) = dispatch(
                &mut users,
                vec![(Type::PlayerJoined, &new_user_short)],
                |_| {},
            ) {
                let mut running = self.running.lock().unwrap();
                *running = false;
            }
        }

        users.push(new_user);

        Ok(LobbyState {
            name: { self.lobby_name.lock().unwrap().clone() },
            users: users.iter().map(UserInfoShort::from).collect(),
            game: { self.game.lock().unwrap().clone() },
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
