use std::sync::mpsc;

use sfml::graphics::{
    Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape, Transformable,
};

use crate::{
    events::{Event, LobbyCardEventData, UIEvent, Window},
    types::LobbyShort,
};

use super::{Clickable, EventHandlerMut, Fixed};

pub struct LobbyCard<'a> {
    event_data: LobbyCardEventData,

    bounds: FloatRect,
    bg: RectangleShape<'a>,
    name: RcText,
    players: RcText,

    selected: bool,
    sender: mpsc::Sender<UIEvent>,
}

impl<'a> LobbyCard<'a> {
    const PADDING: f32 = 10f32;
    const COLOR_NOT_SELECTED: Color = Color::rgb(97, 97, 97);
    const COLOR_SELECTED: Color = Color::rgb(117, 117, 117);

    pub fn new(
        id: u32,
        window: Window,
        data: LobbyShort,
        width: f32,
        height: f32,
        font: &RcFont,
        sender: mpsc::Sender<UIEvent>,
    ) -> LobbyCard<'a> {
        let bounds = FloatRect {
            left: 0f32,
            top: 0f32,
            width,
            height,
        };

        let mut bg = RectangleShape::new();
        bg.set_size((width, height));
        bg.set_position((bounds.left, bounds.top));
        bg.set_fill_color(LobbyCard::COLOR_NOT_SELECTED);

        let mut name = RcText::new(&data.name, font, 32);
        let text_height = name.character_size() as f32;
        name.set_position((
            bounds.left + LobbyCard::PADDING,
            bounds.top + height / 2f32 - text_height / 2f32,
        ));

        let mut players = RcText::new(&format!("Players: {}", data.players), font, 32);
        let text_width = players.local_bounds().width;
        let text_height = players.character_size() as f32;
        players.set_position((
            bounds.left + width - LobbyCard::PADDING - text_width,
            bounds.top + height / 2f32 - text_height / 2f32,
        ));

        // shrink lobby name text so it doesnt overlap
        let mut i = data.name.len();
        if name.find_character_pos(i).x > players.find_character_pos(0).x {
            let mut new_buf = data.name.clone() + "...";
            name.set_string(&new_buf);
            let dots_width = name.find_character_pos(i + 3).x - name.find_character_pos(i).x;

            while name.find_character_pos(i).x + dots_width > players.find_character_pos(0).x {
                if i > 0 {
                    new_buf.remove(i - 1);
                    i -= 1;
                } else {
                    break;
                }
            }

            name.set_string(&new_buf);
        }

        LobbyCard {
            event_data: LobbyCardEventData { id, window, data },
            bounds,
            bg,
            name,
            players,
            selected: false,
            sender,
        }
    }

    pub fn update(&mut self, data: LobbyShort) {
        self.name.set_string(&data.name);
        let text_height = self.name.character_size() as f32;
        self.name.set_position((
            self.bounds.left + LobbyCard::PADDING,
            self.bounds.top + self.bounds.height / 2f32 - text_height / 2f32,
        ));

        self.players
            .set_string(&format!("Players: {}", data.players));
        let text_width = self.players.local_bounds().width;
        let text_height = self.players.character_size() as f32;
        self.players.set_position((
            self.bounds.left + self.bounds.width - LobbyCard::PADDING - text_width,
            self.bounds.top + self.bounds.height / 2f32 - text_height / 2f32,
        ));

        // shrink lobby name text so it doesnt overlap
        let mut i = data.name.len();
        if self.name.find_character_pos(i).x > self.players.find_character_pos(0).x {
            let mut new_buf = data.name.clone() + "...";
            self.name.set_string(&new_buf);
            let dots_width =
                self.name.find_character_pos(i + 3).x - self.name.find_character_pos(i).x;

            while self.name.find_character_pos(i).x + dots_width
                > self.players.find_character_pos(0).x
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

impl<'a> EventHandlerMut for LobbyCard<'a> {
    fn handle_event(&mut self, e: Event) {
        match e {
            _ => {}
        }
    }
}

impl<'a> Clickable for LobbyCard<'a> {
    fn click(&mut self, _x: u32, _y: u32) {
        self.selected = !self.selected;

        if self.selected {
            self.bg.set_fill_color(LobbyCard::COLOR_SELECTED);
        } else {
            self.bg.set_fill_color(LobbyCard::COLOR_NOT_SELECTED);
        }

        if let Err(e) = self
            .sender
            .send(UIEvent::LobbyCardClicked(self.event_data.clone()))
        {
            println!("send error: {e:?}");
        }
    }

    fn no_click(&mut self) {
        self.selected = false;
        self.bg.set_fill_color(LobbyCard::COLOR_NOT_SELECTED);

        // if let Err(e) = self
        //     .sender
        //     .send(UIEvent::LobbyCardNoClicked(self.event_data.clone()))
        // {
        //     println!("send error: {e:?}");
        // }
    }
}

impl<'a> Fixed for LobbyCard<'a> {
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: FloatRect) {
        self.bounds = new_bounds;
        self.set_position((self.bounds.left, self.bounds.top));
    }
}

impl<'a> Drawable for LobbyCard<'a> {
    fn draw<'b: 'shader, 'texture, 'shader, 'shader_texture>(
        &'b self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&self.bg);
        target.draw(&self.name);
        target.draw(&self.players);
    }
}

impl<'a> Transformable for LobbyCard<'a> {
    fn set_position<P: Into<sfml::system::Vector2f>>(&mut self, position: P) {
        let new_pos: sfml::system::Vector2f = position.into();
        let old_pos = self.position();
        let offset = (new_pos.x - old_pos.x, new_pos.y - old_pos.y);

        self.bg.set_position(new_pos);

        let old_pos = self.name.position();
        self.name
            .set_position((old_pos.x + offset.0, old_pos.y + offset.1));

        let old_pos = self.players.position();
        self.players
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
