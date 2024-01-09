use sfml::{graphics::FloatRect, system::Vector2f};

pub trait Fixed {
    fn bounds(&self) -> FloatRect;
    // fn set_bounds(&mut self, new_bounds: FloatRect);
    fn position(&self) -> Vector2f;
    fn set_position(&mut self, position: Vector2f);
}
