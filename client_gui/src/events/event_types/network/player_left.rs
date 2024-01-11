#[derive(Clone, Copy, Debug)]
pub struct PlayerLeftEvent {
    pub user_id: u32,
}

impl PlayerLeftEvent {
    pub fn new(user_id: u32) -> PlayerLeftEvent {
        PlayerLeftEvent { user_id }
    }
}
