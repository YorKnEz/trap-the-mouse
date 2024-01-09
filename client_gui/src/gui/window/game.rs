use std::{cell::RefCell, rc::Rc, sync::mpsc};

use anyhow::anyhow;
use sfml::{
    graphics::{Drawable, RcFont},
    window::mouse,
};

use crate::{
    commands::{check_error, join_lobby_cmd, leave_lobby_cmd},
    events::{Event, NetworkEvent, UIEvent, Window},
    gui::components::{Button, Clicker, EventHandler, Fixed, PlayerCard, Scrollable},
    rc_cell,
    types::{GameStateShared, Lobby, Player, RcCell},
    BUTTON_HEIGHT, BUTTON_WIDTH, PADDING, WINDOW_SIZE,
};

use super::WindowState;

pub struct GameWindow<'a> {
    window: Window,
    state: GameStateShared,
    sender: mpsc::Sender<UIEvent>,

    back: RcCell<Button<'a>>,
    players_scrollable: RcCell<Scrollable<'a, PlayerCard<'a>>>,
    clicker: Clicker<'a>,
    font: &'a RcFont,
}

impl<'a> GameWindow<'a> {
    pub fn new(
        window: Window,
        font: &'a RcFont,
        sender: mpsc::Sender<UIEvent>,
        state: GameStateShared,
    ) -> GameWindow<'a> {
        let lables = vec!["Spectate", "Play", "Close lobby", "Make host", "Back"];

        GameWindow {
            window,
            state,
            back: rc_cell!(Button::new(
                2,
                window,
                WINDOW_SIZE / 2f32 - BUTTON_WIDTH / 2f32,
                WINDOW_SIZE / 2f32 - BUTTON_HEIGHT / 2f32 + 2f32 * (BUTTON_HEIGHT + PADDING),
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
                "Back",
                font,
                sender.clone(),
            )),
            players_scrollable: rc_cell!(Scrollable::new(10f32, 40f32, 240f32, 600f32)),
            clicker: Clicker::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
            font,
            sender,
        }
    }

    pub fn init(&self) {
        self.clicker.add_clickable(self.back.clone());
        self.clicker.add_clickable(self.players_scrollable.clone());
    }

    fn add_player(&self, player: Player) {
        let mut players_scrollable = self.players_scrollable.borrow_mut();
        let bounds = players_scrollable.bounds();

        let mut state = self.state.borrow_mut();
        let players = &mut state.lobby.as_mut().unwrap().players;

        let card = rc_cell!(PlayerCard::new(
            player.id as u32,
            self.window,
            player.clone(),
            bounds.width
                - Scrollable::<PlayerCard>::SCROLLBAR_WIDTH
                - 2f32 * Scrollable::<PlayerCard>::PADDING,
            60f32,
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
                    state.lobby = Some(Lobby {
                        id: lobby.id,
                        addr: lobby.addr,
                        name: lobby_state.name,
                        players: lobby_state.players,
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

        let mut players_scrollable = self.players_scrollable.borrow_mut();
        let bounds = players_scrollable.bounds();

        let players = &state.lobby.as_ref().unwrap().players;

        for player in players {
            let card = rc_cell!(PlayerCard::new(
                player.id as u32,
                self.window,
                player.clone(),
                bounds.width
                    - Scrollable::<PlayerCard>::SCROLLBAR_WIDTH
                    - 2f32 * Scrollable::<PlayerCard>::PADDING,
                60f32,
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
            let id = state.id; // dirty hack sorry

            match leave_lobby_cmd(&id, &mut state.lobby) {
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
            Event::SFML(sfml::window::Event::MouseButtonReleased { button, x, y }) => {
                if button == mouse::Button::Left {
                    self.clicker.click(x, y);
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
        target.draw(&*self.back.borrow());
        target.draw(&*self.players_scrollable.borrow());
    }
}
