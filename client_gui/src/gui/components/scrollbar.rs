use sfml::graphics::{Color, Drawable, RectangleShape, Shape, Transformable, FloatRect};

use super::Fixed;

pub struct Scrollbar<'a> {
    bounds: FloatRect,
    scrollable_height: f32,
    scrollbar: RectangleShape<'a>,
    thumb: RectangleShape<'a>,
    ratio: f32,
}

impl<'a> Scrollbar<'a> {
    pub fn new(
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        scrollable_height: f32,
    ) -> Scrollbar<'a> {
        let bounds = FloatRect {
            left,
            top,
            width,
            height,
        };

        let ratio = height / scrollable_height;

        let mut scrollbar = RectangleShape::new();
        scrollbar.set_size((width, height));
        scrollbar.set_position((left, top));
        scrollbar.set_fill_color(Color::rgb(54, 54, 54));

        let mut thumb = RectangleShape::new();

        if scrollable_height < height {
            thumb.set_size((width, height));
        } else {
            thumb.set_size((width, height * ratio));
        }

        thumb.set_position((left, top));
        thumb.set_fill_color(Color::rgb(145, 145, 145));

        Scrollbar {
            bounds,
            scrollable_height,
            scrollbar,
            thumb,
            ratio,
        }
    }

    pub fn resize_with(&mut self, new_height: f32) {
        let bar = self.scrollbar.bounds();
        self.scrollable_height += new_height;
        self.ratio = bar.height / self.scrollable_height;

        if self.scrollable_height < bar.height {
            self.thumb.set_size((bar.width, bar.height));
        } else {
            self.thumb.set_size((bar.width, bar.height * self.ratio));
        }
    }

    pub fn scroll_by(&mut self, delta: f32, offset: f32) -> f32 {
        let thumb = self.thumb.bounds();
        let bar = self.scrollbar.bounds();

        let new_thumb_offset = if delta > 0f32 {
            bar.top.max(thumb.top - delta * self.ratio * offset)
        } else if delta < 0f32 {
            (bar.top + bar.height).min(thumb.top + thumb.height - delta * self.ratio * offset)
                - thumb.height
        } else {
            0f32
        };

        self.thumb.set_position((thumb.left, new_thumb_offset));

        (thumb.top - new_thumb_offset) / self.ratio
    }

    pub fn scroll_to(&mut self, offset: f32) -> f32 {
        let thumb = self.thumb.bounds();
        let bar = self.scrollbar.bounds();

        let new_thumb_offset = ((bar.top + self.ratio * offset).max(bar.top) + thumb.height)
            .min(bar.top + bar.height)
            - thumb.height;

        self.thumb.set_position((thumb.left, new_thumb_offset));

        (new_thumb_offset - bar.top) / self.ratio
    }

    pub fn scrolled(&self) -> f32 {
        let thumb = self.thumb.bounds();
        let bar = self.scrollbar.bounds();

        (thumb.top - bar.top) / self.ratio
    }
}

impl<'a> Fixed for Scrollbar<'a> {
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: FloatRect) {
        self.bounds = new_bounds;
    }
}

impl<'b> Drawable for Scrollbar<'b> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw(&self.scrollbar);
        target.draw(&self.thumb);
    }
}
