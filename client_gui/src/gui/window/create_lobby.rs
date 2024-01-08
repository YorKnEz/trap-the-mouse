use std::{cell::RefCell, rc::Rc, sync::mpsc};

use sfml::{
    graphics::{Drawable, RcFont},
    window::mouse,
};

use crate::{
    commands::{change_name_cmd, check_error, create_lobby_cmd},
    events::{Event, UIEvent, Window},
    gui::components::{Button, Clicker, EventHandler, Input, EventHandlerMut},
    rc_cell,
    types::{GameStateShared, RcCell, LobbyShort},
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

    input: RcCell<Input>,
    buttons: Vec<RcCell<Button<'a>>>,
    clicker: Clicker<'a>,
}

impl<'a> CreateLobbyWindow<'a> {
    pub fn new(
        window: Window,
        font: &'a RcFont,
        sender: mpsc::Sender<UIEvent>,
        state: GameStateShared,
    ) -> CreateLobbyWindow<'a> {
        let x = WINDOW_SIZE / 2f32 - BUTTON_WIDTH / 2f32;
        let y = WINDOW_SIZE / 2f32 - BUTTON_HEIGHT;
        let offset = BUTTON_HEIGHT + PADDING;

        let mut buttons = vec![];

        let texts = vec!["Create lobby", "Back"];

        for i in 1..=2 {
            buttons.push(rc_cell!(Button::new(
                i,
                window,
                x,
                y + i as f32 * offset,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
                texts[i as usize - 1],
                font,
                sender.clone()
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
                x,
                y + 0f32 * offset,
                BUTTON_WIDTH,
                20f32,
                font,
                "",
                "Lobby name",
                sender.clone(),
            )),
            buttons,
            clicker: Clicker::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
        }
    }

    pub fn init(&self) {
        self.clicker.add_clickable(self.input.clone());
        for button in &self.buttons {
            self.clicker.add_clickable(button.clone());
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
            Event::SFML(sfml::window::Event::MouseButtonReleased { button, x, y }) => {
                if button == mouse::Button::Left {
                    self.clicker.click(x as u32, y as u32);
                }
            }
            Event::UI(UIEvent::InputChanged { value }) => {
                let mut settings = self.settings.borrow_mut();
                settings.name = value.clone();
            }
            Event::UI(UIEvent::ButtonClicked(event_data)) if event_data.window == self.window => {
                match event_data.id {
                    1 => 'create: {
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
                                check_error(e);
                                break 'create;
                            }
                        };

                        state.selected_lobby = Some(lobby);
                    }
                    _ => {}
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
