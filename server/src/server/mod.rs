mod lobby_server;
mod main_server;

pub trait ServerT {
    const THREAD_POOL_SIZE: u32;

    fn start(&self);
    fn drop(self);
}

pub use lobby_server::LobbyServer;
pub use main_server::Server;
