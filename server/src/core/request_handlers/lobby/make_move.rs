use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    game::GameUpdate,
    request_handlers::{dispatch, error_check},
    types::{BoolMutex, Game, UsersVec},
};

use super::{error::ServerError, Request};

pub struct MakeMoveRequest {
    stream: TcpStream,
    user_id: u32,
    user_move: (i32, i32),
    users: UsersVec,
    game: Game,
    running: BoolMutex,
    db_pool: Pool<SqliteConnectionManager>,
}

impl MakeMoveRequest {
    pub fn new(
        stream: TcpStream,
        data: (u32, i32, i32),
        users: UsersVec,
        game: Game,
        running: BoolMutex,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> MakeMoveRequest {
        MakeMoveRequest {
            stream,
            user_id: data.0,
            user_move: (data.1, data.2),
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

        let mut game_state = self.game.lock().unwrap();

        if game_state.is_none() {
            return Err(ServerError::Api {
                message: "game is not started yet".to_string(),
            });
        }

        let game = game_state.as_mut().unwrap();

        if self.user_id != game.angel && self.user_id != game.devil {
            return Err(ServerError::Api {
                message: "you are not playing".to_string(),
            });
        }

        // devil player move
        let user_move = if game.turn {
            if self.user_id != game.devil {
                return Err(ServerError::Api {
                    message: "it's not your turn".to_string(),
                });
            }

            if game.grid[self.user_move.0 as usize][self.user_move.1 as usize]
                || game.angel_pos == self.user_move
            {
                return Err(ServerError::Api {
                    message: "invalid move".to_string(),
                });
            }

            game.grid[self.user_move.0 as usize][self.user_move.1 as usize] = true;

            self.user_move
        }
        // angel player move
        else {
            if self.user_id != game.angel {
                return Err(ServerError::Api {
                    message: "it's not your turn".to_string(),
                });
            }

            if !game.valid_angel_move(self.user_move) {
                return Err(ServerError::Api {
                    message: "invalid move".to_string(),
                });
            }

            game.angel_pos = self.user_move;

            self.user_move
        };

        let path = game.find_path();

        let update = GameUpdate {
            win: (path.is_none(), game.angel_won()),
            turn: game.turn,
            user_move,
        };

        let mut users = self.users.lock().unwrap();

        if let Err(ServerError::InternalShutDown) =
            dispatch(&mut users, vec![(Type::GameUpdated, &update)], |_| {})
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        game.turn = !game.turn;

        if update.win.0 || update.win.1 {
            *game_state = None;
            return Ok(());
        }

        // angel computer move
        if !game.turn && game.angel == 0 {
            let update = if let Some(next_move) = path {
                game.angel_pos = next_move;

                GameUpdate {
                    win: (false, game.angel_won()),
                    turn: game.turn,
                    user_move: game.angel_pos,
                }
            } else {
                GameUpdate {
                    win: (true, false),
                    turn: game.turn,
                    user_move: game.angel_pos,
                }
            };

            if let Err(ServerError::InternalShutDown) =
                dispatch(&mut users, vec![(Type::GameUpdated, &update)], |_| {})
            {
                let mut running = self.running.lock().unwrap();
                *running = false;
            }

            game.turn = !game.turn;

            if update.win.0 || update.win.1 {
                *game_state = None;
            }
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
