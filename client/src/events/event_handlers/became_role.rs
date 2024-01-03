use crate::types::{ActiveLobby, UserType};

use super::{Event, EventError};

pub struct BecameRoleEvent {
    user_id: u32,
    user_type: UserType,
    active_lobby: ActiveLobby,
}

impl BecameRoleEvent {
    pub fn new(data: (u32, UserType), active_lobby: ActiveLobby) -> BecameRoleEvent {
        BecameRoleEvent {
            user_id: data.0,
            user_type: data.1,
            active_lobby,
        }
    }
}

impl Event for BecameRoleEvent {
    fn execute(&self) -> Result<(), EventError> {
        {
            let mut active_lobby = self.active_lobby.lock().unwrap();

            if active_lobby.1 {
                active_lobby.0.players.iter_mut().for_each(|p| {
                    if p.id == self.user_id {
                        p.user_type = self.user_type;
                    }
                });

                println!("player {} became {:?}", self.user_id, self.user_type);
            }
        }

        Ok(())
    }
}
