mod join_lobby;
mod leave_lobby;

pub use join_lobby::JoinLobbyRequest;
pub use leave_lobby::LeaveLobbyRequest;

use super::{error, Request};
