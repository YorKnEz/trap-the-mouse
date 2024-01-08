use std::sync::mpsc;

use sfml::graphics::{
    Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape, Transformable,
};

use crate::{
    events::{Event, UIEvent},
    gui::components::Fixed,
};

use super::{ActiveInput, InputState};

pub struct InactiveInput<'a> {
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
    const LEFT_PADDING: f32 = 10f32;
    const TOP_PADDING: f32 = 10f32;
    const BORDER: f32 = 2f32;

    pub fn new(
        left: f32,
        top: f32,
        width: f32,
        text_height: f32,
        font: &RcFont,
        value: &str,
        placeholder: &str,
        sender: mpsc::Sender<UIEvent>,
    ) -> InactiveInput<'a> {
        let bounds = FloatRect {
            left,
            top,
            width,
            height: text_height + 2f32 * InactiveInput::TOP_PADDING + 2f32 * InactiveInput::BORDER,
        };

        let mut bg = RectangleShape::new();
        bg.set_size((
            bounds.width - 2f32 * InactiveInput::BORDER,
            bounds.height - 2f32 * InactiveInput::BORDER,
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
            bounds.height - 2f32 * InactiveInput::BORDER,
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

        let mut text = if value.len() == 0 {
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
        cursor.set_size((1f32, text_height + 2f32 * InactiveInput::BORDER));
        cursor.set_position((
            bounds.left + InactiveInput::BORDER + InactiveInput::LEFT_PADDING,
            bounds.top + InactiveInput::BORDER + InactiveInput::TOP_PADDING,
        ));
        cursor.set_fill_color(Color::BLACK);

        InactiveInput {
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

        if self.buf.len() > 0 {
            let left = self.side_bg[0].bounds();
            let limit = (left.left + left.width, self.side_bg[1].bounds().left);

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
        match e {
            Event::UI(UIEvent::InputClicked(_)) => {
                return Box::new(ActiveInput::from(*self));
            }
            _ => {}
        }

        return self;
    }

    fn get_value(&self) -> String {
        self.buf.clone()
    }

    fn set_value(&mut self, value: String) {
        self.buf = value;
        if self.buf.len() == 0 {
            self.text.set_string(&self.placeholder);
            self.copy_text.set_string(&self.placeholder);
        } else {
            self.text.set_string(&self.buf);
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

    fn set_bounds(&mut self, new_bounds: FloatRect) {
        self.bounds = new_bounds;
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
