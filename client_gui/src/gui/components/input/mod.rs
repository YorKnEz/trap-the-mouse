mod active_input;
mod inactive_input;

use std::sync::mpsc;

use crate::events::{Event, EventData, UIEvent, Window};

use active_input::ActiveInput;
use inactive_input::InactiveInput;
use sfml::graphics::{
    Color, Drawable, FloatRect, RcFont, RcText, RectangleShape, Shape, Transformable,
};

use super::{EventHandlerMut, Fixed, MouseEventObserver};

pub trait InputState: Fixed {
    fn handle_event(self: Box<Self>, e: Event) -> Box<dyn InputState>;
    fn get_value(&self) -> String;
    fn set_value(&mut self, value: String);
    fn as_drawable(&self) -> &dyn Drawable;
}

pub struct InputColors {
    border: Color,
    bg: Color,
    cursor: Color,
    value: Color,
    placeholder: Color,
}

pub struct InputBuilder {
    bounds: Option<FloatRect>,
    border: Option<f32>,
    colors: Option<InputColors>,
    placeholder: Option<String>,
    value: Option<String>,
    font_size: Option<u32>,
}

impl InputBuilder {
    const INPUT_WIDTH: f32 = 240.0;
    const LEFT_PADDING: f32 = 16.0;
    const TOP_PADDING: f32 = 10.0;

    const INPUT_COLORS: InputColors = InputColors {
        border: Color::rgb(190, 190, 190),
        bg: Color::rgb(238, 238, 238),
        cursor: Color::BLACK,
        value: Color::BLACK,
        placeholder: Color::rgb(45, 45, 45),
    };

    pub fn set_bounds(mut self, left: f32, top: f32, width: f32) -> Self {
        self.bounds = Some(FloatRect::new(left, top, width, 0.0));
        self
    }

    // pub fn set_border(mut self, border: f32) -> Self {
    //     self.border = Some(border);
    //     self
    // }

    pub fn set_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(String::from(placeholder));
        self
    }

    // pub fn set_value(mut self, value: &str) -> Self {
    //     self.value = Some(String::from(value));
    //     self
    // }

    pub fn set_font_size(mut self, font_size: u32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub fn build<'a>(
        self,
        id: u32,
        window: Window,
        sender: mpsc::Sender<UIEvent>,
        font: &'a RcFont,
    ) -> Input {
        let font_size = self.font_size.unwrap_or(20);
        let border = self.border.unwrap_or(4.0);
        let mut bounds =
            self.bounds
                .unwrap_or(FloatRect::new(0.0, 0.0, InputBuilder::INPUT_WIDTH, 0.0));
        bounds.height = font_size as f32 + 2.0 * InputBuilder::TOP_PADDING + 2.0 * border;
        let colors = self.colors.unwrap_or(InputBuilder::INPUT_COLORS);
        let placeholder = self.placeholder.unwrap_or("Type something".to_string());
        let value = self.value.unwrap_or("".to_string());

        let mut bg = RectangleShape::new();
        bg.set_size((bounds.width - 2.0 * border, bounds.height - 2.0 * border));
        bg.set_position((bounds.left + border, bounds.top + border));
        bg.set_fill_color(colors.bg);

        bg.set_outline_thickness(border);
        bg.set_outline_color(colors.border);

        let mut side_bg = RectangleShape::new();
        side_bg.set_size((InputBuilder::LEFT_PADDING, bounds.height - 2.0 * border));
        side_bg.set_position((bounds.left + border, bounds.top + border));
        side_bg.set_fill_color(colors.bg);

        let mut side_bg = [side_bg.clone(), side_bg];
        side_bg[1].set_position((
            bounds.left + bounds.width - border - InputBuilder::LEFT_PADDING,
            bounds.top + border,
        ));

        let mut text = if value.is_empty() {
            let mut text = RcText::new(&placeholder, font, font_size);
            text.set_fill_color(colors.placeholder);
            text
        } else {
            let mut text = RcText::new(&value, font, font_size);
            text.set_fill_color(colors.value);
            text
        };

        text.set_position((
            bounds.left + border + InputBuilder::LEFT_PADDING,
            bounds.top + border + InputBuilder::TOP_PADDING,
        ));

        let mut cursor = RectangleShape::new();
        cursor.set_size((1.0, font_size as f32 + 2.0 * border));
        cursor.set_position((
            bounds.left + border + InputBuilder::LEFT_PADDING,
            bounds.top + border + InputBuilder::TOP_PADDING,
        ));
        cursor.set_fill_color(colors.cursor);

        Input {
            event_data: EventData { window, id },
            sender: sender.clone(),
            state: Some(Box::new(InactiveInput {
                event_data: EventData { window, id },
                bounds,
                bg,
                side_bg,
                colors,
                range: (0, 0),
                value,
                placeholder,
                copy_text: text.clone(),
                text,
                sender,
            })),
        }
    }
}

pub struct Input {
    event_data: EventData,
    sender: mpsc::Sender<UIEvent>,
    state: Option<Box<dyn InputState>>,
}

impl Input {
    pub fn builder() -> InputBuilder {
        InputBuilder {
            bounds: None,
            border: None,
            colors: None,
            placeholder: None,
            value: None,
            font_size: None,
        }
    }

    pub fn set_value(&mut self, value: String) {
        self.state.as_mut().unwrap().set_value(value);
    }
}

impl EventHandlerMut for Input {
    fn handle_event(&mut self, e: Event) {
        match e {
            Event::UI(UIEvent::InputClicked(event_data)) => {
                if event_data != self.event_data {
                    return;
                }
            }
            Event::UI(UIEvent::InputNoClicked(event_data)) => {
                if event_data != self.event_data {
                    return;
                }
            }
            _ => {}
        }

        if let Some(state_) = self.state.take() {
            self.state = Some(state_.handle_event(e));
        }
    }
}

impl MouseEventObserver for Input {
    fn get_id(&self) -> u32 {
        self.event_data.id
    }

    fn before_click(&mut self, _x: u32, _y: u32) {}

    fn click(&mut self, _x: u32, _y: u32) {
        if let Err(e) = self.sender.send(UIEvent::InputClicked(self.event_data)) {
            println!("send error: {e:?}");
        }
    }

    fn no_click(&mut self) {
        if let Err(e) = self.sender.send(UIEvent::InputNoClicked(self.event_data)) {
            println!("send error: {e:?}");
        }
    }

    fn hover(&mut self, _x: u32, _y: u32) {}
    fn no_hover(&mut self) {}
}

impl Fixed for Input {
    fn bounds(&self) -> FloatRect {
        self.state.as_ref().unwrap().bounds()
    }

    // fn set_bounds(&mut self, new_bounds: FloatRect) {
    //     self.state.as_mut().unwrap().set_bounds(new_bounds);
    // }

    fn position(&self) -> sfml::system::Vector2f {
        self.state.as_ref().unwrap().position()
    }

    fn set_position(&mut self, position: sfml::system::Vector2f) {
        self.state.as_mut().unwrap().set_position(position);
    }
}

impl Drawable for Input {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(self.state.as_ref().unwrap().as_drawable());
    }
}
