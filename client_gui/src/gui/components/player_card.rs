use std::sync::mpsc;

use sfml::graphics::{
    Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape, Transformable,
};

use crate::{
    events::{Event, PlayerCardEventData, UIEvent, Window},
    types::Player,
};

use super::{Clickable, EventHandlerMut, Fixed};

pub struct PlayerCard<'a> {
    event_data: PlayerCardEventData,

    bounds: FloatRect,
    bg: RectangleShape<'a>,
    name: RcText,
    user_type: RcText,

    selected: bool,
    sender: mpsc::Sender<UIEvent>,
}

impl<'a> PlayerCard<'a> {
    const LEFT_PADDING: f32 = 10f32;
    const TOP_PADDING: f32 = 4f32;
    const COLOR_NOT_SELECTED: Color = Color::rgb(97, 97, 97);
    const COLOR_SELECTED: Color = Color::rgb(117, 117, 117);

    pub fn new(
        id: u32,
        window: Window,
        data: Player,
        width: f32,
        height: f32,
        font: &RcFont,
        sender: mpsc::Sender<UIEvent>,
    ) -> PlayerCard<'a> {
        let bounds = FloatRect {
            left: 0f32,
            top: 0f32,
            width,
            height,
        };

        let mut bg = RectangleShape::new();
        bg.set_size((width, height));
        bg.set_position((bounds.left, bounds.top));
        bg.set_fill_color(PlayerCard::COLOR_NOT_SELECTED);

        let mut name = RcText::new(&data.name, font, 24);
        name.set_position((
            bounds.left + PlayerCard::LEFT_PADDING,
            bounds.top + PlayerCard::TOP_PADDING,
        ));
        name.set_fill_color(Color::WHITE);

        let mut user_type = RcText::new(&format!("{:?}", data.user_type), font, 16);
        let text_height = user_type.character_size() as f32;
        user_type.set_position((
            bounds.left + PlayerCard::LEFT_PADDING,
            bounds.top + bounds.height - PlayerCard::LEFT_PADDING - text_height,
        ));
        user_type.set_fill_color(Color::rgb(200, 200, 200));

        // shrink lobby name text so it doesnt overflow
        let mut i = data.name.len();
        if name.find_character_pos(i).x > bounds.left + bounds.width {
            let mut new_buf = data.name.clone() + "...";
            name.set_string(&new_buf);
            let dots_width = name.find_character_pos(i + 3).x - name.find_character_pos(i).x;

            while name.find_character_pos(i).x + dots_width > bounds.left + bounds.width {
                if i > 0 {
                    new_buf.remove(i - 1);
                    i -= 1;
                } else {
                    break;
                }
            }

            name.set_string(&new_buf);
        }

        PlayerCard {
            event_data: PlayerCardEventData { id, window, data },
            bounds,
            bg,
            name,
            user_type,
            selected: false,
            sender,
        }
    }

    pub fn update(&mut self, data: Player) {
        self.name.set_string(&data.name);
        self.name.set_position((
            self.bounds.left + PlayerCard::LEFT_PADDING,
            self.bounds.top + PlayerCard::TOP_PADDING,
        ));
        self.name.set_fill_color(Color::WHITE);

        self.user_type.set_string(&format!("{:?}", data.user_type));
        let text_height = self.user_type.character_size() as f32;
        self.user_type.set_position((
            self.bounds.left + PlayerCard::LEFT_PADDING,
            self.bounds.top + self.bounds.height - PlayerCard::LEFT_PADDING - text_height,
        ));
        self.user_type.set_fill_color(Color::rgb(200, 200, 200));

        // shrink lobby self.name text so it doesnt overflow
        let mut i = data.name.len();
        if self.name.find_character_pos(i).x > self.bounds.left + self.bounds.width {
            let mut new_buf = data.name.clone() + "...";
            self.name.set_string(&new_buf);
            let dots_width =
                self.name.find_character_pos(i + 3).x - self.name.find_character_pos(i).x;

            while self.name.find_character_pos(i).x + dots_width
                > self.bounds.left + self.bounds.width
            {
                if i > 0 {
                    new_buf.remove(i - 1);
                    i -= 1;
                } else {
                    break;
                }
            }

            self.name.set_string(&new_buf);
        }
    }
}

impl<'a> EventHandlerMut for PlayerCard<'a> {
    fn handle_event(&mut self, e: Event) {
        match e {
            _ => {}
        }
    }
}

impl<'a> Clickable for PlayerCard<'a> {
    fn get_id(&self) -> u32 {
        self.event_data.id
    }

    fn click(&mut self, _x: u32, _y: u32) {
        self.selected = !self.selected;

        if self.selected {
            self.bg.set_fill_color(PlayerCard::COLOR_SELECTED);
        } else {
            self.bg.set_fill_color(PlayerCard::COLOR_NOT_SELECTED);
        }

        if let Err(e) = self
            .sender
            .send(UIEvent::PlayerCardClicked(self.event_data.clone()))
        {
            println!("send error: {e:?}");
        }
    }

    fn no_click(&mut self) {
        self.selected = false;
        self.bg.set_fill_color(PlayerCard::COLOR_NOT_SELECTED);

        // if let Err(e) = self
        //     .sender
        //     .send(UIEvent::PlayerCardNoClicked(self.event_data.clone()))
        // {
        //     println!("send error: {e:?}");
        // }
    }
}

impl<'a> Fixed for PlayerCard<'a> {
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: FloatRect) {
        self.bounds = new_bounds;
        self.set_position((self.bounds.left, self.bounds.top));
    }
}

impl<'a> Drawable for PlayerCard<'a> {
    fn draw<'b: 'shader, 'texture, 'shader, 'shader_texture>(
        &'b self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&self.bg);
        target.draw(&self.name);
        target.draw(&self.user_type);
    }
}

impl<'a> Transformable for PlayerCard<'a> {
    fn set_position<P: Into<sfml::system::Vector2f>>(&mut self, position: P) {
        let new_pos: sfml::system::Vector2f = position.into();
        let old_pos = self.position();
        let offset = (new_pos.x - old_pos.x, new_pos.y - old_pos.y);

        self.bg.set_position(new_pos);

        let old_pos = self.name.position();
        self.name
            .set_position((old_pos.x + offset.0, old_pos.y + offset.1));

        let old_pos = self.user_type.position();
        self.user_type
            .set_position((old_pos.x + offset.0, old_pos.y + offset.1));
    }

    fn position(&self) -> sfml::system::Vector2f {
        self.bg.position()
    }

    fn set_rotation(&mut self, angle: f32) {
        todo!()
    }

    fn set_scale<S: Into<sfml::system::Vector2f>>(&mut self, scale: S) {
        todo!()
    }

    fn set_origin<O: Into<sfml::system::Vector2f>>(&mut self, origin: O) {
        todo!()
    }

    fn rotation(&self) -> f32 {
        todo!()
    }

    fn get_scale(&self) -> sfml::system::Vector2f {
        todo!()
    }

    fn origin(&self) -> sfml::system::Vector2f {
        todo!()
    }

    fn move_<O: Into<sfml::system::Vector2f>>(&mut self, offset: O) {
        todo!()
    }

    fn rotate(&mut self, angle: f32) {
        todo!()
    }

    fn scale<F: Into<sfml::system::Vector2f>>(&mut self, factors: F) {
        todo!()
    }

    fn transform(&self) -> &sfml::graphics::Transform {
        todo!()
    }

    fn inverse_transform(&self) -> &sfml::graphics::Transform {
        todo!()
    }
}
