use std::{cell::RefCell, rc::Rc, sync::mpsc};

use sfml::{
    graphics::{Drawable, RcFont},
    window::mouse,
};

use super::WindowState;
use crate::{
    events::{Event, UIEvent, Window},
    gui::components::{Button, EventHandler, MouseObserver},
    rc_cell,
    types::RcCell,
    BUTTON_HEIGHT, BUTTON_WIDTH, PADDING, WINDOW_SIZE,
};

pub struct StartWindow<'a> {
    // window: Window,
    // state: GameStateShared,
    buttons: Vec<RcCell<Button<'a>>>,
    mouse_observer: MouseObserver<'a>,
}

impl<'a> StartWindow<'a> {
    pub fn new(
        window: Window,
        font: &'a RcFont,
        sender: mpsc::Sender<UIEvent>,
        // state: GameStateShared,
    ) -> StartWindow<'a> {
        let x = WINDOW_SIZE / 2.0 - BUTTON_WIDTH / 2.0;
        let y = WINDOW_SIZE / 2.0 - BUTTON_HEIGHT;
        let offset = BUTTON_HEIGHT + PADDING;

        let mut buttons = vec![];
        let texts = ["Create lobby", "Join lobby", "Settings", "Exit"];

        for (i, text) in texts.into_iter().enumerate() {
            buttons.push(rc_cell!(Button::builder()
                .set_position(x, y + (i + 1) as f32 * offset)
                .set_text(text)
                .build(i as u32 + 1, window, sender.clone(), font)));
        }

        StartWindow {
            // window,
            // state,
            buttons,
            mouse_observer: MouseObserver::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
        }
    }

    pub fn init(&self) {
        for button in &self.buttons {
            self.mouse_observer.add_observer(button.clone());
        }
    }
}

impl<'a> WindowState for StartWindow<'a> {
    fn as_drawable(&self) -> &dyn Drawable {
        self
    }

    fn enter(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn exit(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<'a> EventHandler for StartWindow<'a> {
    fn handle_event(&self, e: Event) {
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
            _ => {}
        }
    }
}

impl<'a> Drawable for StartWindow<'a> {
    fn draw<'z: 'shader, 'texture, 'shader, 'shader_texture>(
        &'z self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        for button in &self.buttons {
            target.draw(&*button.borrow());
        }
    }
}
