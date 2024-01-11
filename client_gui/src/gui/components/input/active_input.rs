use std::sync::mpsc;

use sfml::{
    graphics::{Drawable, FloatRect, RcText, RectangleShape, Shape, Transformable},
    system::Vector2f,
    window::Key,
};

use crate::{
    events::{Event, EventData, InputChangedEventData, UIEvent},
    gui::components::Fixed,
};

use super::{InactiveInput, InputColors, InputState};

pub struct ActiveInput<'a> {
    pub event_data: EventData,
    pub bounds: FloatRect,
    pub bg: RectangleShape<'a>,
    pub side_bg: [RectangleShape<'a>; 2],
    pub colors: InputColors,

    pub range: (usize, usize),
    pub value: String,
    pub placeholder: String,
    pub copy_text: RcText,
    pub text: RcText,

    cursor: RectangleShape<'a>,
    cursor_pos: usize,

    pub sender: mpsc::Sender<UIEvent>,
}

impl<'a> ActiveInput<'a> {
    const LEFT_PADDING: f32 = 16.0;
    const TOP_PADDING: f32 = 10.0;
    const BORDER: f32 = 2.0;
    const MAX_LEN: usize = 256;

    pub fn from(from: InactiveInput<'a>) -> ActiveInput<'a> {
        let cursor_pos = from.value.len();
        let mut cursor = RectangleShape::new();
        cursor.set_size((
            1.0,
            from.text.character_size() as f32 + 2.0 * ActiveInput::BORDER,
        ));
        cursor.set_position((
            from.text.find_character_pos(cursor_pos).x,
            from.bounds.top + ActiveInput::BORDER + ActiveInput::TOP_PADDING,
        ));
        cursor.set_fill_color(from.colors.cursor);

        ActiveInput {
            event_data: from.event_data,
            bounds: from.bounds,
            bg: from.bg,
            side_bg: from.side_bg,
            colors: from.colors,
            range: from.range,
            value: from.value,
            placeholder: from.placeholder,
            copy_text: from.copy_text,
            text: from.text,
            cursor,
            cursor_pos,
            sender: from.sender,
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            let cursor_pos = self.cursor.position();
            let mut new_left = self.text.find_character_pos(self.cursor_pos).x;

            let left_limit = self.side_bg[0].global_bounds();

            if new_left < left_limit.left + left_limit.width {
                let text_pos = self.text.position();
                self.text.set_position((
                    text_pos.x + left_limit.left + left_limit.width - new_left,
                    text_pos.y,
                ));
                new_left = left_limit.left + left_limit.width;
            }

            self.cursor.set_position((new_left, cursor_pos.y));
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.value.len() {
            self.cursor_pos += 1;
            let cursor_pos = self.cursor.position();
            let mut new_left = self.text.find_character_pos(self.cursor_pos).x;

            let right_limit = self.side_bg[1].global_bounds();

            if new_left > right_limit.left {
                let text_pos = self.text.position();
                self.text
                    .set_position((text_pos.x - (new_left - right_limit.left), text_pos.y));
                new_left = right_limit.left;
            }

            self.cursor.set_position((new_left, cursor_pos.y));
        }
    }

    fn shift(&mut self) {
        let cursor = self.text.find_character_pos(self.value.len()).x;
        let left = self.side_bg[0].global_bounds();
        let limit = (left.left + left.width, self.side_bg[1].global_bounds().left);

        if cursor < limit.1 {
            let pos = self.text.position();
            self.text
                .set_position((pos.x + (limit.1 - cursor).min(limit.0 - pos.x), pos.y));
            let cursor = self.cursor.position();
            self.cursor
                .set_position((self.text.find_character_pos(self.cursor_pos).x, cursor.y));
        }
    }

    fn new_range(&mut self) {
        self.range = (0, self.value.len());

        if !self.value.is_empty() {
            let left = self.side_bg[0].global_bounds();
            let limit = (left.left + left.width, self.side_bg[1].global_bounds().left);

            while self.text.find_character_pos(self.range.0 + 1).x <= limit.0 {
                self.range.0 += 1;
            }

            while self.text.find_character_pos(self.range.1 - 1).x >= limit.1 {
                self.range.1 -= 1;
            }
        }

        self.copy_text
            .set_string(&self.value[self.range.0..self.range.1]);
        let pos = self.copy_text.position();
        self.copy_text
            .set_position((self.text.find_character_pos(self.range.0).x, pos.y));
    }
}

impl InputState for ActiveInput<'static> {
    fn handle_event(mut self: Box<Self>, e: Event) -> Box<dyn InputState> {
        match e {
            Event::UI(UIEvent::InputNoClicked(_)) => {
                return Box::new(InactiveInput::from(*self));
            }
            Event::Sfml(sfml::window::Event::KeyPressed {
                code,
                scan: _,
                alt: _,
                ctrl: _,
                shift: _,
                system: _,
            }) => match code {
                Key::Backspace => {
                    if !(0 < self.cursor_pos && self.cursor_pos <= self.value.len()) {
                        return self;
                    }

                    self.value.remove(self.cursor_pos - 1);
                    self.text.set_string(&self.value);
                    self.move_cursor_left();
                    self.shift();
                    self.new_range();

                    if let Err(e) = self
                        .sender
                        .send(UIEvent::InputChanged(InputChangedEventData {
                            id: self.event_data.id,
                            window: self.event_data.window,
                            data: self.value.clone(),
                        }))
                    {
                        println!("send error: {e:?}");
                    }

                    if self.range.0 == self.range.1 {
                        self.copy_text.set_fill_color(self.colors.placeholder);
                        self.copy_text.set_string(&self.placeholder);
                    }
                }
                Key::Left => {
                    self.move_cursor_left();
                    self.new_range();
                }
                Key::Right => {
                    self.move_cursor_right();
                    self.new_range();
                }
                _ => {}
            },
            Event::Sfml(sfml::window::Event::TextEntered { unicode }) => {
                if !(unicode.is_ascii_alphanumeric() || unicode.is_ascii_punctuation())
                    || self.value.len() >= ActiveInput::MAX_LEN
                {
                    return self;
                }

                self.value.insert(self.cursor_pos, unicode);
                self.text.set_string(&self.value);
                self.move_cursor_right();
                self.new_range();

                if let Err(e) = self
                    .sender
                    .send(UIEvent::InputChanged(InputChangedEventData {
                        id: self.event_data.id,
                        window: self.event_data.window,
                        data: self.value.clone(),
                    }))
                {
                    println!("send error: {e:?}");
                }

                if self.range.0 < self.range.1 {
                    self.copy_text.set_fill_color(self.colors.value);
                }
            }
            _ => {}
        }

        self
    }

    fn get_value(&self) -> String {
        self.value.clone()
    }

    fn set_value(&mut self, value: String) {
        self.value = value;
        while self.cursor_pos > 0 {
            self.move_cursor_left();
        }
        if self.value.is_empty() {
            self.text.set_string(&self.placeholder);
            self.text.set_position((
                self.bounds.left + ActiveInput::BORDER + ActiveInput::LEFT_PADDING,
                self.bounds.top + ActiveInput::BORDER + ActiveInput::TOP_PADDING,
            ));
            self.copy_text.set_string(&self.placeholder);
        } else {
            self.text.set_string(&self.value);
            self.text.set_position((
                self.bounds.left + ActiveInput::BORDER + ActiveInput::LEFT_PADDING,
                self.bounds.top + ActiveInput::BORDER + ActiveInput::TOP_PADDING,
            ));
            while self.cursor_pos < self.value.len() {
                self.move_cursor_right();
            }
            self.new_range();
        }
    }

    fn as_drawable(&self) -> &dyn Drawable {
        self
    }
}

impl<'a> Fixed for ActiveInput<'a> {
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

        old_pos = self.bg.position();
        self.bg
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.side_bg[0].position();
        self.side_bg[0].set_position((old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.side_bg[1].position();
        self.side_bg[1].set_position((old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.text.position();
        self.text
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.copy_text.position();
        self.copy_text
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.cursor.position();
        self.cursor
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));
    }
}

impl<'b> Drawable for ActiveInput<'b> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&self.bg);
        target.draw(&self.copy_text);
        target.draw(&self.side_bg[0]);
        target.draw(&self.side_bg[1]);
        target.draw(&self.cursor);
    }
}
