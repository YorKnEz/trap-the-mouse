mod lobby_closing;
mod player_joined;
mod player_left;
mod became_role;

pub use lobby_closing::LobbyClosingEvent;
pub use player_joined::PlayerJoinedEvent;
pub use player_left::PlayerLeftEvent;
pub use became_role::BecameRoleEvent;

use super::EventError;

pub trait Event {
    fn execute(&self) -> Result<(), EventError>;
}
