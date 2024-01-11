use std::sync::mpsc;

use sfml::{
    graphics::{Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape, Transformable},
    system::Vector2f,
};

use crate::{
    events::{Event, PlayerCardEventData, UIEvent, Window},
    types::{Player, UserType},
};

use super::{EventHandlerMut, Fixed, MouseEventObserver};

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
    const LEFT_PADDING: f32 = 10.0;
    const TOP_PADDING: f32 = 4.0;

    const COLOR_NOT_SELECTED: Color = Color::rgb(97, 97, 97);
    const COLOR_HOVERED: Color = Color::rgb(107, 107, 107);
    const COLOR_SELECTED: Color = Color::rgb(117, 117, 117);

    const HOST_COLOR: Color = Color::rgb(235, 64, 52);
    const PLAYER_COLOR: Color = Color::rgb(52, 235, 226);
    const SPECTATOR_COLOR: Color = Color::WHITE;

    pub fn new(
        id: u32,
        window: Window,
        data: Player,
        bounds: FloatRect,
        font: &RcFont,
        sender: mpsc::Sender<UIEvent>,
    ) -> PlayerCard<'a> {
        let mut bg = RectangleShape::new();
        bg.set_size((bounds.width, bounds.height));
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
        match data.user_type {
            UserType::Host => user_type.set_fill_color(PlayerCard::HOST_COLOR),
            UserType::Player => user_type.set_fill_color(PlayerCard::PLAYER_COLOR),
            UserType::Spectator => user_type.set_fill_color(PlayerCard::SPECTATOR_COLOR),
        }

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
        match data.user_type {
            UserType::Host => self.user_type.set_fill_color(PlayerCard::HOST_COLOR),
            UserType::Player => self.user_type.set_fill_color(PlayerCard::PLAYER_COLOR),
            UserType::Spectator => self.user_type.set_fill_color(PlayerCard::SPECTATOR_COLOR),
        }

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
    fn handle_event(&mut self, _e: Event) {}
}

impl<'a> MouseEventObserver for PlayerCard<'a> {
    fn get_id(&self) -> u32 {
        self.event_data.id
    }

    fn before_click(&mut self, _x: u32, _y: u32) {}

    fn click(&mut self, _x: u32, _y: u32) {
        self.selected = true;
        self.bg.set_fill_color(PlayerCard::COLOR_SELECTED);

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
    }

    fn hover(&mut self, _x: u32, _y: u32) {
        if !self.selected {
            self.bg.set_fill_color(PlayerCard::COLOR_HOVERED);
        }
    }

    fn no_hover(&mut self) {
        if !self.selected {
            self.bg.set_fill_color(PlayerCard::COLOR_NOT_SELECTED);
        }
    }
}

impl<'a> Fixed for PlayerCard<'a> {
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    // fn set_bounds(&mut self, new_bounds: FloatRect) {
    //     self.bounds = new_bounds;
    //     self.set_position((self.bounds.left, self.bounds.top));
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
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.name.position();
        self.name
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.user_type.position();
        self.user_type
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));
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
