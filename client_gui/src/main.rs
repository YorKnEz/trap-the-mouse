mod commands;
mod events;
mod gui;
mod types;

use commands::{check_error, connect_cmd, disconnect_cmd};
use gui::components::{Button, ButtonVariant, MouseEventObserver, MouseObserver};
use gui::window::{CreateLobbyWindow, GameWindow, SettingsWindow};
use types::{GameState, GameStateShared, RcCell};

use std::cell::RefCell;
use std::collections::VecDeque;
use std::env::args;
use std::net::SocketAddr;
use std::rc::Rc;

thread_local! {
    pub static SERVER_ADDR: SocketAddr = args().nth(1).expect("no address provided").parse().expect("invalid address");
}

use events::{EventLoop, UIEvent, Window};
use sfml::graphics::{
    Color, RcFont, RenderTarget, RenderWindow, Sprite, TextStyle, Texture, Transformable,
};
use sfml::window::{mouse, Style, VideoMode};

use crate::events::{Event, NetworkEvent};
use crate::gui::window::{LobbiesWindow, StartWindow, WindowState};

const WINDOW_SIZE: f32 = 800.0;
const WINDOW_TITLE: &str = "Test";
const BUTTON_WIDTH: f32 = 240.0;
const BUTTON_HEIGHT: f32 = 60.0;
const PADDING: f32 = 10.0;
const DEFAULT_NAME: &str = "Player";

fn main() {
    let game_state: GameStateShared = rc_cell!(GameState {
        id: 0, // invalid id, doesn't matter because we connect before using the id
        name: String::from(DEFAULT_NAME),
        lobby: None,
        selected_lobby: None,
    });

    let event_loop = EventLoop::new().unwrap();

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

    let bg_texture = Texture::from_file("./client_gui/assets/bg.png").unwrap();
    let mut bg = Sprite::with_texture(&bg_texture);
    bg.set_position((0.0, 0.0));

    let font =
        RcFont::from_file("./client_gui/assets/montserrat-regular.ttf").expect("cannot load font");
    let _bold_font =
        RcFont::from_file("./client_gui/assets/montserrat-bold.ttf").expect("cannot load font");

    let start_window = StartWindow::new(
        Window::Start,
        &font,
        event_loop.sender.clone(),
        // Rc::clone(&game_state),
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

    let mut err_queue = VecDeque::new();
    let mut current_err = None;
    let global_mouse_observer = MouseObserver::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32);

    while window.is_open() {
        while let Some(e) = event_loop.get_event() {
            current_window.borrow().handle_event(e.clone());

            match e.clone() {
                Event::UI(UIEvent::Error(err)) => {
                    err_queue.push_back(format!("Error: {err}"));

                    if current_err.is_none() {
                        current_err = Some(rc_cell!(Button::builder()
                            .set_bounds(10.0, 710.0, 400.0, 80.0)
                            .set_border(2.0)
                            .set_colors(ButtonVariant::Red)
                            .set_text(&err_queue.pop_front().unwrap())
                            .set_font_size(16)
                            .set_font_style(TextStyle::REGULAR)
                            .set_center_text(false)
                            .build(0, Window::Global, event_loop.sender.clone(), &font)));

                        global_mouse_observer.add_observer(current_err.as_ref().unwrap().clone());
                    }
                }
                Event::UI(UIEvent::ButtonClicked(event_data)) => match event_data.window {
                    Window::Global => {
                        if event_data.id == 0 {
                            global_mouse_observer
                                .remove_observer(current_err.as_ref().unwrap().borrow().get_id());
                            current_err = None;
                        }
                    }
                    Window::Start => match event_data.id {
                        1 => switch_state(current_window.clone(), &create_lobby_window),
                        2 => switch_state(current_window.clone(), &lobbies_window),
                        3 => switch_state(current_window.clone(), &settings_window),
                        4 => window.close(),
                        _ => {}
                    },
                    Window::CreateLobby => match event_data.id {
                        1 => switch_state(current_window.clone(), &game_window),
                        2 => switch_state(current_window.clone(), &start_window),
                        _ => {}
                    },
                    Window::Lobbies => match event_data.id {
                        1 => switch_state(current_window.clone(), &game_window),
                        2 => switch_state(current_window.clone(), &start_window),
                        _ => {}
                    },
                    Window::Settings => match event_data.id {
                        1 => switch_state(current_window.clone(), &start_window),
                        2 => switch_state(current_window.clone(), &start_window),
                        _ => {}
                    },
                    Window::Game => {
                        if event_data.id == 5 {
                            switch_state(current_window.clone(), &start_window);
                        }
                    }
                },
                Event::Network(NetworkEvent::LobbyClosing(_)) => {
                    switch_state(current_window.clone(), &lobbies_window);
                }
                _ => {}
            }
        }

        while let Some(e) = window.poll_event() {
            current_window.borrow().handle_event(Event::Sfml(e));

            match e {
                sfml::window::Event::Closed => window.close(),
                sfml::window::Event::MouseButtonReleased { button, x, y } => {
                    if button == mouse::Button::Left {
                        global_mouse_observer.click(x, y);
                    }
                }
                _ => {}
            }
        }

        window.clear(Color::rgb(57, 11, 74));

        window.draw(&bg);
        window.draw(current_window.borrow().as_drawable());

        if current_err.is_some() {
            window.draw(&*current_err.as_ref().unwrap().borrow());
        }

        window.display();
    }

    {
        if let Err(e) = current_window.borrow().exit() {
            panic!("cannot disconnect game: {e:?}");
        }
    }

    {
        let mut game_state = game_state.borrow_mut();

        match disconnect_cmd(&game_state.id) {
            Ok(_) => game_state.id = 0,
            Err(e) => {
                check_error(e);
            }
        }
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
