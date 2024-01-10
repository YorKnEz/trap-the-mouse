use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::core::{
    db::UserOps,
    request_handlers::{dispatch, error_check},
    types::{BoolMutex, Game, UserInfoShort, UserType, UsersVec},
};

use super::{error::ServerError, Request};

pub struct BecomeRoleRequest {
    stream: TcpStream,
    user_id: u32,
    new_role: UserType,
    users: UsersVec,
    game: Game,
    running: BoolMutex,
    db_pool: Pool<SqliteConnectionManager>,
}

impl BecomeRoleRequest {
    pub fn new(
        stream: TcpStream,
        data: (u32, UserType),
        users: UsersVec,
        game: Game,
        running: BoolMutex,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> BecomeRoleRequest {
        BecomeRoleRequest {
            stream,
            user_id: data.0,
            new_role: data.1,
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

        let mut users = self.users.lock().unwrap();

        let mut new_user = match users.iter().find(|user| user.id == self.user_id) {
            Some(user) => UserInfoShort::from(user),
            None => {
                return Err(ServerError::Api {
                    message: "you are not connected to this lobby".to_string(),
                })
            }
        };

        {
            if self.game.lock().unwrap().is_some() {
                return Err(ServerError::Api {
                    message: "cannot change roles while a game is going on".to_string(),
                });
            }
        }

        if new_user.user_type == UserType::Host {
            return Err(ServerError::Api {
                message: "you need to make someone else host".to_string(),
            });
        }

        if new_user.user_type == self.new_role {
            return Err(ServerError::Api {
                message: "you already have this role".to_string(),
            });
        }

        match self.new_role {
            UserType::Host => {
                return Err(ServerError::Api {
                    message: "you cannot become host".to_string(),
                })
            }
            UserType::Player => {
                if users.iter().any(|user| user.user_type == UserType::Player) {
                    return Err(ServerError::Api {
                        message: "cannot become player".to_string(),
                    });
                }
            }
            UserType::Spectator => {}
        }

        new_user.user_type = self.new_role;

        if let Err(ServerError::InternalShutDown) = dispatch(
            &mut users,
            vec![(Type::PlayerUpdated, &new_user)],
            |user| {
                if user.id == new_user.id {
                    user.user_type = new_user.user_type;
                }
            },
        ) {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        Ok(())
    }
}

impl Request for BecomeRoleRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
