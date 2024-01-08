#[derive(Clone, Copy, Debug)]
pub struct LobbyClosingEvent {}

impl LobbyClosingEvent {
    pub fn new(_: ()) -> LobbyClosingEvent {
        LobbyClosingEvent {}
    }
}
