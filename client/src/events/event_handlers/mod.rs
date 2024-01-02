mod lobby_closing;
mod player_joined;
mod player_left;

pub use lobby_closing::LobbyClosingEvent;
pub use player_joined::PlayerJoinedEvent;
pub use player_left::PlayerLeftEvent;

use super::EventError;

pub trait Event {
    fn execute(&self) -> Result<(), EventError>;
}
