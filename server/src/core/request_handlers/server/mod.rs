mod connect;
mod disconnect;

mod create_lobby;
mod get_lobbies;
mod change_name;

pub use connect::ConnectRequest;
pub use create_lobby::CreateLobbyRequest;
pub use disconnect::DisconnectRequest;
pub use get_lobbies::GetLobbiesRequest;
pub use change_name::ChangeNameRequest;

use super::{error, Request};
