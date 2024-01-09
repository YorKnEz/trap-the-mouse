mod event_types;
mod event_loop;

pub use event_types::*;
pub use event_loop::EventLoop;

/// Event loop events, includes SFML, Network and UI events
#[derive(Clone, Debug)]
pub enum Event {
    SFML(sfml::window::Event),
    Network(NetworkEvent),
    UI(UIEvent),
}

#[derive(Clone, Debug)]
pub enum NetworkEvent {
    PlayerUpdated(PlayerUpdatedEvent),
    PlayerJoined(PlayerJoinedEvent),
    PlayerLeft(PlayerLeftEvent),
    LobbyClosing(LobbyClosingEvent),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Window {
    Start,
    Settings,
    Lobbies,
    CreateLobby,
    Game,
}

/// Data about the location of the event, contains the window where the event happened and the id of the component
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventData {
    pub window: Window,
    pub id: u32,
}

#[derive(Clone, Debug)]
pub enum UIEvent {
    ButtonClicked(EventData),
    InputClicked(EventData),
    InputNoClicked(EventData),
    InputChanged{value: String},
    LobbyCardClicked(LobbyCardEventData),
    // LobbyCardNoClicked(LobbyCardEventData),
    PlayerCardClicked(PlayerCardEventData),
    // PlayerCardNoClicked(PlayerCardEventData),
}
