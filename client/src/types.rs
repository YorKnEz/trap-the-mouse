use std::{
    cell::RefCell,
    fmt::Display,
    net::SocketAddr,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::events::Event;
use serde_derive::{Deserialize, Serialize};

pub type BoolMutex = Arc<Mutex<bool>>;

pub type EventQueue = Arc<Mutex<Vec<EventQueueItem>>>;
pub type EventQueueItem = Event;
pub type UserId = u32;
pub type UserName = String;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum UserType {
    Host,
    Player,
    Spectator,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct LobbyAddr {
    pub id: u16,
    pub addr: SocketAddr,
}

#[derive(Debug, Deserialize)]
pub struct LobbyState {
    pub name: String,
    pub players: Vec<Player>,
}

#[derive(Debug, Deserialize)]
pub struct LobbyStateShort {
    pub name: String,
    pub players: u32,
}

#[derive(Debug)]
pub struct Lobby {
    pub id: u16,
    pub addr: SocketAddr,
    pub name: String,
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

#[derive(Clone, Debug)]
pub struct LobbyShort {
    pub id: u16,
    pub addr: SocketAddr,
    pub name: String,
    pub players: u32,
}

pub type LobbyVec = Vec<LobbyShort>;
pub type LobbyAddrVec = Vec<LobbyAddr>;

/// The game state to be shared across windows
/// Option is used because initially there is no state, throughout the code unwrap will be unused because we know for sure that the values exist because in order to get to window X part Y of state must be initialized
pub struct GameState {
    pub id: UserId,
    pub name: UserName,
    pub lobby: Option<Lobby>,
    pub selected_lobby: Option<LobbyShort>,
}
pub type GameStateShared = RcCell<GameState>;

pub type RcCell<T> = Rc<RefCell<T>>;

#[macro_export]
macro_rules! rc_cell {
    ($value:expr) => {
        Rc::new(RefCell::new($value))
    };
}

