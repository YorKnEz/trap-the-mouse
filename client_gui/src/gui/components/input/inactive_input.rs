use std::sync::mpsc;

use sfml::graphics::{Drawable, FloatRect, RcText, RectangleShape, Shape, Transformable};

use crate::{
    events::{Event, EventData, UIEvent},
    gui::components::Fixed,
};

use super::{ActiveInput, InputColors, InputState, InputBuilder};

pub struct InactiveInput<'a> {
    pub event_data: EventData,
    pub bounds: FloatRect,
    pub bg: RectangleShape<'a>,
    pub side_bg: [RectangleShape<'a>; 2],
    pub border: f32,
    pub colors: InputColors,

    pub range: (usize, usize),
    pub value: String,
    pub placeholder: String,
    pub copy_text: RcText,
    pub text: RcText,
    pub sender: mpsc::Sender<UIEvent>,
}

impl<'a> InactiveInput<'a> {
    pub fn from(from: ActiveInput<'a>) -> InactiveInput<'a> {
        InactiveInput {
            event_data: from.event_data,
            bounds: from.bounds,
            bg: from.bg,
            side_bg: from.side_bg,
            border: from.border,
            colors: from.colors,
            range: from.range,
            value: from.value,
            placeholder: from.placeholder,
            copy_text: from.copy_text,
            text: from.text,
            sender: from.sender,
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

impl InputState for InactiveInput<'static> {
    fn handle_event(self: Box<Self>, e: Event) -> Box<dyn InputState> {
        if let Event::UI(UIEvent::InputClicked(_)) = e {
            return Box::new(ActiveInput::from(*self));
        }

        self
    }

    fn get_value(&self) -> String {
        self.value.clone()
    }

    fn set_value(&mut self, value: String) {
        self.value = value;
        if self.value.is_empty() {
            self.text.set_string(&self.placeholder);
            self.text.set_position((
                self.bounds.left + self.border + InputBuilder::LEFT_PADDING,
                self.bounds.top + self.border + InputBuilder::TOP_PADDING,
            ));
            self.copy_text.set_string(&self.placeholder);
        } else {
            self.text.set_string(&self.value);
            self.text.set_position((
                self.bounds.left + self.border + InputBuilder::LEFT_PADDING,
                self.bounds.top + self.border + InputBuilder::TOP_PADDING,
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
