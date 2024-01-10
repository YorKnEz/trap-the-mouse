use std::{cell::RefCell, rc::Rc, sync::mpsc};

use anyhow::anyhow;
use sfml::{
    graphics::{Drawable, FloatRect, RcFont, RcText, Transformable},
    system::Vector2f,
    window::mouse,
};

use crate::{
    commands::{
        become_role_cmd, check_error, close_lobby_cmd, join_lobby_cmd, leave_lobby_cmd,
        make_host_cmd, make_move_cmd, start_game_cmd,
    },
    events::{Event, NetworkEvent, UIEvent, Window},
    gui::components::{
        game::Game, Button, Clickable, Clicker, EventHandler, EventHandlerMut, Fixed, PlayerCard,
        Scrollable,
    },
    rc_cell,
    types::{GameStateShared, Lobby, Player, RcCell, UserType},
    BUTTON_HEIGHT, PADDING, WINDOW_SIZE,
};

use super::WindowState;

pub struct GameWindow<'a> {
    window: Window,
    state: GameStateShared,
    selected_player: RefCell<Option<Player>>,
    sender: mpsc::Sender<UIEvent>,

    font: &'a RcFont,
    game_state: RefCell<RcText>,
    buttons: Vec<RcCell<Button<'a>>>,
    show_buttons: RefCell<Vec<usize>>,
    players_scrollable: RcCell<Scrollable<'a, PlayerCard<'a>>>,
    game: RcCell<Game>,
    clicker: Clicker<'a>,
}

impl<'a> GameWindow<'a> {
    const GAME_WIDTH: f32 = 600.0;
    const GAME_HEIGHT: f32 = 480.0;

    pub fn new(
        window: Window,
        font: &'a RcFont,
        sender: mpsc::Sender<UIEvent>,
        state: GameStateShared,
    ) -> GameWindow<'a> {
        let texts = [
            "Start game",
            "Make host",
            "Close lobby",
            "Spectate",
            "Play",
            "Back",
        ];
        let mut buttons = vec![];

        for (i, text) in texts.iter().enumerate() {
            buttons.push(rc_cell!(Button::new(
                i as u32,
                window,
                FloatRect::new(
                    0.0,
                    0.0,
                    WINDOW_SIZE - (GameWindow::GAME_WIDTH + 3.0 * PADDING),
                    BUTTON_HEIGHT
                ),
                text,
                font,
                sender.clone(),
            )));
        }

        let mut game_state = RcText::new("Waiting for host to start a new game", font, 20);
        let text_height = game_state.character_size() as f32;
        let text_width = game_state.local_bounds().width;
        game_state.set_position((
            WINDOW_SIZE - 10.0 - GameWindow::GAME_WIDTH + GameWindow::GAME_WIDTH / 2.0
                - text_width / 2.0,
            10.0 + GameWindow::GAME_HEIGHT / 2.0 + text_height / 2.0,
        ));

        GameWindow {
            game_state: RefCell::new(game_state),
            window,
            state,
            selected_player: RefCell::new(None),
            buttons,
            show_buttons: RefCell::new(vec![]),
            players_scrollable: rc_cell!(Scrollable::new(
                6,
                window,
                FloatRect::new(
                    WINDOW_SIZE - PADDING - 240.0,
                    PADDING + GameWindow::GAME_HEIGHT + PADDING,
                    240.0,
                    WINDOW_SIZE - (GameWindow::GAME_HEIGHT + 3.0 * PADDING)
                ),
            )),
            game: rc_cell!(Game::new(
                7,
                window,
                FloatRect::new(
                    WINDOW_SIZE - 10.0 - GameWindow::GAME_WIDTH,
                    10.0,
                    GameWindow::GAME_WIDTH,
                    GameWindow::GAME_HEIGHT
                ),
                sender.clone()
            )),
            clicker: Clicker::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
            font,
            sender,
        }
    }

    pub fn init(&self) {
        self.clicker.add_clickable(self.players_scrollable.clone());
    }

    fn add_player(&self, player: Player) {
        let mut players_scrollable = self.players_scrollable.borrow_mut();
        let bounds = players_scrollable.bounds();

        let mut state = self.state.borrow_mut();
        let players = &mut state.lobby.as_mut().unwrap().players;

        let card = rc_cell!(PlayerCard::new(
            player.id,
            self.window,
            player.clone(),
            FloatRect::new(
                0.0,
                0.0,
                bounds.width
                    - Scrollable::<PlayerCard>::SCROLLBAR_WIDTH
                    - 2.0 * Scrollable::<PlayerCard>::PADDING,
                60.0
            ),
            self.font,
            self.sender.clone(),
        ));

        players_scrollable.add(card.clone());
        players.push(player);
    }

    fn remove_player(&self, id: u32) {
        let mut state = self.state.borrow_mut();
        let players = &mut state.lobby.as_mut().unwrap().players;

        if let Some(index) = players.iter().position(|p| p.id == id) {
            let mut players_scrollable = self.players_scrollable.borrow_mut();

            players_scrollable.remove(index);
            players.remove(index);
        }
    }

    fn update_state(&self, user_type: UserType) {
        let mut show_buttons = self.show_buttons.borrow_mut();

        // remove old buttons
        for &i in show_buttons.iter() {
            self.clicker
                .remove_clickable(self.buttons[i].borrow().get_id());
        }

        show_buttons.clear();

        match user_type {
            UserType::Host => show_buttons.extend_from_slice(&[0, 1, 2, 5]),
            UserType::Player => show_buttons.extend_from_slice(&[3, 5]),
            UserType::Spectator => show_buttons.extend_from_slice(&[4, 5]),
        }

        let x = 10.0;
        let y = 10.0;

        for (i, b) in show_buttons.iter().enumerate() {
            let button = &self.buttons[*b];
            button
                .borrow_mut()
                .set_position(Vector2f::new(x, y + i as f32 * (PADDING + BUTTON_HEIGHT)));
            self.clicker.add_clickable(button.clone());
        }
    }

    fn set_game_state(&self, value: &str) {
        let mut game_state = self.game_state.borrow_mut();

        game_state.set_string(value);
        let text_height = game_state.character_size() as f32;
        let text_width = game_state.local_bounds().width;
        game_state.set_position((
            WINDOW_SIZE - 10.0 - GameWindow::GAME_WIDTH + GameWindow::GAME_WIDTH / 2.0
                - text_width / 2.0,
            10.0 + GameWindow::GAME_HEIGHT / 2.0 + text_height / 2.0,
        ));
    }
}

impl<'a> WindowState for GameWindow<'a> {
    fn as_drawable(&self) -> &dyn Drawable {
        self
    }

    fn enter(&self) -> anyhow::Result<()> {
        let mut state = self.state.borrow_mut();
        if let Some(lobby) = state.selected_lobby.as_ref() {
            match join_lobby_cmd(&state.id, lobby.addr, &state.lobby) {
                Ok(lobby_state) => {
                    let user_type = lobby_state
                        .players
                        .iter()
                        .find(|p| p.id == state.id)
                        .unwrap()
                        .user_type; // unwrap because user just joined lobby, so he's on the list
                    state.lobby = Some(Lobby {
                        id: lobby.id,
                        addr: lobby.addr,
                        name: lobby_state.name,
                        players: lobby_state.players,
                        user_type,
                    });
                }
                Err(e) => {
                    check_error(e);
                    return Err(anyhow!("couldn't join"));
                }
            }
        } else {
            return Err(anyhow!("no lobby selected"));
        }

        let lobby = state.lobby.as_ref().unwrap();

        self.update_state(lobby.user_type);
        self.set_game_state("Waiting for host to start a new game");

        let mut players_scrollable = self.players_scrollable.borrow_mut();
        players_scrollable.clear();

        let bounds = players_scrollable.bounds();

        let players = &lobby.players;

        for player in players {
            let card = rc_cell!(PlayerCard::new(
                player.id,
                self.window,
                player.clone(),
                FloatRect::new(
                    0.0,
                    0.0,
                    bounds.width
                        - Scrollable::<PlayerCard>::SCROLLBAR_WIDTH
                        - 2.0 * Scrollable::<PlayerCard>::PADDING,
                    60.0
                ),
                self.font,
                self.sender.clone(),
            ));
            players_scrollable.add(card.clone());
        }

        Ok(())
    }

    fn exit(&self) -> anyhow::Result<()> {
        let mut state = self.state.borrow_mut();
        if state.lobby.is_some() {
            match leave_lobby_cmd(&state.id.clone(), &mut state.lobby) {
                Ok(_) => {
                    self.players_scrollable.borrow_mut().clear();
                }
                Err(e) => {
                    check_error(e);
                    return Err(anyhow!("cannot leave lobby"));
                }
            }
        }

        Ok(())
    }
}

impl<'a> EventHandler for GameWindow<'a> {
    fn handle_event(&self, e: Event) {
        match e {
            Event::Sfml(sfml::window::Event::MouseButtonReleased { button, x, y }) => {
                if button == mouse::Button::Left {
                    self.clicker.click(x, y);
                }
            }
            Event::UI(UIEvent::ButtonClicked(event_data)) if event_data.window == self.window => {
                let mut state = self.state.borrow_mut();
                match event_data.id {
                    0 => match start_game_cmd(&state.id, &state.lobby) {
                        Ok(_) => {}
                        Err(e) => check_error(e),
                    },
                    1 => {
                        if let Some(player) = self.selected_player.borrow().as_ref() {
                            match make_host_cmd(&state.id, player.id, &state.lobby) {
                                Ok(_) => {}
                                Err(e) => check_error(e),
                            }
                        }
                    }
                    2 => match close_lobby_cmd(&state.id.clone(), &mut state.lobby) {
                        Ok(_) => {}
                        Err(e) => check_error(e),
                    },
                    3 => match become_role_cmd(&state.id, UserType::Spectator, &state.lobby) {
                        Ok(_) => {}
                        Err(e) => check_error(e),
                    },
                    4 => match become_role_cmd(&state.id, UserType::Player, &state.lobby) {
                        Ok(_) => {}
                        Err(e) => check_error(e),
                    },
                    _ => {}
                }
            }
            Event::UI(UIEvent::PlayerCardClicked(event_data))
                if event_data.window == self.window =>
            {
                *self.selected_player.borrow_mut() = Some(event_data.data);
            }
            // Event::UI(UIEvent::PlayerCardNoClicked(event_data))
            //     if event_data.window == self.window => {}
            Event::UI(UIEvent::GameMove(e)) => {
                let state = self.state.borrow();
                match make_move_cmd(&state.id, (e.x, e.y), &state.lobby) {
                    Ok(_) => {}
                    Err(e) => check_error(e),
                }
            }
            Event::Network(NetworkEvent::PlayerJoined(e)) => {
                self.add_player(e.player);
            }
            Event::Network(NetworkEvent::PlayerLeft(e)) => {
                self.remove_player(e.user_id);
            }
            Event::Network(NetworkEvent::PlayerUpdated(e)) => {
                let mut state = self.state.borrow_mut();

                // update the UI if the user type changed
                if e.player.id == state.id {
                    self.update_state(e.player.user_type);
                }

                let players = &mut state.lobby.as_mut().unwrap().players;
                let mut players_scrollable = self.players_scrollable.borrow_mut();

                for (i, player) in players.iter_mut().enumerate() {
                    if player.id == e.player.id {
                        *player = e.player.clone();
                        let card = players_scrollable.get(i);
                        card.borrow_mut().update(e.player.clone());
                    }
                }
            }
            Event::Network(NetworkEvent::GameStarted(e)) => {
                self.game.borrow_mut().start(e);
                self.clicker.add_clickable(self.game.clone());
            }
            Event::Network(NetworkEvent::GameUpdated(e)) => {
                let mut host = String::new();
                let mut player = String::from("computer");

                self.state
                    .borrow()
                    .lobby
                    .as_ref()
                    .unwrap()
                    .players
                    .iter()
                    .for_each(|p| match p.user_type {
                        UserType::Host => host = p.name.clone(),
                        UserType::Player => player = p.name.clone(),
                        _ => {}
                    });

                if e.win.0 {
                    self.set_game_state(&format!(
                        "Player {host} won! Waiting for host to start a new game"
                    ));
                    self.clicker.remove_clickable(self.game.borrow().get_id());
                } else if e.win.1 {
                    self.set_game_state(&format!(
                        "Player {player} won! Waiting for host to start a new game"
                    ));
                    self.clicker.remove_clickable(self.game.borrow().get_id());
                }

                self.game.borrow_mut().update(e);
            }
            Event::Network(NetworkEvent::LobbyClosing(_)) => {
                let mut state = self.state.borrow_mut();
                state.lobby = None;
            }
            _ => {}
        }
    }
}

impl<'a> Drawable for GameWindow<'a> {
    fn draw<'z: 'shader, 'texture, 'shader, 'shader_texture>(
        &'z self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        for i in self.show_buttons.borrow().iter() {
            target.draw(&*self.buttons[*i].borrow());
        }
        target.draw(&*self.players_scrollable.borrow());
        target.draw(&*self.game.borrow());

        if !self.game.borrow().began {
            target.draw(&*self.game_state.borrow());
        }
    }
}
