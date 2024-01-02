use super::{Event, EventError};

pub struct PlayerJoinedEvent {
    user_id: u32,
    name: String,
}

impl PlayerJoinedEvent {
    pub fn new(data: (u32, String)) -> PlayerJoinedEvent {
        PlayerJoinedEvent {
            user_id: data.0,
            name: data.1,
        }
    }
}

impl Event for PlayerJoinedEvent {
    fn execute(&self) -> Result<(), EventError> {
        println!("player {} with id {} joined", self.name, self.user_id);
        Ok(())
    }
}
