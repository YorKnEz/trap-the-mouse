mod create_lobby;
mod game;
mod lobbies;
mod settings;
mod start;

use anyhow::Result;
use sfml::graphics::Drawable;

pub use create_lobby::CreateLobbyWindow;
pub use game::GameWindow;
pub use lobbies::LobbiesWindow;
pub use settings::SettingsWindow;
pub use start::StartWindow;

use super::components::EventHandler;

pub trait WindowState: EventHandler {
    fn as_drawable(&self) -> &dyn Drawable;
    fn enter(&self) -> Result<()>;
    fn exit(&self) -> Result<()>;
}
