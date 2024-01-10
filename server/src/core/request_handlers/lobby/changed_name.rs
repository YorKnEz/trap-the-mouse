use std::net::TcpStream;

use anyhow::{anyhow, Result};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use network::{SendRecv, Type};

use crate::core::{
    db::UserOps,
    request_handlers::{dispatch, error_check},
    types::{BoolMutex, UserInfoShort, UsersVec},
};

use super::{error::ServerError, Request};

pub struct ChangedNameRequest {
    stream: TcpStream,
    user_id: u32,
    users: UsersVec,
    running: BoolMutex,
    db_pool: Pool<SqliteConnectionManager>,
}

impl ChangedNameRequest {
    pub fn new(
        stream: TcpStream,
        user_id: u32,
        users: UsersVec,
        running: BoolMutex,
        db_pool: Pool<SqliteConnectionManager>,
    ) -> ChangedNameRequest {
        ChangedNameRequest {
            stream,
            user_id,
            users,
            running,
            db_pool,
        }
    }

    fn handler(&self) -> Result<(), ServerError> {
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

        let mut new_user = match users.iter().find(|user| user.id == self.user_id) {
            Some(user) => user.clone(),
            None => {
                return Err(ServerError::Api {
                    message: "you are not connected to this lobby".to_string(),
                })
            }
        };

        new_user.name = db_user.name.clone();
        let new_user = UserInfoShort::from(&new_user);

        if let Err(ServerError::InternalShutDown) = dispatch(
            &mut users,
            vec![(Type::PlayerUpdated, &new_user)],
            |user| {
                if user.id == self.user_id {
                    user.name = db_user.name.clone();
                }
            },
        ) {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        Ok(())
    }
}

impl Request for ChangedNameRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
