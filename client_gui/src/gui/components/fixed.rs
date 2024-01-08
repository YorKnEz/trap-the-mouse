use sfml::graphics::{FloatRect, RectangleShape, Shape, Transformable};

pub trait Fixed {
    fn bounds(&self) -> FloatRect;
    fn set_bounds(&mut self, new_bounds: FloatRect);
}

impl<'a> Fixed for RectangleShape<'a> {
    fn bounds(&self) -> FloatRect {
        self.global_bounds()
    }

    fn set_bounds(&mut self, new_bounds: FloatRect) {
        self.set_position((new_bounds.left, new_bounds.top));
        self.set_size((new_bounds.width, new_bounds.height));
    }
}
