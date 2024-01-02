mod lobby;
mod server;

mod exit;
mod invalid;
mod ping;

mod error;

use anyhow::Result;

use serde::Serialize;

use network::Type;

pub use invalid::InvalidRequest;
pub use ping::PingRequest;
pub use exit::ExitRequest;

pub use lobby::*;
pub use server::*;

use error::ServerError;

pub trait Request {
    fn execute(&mut self) -> Result<()>;
}

pub fn error_check<T: Serialize>(res: Result<T, ServerError>) -> Result<(Type, Vec<u8>)> {
    Ok(match res {
        Ok(res) => (Type::Success, bincode::serialize(&res)?),
        Err(e) => {
            println!("request error: {e:?}");
            match e {
                ServerError::API { message } => (Type::Error, bincode::serialize(&message)?),
                ServerError::APINotConnected => (Type::Error, bincode::serialize(&"you are not connected")?),
                _ => (Type::Error, bincode::serialize("internal error")?),
            }
        }
    })
}
