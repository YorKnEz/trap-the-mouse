use std::{cell::RefCell, rc::Rc, sync::mpsc};

use sfml::{
    graphics::{Drawable, FloatRect, RcFont},
    window::mouse,
};

use crate::{
    commands::{check_error, create_lobby_cmd},
    events::{Event, UIEvent, Window},
    gui::components::{Button, ButtonVariant, EventHandler, EventHandlerMut, Input, MouseObserver},
    rc_cell,
    types::{GameStateShared, LobbyShort, RcCell},
    BUTTON_HEIGHT, BUTTON_WIDTH, DEFAULT_NAME, PADDING, WINDOW_SIZE,
};

use super::WindowState;

struct Settings {
    name: String,
}

pub struct CreateLobbyWindow<'a> {
    window: Window,
    state: GameStateShared,
    settings: RefCell<Settings>,
    sender: mpsc::Sender<UIEvent>,

    input: RcCell<Input>,
    buttons: Vec<RcCell<Button<'a>>>,
    mouse_observer: MouseObserver<'a>,
}

impl<'a> CreateLobbyWindow<'a> {
    pub fn new(
        window: Window,
        font: &'a RcFont,
        sender: mpsc::Sender<UIEvent>,
        state: GameStateShared,
    ) -> CreateLobbyWindow<'a> {
        let x = WINDOW_SIZE / 2.0 - BUTTON_WIDTH / 2.0;
        let y = WINDOW_SIZE / 2.0 - BUTTON_HEIGHT;
        let offset = BUTTON_HEIGHT + PADDING;

        let mut buttons = vec![];

        let texts = ["Create lobby", "Back"];

        for i in 1..=2 {
            buttons.push(rc_cell!(Button::new(
                i,
                window,
                FloatRect::new(x, y + i as f32 * offset, BUTTON_WIDTH, BUTTON_HEIGHT),
                texts[i as usize - 1],
                font,
                sender.clone(),
                ButtonVariant::Green,
            )));
        }

        CreateLobbyWindow {
            window,
            state,
            settings: RefCell::new(Settings {
                name: String::from(DEFAULT_NAME),
            }),
            input: rc_cell!(Input::new(
                0,
                window,
                FloatRect::new(x, y + 0.0 * offset, BUTTON_WIDTH, 0.0),
                20.0,
                font,
                "Lobby name",
                sender.clone(),
            )),
            buttons,
            mouse_observer: MouseObserver::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
            sender,
        }
    }

    pub fn init(&self) {
        self.mouse_observer.add_observer(self.input.clone());
        for button in &self.buttons {
            self.mouse_observer.add_observer(button.clone());
        }
    }
}

impl<'a> WindowState for CreateLobbyWindow<'a> {
    fn as_drawable(&self) -> &dyn Drawable {
        self
    }

    fn enter(&self) -> anyhow::Result<()> {
        let mut settings = self.settings.borrow_mut();
        settings.name.clear();

        let mut input = self.input.borrow_mut();
        input.set_value(settings.name.clone());

        Ok(())
    }

    fn exit(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<'a> EventHandler for CreateLobbyWindow<'a> {
    fn handle_event(&self, e: Event) {
        self.input.borrow_mut().handle_event(e.clone());

        match e {
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
            Event::UI(UIEvent::InputChanged(e)) if e.window == self.window => {
                let mut settings = self.settings.borrow_mut();
                settings.name = e.data;
            }
            Event::UI(UIEvent::ButtonClicked(e)) if e.window == self.window => {
                if e.id == 1 {
                    'create: {
                        let mut state = self.state.borrow_mut();
                        let settings = self.settings.borrow();
                        let lobby = match create_lobby_cmd(&state.id, settings.name.clone()) {
                            Ok(lobby) => LobbyShort {
                                id: lobby.id,
                                addr: lobby.addr,
                                name: settings.name.clone(),
                                players: 0,
                            },
                            Err(e) => {
                                if let Err(e) = self.sender.send(UIEvent::Error(check_error(e))) {
                                    println!("send error: {e:?}");
                                }
                                break 'create;
                            }
                        };

                        state.selected_lobby = Some(lobby);
                    }
                }
            }
            _ => {}
        }
    }
}

impl<'a> Drawable for CreateLobbyWindow<'a> {
    fn draw<'z: 'shader, 'texture, 'shader, 'shader_texture>(
        &'z self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&*self.input.borrow());
        for button in &self.buttons {
            target.draw(&*button.borrow());
        }
    }
}
