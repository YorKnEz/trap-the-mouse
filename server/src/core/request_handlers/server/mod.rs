mod connect;
mod disconnect;

mod change_name;
mod create_lobby;
mod get_lobbies;

pub use change_name::ChangeNameRequest;
pub use connect::ConnectRequest;
pub use create_lobby::CreateLobbyRequest;
pub use disconnect::DisconnectRequest;
pub use get_lobbies::GetLobbiesRequest;

use super::{error, Request};
