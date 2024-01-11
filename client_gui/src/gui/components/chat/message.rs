use sfml::{
    graphics::{Color, Drawable, FloatRect, RcFont, RcText, Transformable},
    system::Vector2f,
};

use crate::gui::components::{Fixed, EventHandlerMut};

pub struct Message {
    bounds: FloatRect,
    author: RcText,
    text_lines: Vec<RcText>,
}

impl Message {
    // const PADDING: f32 = 10.0;
    const LINE_SPACE: f32 = 2.0;
    const AUTHOR_COLOR: Color = Color::WHITE;
    const TEXT_COLOR: Color = Color::rgb(200, 200, 200);

    // messages won't be longer than 256
    pub fn new(bounds: FloatRect, author_: String, text_: String, font: &RcFont) -> Message {
        let mut bounds = bounds;

        let text_height = 16.0;
        let mut author = RcText::new(&format!("{author_}: "), font, text_height as u32);
        author.set_fill_color(Message::AUTHOR_COLOR);
        author.set_position((bounds.left, bounds.top));

        let mut text_lines = vec![];

        // author name shall not take more than one line of text
        let mut i = author_.len();
        if author.find_character_pos(i).x > bounds.left + bounds.width {
            let mut new_buf = format!("{author_}...: ");
            author.set_string(&new_buf);
            let dots_width = author.find_character_pos(i + 3).x - author.find_character_pos(i).x;

            while author.find_character_pos(i).x + dots_width
                > bounds.left + bounds.width
            {
                if i > 0 {
                    new_buf.remove(i - 3);
                    i -= 1;
                } else {
                    break;
                }
            }

            i = 0;

            author.set_string(&new_buf);
        }
        // if the author couldn't be shrinked, it means some text might fit on that line after it
        else {
            let mut text = RcText::new(&text_, font, text_height as u32);
            text.set_position((
                bounds.left + author.local_bounds().width,
                bounds.top,
            ));
            text.set_fill_color(Message::TEXT_COLOR);

            i = 0;
            while i < text_.len() {
                if text.find_character_pos(i).x > bounds.left + bounds.width {
                    break;
                }

                i += 1;
            }

            if i > 0 {
                text.set_string(&text_[0..i]);
                text_lines.push(text);
            }
        }

        bounds.height = text_height;

        // fit the rest of the text on new lines
        while i < text_.len() {
            let mut text = RcText::new(&text_[i..], font, text_height as u32);
            text.set_position((
                bounds.left,
                bounds.top + bounds.height + Message::LINE_SPACE,
            ));
            text.set_fill_color(Message::TEXT_COLOR);

            let start_i = i;

            while i < text_.len() {
                if text.find_character_pos(i - start_i).x > bounds.left + bounds.width {
                    break;
                }

                i += 1;
            }

            text.set_string(&text_[start_i..i]);
            text_lines.push(text);
            bounds.height += Message::LINE_SPACE + text_height;

            // no i += 1 needed here because the while above is guaranteed to advance by at least 1 every time
        }

        Message {
            bounds,
            author,
            text_lines,
        }
    }
}

impl EventHandlerMut for Message {
    fn handle_event(&mut self, _: crate::events::Event) {}
}

impl Fixed for Message {
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    fn position(&self) -> sfml::system::Vector2f {
        (self.bounds.left, self.bounds.top).into()
    }

    fn set_position(&mut self, position: sfml::system::Vector2f) {
        let mut old_pos = self.position();
        let offset = Vector2f::new(position.x - old_pos.x, position.y - old_pos.y);

        self.bounds.left = position.x;
        self.bounds.top = position.y;

        old_pos = self.author.position();
        self.author
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));

        for line in &mut self.text_lines {
            old_pos = line.position();
            line.set_position((old_pos.x + offset.x, old_pos.y + offset.y));
        }
    }
}

impl Drawable for Message {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&self.author);
        for line in &self.text_lines {
            target.draw(line);
        }
    }
}
