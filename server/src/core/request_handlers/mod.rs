mod lobby;
mod server;

mod exit;
mod invalid;
mod ping;

mod error;

use anyhow::Result;

use serde::Serialize;

use network::{request, Type};

pub use exit::ExitRequest;
pub use invalid::InvalidRequest;
pub use ping::PingRequest;

pub use lobby::*;
pub use server::*;

use error::ServerError;

use super::types::{UserInfo, UserInfoShort, UserType};

pub trait Request {
    fn execute(&mut self) -> Result<()>;
}

pub fn error_check<T: Serialize>(res: Result<T, ServerError>) -> Result<(Type, Vec<u8>)> {
    Ok(match res {
        Ok(res) => (Type::Success, bincode::serialize(&res)?),
        Err(e) => {
            println!("request error: {e:?}");
            match e {
                ServerError::Api { message } => (Type::Error, bincode::serialize(&message)?),
                ServerError::ApiNotConnected => {
                    (Type::Error, bincode::serialize(&"you are not connected")?)
                }
                _ => (Type::Error, bincode::serialize("internal error")?),
            }
        }
    })
}

pub fn dispatch<S: Serialize, F: Fn(&mut UserInfo)>(
    users: &mut Vec<UserInfo>,
    events: Vec<(Type, &S)>,
    cb: F,
) -> Result<(), ServerError> {
    let mut removed_users = vec![];
    let mut replace_host = false;
    let mut new_host = None;

    for user in users.iter_mut() {
        cb(user);

        for event in &events {
            match request(user.addr, event.0, event.1) {
                // cache a new host in case the host left
                Ok(()) => {
                    if new_host.is_none() {
                        new_host = Some(UserInfoShort::from(user));
                        new_host.as_mut().unwrap().user_type = UserType::Host;
                    }
                }
                Err(_) => {
                    // if the host lost connection, use the new found host
                    if user.user_type == UserType::Host {
                        replace_host = true;
                    }

                    removed_users.push(user.id);
                    break;
                }
            }
        }
    }

    // if no new host has been found, it means the lobby's empty
    if replace_host && new_host.is_none() {
        return Err(ServerError::InternalShutDown);
    }

    // announce that some players lost connection, if this request fails it doesn't matter
    // because subsequent requests should remove those players as well
    let mut i = 0;
    users.retain_mut(|user| {
        if i < removed_users.len() && removed_users[i] == user.id {
            i += 1;
            return false;
        }

        for id in &removed_users {
            if let Ok(()) = request(user.addr, Type::PlayerLeft, &id) {}
        }

        // replace the host
        if replace_host {
            if user.id == new_host.as_ref().unwrap().id {
                user.user_type = UserType::Host;
            }
            if let Ok(()) = request(user.addr, Type::PlayerUpdated, new_host.as_ref().unwrap()) {}
        }

        true
    });

    Ok(())
}
