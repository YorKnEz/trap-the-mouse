mod message;

use std::{cell::RefCell, rc::Rc, sync::mpsc};

use crate::{
    events::{Event, NetworkEvent, UIEvent, Window},
    rc_cell,
    types::RcCell,
    BUTTON_HEIGHT,
};

use message::Message;
use sfml::{
    graphics::{Drawable, FloatRect, RcFont, TextStyle},
    system::Vector2f,
};

use super::{Button, EventHandlerMut, Fixed, Input, MouseObserver, Scrollable};

pub struct Chat<'a> {
    window: Window,
    message: String,
    bounds: FloatRect,
    submit: RcCell<Button<'a>>,
    input: RcCell<Input>,
    messages: Scrollable<'a, Message>,
    font: &'a RcFont,
}

impl<'a> Chat<'a> {
    pub fn new(
        submit_id: u32,
        input_id: u32,
        scrollable_id: u32,
        window: Window,
        bounds: FloatRect,
        font: &'a RcFont,
        sender: mpsc::Sender<UIEvent>,
    ) -> Chat<'a> {
        let mut input = Input::new(
            input_id,
            window,
            FloatRect::new(bounds.left, 0.0, bounds.width - BUTTON_HEIGHT, 0.0),
            16.0,
            font,
            "Your message",
            sender.clone(),
        );
        let input_bounds = input.bounds();
        input.set_position(Vector2f::new(
            bounds.left,
            bounds.top + bounds.height - input_bounds.height,
        ));

        let submit = Button::builder()
            .set_bounds(
                bounds.left + bounds.width - BUTTON_HEIGHT,
                bounds.top + bounds.height - input_bounds.height,
                BUTTON_HEIGHT,
                input_bounds.height,
            )
            .set_border(2.0)
            .set_text("Send")
            .set_font_size(16)
            .set_font_style(TextStyle::REGULAR)
            .build(submit_id, window, sender.clone(), font);

        let messages = Scrollable::new(
            scrollable_id,
            window,
            FloatRect::new(
                bounds.left,
                bounds.top,
                bounds.width,
                bounds.height - input_bounds.height,
            ),
            10.0,
        );

        Chat {
            window,
            message: String::new(),
            bounds,
            submit: rc_cell!(submit),
            input: rc_cell!(input),
            messages,
            font,
        }
    }

    pub fn register_observers(&self, mouse_observer: &MouseObserver<'a>) {
        mouse_observer.add_observer(self.input.clone());
        mouse_observer.add_observer(self.submit.clone());
    }

    pub fn add_message(&mut self, author: String, text: String) {
        self.messages.add(rc_cell!(Message::new(
            FloatRect::new(
                0.0,
                0.0,
                self.bounds.width - 2.0 * self.messages.padding - 20.0,
                0.0
            ),
            author,
            text,
            self.font
        )))
    }

    pub fn get_message(&mut self) -> String {
        let ret = self.message.clone();
        self.message.clear();
        self.input.borrow_mut().set_value(self.message.clone());
        ret
    }
}

impl<'a> EventHandlerMut for Chat<'a> {
    fn handle_event(&mut self, e: crate::events::Event) {
        self.messages.handle_event(e.clone());
        self.input.borrow_mut().handle_event(e.clone());

        match e {
            Event::UI(UIEvent::InputChanged(e)) if e.window == self.window => {
                self.message = e.data;
            }
            Event::Network(NetworkEvent::Message(e)) => {
                self.add_message(e.author, e.text);
            }
            _ => {}
        }
    }
}

impl<'a> Fixed for Chat<'a> {
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    fn position(&self) -> Vector2f {
        (self.bounds.left, self.bounds.top).into()
    }

    fn set_position(&mut self, position: Vector2f) {
        let mut old_pos = self.position();
        let offset = Vector2f::new(position.x - old_pos.x, position.y - old_pos.y);

        self.bounds.left = position.x;
        self.bounds.top = position.y;

        let mut input = self.input.borrow_mut();
        old_pos = input.position();
        input.set_position(Vector2f::new(old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.messages.position();
        self.messages
            .set_position(Vector2f::new(old_pos.x + offset.x, old_pos.y + offset.y));
    }
}

impl<'a> Drawable for Chat<'a> {
    fn draw<'b: 'shader, 'texture, 'shader, 'shader_texture>(
        &'b self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&self.messages);
        target.draw(&*self.input.borrow());
        target.draw(&*self.submit.borrow());
    }
}
