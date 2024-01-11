mod active_input;
mod inactive_input;

use std::sync::mpsc;

use crate::events::{Event, EventData, UIEvent, Window};

use active_input::ActiveInput;
use inactive_input::InactiveInput;
use sfml::graphics::{Drawable, FloatRect, RcFont};

use super::{EventHandlerMut, Fixed, MouseEventObserver};

pub trait InputState: Fixed {
    fn handle_event(self: Box<Self>, e: Event) -> Box<dyn InputState>;
    fn get_value(&self) -> String;
    fn set_value(&mut self, value: String);
    fn as_drawable(&self) -> &dyn Drawable;
}

pub struct Input {
    event_data: EventData,
    sender: mpsc::Sender<UIEvent>,
    state: Option<Box<dyn InputState>>,
}

impl Input {
    pub fn new(
        id: u32,
        window: Window,
        bounds: FloatRect,
        text_height: f32,
        font: &RcFont,
        placeholder: &str,
        sender: mpsc::Sender<UIEvent>,
    ) -> Input {
        Input {
            event_data: EventData { window, id },
            state: Some(Box::new(InactiveInput::new(
                bounds,
                text_height,
                font,
                "",
                placeholder,
                sender.clone(),
            ))),
            sender,
        }
    }

    pub fn set_value(&mut self, value: String) {
        self.state.as_mut().unwrap().set_value(value);
    }
}

impl EventHandlerMut for Input {
    fn handle_event(&mut self, e: Event) {
        match e {
            Event::UI(UIEvent::InputClicked(event_data)) => {
                if event_data != self.event_data {
                    return;
                }
            }
            Event::UI(UIEvent::InputNoClicked(event_data)) => {
                if event_data != self.event_data {
                    return;
                }
            }
            _ => {}
        }

        if let Some(state_) = self.state.take() {
            self.state = Some(state_.handle_event(e));
        }
    }
}

impl MouseEventObserver for Input {
    fn get_id(&self) -> u32 {
        self.event_data.id
    }

    fn before_click(&mut self, _x: u32, _y: u32) {}

    fn click(&mut self, _x: u32, _y: u32) {
        if let Err(e) = self.sender.send(UIEvent::InputClicked(self.event_data)) {
            println!("send error: {e:?}");
        }
    }

    fn no_click(&mut self) {
        if let Err(e) = self.sender.send(UIEvent::InputNoClicked(self.event_data)) {
            println!("send error: {e:?}");
        }
    }

    fn hover(&mut self, _x: u32, _y: u32) {}
    fn no_hover(&mut self) {}
}

impl Fixed for Input {
    fn bounds(&self) -> FloatRect {
        self.state.as_ref().unwrap().bounds()
    }

    // fn set_bounds(&mut self, new_bounds: FloatRect) {
    //     self.state.as_mut().unwrap().set_bounds(new_bounds);
    // }

    fn position(&self) -> sfml::system::Vector2f {
        self.state.as_ref().unwrap().position()
    }

    fn set_position(&mut self, position: sfml::system::Vector2f) {
        self.state.as_mut().unwrap().set_position(position);
    }
}

impl Drawable for Input {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(self.state.as_ref().unwrap().as_drawable());
    }
}
