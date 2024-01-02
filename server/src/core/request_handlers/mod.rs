mod lobby;
mod server;

mod exit;
mod invalid;
mod ping;

mod error;

use anyhow::Result;


pub use invalid::InvalidRequest;
pub use ping::PingRequest;
pub use exit::ExitRequest;

pub use lobby::*;
pub use server::*;

pub trait Request {
    fn execute(&mut self) -> Result<()>;
}
