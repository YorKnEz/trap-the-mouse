use std::sync::mpsc;

use sfml::graphics::{
    Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape, Transformable,
};

use crate::{
    events::{Event, EventData, UIEvent, Window},
    gui::components::Fixed,
};

use super::{ActiveInput, InputState};

pub struct InactiveInput<'a> {
    pub event_data: EventData,
    pub bounds: FloatRect,
    pub bg: RectangleShape<'a>,
    pub side_bg: [RectangleShape<'a>; 2],

    pub range: (usize, usize),
    pub buf: String,
    pub placeholder: String,
    pub copy_text: RcText,
    pub text: RcText,
    pub sender: mpsc::Sender<UIEvent>,
}

impl<'a> InactiveInput<'a> {
    const LEFT_PADDING: f32 = 16.0;
    const TOP_PADDING: f32 = 10.0;
    const BORDER: f32 = 2.0;

    pub fn new(
        id: u32,
        window: Window,
        bounds: FloatRect,
        text_height: f32,
        font: &RcFont,
        value: &str,
        placeholder: &str,
        sender: mpsc::Sender<UIEvent>,
    ) -> InactiveInput<'a> {
        let bounds = FloatRect {
            height: text_height + 2.0 * InactiveInput::TOP_PADDING + 2.0 * InactiveInput::BORDER,
            ..bounds
        };

        let mut bg = RectangleShape::new();
        bg.set_size((
            bounds.width - 2.0 * InactiveInput::BORDER,
            bounds.height - 2.0 * InactiveInput::BORDER,
        ));
        bg.set_position((
            bounds.left + InactiveInput::BORDER,
            bounds.top + InactiveInput::BORDER,
        ));
        bg.set_fill_color(Color::rgb(238, 238, 238));

        bg.set_outline_thickness(InactiveInput::BORDER);
        bg.set_outline_color(Color::rgb(190, 190, 190));

        let mut side_bg = RectangleShape::new();
        side_bg.set_size((
            InactiveInput::LEFT_PADDING,
            bounds.height - 2.0 * InactiveInput::BORDER,
        ));
        side_bg.set_position((
            bounds.left + InactiveInput::BORDER,
            bounds.top + InactiveInput::BORDER,
        ));
        side_bg.set_fill_color(Color::rgb(238, 238, 238));

        let mut side_bg = [side_bg.clone(), side_bg];
        side_bg[1].set_position((
            bounds.left + bounds.width - InactiveInput::BORDER - InactiveInput::LEFT_PADDING,
            bounds.top + InactiveInput::BORDER,
        ));

        let mut text = if value.is_empty() {
            RcText::new(placeholder, font, text_height as u32)
        } else {
            RcText::new(value, font, text_height as u32)
        };

        text.set_position((
            bounds.left + InactiveInput::BORDER + InactiveInput::LEFT_PADDING,
            bounds.top + InactiveInput::BORDER + InactiveInput::TOP_PADDING,
        ));
        text.set_fill_color(Color::rgb(45, 45, 45));

        let mut cursor = RectangleShape::new();
        cursor.set_size((1.0, text_height + 2.0 * InactiveInput::BORDER));
        cursor.set_position((
            bounds.left + InactiveInput::BORDER + InactiveInput::LEFT_PADDING,
            bounds.top + InactiveInput::BORDER + InactiveInput::TOP_PADDING,
        ));
        cursor.set_fill_color(Color::BLACK);

        InactiveInput {
            event_data: EventData { window, id },
            bounds,
            bg,
            side_bg,
            range: (0, 0),
            buf: String::from(value),
            placeholder: String::from(placeholder),
            copy_text: text.clone(),
            text,
            sender,
        }
    }

    pub fn from(from: ActiveInput<'a>) -> InactiveInput<'a> {
        InactiveInput {
            event_data: from.event_data,
            bounds: from.bounds,
            bg: from.bg,
            side_bg: from.side_bg,
            range: from.range,
            buf: from.buf,
            placeholder: from.placeholder,
            copy_text: from.copy_text,
            text: from.text,
            sender: from.sender,
        }
    }

    fn new_range(&mut self) {
        self.range = (0, self.buf.len());

        if !self.buf.is_empty() {
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
            .set_string(&self.buf[self.range.0..self.range.1]);
        let pos = self.copy_text.position();
        self.copy_text
            .set_position((self.text.find_character_pos(self.range.0).x, pos.y));
    }
}

impl InputState for InactiveInput<'static> {
    fn handle_event(self: Box<Self>, e: Event) -> Box<dyn InputState> {
        if let Event::UI(UIEvent::InputClicked(_)) = e {
            return Box::new(ActiveInput::from(*self));
        }

        self
    }

    fn get_value(&self) -> String {
        self.buf.clone()
    }

    fn set_value(&mut self, value: String) {
        self.buf = value;
        if self.buf.is_empty() {
            self.text.set_string(&self.placeholder);
            self.text.set_position((
                self.bounds.left + InactiveInput::BORDER + InactiveInput::LEFT_PADDING,
                self.bounds.top + InactiveInput::BORDER + InactiveInput::TOP_PADDING,
            ));
            self.copy_text.set_string(&self.placeholder);
        } else {
            self.text.set_string(&self.buf);
            self.text.set_position((
                self.bounds.left + InactiveInput::BORDER + InactiveInput::LEFT_PADDING,
                self.bounds.top + InactiveInput::BORDER + InactiveInput::TOP_PADDING,
            ));
            self.new_range();
        }
    }

    fn as_drawable(&self) -> &dyn Drawable {
        self
    }
}

impl<'a> Fixed for InactiveInput<'a> {
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    // fn set_bounds(&mut self, new_bounds: FloatRect) {
    //     self.bounds = new_bounds;
    // }

    fn position(&self) -> sfml::system::Vector2f {
        (self.bounds.left, self.bounds.top).into()
    }

    fn set_position(&mut self, position: sfml::system::Vector2f) {
        let mut old_pos = self.position();
        let offset = sfml::system::Vector2f::new(position.x - old_pos.x, position.y - old_pos.y);

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
    }
}

impl<'b> Drawable for InactiveInput<'b> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&self.bg);
        target.draw(&self.copy_text);
        target.draw(&self.side_bg[0]);
        target.draw(&self.side_bg[1]);
    }
}
