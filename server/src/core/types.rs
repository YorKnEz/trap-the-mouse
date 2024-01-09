use std::{
    cell::RefCell,
    net::SocketAddr,
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
};

use serde_derive::{Deserialize, Serialize};

use super::{request_handlers::Request, game::GameState};

pub type BoolMutex = Arc<Mutex<bool>>;

pub type HandleVec = RefCell<Vec<JoinHandle<()>>>;

pub type RequestQueue = Arc<(Mutex<Vec<RequestQueueItem>>, Condvar)>;
pub type RequestQueueItem = Box<dyn Request + Send>;

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum UserType {
    Host,
    Player,
    Spectator,
}

#[derive(Clone)]
pub struct UserInfo {
    pub id: u32,
    pub user_type: UserType,
    pub name: String,
    pub addr: SocketAddr,
}

#[derive(Serialize)]
pub struct UserInfoShort {
    pub id: u32,
    pub user_type: UserType,
    pub name: String,
}

impl UserInfoShort {
    pub fn from(user_info: &UserInfo) -> UserInfoShort {
        UserInfoShort {
            id: user_info.id,
            user_type: user_info.user_type,
            name: user_info.name.clone(),
        }
    }
}

pub type UsersVec = Arc<Mutex<Vec<UserInfo>>>;

pub type LobbyId = Arc<Mutex<u16>>;
pub type LobbyName = Arc<Mutex<String>>;
pub type LobbyInfo = (u16, SocketAddr, BoolMutex, JoinHandle<()>);
pub type LobbyVec = Arc<Mutex<Vec<LobbyInfo>>>;

#[derive(Serialize)]
pub struct LobbyAddr {
    pub id: u16,
    pub addr: SocketAddr,
}

#[derive(Serialize)]
pub struct LobbyState {
    pub name: String,
    pub users: Vec<UserInfoShort>,
    pub game: Option<GameState>,
}

#[derive(Serialize)]
pub struct LobbyStateShort {
    pub name: String,
    pub users: u32,
    pub game_going: bool,
}

pub type Game = Arc<Mutex<Option<GameState>>>;
