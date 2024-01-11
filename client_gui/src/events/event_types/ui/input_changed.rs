use crate::events::Window;

#[derive(Clone, Debug)]
pub struct InputChangedEventData {
    pub window: Window,
    pub id: u32,
    pub data: String,
}

impl PartialEq for InputChangedEventData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.window == other.window
    }
}
