use sfml::{
    graphics::{
        Color, Drawable, FloatRect, IntRect, RectangleShape, RenderTarget, RenderTexture, Shape,
        Sprite, Transformable,
    },
    system::Vector2f,
    window::Key,
};

use crate::{
    events::{Event, EventData, Window},
    types::RcCell,
    WINDOW_SIZE,
};

use super::{Clickable, EventHandlerMut, Fixed, Scrollbar};

/// Component that retains a list of T and renders them in a scrollable environment
pub struct Scrollable<'a, T> {
    event_data: EventData,
    bounds: FloatRect,
    scrollbar: Scrollbar<'a>,
    bg: RectangleShape<'a>,
    list: Vec<RcCell<T>>,
}

impl<'a, T> Scrollable<'a, T>
where
    T: Fixed,
{
    pub const PADDING: f32 = 10.0;
    pub const SCROLLBAR_WIDTH: f32 = 20.0;

    pub fn new(id: u32, window: Window, bounds: FloatRect) -> Scrollable<'a, T> {
        let scrollbar = Scrollbar::new(
            FloatRect::new(
                bounds.left + bounds.width - Scrollable::<T>::SCROLLBAR_WIDTH,
                bounds.top,
                Scrollable::<T>::SCROLLBAR_WIDTH,
                bounds.height,
            ),
            0.0,
        );

        let mut bg = RectangleShape::new();
        bg.set_size((bounds.width, bounds.height));
        bg.set_position((bounds.left, bounds.top));
        bg.set_fill_color(Color::rgb(45, 45, 45));

        Scrollable {
            event_data: EventData { id, window },
            bounds,
            scrollbar,
            bg,
            list: vec![],
        }
    }

    pub fn add(&mut self, item: RcCell<T>) {
        {
            let mut item_inner = item.borrow_mut();
            let pos = item_inner.bounds();

            let offset = self.scrollbar.scrolled();

            if let Some(last_item) = self.list.last_mut() {
                let last_item = last_item.borrow_mut();
                self.scrollbar
                    .resize_with(pos.height + Scrollable::<T>::PADDING);
                let last_pos = last_item.bounds();
                item_inner.set_position(Vector2f::new(
                    self.bounds.left + Scrollable::<T>::PADDING,
                    last_pos.top + last_pos.height + Scrollable::<T>::PADDING,
                ));
            } else {
                self.scrollbar
                    .resize_with(pos.height + 2.0 * Scrollable::<T>::PADDING);
                item_inner.set_position(Vector2f::new(
                    self.bounds.left + Scrollable::<T>::PADDING,
                    self.bounds.top + Scrollable::<T>::PADDING,
                ));
            }

            self.scrollbar.scroll_to(offset);
        }

        self.list.push(item);
    }

    pub fn remove(&mut self, index: usize) -> RcCell<T> {
        let rem_item = self.list.remove(index);
        let rem_bounds = rem_item.borrow().bounds();
        let rem = rem_bounds.height + Scrollable::<T>::PADDING;

        if self.list.is_empty() {
            self.scrollbar
                .resize_with(-(rem + Scrollable::<T>::PADDING));
            return rem_item;
        }

        let shift = -rem;

        // if the removed item is above the view bring the items above it down
        let (range, shift, scroll_offset) =
            // 0.01 because floats are funny and sizes are big so 0.01 is really small in comparison
            if rem_bounds.top + rem_bounds.height < self.bounds.top + Scrollable::<T>::PADDING - 0.01 {
                // if there are no items to bring down we need to move the whole view
                (0..index, rem, shift)
            }
            // if the removed item is in or below the view bring the items below it up
            else {
                // after this move it is possible that the elements will be shifted too much upwards
                // in this case we must shift the whole view in place
                let last = self.list.last().unwrap().borrow().bounds(); // we know list is not empty by this point

                // between the last item and the scrollable there is free space
                if last.top + shift + last.height
                    < self.bounds.top + self.bounds.height - Scrollable::<T>::PADDING
                {
                    // we need to shift the whole view by the minimum of the distance between last and scrollable end or first and scrollable start

                    // apply the old shift first, because in this case two shifts take place
                    // one for the elements below the removed item and one for all elements
                    // i prefer to shift elements below here and shift all in the final for
                    let temp_range = index..self.list.len();
                    for item in &mut self.list[temp_range] {
                        let mut item = item.borrow_mut();
                        let bounds = item.bounds();
                        item.set_position(Vector2f::new(
                            bounds.left,
                            bounds.top + shift,
                        ));
                    }

                    let first = self.list.first().unwrap().borrow().bounds(); // we know list is not empty by this point

                    let shift = (self.bounds.top + self.bounds.height
                        - Scrollable::<T>::PADDING
                        - (last.top + shift + last.height))
                        .min((self.bounds.top + Scrollable::<T>::PADDING - first.top).max(0.0));

                    (0..self.list.len(), shift, shift)
                } else {
                    (index..self.list.len(), shift, 0.0)
                }
            };

        let scrolled = self.scrollbar.scrolled();
        self.scrollbar.resize_with(-rem);
        self.scrollbar.scroll_to(scrolled + scroll_offset);

        for item in &mut self.list[range] {
            let mut item = item.borrow_mut();
            let bounds = item.bounds();
            item.set_position(Vector2f::new(bounds.left, bounds.top + shift));
        }

        rem_item
    }

    pub fn get(&mut self, index: usize) -> RcCell<T> {
        self.list[index].clone()
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    pub fn scroll_by(&mut self, delta: f32, offset: f32) {
        let offset = self.scrollbar.scroll_by(delta, offset);
        for item in &mut self.list {
            let mut item = item.borrow_mut();
            let pos = item.bounds();
            item.set_position(Vector2f::new(pos.left, pos.top + offset));
        }
    }

    pub fn scroll_to(&mut self, offset: f32) {
        let offset = self.scrollbar.scroll_to(offset);
        let mut top = self.bounds.top + Scrollable::<'a, T>::PADDING;
        for item in &mut self.list {
            let mut item = item.borrow_mut();
            let pos = item.bounds();
            item.set_position(Vector2f::new(pos.left, top - offset));

            top += pos.height + Scrollable::<'a, T>::PADDING;
        }
    }
}

impl<'a, T> EventHandlerMut for Scrollable<'a, T>
where
    T: Fixed + EventHandlerMut,
{
    fn handle_event(&mut self, e: Event) {
        match e.clone() {
            Event::Sfml(sfml::window::Event::MouseWheelScrolled {
                wheel: _,
                delta,
                x: _,
                y: _,
            }) => self.scroll_by(delta, 100.0),
            Event::Sfml(sfml::window::Event::KeyPressed {
                code,
                scan: _,
                alt: _,
                ctrl: _,
                shift: _,
                system: _,
            }) => match code {
                Key::Up => self.scroll_to(0.0),
                Key::Down => self.scroll_to(9999.0),
                _ => {}
            },
            _ => {}
        }

        for item in &mut self.list {
            let mut item = item.borrow_mut();
            item.handle_event(e.clone());
        }
    }
}

// implement clickable trait for scrollables that have clickabels in order to let the scrollable decide which object is clicked
// we use a raw check instead of using the clicker structure because a scrollable object often updates which would cause a lot of buffer updates on the clickable
impl<'a, T> Clickable for Scrollable<'a, T>
where
    T: Fixed + Clickable,
{
    fn get_id(&self) -> u32 {
        self.event_data.id
    }

    fn click(&mut self, x: u32, y: u32) {
        // decide where if the user clicked on a visible component
        for item in &mut self.list {
            let mut item = item.borrow_mut();
            let bounds = item.bounds();

            if bounds.top + bounds.height < self.bounds.top {
                item.no_click();
                continue;
            }

            if bounds.top > self.bounds.top + self.bounds.height {
                item.no_click();
                continue;
            }

            // the item is on the screen, check if it has been clicked
            if bounds.left <= x as f32
                && x as f32 <= bounds.left + bounds.width
                && bounds.top <= y as f32
                && y as f32 <= bounds.top + bounds.height
            {
                item.click(x, y);
            } else {
                item.no_click();
            }
        }
    }

    fn no_click(&mut self) {
        // delegate the no click event to all items
        for item in &mut self.list {
            let mut item = item.borrow_mut();
            item.no_click();
        }
    }
}

impl<'b, T> Fixed for Scrollable<'b, T>
where
    T: Fixed,
{
    fn bounds(&self) -> FloatRect {
        self.bounds
    }

    // fn set_bounds(&mut self, new_bounds: FloatRect) {
    //     let offset = (
    //         new_bounds.left - self.bounds.left,
    //         new_bounds.top - self.bounds.top,
    //     );
    //
    //     for item in &mut self.list {
    //         let mut item = item.borrow_mut();
    //         let pos = item.bounds();
    //         item.set_position((pos.left + offset.0, pos.top + offset.1));
    //     }
    //
    //     self.bounds = new_bounds;
    // }

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
            .set_position((old_pos.x + offset.x, old_pos.y + offset.y));

        old_pos = self.scrollbar.position();
        self.scrollbar
            .set_position(Vector2f::new(old_pos.x + offset.x, old_pos.y + offset.y));

        for item in &mut self.list {
            let mut item = item.borrow_mut();
            old_pos = item.position();
            item.set_position(Vector2f::new(old_pos.x + offset.x, old_pos.y + offset.y));
        }
    }
}

impl<'b, T> Drawable for Scrollable<'b, T>
where
    T: Drawable + Fixed,
{
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        _target: &mut dyn RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let bounds = self.bounds();
        let mut target = loop {
            let target = RenderTexture::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32);
            if let Some(target) = target {
                break target;
            }
        };

        target.draw(&self.bg);

        for item in &self.list {
            let item = item.borrow();
            let bounds = item.bounds();

            if bounds.top + bounds.height < self.bounds.top {
                continue;
            }

            if bounds.top > self.bounds.top + self.bounds.height {
                continue;
            }

            target.draw(&*item);
        }

        target.draw(&self.scrollbar);
        target.display();

        let mut sprite = Sprite::with_texture_and_rect(
            target.texture(),
            IntRect {
                left: bounds.left as i32,
                top: bounds.top as i32,
                width: bounds.width as i32,
                height: bounds.height as i32,
            },
        );
        sprite.set_position((bounds.left, bounds.top));

        _target.draw(&sprite);
    }
}
