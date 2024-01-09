use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{request, SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    game::GameState,
    request_handlers::error_check,
    types::{Game, UserType, UsersVec},
};

use super::{error::ServerError, Request};

pub struct StartGameRequest {
    stream: TcpStream,
    user_id: u32,
    users: UsersVec,
    game: Game,
    db_pool: Pool<SqliteConnectionManager>,
}

impl StartGameRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        users: UsersVec,
        game: Game,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> StartGameRequest {
        StartGameRequest {
            stream,
            user_id,
            users,
            game,
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

        let mut game = self.game.lock().unwrap();

        if game.is_some() {
            return Err(ServerError::API {
                message: "game is already started".to_string(),
            });
        }

        let users = self.users.lock().unwrap();

        let (mut angel, mut devil) = (0, 0);
        users.iter().for_each(|u| match u.user_type {
            UserType::Host => devil = u.id,
            UserType::Player => angel = u.id,
            _ => {}
        });

        if devil != self.user_id {
            return Err(ServerError::API {
                message: "you are not the host".to_string(),
            });
        }

        let game_state = GameState::new(angel, devil);

        for user in users.iter() {
            request(user.addr, Type::GameStarted, &game_state)?;
        }

        *game = Some(game_state);

        Ok(())
    }
}

impl Request for StartGameRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
