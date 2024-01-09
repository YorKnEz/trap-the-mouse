use std::sync::mpsc;

use sfml::{
    graphics::{
        Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape, TextStyle, Transformable,
    }, system::Vector2f,
};

use crate::events::{EventData, UIEvent, Window};

use super::{clicker::Clickable, Fixed};

pub struct Button<'a> {
    event_data: EventData,
    bounds: FloatRect,
    rect: RectangleShape<'a>,
    text: RcText,
    sender: mpsc::Sender<UIEvent>,
}

impl<'a> Button<'a> {
    const BORDER: f32 = 4f32;

    pub fn new(
        id: u32,
        window: Window,

        left: f32,
        top: f32,
        width: f32,
        height: f32,
        text: &str,
        font: &RcFont,
        sender: mpsc::Sender<UIEvent>,
    ) -> Button<'a> {
        let bounds = FloatRect {
            top,
            left,
            width,
            height,
        };

        let mut rect = RectangleShape::new();
        rect.set_size(Vector2f::new(
            width as f32 - 2f32 * Button::BORDER,
            height as f32 - 2f32 * Button::BORDER,
        ));
        rect.set_position((left + Button::BORDER, top + Button::BORDER));
        rect.set_fill_color(Color::rgb(53, 232, 101));

        rect.set_outline_thickness(Button::BORDER);
        rect.set_outline_color(Color::rgb(45, 168, 78));

        let mut text = RcText::new(text, font, 20);
        text.set_style(TextStyle::BOLD);
        let text_width = text.local_bounds().width;
        let text_height = text.character_size() as f32;

        text.set_position((
            left + width / 2f32 - text_width / 2f32,
            top + height / 2f32 - text_height / 2f32,
        ));

        Button {
            event_data: EventData { window, id },
            bounds,
            rect,
            text,
            sender,
        }
    }
}

impl<'a> Fixed for Button<'a> {
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    fn set_bounds(&mut self, _new_bounds: FloatRect) {
        // don't allow bounds setting
        // self.bounds = new_bounds;
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
        target.draw(&self.rect);
        target.draw(&self.text);
    }
}

impl<'a> Transformable for Button<'a> {
    fn set_position<P: Into<Vector2f>>(&mut self, position: P) {
        let new_pos: sfml::system::Vector2f = position.into();

        let old_pos = self.position();
        let offset = (new_pos.x - old_pos.x, new_pos.y - old_pos.y);

        // update bounds as well
        self.bounds.left = new_pos.x;
        self.bounds.top = new_pos.y;

        self.rect.set_position(new_pos);

        let old_pos = self.text.position();
        self.text
            .set_position((old_pos.x + offset.0, old_pos.y + offset.1));
    }

    fn position(&self) -> Vector2f {
        self.rect.position()
    }

    fn set_rotation(&mut self, angle: f32) {
        todo!()
    }

    fn set_scale<S: Into<Vector2f>>(&mut self, scale: S) {
        todo!()
    }

    fn set_origin<O: Into<Vector2f>>(&mut self, origin: O) {
        todo!()
    }

    fn rotation(&self) -> f32 {
        todo!()
    }

    fn get_scale(&self) -> Vector2f {
        todo!()
    }

    fn origin(&self) -> Vector2f {
        todo!()
    }

    fn move_<O: Into<Vector2f>>(&mut self, offset: O) {
        todo!()
    }

    fn rotate(&mut self, angle: f32) {
        todo!()
    }

    fn scale<F: Into<Vector2f>>(&mut self, factors: F) {
        todo!()
    }

    fn transform(&self) -> &sfml::graphics::Transform {
        todo!()
    }

    fn inverse_transform(&self) -> &sfml::graphics::Transform {
        todo!()
    }
}
