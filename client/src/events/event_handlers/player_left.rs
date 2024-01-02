use super::{Event, EventError};

pub struct PlayerLeftEvent {
    user_id: u32,
}

impl PlayerLeftEvent {
    pub fn new(user_id: u32) -> PlayerLeftEvent {
        PlayerLeftEvent { user_id }
    }
}

impl Event for PlayerLeftEvent {
    fn execute(&self) -> Result<(), EventError> {
        println!("player with id {} left", self.user_id);
        Ok(())
    }
}
