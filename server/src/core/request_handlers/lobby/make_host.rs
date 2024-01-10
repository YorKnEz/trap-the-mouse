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

pub struct MakeHostRequest {
    stream: TcpStream,
    user_id: u32,
    new_host_id: u32,
    users: UsersVec,
    game: Game,
    running: BoolMutex,
    db_pool: Pool<SqliteConnectionManager>,
}

impl MakeHostRequest {
    pub fn new(
        stream: TcpStream,
        data: (u32, u32),
        users: UsersVec,
        game: Game,
        running: BoolMutex,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> MakeHostRequest {
        MakeHostRequest {
            stream,
            user_id: data.0,
            new_host_id: data.1,
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

        let (mut new_host, mut old_host) = {
            let host = match users.iter().find(|user| user.id == self.user_id) {
                Some(index) => index,
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

            if host.user_type != UserType::Host {
                return Err(ServerError::Api {
                    message: "you are not the host".to_string(),
                });
            }

            let user = match users.iter().find(|user| user.id == self.new_host_id) {
                Some(index) => index,
                None => {
                    return Err(ServerError::Api {
                        message: "user is not connected to this lobby".to_string(),
                    })
                }
            };

            (UserInfoShort::from(user), UserInfoShort::from(host))
        };

        old_host.user_type = new_host.user_type;
        new_host.user_type = UserType::Host;

        if let Err(ServerError::InternalShutDown) = dispatch(
            &mut users,
            vec![
                (Type::PlayerUpdated, &old_host),
                (Type::PlayerUpdated, &new_host),
            ],
            |user| {
                if user.id == old_host.id {
                    user.user_type = old_host.user_type;
                }

                if user.id == new_host.id {
                    user.user_type = new_host.user_type;
                }
            },
        ) {
            let mut running = self.running.lock().unwrap();
            *running = false;
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
