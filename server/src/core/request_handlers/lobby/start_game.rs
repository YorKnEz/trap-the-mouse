use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    game::GameState,
    request_handlers::{dispatch, error_check},
    types::{BoolMutex, Game, UserType, UsersVec},
};

use super::{error::ServerError, Request};

pub struct StartGameRequest {
    stream: TcpStream,
    user_id: u32,
    users: UsersVec,
    game: Game,
    running: BoolMutex,
    db_pool: Pool<SqliteConnectionManager>,
}

impl StartGameRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        users: UsersVec,
        game: Game,
        running: BoolMutex,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> StartGameRequest {
        StartGameRequest {
            stream,
            user_id,
            users,
            game,
            running,
            db_pool,
        }
    }

    fn handler(&self) -> Result<(), ServerError> {
        let conn = self.db_pool.get()?;

        let _ = match conn.is_connected(self.user_id) {
            Ok(Some(db_user)) => db_user,
            Ok(None) => return Err(ServerError::ApiNotConnected),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(ServerError::Api {
                    message: "invalid id".to_string(),
                })
            }
            Err(e) => return Err(ServerError::InternalRusqlite(e)),
        };

        let mut game = self.game.lock().unwrap();

        if game.is_some() {
            return Err(ServerError::Api {
                message: "game is already started".to_string(),
            });
        }

        let mut users = self.users.lock().unwrap();

        let (mut angel, mut devil) = (0, 0);
        users.iter().for_each(|u| match u.user_type {
            UserType::Host => devil = u.id,
            UserType::Player => angel = u.id,
            _ => {}
        });

        if devil != self.user_id {
            return Err(ServerError::Api {
                message: "you are not the host".to_string(),
            });
        }

        let game_state = GameState::new(angel, devil);

        if let Err(ServerError::InternalShutDown) = dispatch(
            &mut users,
            vec![(Type::GameStarted, &game_state)],
            |_| {},
        ) {
            let mut running = self.running.lock().unwrap();
            *running = false;
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
