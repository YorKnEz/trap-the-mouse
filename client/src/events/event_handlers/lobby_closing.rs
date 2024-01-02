use super::{Event, EventError};

pub struct LobbyClosingEvent {}

impl LobbyClosingEvent {
    pub fn new(_: ()) -> LobbyClosingEvent {
        LobbyClosingEvent {}
    }
}

impl Event for LobbyClosingEvent {
    fn execute(&self) -> Result<(), EventError> {
        println!("lobby closing");
        Ok(())
    }
}
