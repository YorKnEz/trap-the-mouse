use std::sync::mpsc;

use sfml::{
    graphics::{
        Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape, TextStyle, Transformable,
    },
    system::Vector2f,
};

use crate::events::{EventData, UIEvent, Window};

use super::{Fixed, MouseEventObserver};

pub struct ButtonBuilder {
    position: Option<Vector2f>,
    size: Option<Vector2f>,
    border: Option<f32>,
    colors: Option<ButtonColors>,
    text: Option<String>,
    font_size: Option<u32>,
    font_style: Option<TextStyle>,
    center_text: Option<bool>,
}

impl ButtonBuilder {
    const BUTTON_WIDTH: f32 = 240.0;
    const BUTTON_HEIGHT: f32 = 60.0;

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

    pub fn set_position(mut self, left: f32, top: f32) -> Self {
        self.position = Some(Vector2f::new(left, top));
        self
    }

    pub fn set_size(mut self, width: f32, height: f32) -> Self {
        self.size = Some(Vector2f::new(width, height));
        self
    }

    pub fn set_bounds(mut self, left: f32, top: f32, width: f32, height: f32) -> Self {
        self.position = Some(Vector2f::new(left, top));
        self.size = Some(Vector2f::new(width, height));
        self
    }

    pub fn set_border(mut self, border: f32) -> Self {
        self.border = Some(border);
        self
    }

    pub fn set_colors(mut self, variant: ButtonVariant) -> Self {
        self.colors = Some(match variant {
            ButtonVariant::Red => ButtonBuilder::RED_BUTTON,
            // ButtonVariant::Green => ButtonBuilder::GREEN_BUTTON,
        });
        self
    }

    pub fn set_text(mut self, text: &str) -> Self {
        self.text = Some(String::from(text));
        self
    }

    pub fn set_font_size(mut self, font_size: u32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub fn set_font_style(mut self, font_style: TextStyle) -> Self {
        self.font_style = Some(font_style);
        self
    }

    pub fn set_center_text(mut self, center_text: bool) -> Self {
        self.center_text = Some(center_text);
        self
    }

    pub fn build<'a>(
        self,
        id: u32,
        window: Window,
        sender: mpsc::Sender<UIEvent>,
        font: &'a RcFont,
    ) -> Button<'a> {
        let position = self.position.unwrap_or(Vector2f::new(0.0, 0.0));
        let size = self.size.unwrap_or(Vector2f::new(
            ButtonBuilder::BUTTON_WIDTH,
            ButtonBuilder::BUTTON_HEIGHT,
        ));
        let bounds = FloatRect::new(position.x, position.y, size.x, size.y);
        let border = self.border.unwrap_or(4.0);
        let colors = self.colors.unwrap_or(ButtonBuilder::GREEN_BUTTON);
        let text = self.text.unwrap_or("My Button".to_string());
        let font_size = self.font_size.unwrap_or(20);
        let font_style = self.font_style.unwrap_or(TextStyle::BOLD);
        let center_text = self.center_text.unwrap_or(true);

        let mut bg = RectangleShape::new();
        bg.set_size(Vector2f::new(
            bounds.width - 2.0 * border,
            bounds.height - 2.0 * border,
        ));
        bg.set_position((bounds.left + border, bounds.top + border));
        bg.set_outline_thickness(border);

        bg.set_fill_color(colors.bg);
        bg.set_outline_color(colors.border);

        let mut text = RcText::new(&text, font, font_size);
        text.set_style(font_style);
        let text_width = text.local_bounds().width;

        if center_text {
            text.set_position((
                bounds.left + bounds.width / 2.0 - text_width / 2.0,
                bounds.top + bounds.height / 2.0 - font_size as f32 / 2.0,
            ));
        } else {
            text.set_position((
                bounds.left,
                bounds.top + bounds.height / 2.0 - font_size as f32 / 2.0,
            ));
        }

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
    // Green,
    Red,
}

struct ButtonColors {
    border: Color,
    bg: Color,
    bg_hover: Color,
    bg_clicked: Color,
}

impl<'a> Button<'a> {
    pub fn builder() -> ButtonBuilder {
        ButtonBuilder {
            position: None,
            size: None,
            border: None,
            colors: None,
            text: None,
            font_size: None,
            font_style: None,
            center_text: None,
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
