use std::sync::mpsc;

use sfml::{
    graphics::{
        Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape, TextStyle, Transformable,
    },
    system::Vector2f,
};

use crate::events::{EventData, UIEvent, Window};

use super::{clicker::Clickable, Fixed};

pub struct Button<'a> {
    event_data: EventData,
    bounds: FloatRect,
    bg: RectangleShape<'a>,
    text: RcText,
    sender: mpsc::Sender<UIEvent>,
}

impl<'a> Button<'a> {
    const BORDER: f32 = 4.0;

    pub fn new(
        id: u32,
        window: Window,
        bounds: FloatRect,
        text: &str,
        font: &RcFont,
        sender: mpsc::Sender<UIEvent>,
    ) -> Button<'a> {
        let mut bg = RectangleShape::new();
        bg.set_size(Vector2f::new(
            bounds.width - 2.0 * Button::BORDER,
            bounds.height - 2.0 * Button::BORDER,
        ));
        bg.set_position((bounds.left + Button::BORDER, bounds.top + Button::BORDER));
        bg.set_fill_color(Color::rgb(53, 232, 101));

        bg.set_outline_thickness(Button::BORDER);
        bg.set_outline_color(Color::rgb(45, 168, 78));

        let mut text = RcText::new(text, font, 20);
        text.set_style(TextStyle::BOLD);
        let text_width = text.local_bounds().width;
        let text_height = text.character_size() as f32;

        text.set_position((
            bounds.left + bounds.width / 2.0 - text_width / 2.0,
            bounds.top + bounds.height / 2.0 - text_height / 2.0,
        ));

        Button {
            event_data: EventData { window, id },
            bounds,
            bg,
            text,
            sender,
        }
    }
}

impl<'a> Fixed for Button<'a> {
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    // fn set_bounds(&mut self, _new_bounds: FloatRect) {
    //     // don't allow bounds setting
    //     // self.bounds = new_bounds;
    // }

    fn position(&self) -> Vector2f {
        (self.bounds.left, self.bounds.top).into()
    }

    fn set_position(&mut self, position: Vector2f) {
        let mut old_pos = self.position();
        let offset = Vector2f::new(position.x - old_pos.x, position.y - old_pos.y);

        self.bounds.left = position.x;
        self.bounds.top = position.y;

        old_pos = self.bg.position();
        self.bg
            .set_position(Vector2f::new(old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.text.position();
        self.text
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));
    }
}

impl<'a> Clickable for Button<'a> {
    fn get_id(&self) -> u32 {
        self.event_data.id
    }

    fn click(&mut self, _x: u32, _y: u32) {
        if let Err(e) = self.sender.send(UIEvent::ButtonClicked(self.event_data)) {
            println!("send error: {e:?}");
        }
    }

    fn no_click(&mut self) {}
}

impl<'a> Drawable for Button<'a> {
    fn draw<'b: 'shader, 'texture, 'shader, 'shader_texture>(
        &'b self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&self.bg);
        target.draw(&self.text);
    }
}
