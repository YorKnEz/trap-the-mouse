use std::cell::RefCell;

use sfml::graphics::FloatRect;

use crate::types::RcCell;

use super::Fixed;

type Observer<'a> = RcCell<dyn MouseEventObserver + 'a>;

/// This structure holds arrays of elements for each relevant mouse event and when that event triggers it finds out in O(1) which element should get notified by the mouse event
pub struct MouseObserver<'a> {
    width: u32,
    height: u32,
    buffer: RefCell<Vec<Vec<u8>>>,
    observers: RefCell<Vec<Observer<'a>>>,
}

impl<'a> MouseObserver<'a> {
    pub fn new(width: u32, height: u32) -> MouseObserver<'a> {
        MouseObserver {
            width,
            height,
            buffer: RefCell::new(vec![vec![u8::MAX; height as usize]; width as usize]),
            observers: RefCell::new(vec![]),
        }
    }

    fn add_to_buffer(&self, id: u32, bounds: FloatRect) -> Option<()> {
        if id >= u8::MAX as u32 {
            return None;
        }

        let mut buffer = self.buffer.borrow_mut();

        for i in 0..bounds.height as usize {
            for j in 0..bounds.width as usize {
                buffer[bounds.top as usize + i][bounds.left as usize + j] = id as u8;
            }
        }

        Some(())
    }

    fn remove_from_buffer(&self, id: u32, bounds: FloatRect) {
        let mut buffer = self.buffer.borrow_mut();

        for i in 0..bounds.height as usize {
            for j in 0..bounds.width as usize {
                if buffer[bounds.top as usize + i][bounds.left as usize + j] == id as u8 {
                    buffer[bounds.top as usize + i][bounds.left as usize + j] = u8::MAX;
                }
            }
        }
    }

    pub fn add_observer(&self, observer: Observer<'a>) {
        let mut observers = self.observers.borrow_mut();

        if observers.len() == u8::MAX as usize {
            return;
        }

        let (id, bounds) = {
            let observer = observer.borrow();
            (observer.get_id(), observer.bounds())
        };

        if self.add_to_buffer(id, bounds).is_none() {
            return;
        }

        observers.push(observer);
    }

    pub fn remove_observer(&self, id: u32) {
        let mut observers = self.observers.borrow_mut();
        if let Some(index) = observers.iter().position(|c| c.borrow().get_id() == id) {
            let observer = observers.remove(index);

            let (id, bounds) = {
                let observer = observer.borrow();
                (observer.get_id(), observer.bounds())
            };

            self.remove_from_buffer(id, bounds);
        }
    }

    pub fn before_click(&self, x: i32, y: i32) {
        if !(0 <= x && x < self.width as i32 && 0 <= y && y < self.height as i32) {
            return;
        }

        let id = self.buffer.borrow()[y as usize][x as usize];

        for c in self.observers.borrow().iter() {
            let mut c = c.borrow_mut();
            if c.get_id() as u8 == id && id != u8::MAX {
                c.before_click(x as u32, y as u32);
            } else {
                c.no_click();
            }
        }
    }

    pub fn click(&self, x: i32, y: i32) {
        if !(0 <= x && x < self.width as i32 && 0 <= y && y < self.height as i32) {
            return;
        }

        let id = self.buffer.borrow()[y as usize][x as usize];

        for c in self.observers.borrow().iter() {
            let mut c = c.borrow_mut();
            if c.get_id() as u8 == id && id != u8::MAX {
                c.click(x as u32, y as u32);
            } else {
                c.no_click();
            }
        }
    }

    pub fn hover(&self, x: i32, y: i32) {
        if !(0 <= x && x < self.width as i32 && 0 <= y && y < self.height as i32) {
            return;
        }

        let id = self.buffer.borrow()[y as usize][x as usize];

        for c in self.observers.borrow().iter() {
            let mut c = c.borrow_mut();
            if c.get_id() as u8 == id && id != u8::MAX {
                c.hover(x as u32, y as u32);
            } else {
                c.no_hover();
            }
        }
    }
}

/// EventGenerator trait
/// Each component that generates events (buttons, inputs) must implement this trait
pub trait MouseEventObserver: Fixed {
    /// get id of current observer
    fn get_id(&self) -> u32;

    /// x and y specify the coordonates where the click occured
    fn before_click(&mut self, x: u32, y: u32);
    fn click(&mut self, x: u32, y: u32);
    fn no_click(&mut self);

    /// x and y specify the coordonates where the hover occured
    fn hover(&mut self, x: u32, y: u32);
    fn no_hover(&mut self);
}
