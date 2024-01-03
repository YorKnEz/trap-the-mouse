use crate::types::{ActiveLobby, UserType};

use super::{Event, EventError};

pub struct BecameHostEvent {
    user_id: u32,
    active_lobby: ActiveLobby,
}

impl BecameHostEvent {
    pub fn new(user_id: u32, active_lobby: ActiveLobby) -> BecameHostEvent {
        BecameHostEvent {
            user_id,
            active_lobby,
        }
    }
}

impl Event for BecameHostEvent {
    fn execute(&self) -> Result<(), EventError> {
        {
            let mut active_lobby = self.active_lobby.lock().unwrap();

            if active_lobby.1 {
                active_lobby.0.players.iter_mut().for_each(|p| {
                    if p.id == self.user_id {
                        p.user_type = UserType::Host;
                    }
                });

                println!("player {} became host", self.user_id);
            }
        }

        Ok(())
    }
}
