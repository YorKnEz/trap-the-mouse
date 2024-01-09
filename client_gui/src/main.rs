mod commands;
mod events;
mod gui;
mod types;

use commands::{check_error, connect_cmd, disconnect_cmd};
use gui::window::{GameWindow, SettingsWindow, CreateLobbyWindow};
use types::{GameState, GameStateShared, RcCell};

use std::cell::RefCell;
use std::net::{Ipv4Addr, SocketAddr};
use std::rc::Rc;

const SERVER_ADDR: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 20000);

use events::{EventLoop, UIEvent, Window};
use sfml::graphics::{Color, RcFont, RenderTarget, RenderWindow};
use sfml::window::{Style, VideoMode};

use crate::events::{Event, NetworkEvent};
use crate::gui::window::{LobbiesWindow, StartWindow, WindowState};

const WINDOW_SIZE: f32 = 800f32;
const WINDOW_TITLE: &str = "Test";
const BUTTON_WIDTH: f32 = 240f32;
const BUTTON_HEIGHT: f32 = 60f32;
const PADDING: f32 = 10f32;
const DEFAULT_NAME: &str = "Player";

fn main() {
    let game_state: GameStateShared = rc_cell!(GameState {
        id: 0, // invalid id, doesn't matter because we connect before using the id
        name: String::from(DEFAULT_NAME),
        lobby: None,
        selected_lobby: None,
    });

    let mut window = RenderWindow::new(
        (WINDOW_SIZE as u32, WINDOW_SIZE as u32),
        WINDOW_TITLE,
        Style::CLOSE,
        &Default::default(),
    );
    window.set_framerate_limit(60);

    let desktop = VideoMode::desktop_mode();
    window.set_position(
        (
            (desktop.width / 2 - WINDOW_SIZE as u32 / 2) as i32,
            (desktop.height / 2 - WINDOW_SIZE as u32 / 2) as i32,
        )
            .into(),
    );

    let event_loop = EventLoop::new().unwrap();

    let font =
        RcFont::from_file("./client_gui/assets/montserrat-regular.ttf").expect("cannot load font");
    let _bold_font =
        RcFont::from_file("./client_gui/assets/montserrat-bold.ttf").expect("cannot load font");

    let start_window = StartWindow::new(
        Window::Start,
        &font,
        event_loop.sender.clone(),
        Rc::clone(&game_state),
    );
    let create_lobby_window = CreateLobbyWindow::new(
        Window::CreateLobby,
        &font,
        event_loop.sender.clone(),
        Rc::clone(&game_state),
    );
    let lobbies_window = LobbiesWindow::new(
        Window::Lobbies,
        &font,
        event_loop.sender.clone(),
        Rc::clone(&game_state),
    );
    let settings_window = SettingsWindow::new(
        Window::Settings,
        &font,
        event_loop.sender.clone(),
        Rc::clone(&game_state),
    );
    let game_window = GameWindow::new(
        Window::Game,
        &font,
        event_loop.sender.clone(),
        Rc::clone(&game_state),
    );

    start_window.init();
    create_lobby_window.init();
    lobbies_window.init();
    settings_window.init();
    game_window.init();

    let current_window: RcCell<&dyn WindowState> = rc_cell!(&start_window);

    {
        if let Err(e) = current_window.borrow().enter() {
            panic!("cannot start game: {e:?}");
        }
    }

    {
        let mut game_state = game_state.borrow_mut();

        match connect_cmd(game_state.name.clone(), event_loop.addr) {
            Ok(id) => game_state.id = id,
            Err(e) => {
                check_error(e);
                panic!("cannot connect to server");
            }
        }
    }

    while window.is_open() {
        while let Some(e) = event_loop.get_event() {
            current_window.borrow().handle_event(e.clone());

            match e.clone() {
                Event::UI(UIEvent::ButtonClicked(event_data)) => match event_data.window {
                    Window::Start => match event_data.id {
                        1 => switch_state(current_window.clone(), &create_lobby_window),
                        2 => switch_state(current_window.clone(), &lobbies_window),
                        3 => switch_state(current_window.clone(), &settings_window),
                        4 => window.close(),
                        _ => {}
                    }
                    Window::CreateLobby => match event_data.id {
                        1 => switch_state(current_window.clone(), &game_window),
                        2 => switch_state(current_window.clone(), &start_window),
                        _ => {}
                    }
                    Window::Lobbies => match event_data.id {
                        0 => println!("search"),
                        2 => switch_state(current_window.clone(), &game_window),
                        3 => switch_state(current_window.clone(), &start_window),
                        _ => {}
                    }
                    Window::Settings => match event_data.id {
                        2 => switch_state(current_window.clone(), &start_window),
                        _ => {}
                    }
                    Window::Game => match event_data.id {
                        2 => switch_state(current_window.clone(), &start_window),
                        _ => {}
                    }
                }
                Event::Network(NetworkEvent::LobbyClosing(_)) => {
                    switch_state(current_window.clone(), &lobbies_window);
                }
                _ => {}
            }
        }

        while let Some(e) = window.poll_event() {
            current_window.borrow().handle_event(Event::SFML(e));

            match e {
                sfml::window::Event::Closed => window.close(),
                _ => {}
            }
        }

        window.clear(Color::rgb(57, 11, 74));

        window.draw(current_window.borrow().as_drawable());

        window.display();
    }

    let mut game_state = game_state.borrow_mut();

    match disconnect_cmd(&game_state.id) {
        Ok(_) => game_state.id = 0,
        Err(e) => check_error(e),
    }
}

fn switch_state<'a>(current: RcCell<&'a dyn WindowState>, to: &'a dyn WindowState) {
    let mut current = current.borrow_mut();
    if let Err(e) = current.exit() {
        println!("error exit: {e:?}");
        return;
    }

    if let Err(e) = to.enter() {
        println!("error enter: {e:?}");
        return;
    }

    *current = to;
}