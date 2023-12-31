mod create_lobby;
mod delete_lobby;
mod get_lobbies;
mod exit;
mod invalid;
mod ping;
mod error;

use anyhow::Result;

pub use create_lobby::CreateLobbyRequest;
pub use delete_lobby::DeleteLobbyRequest;
pub use get_lobbies::GetLobbiesRequest;
pub use exit::ExitRequest;
pub use invalid::InvalidRequest;
pub use ping::PingRequest;

pub trait Request {
    fn execute(&mut self) -> Result<()>;
}
