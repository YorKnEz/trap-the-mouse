use std::{cell::RefCell, rc::Rc, sync::mpsc};

use anyhow::{anyhow, Result};
use sfml::{
    graphics::{Drawable, FloatRect, RcFont},
    window::mouse,
};

use crate::{
    commands::{check_error, get_lobbies_cmd, get_lobby_state},
    events::{Event, UIEvent, Window},
    gui::components::Fixed,
    gui::components::{
        Button, ButtonVariant, EventHandler, EventHandlerMut, LobbyCard, MouseObserver, Scrollable,
    },
    rc_cell,
    types::{GameStateShared, LobbyAddr, LobbyShort, LobbyVec, RcCell},
    BUTTON_HEIGHT, BUTTON_WIDTH, PADDING, WINDOW_SIZE,
};

use super::WindowState;

pub struct LobbiesWindow<'a> {
    window: Window,
    state: GameStateShared,
    lobbies: RefCell<LobbyVec>,
    range: (u32, u32),
    sender: mpsc::Sender<UIEvent>,

    search: RcCell<Button<'a>>,
    join_lobby: RcCell<Button<'a>>,
    back: RcCell<Button<'a>>,
    mouse_observer: MouseObserver<'a>,
    lobbies_scrollable: RcCell<Scrollable<'a, LobbyCard<'a>>>,
    font: &'a RcFont,
}

impl<'a> LobbiesWindow<'a> {
    pub fn new(
        window: Window,
        font: &'a RcFont,
        sender: mpsc::Sender<UIEvent>,
        state: GameStateShared,
    ) -> LobbiesWindow<'a> {
        let x =
            (WINDOW_SIZE - (BUTTON_HEIGHT + PADDING + BUTTON_WIDTH + PADDING + BUTTON_WIDTH)) / 2.0;

        LobbiesWindow {
            window,
            state,
            lobbies: RefCell::new(vec![]),
            range: (0, 10),
            search: rc_cell!(Button::new(
                0,
                window,
                FloatRect::new(x, 40.0 + 600.0 + 10.0, BUTTON_HEIGHT, BUTTON_HEIGHT),
                "S",
                font,
                sender.clone(),
                ButtonVariant::Green,
            )),
            join_lobby: rc_cell!(Button::new(
                2,
                window,
                FloatRect::new(
                    x + PADDING + BUTTON_HEIGHT,
                    40.0 + 600.0 + 10.0,
                    BUTTON_WIDTH,
                    BUTTON_HEIGHT
                ),
                "Join Lobby",
                font,
                sender.clone(),
                ButtonVariant::Green,
            )),
            back: rc_cell!(Button::new(
                3,
                window,
                FloatRect::new(
                    x + PADDING + BUTTON_HEIGHT + PADDING + BUTTON_WIDTH,
                    40.0 + 600.0 + 10.0,
                    BUTTON_WIDTH,
                    BUTTON_HEIGHT
                ),
                "Back",
                font,
                sender.clone(),
                ButtonVariant::Green,
            )),
            mouse_observer: MouseObserver::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
            lobbies_scrollable: rc_cell!(Scrollable::new(
                4,
                window,
                FloatRect::new(WINDOW_SIZE / 2.0 - 300.0, 40.0, 600.0, 600.0),
            )),
            font,
            sender,
        }
    }

    pub fn init(&self) {
        self.mouse_observer.add_observer(self.search.clone());
        self.mouse_observer.add_observer(self.join_lobby.clone());
        self.mouse_observer.add_observer(self.back.clone());
        self.mouse_observer.add_observer(self.lobbies_scrollable.clone());
    }

    fn search(&self) -> Result<()> {
        let state = self.state.borrow();

        let new_lobbies = match get_lobbies_cmd(&state.id, self.range.0, self.range.1) {
            Ok(lobbies) => lobbies,
            Err(e) => {
                check_error(e);
                return Err(anyhow!("get lobbies"));
            }
        };

        let mut lobbies = self.lobbies.borrow_mut();
        let mut lobbies_scrollable = self.lobbies_scrollable.borrow_mut();
        let bounds = lobbies_scrollable.bounds();

        // refresh old lobbies
        let mut index = 0;
        while index < lobbies.len() {
            let lobby = &mut lobbies[index];
            let lobby_addr = LobbyAddr {
                id: lobby.id,
                addr: lobby.addr,
            };

            let lobby_state = match get_lobby_state(lobby_addr) {
                Ok(lobby_state) => LobbyShort {
                    id: lobby.id,
                    addr: lobby.addr,
                    name: lobby_state.name,
                    players: lobby_state.players,
                },
                Err(e) => {
                    check_error(e);
                    lobbies.remove(index);
                    lobbies_scrollable.remove(index);
                    continue;
                }
            };

            *lobby = lobby_state.clone();
            let card = lobbies_scrollable.get(index);
            card.borrow_mut().update(lobby_state);

            index += 1;
        }

        for lobby in new_lobbies {
            // skip old lobbies
            if lobbies.iter().any(|l| l.id == lobby.id) {
                continue;
            }

            let lobby_state = match get_lobby_state(lobby.clone()) {
                Ok(lobby_state) => LobbyShort {
                    id: lobby.id,
                    addr: lobby.addr,
                    name: lobby_state.name,
                    players: lobby_state.players,
                },
                Err(e) => {
                    check_error(e);
                    continue;
                }
            };

            lobbies.push(lobby_state.clone());
            let card = rc_cell!(LobbyCard::new(
                lobby.id as u32,
                self.window,
                lobby_state,
                FloatRect::new(
                    0.0,
                    0.0,
                    bounds.width
                        - Scrollable::<LobbyCard>::SCROLLBAR_WIDTH
                        - 2.0 * Scrollable::<LobbyCard>::PADDING,
                    60.0
                ),
                self.font,
                self.sender.clone(),
            ));
            lobbies_scrollable.add(card.clone());
        }
        Ok(())
    }
}

impl<'a> WindowState for LobbiesWindow<'a> {
    fn as_drawable(&self) -> &dyn Drawable {
        self
    }

    fn enter(&self) -> anyhow::Result<()> {
        self.search()?;
        Ok(())
    }

    fn exit(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<'a> EventHandler for LobbiesWindow<'a> {
    fn handle_event(&self, e: Event) {
        match e.clone() {
            Event::Sfml(sfml::window::Event::MouseButtonPressed { button, x, y }) => {
                if button == mouse::Button::Left {
                    self.mouse_observer.before_click(x, y);
                }
            }
            Event::Sfml(sfml::window::Event::MouseButtonReleased { button, x, y }) => {
                if button == mouse::Button::Left {
                    self.mouse_observer.click(x, y);
                }
            }
            Event::Sfml(sfml::window::Event::MouseMoved { x, y }) => {
                self.mouse_observer.hover(x, y);
            }
            Event::UI(UIEvent::ButtonClicked(event_data)) if event_data.window == self.window => {
                match event_data.id {
                    0 => match self.search() {
                        Ok(_) => {}
                        Err(e) => println!("{e:?}"),
                    },
                    2 => println!("selected: {:?}", self.state.borrow().selected_lobby),
                    _ => {}
                }
            }
            Event::UI(UIEvent::LobbyCardClicked(event_data))
                if event_data.window == self.window =>
            {
                self.state.borrow_mut().selected_lobby = Some(event_data.data);
            }
            // Event::UI(UIEvent::LobbyCardNoClicked(event_data))
            //     if event_data.window == self.window =>
            // {
            //     let mut selected = self.selected.borrow_mut();
            //
            //     if let Some(data) = selected.as_ref() {
            //         if data.id == event_data.data.id {
            //             *selected = None;
            //         }
            //     }
            // }
            _ => {}
        }

        self.lobbies_scrollable.borrow_mut().handle_event(e);
    }
}

impl<'a> Drawable for LobbiesWindow<'a> {
    fn draw<'z: 'shader, 'texture, 'shader, 'shader_texture>(
        &'z self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&*self.search.borrow());
        target.draw(&*self.join_lobby.borrow());
        target.draw(&*self.back.borrow());
        target.draw(&*self.lobbies_scrollable.borrow());
    }
}
