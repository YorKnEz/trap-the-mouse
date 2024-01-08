mod game;
mod lobbies;
mod start;
mod settings;
mod create_lobby;

use anyhow::Result;
use sfml::graphics::Drawable;

pub use game::GameWindow;
pub use lobbies::LobbiesWindow;
pub use start::StartWindow;
pub use settings::SettingsWindow;
pub use create_lobby::CreateLobbyWindow;

use super::components::EventHandler;

pub trait WindowState: EventHandler {
    fn as_drawable(&self) -> &dyn Drawable;
    fn enter(&self) -> Result<()>;
    fn exit(&self) -> Result<()>;
}
