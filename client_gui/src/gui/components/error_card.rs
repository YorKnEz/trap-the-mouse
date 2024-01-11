use std::sync::mpsc;

use sfml::{
    graphics::{
        Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape,Transformable,
    },
    system::Vector2f,
};

use crate::{
    events::{EventData, UIEvent, Window},
    gui::components::{EventHandlerMut, Fixed},
};

use super::MouseEventObserver;

pub struct ErrorCard<'a> {
    event_data: EventData,
    bounds: FloatRect,
    bg: RectangleShape<'a>,
    text_lines: Vec<RcText>,
    sender: mpsc::Sender<UIEvent>,
    selected: bool,
    colors: ErrorCardColors,
}

pub struct ErrorCardColors {
    bg: Color,
    bg_hover: Color,
    bg_clicked: Color,
    border: Color,
    text: Color,
}

pub struct ErrorCardBuilder {
    position: Option<Vector2f>,
    width: Option<f32>,
    border: Option<f32>,
    colors: Option<ErrorCardColors>,
    error: Option<String>,
    font_size: Option<u32>,
}

impl ErrorCardBuilder {
    const WIDTH: f32 = 240.0;

    const PADDING: f32 = 10.0;
    const LINE_SPACE: f32 = 2.0;

    // pub fn set_position(mut self, left: f32, top: f32) -> Self {
    //     self.position = Some(Vector2f::new(left, top));
    //     self
    // }

    // pub fn set_width(mut self, width: f32) -> Self {
    //     self.width = Some(width);
    //     self
    // }

    pub fn set_bounds(mut self, left: f32, top: f32, width: f32) -> Self {
        self.position = Some(Vector2f::new(left, top));
        self.width = Some(width);
        self
    }

    // pub fn set_border(mut self, border: f32) -> Self {
    //     self.border = Some(border);
    //     self
    // }

    // pub fn set_colors(mut self, colors: ErrorCardColors) -> Self {
    //     self.colors = Some(colors);
    //     self
    // }

    pub fn set_error(mut self, error: &str) -> Self {
        self.error = Some(String::from(error));
        self
    }

    // pub fn set_font_size(mut self, font_size: u32) -> Self {
    //     self.font_size = Some(font_size);
    //     self
    // }

    pub fn build(
        self,
        id: u32,
        window: Window,
        sender: mpsc::Sender<UIEvent>,
        font: &RcFont,
    ) -> ErrorCard {
        let position = self.position.unwrap_or(Vector2f::new(0.0, 0.0));
        let width = self.width.unwrap_or(ErrorCardBuilder::WIDTH);
        let mut bounds = FloatRect::new(position.x, position.y, width, 0.0);
        let border = self.border.unwrap_or(2.0);
        let colors = self.colors.unwrap_or(ErrorCardColors {
            border: Color::rgb(168, 45, 45),
            bg: Color::rgb(232, 53, 53),
            bg_hover: Color::rgb(235, 71, 71),
            bg_clicked: Color::rgb(237, 90, 90),
            text: Color::WHITE,
        });
        let error = self.error.unwrap_or("Error goes here".to_string());
        let font_size = self.font_size.unwrap_or(20);

        let mut text_lines = vec![];

        let mut i = 0;
        bounds.height = border + ErrorCardBuilder::PADDING;

        // enlarge the box vertically until all text fits
        while i < error.len() {
            let mut text = RcText::new(&error[i..], font, font_size);
            text.set_position((
                bounds.left + ErrorCardBuilder::PADDING,
                bounds.top + bounds.height,
            ));
            text.set_fill_color(colors.text);

            let start_i = i;

            while i < error.len() {
                if text.find_character_pos(i - start_i).x > bounds.left + bounds.width - ErrorCardBuilder::PADDING {
                    break;
                }

                i += 1;
            }

            text.set_string(&error[start_i..i]);
            text_lines.push(text);
            bounds.height += ErrorCardBuilder::LINE_SPACE + font_size as f32;

            // no i += 1 needed here because the while above is guaranteed to advance by at least 1 every time
        }

        bounds.height += ErrorCardBuilder::PADDING - ErrorCardBuilder::LINE_SPACE + border;
        bounds.top = bounds.top - bounds.height;

        let mut bg = RectangleShape::new();
        bg.set_size(Vector2f::new(
            bounds.width - 2.0 * border,
            bounds.height - 2.0 * border,
        ));
        bg.set_position((bounds.left + border, bounds.top + border));
        bg.set_outline_thickness(border);

        bg.set_fill_color(colors.bg);
        bg.set_outline_color(colors.border);

        for line in &mut text_lines {
            let pos = line.position();
            line.set_position((pos.x, pos.y - bounds.height));
        }

        ErrorCard {
            event_data: EventData { id, window },
            bounds,
            bg,
            text_lines,
            sender,
            selected: false,
            colors,
        }
    }
}

impl<'a> ErrorCard<'a> {
    // messages won't be longer than 256
    pub fn builder() -> ErrorCardBuilder {
        ErrorCardBuilder {
            position: None,
            width: None,
            border: None,
            colors: None,
            error: None,
            font_size: None,
        }
    }
}

impl<'a> EventHandlerMut for ErrorCard<'a> {
    fn handle_event(&mut self, _: crate::events::Event) {}
}

impl<'a> Fixed for ErrorCard<'a> {
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

        old_pos = self.bg.position();
        self.bg
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));

        for line in &mut self.text_lines {
            old_pos = line.position();
            line.set_position((old_pos.x + offset.x, old_pos.y + offset.y));
        }
    }
}

impl<'a> MouseEventObserver for ErrorCard<'a> {
    fn get_id(&self) -> u32 {
        self.event_data.id
    }

    fn before_click(&mut self, _x: u32, _y: u32) {
        self.selected = true;
        self.bg.set_fill_color(self.colors.bg_clicked);
    }

    fn click(&mut self, _x: u32, _y: u32) {
        if let Err(e) = self.sender.send(UIEvent::ButtonClicked(self.event_data)) {
            println!("send error: {e:?}");
        }

        self.selected = false;
        self.bg.set_fill_color(self.colors.bg);
    }

    fn no_click(&mut self) {
        self.selected = false;
        self.bg.set_fill_color(self.colors.bg);
    }

    fn hover(&mut self, _x: u32, _y: u32) {
        if !self.selected {
            self.bg.set_fill_color(self.colors.bg_hover);
        }
    }

    fn no_hover(&mut self) {
        if !self.selected {
            self.bg.set_fill_color(self.colors.bg);
        }
    }
}

impl<'a> Drawable for ErrorCard<'a> {
    fn draw<'b: 'shader, 'texture, 'shader, 'shader_texture>(
        &'b self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&self.bg);
        for line in &self.text_lines {
            target.draw(line);
        }
    }
}
