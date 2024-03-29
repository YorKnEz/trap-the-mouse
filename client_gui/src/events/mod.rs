mod event_loop;
mod event_types;

pub use event_loop::EventLoop;
pub use event_types::{network::*, ui::*};

/// Event loop events, includes SFML, Network and UI events
#[derive(Clone, Debug)]
pub enum Event {
    Sfml(sfml::window::Event),
    Network(NetworkEvent),
    UI(UIEvent),
}

#[derive(Clone, Debug)]
pub enum NetworkEvent {
    PlayerUpdated(PlayerUpdatedEvent),
    PlayerJoined(PlayerJoinedEvent),
    PlayerLeft(PlayerLeftEvent),
    GameStarted(GameStartedEvent),
    GameUpdated(GameUpdatedEvent),
    LobbyClosing(LobbyClosingEvent),
    Message(MessageEvent),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Window {
    Global,
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
    InputChanged(InputChangedEventData),
    LobbyCardClicked(LobbyCardEventData),
    // LobbyCardNoClicked(LobbyCardEventData),
    PlayerCardClicked(PlayerCardEventData),
    // PlayerCardNoClicked(PlayerCardEventData),
    GameMove(GameMoveEventData),
    Error(String),
}
