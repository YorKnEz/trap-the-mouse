use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, Condvar},
    thread::JoinHandle, cell::RefCell,
};

use serde_derive::Serialize;

use super::request_handlers::Request;

pub type BoolMutex = Arc<Mutex<bool>>;

pub type HandleVec = RefCell<Vec<JoinHandle<()>>>;

#[derive(PartialEq, Clone, Copy, Serialize)]
pub enum UserType {
    Host,
    Player,
    Spectator,
}

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
        UserInfoShort { id: user_info.id, user_type: user_info.user_type, name: user_info.name.clone() }
    }
}

pub type UsersVec = Arc<Mutex<Vec<UserInfo>>>;

pub type LobbyId = Arc<Mutex<u16>>;
pub type LobbyVec = Arc<Mutex<Vec<(u16, SocketAddr, BoolMutex, JoinHandle<()>)>>>;

#[derive(Serialize)]
pub struct LobbyState {
    pub id: u16,
    pub users: Vec<UserInfoShort>,
}

#[derive(Serialize)]
pub struct LobbyAddr {
    pub id: u16,
    pub addr: SocketAddr,
}

pub type RequestQueue = Arc<(Mutex<Vec<RequestQueueItem>>, Condvar)>;
pub type RequestQueueItem = Box<dyn Request + Send>;

