use std::{cell::RefCell, rc::Rc, sync::mpsc};

use sfml::{
    graphics::{Drawable, RcFont},
    window::mouse,
};

use super::WindowState;
use crate::{
    events::{Event, UIEvent, Window},
    gui::components::{Button, Clicker, EventHandler},
    rc_cell,
    types::{GameStateShared, RcCell},
    BUTTON_HEIGHT, BUTTON_WIDTH, PADDING, WINDOW_SIZE
};

pub struct StartWindow<'a> {
    window: Window,
    state: GameStateShared,

    buttons: Vec<RcCell<Button<'a>>>,
    clicker: Clicker<'a>,
}

impl<'a> StartWindow<'a> {
    pub fn new(
        window: Window,
        font: &'a RcFont,
        sender: mpsc::Sender<UIEvent>,
        state: GameStateShared,
    ) -> StartWindow<'a> {
        let x = WINDOW_SIZE / 2f32 - BUTTON_WIDTH / 2f32;
        let y = WINDOW_SIZE / 2f32 - BUTTON_HEIGHT;
        let offset = BUTTON_HEIGHT + PADDING;

        let mut buttons = vec![];

        let texts = vec!["Create lobby", "Join lobby", "Settings", "Exit"];

        for i in 1..=4 {
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

        StartWindow {
            window,
            state,
            buttons,
            clicker: Clicker::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32),
        }
    }

    pub fn init(&self) {
        for button in &self.buttons {
            self.clicker.add_clickable(button.clone());
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
            Event::SFML(sfml::window::Event::MouseButtonReleased { button, x, y }) => {
                if button == mouse::Button::Left {
                    self.clicker.click(x as u32, y as u32);
                }
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
