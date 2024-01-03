use std::{
    cell::RefCell,
    fmt::Display,
    net::SocketAddr,
    rc::Rc,
    sync::{Arc, Mutex},
};

use serde_derive::{Deserialize, Serialize};

use crate::{commands::Command, events::Event};

pub type BoolMutex = Arc<Mutex<bool>>;

pub type EventQueue = Arc<Mutex<Vec<EventQueueItem>>>;
pub type EventQueueItem = Box<dyn Event + Send>;

pub type Cmd = Box<dyn Command>;
pub type CmdQueue = Rc<RefCell<Vec<Cmd>>>;

pub type UserId = Rc<RefCell<u32>>;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum UserType {
    Host,
    Player,
    Spectator,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Player {
    pub id: u32,
    pub user_type: UserType,
    pub name: String,
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "(id: {}, user_type: {:?}, name: {})",
            self.id, self.user_type, self.name
        ))
    }
}

#[derive(Debug)]
pub struct Lobby {
    pub id: u16,
    pub addr: SocketAddr,
    pub players: Vec<Player>,
}

impl Display for Lobby {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display = String::new();

        display += &format!("id: {}\naddr: {:?}\nplayers:\n", self.id, self.addr);

        for player in self.players.iter() {
            display += &format!("  {}\n", player);
        }

        f.write_str(&display)
    }
}

#[derive(Debug)]
pub struct LobbyState {
    pub id: u16,
    pub users: Vec<Player>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct LobbyAddr {
    pub id: u16,
    pub addr: SocketAddr,
}

pub type LobbyVec = Arc<Mutex<Vec<LobbyAddr>>>;

pub type ActiveLobby = Arc<Mutex<(Lobby, bool)>>;
