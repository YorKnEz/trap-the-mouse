use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{request, SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    game::GameUpdate,
    request_handlers::error_check,
    types::{Game, UsersVec},
};

use super::{error::ServerError, Request};

pub struct MakeMoveRequest {
    stream: TcpStream,
    user_id: u32,
    user_move: (i32, i32),
    users: UsersVec,
    game: Game,
    db_pool: Pool<SqliteConnectionManager>,
}

impl MakeMoveRequest {
    pub fn new(
        stream: TcpStream,
        data: (u32, i32, i32),
        users: UsersVec,
        game: Game,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> MakeMoveRequest {
        MakeMoveRequest {
            stream,
            user_id: data.0,
            user_move: (data.1, data.2),
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

        let mut game_state = self.game.lock().unwrap();

        if game_state.is_none() {
            return Err(ServerError::API {
                message: "game is not started yet".to_string(),
            });
        }

        let game = game_state.as_mut().unwrap();

        if self.user_id != game.angel && self.user_id != game.devil {
            return Err(ServerError::API {
                message: "you are not playing".to_string(),
            });
        }

        // devil player move
        let user_move = if game.turn {
            if self.user_id != game.devil {
                return Err(ServerError::API {
                    message: "it's not your turn".to_string(),
                });
            }

            if game.grid[self.user_move.0 as usize][self.user_move.1 as usize]
                || game.angel_pos == self.user_move
            {
                return Err(ServerError::API {
                    message: "invalid move".to_string(),
                });
            }

            game.grid[self.user_move.0 as usize][self.user_move.1 as usize] = true;

            self.user_move
        }
        // angel player move
        else {
            if self.user_id != game.angel {
                return Err(ServerError::API {
                    message: "it's not your turn".to_string(),
                });
            }

            if !game.valid_angel_move(self.user_move) {
                return Err(ServerError::API {
                    message: "invalid move".to_string(),
                });
            }

            game.angel_pos = self.user_move;

            self.user_move
        };

        let update = GameUpdate {
            win: (game.devil_won(), game.angel_won()),
            turn: game.turn,
            user_move,
        };

        game.turn = !game.turn;

        if update.win.0 || update.win.1 {
            *game_state = None;
        }

        let users = self.users.lock().unwrap();

        for user in users.iter() {
            request(user.addr, Type::GameUpdated, &update)?;
        }

        Ok(())
    }
}

impl Request for MakeMoveRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
