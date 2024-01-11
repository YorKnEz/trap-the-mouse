use std::sync::mpsc;

use sfml::{
    graphics::{
        Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape, TextStyle, Transformable,
    },
    system::Vector2f,
};

use crate::events::{EventData, UIEvent, Window};

use super::{Fixed, MouseEventObserver};

pub struct Button<'a> {
    event_data: EventData,
    bounds: FloatRect,
    bg: RectangleShape<'a>,
    text: RcText,
    sender: mpsc::Sender<UIEvent>,
    selected: bool,
    colors: ButtonColors,
}

#[derive(PartialEq)]
pub enum ButtonVariant {
    Green,
    Red,
}

struct ButtonColors {
    border: Color,
    bg: Color,
    bg_hover: Color,
    bg_clicked: Color,
}

impl<'a> Button<'a> {
    const BORDER: f32 = 4.0;
    const GREEN_BUTTON: ButtonColors = ButtonColors {
        border: Color::rgb(45, 168, 78),
        bg: Color::rgb(53, 232, 101),
        bg_hover: Color::rgb(71, 235, 115),
        bg_clicked: Color::rgb(90, 237, 129),
    };
    const RED_BUTTON: ButtonColors = ButtonColors {
        border: Color::rgb(168, 45, 45),
        bg: Color::rgb(232, 53, 53),
        bg_hover: Color::rgb(235, 71, 71),
        bg_clicked: Color::rgb(237, 90, 90),
    };

    pub fn new(
        id: u32,
        window: Window,
        bounds: FloatRect,
        text: &str,
        font: &RcFont,
        sender: mpsc::Sender<UIEvent>,
        variant: ButtonVariant,
    ) -> Button<'a> {
        let mut bg = RectangleShape::new();
        bg.set_size(Vector2f::new(
            bounds.width - 2.0 * Button::BORDER,
            bounds.height - 2.0 * Button::BORDER,
        ));
        bg.set_position((bounds.left + Button::BORDER, bounds.top + Button::BORDER));
        bg.set_outline_thickness(Button::BORDER);

        let colors = match variant {
            ButtonVariant::Red => Button::RED_BUTTON,
            ButtonVariant::Green => Button::GREEN_BUTTON,
        };

        bg.set_fill_color(colors.bg);
        bg.set_outline_color(colors.border);

        let mut text = RcText::new(text, font, 20);
        text.set_style(TextStyle::BOLD);
        let text_width = text.local_bounds().width;
        let text_height = text.character_size() as f32;

        text.set_position((
            bounds.left + bounds.width / 2.0 - text_width / 2.0,
            bounds.top + bounds.height / 2.0 - text_height / 2.0,
        ));

        Button {
            event_data: EventData { window, id },
            bounds,
            bg,
            text,
            sender,
            selected: false,
            colors,
        }
    }
}

impl<'a> Fixed for Button<'a> {
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
            .set_position(Vector2f::new(old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.text.position();
        self.text
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));
    }
}

impl<'a> MouseEventObserver for Button<'a> {
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

impl<'a> Drawable for Button<'a> {
    fn draw<'b: 'shader, 'texture, 'shader, 'shader_texture>(
        &'b self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&self.bg);
        target.draw(&self.text);
    }
}
