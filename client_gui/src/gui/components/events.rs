use crate::events::Event;

pub trait EventHandler {
    fn handle_event(&self, e: Event);
}

pub trait EventHandlerMut {
    fn handle_event(&mut self, e: Event);
}
