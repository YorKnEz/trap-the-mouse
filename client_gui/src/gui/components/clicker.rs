use std::cell::RefCell;

use crate::types::RcCell;

use super::Fixed;

type Clicky<'a> = RcCell<dyn Clickable + 'a>;

// structure used to determine element getting clicked in O(1)
// u8 will be an index into the clickables vector
pub struct Clicker<'a> {
    width: u32,
    height: u32,
    buffer: RefCell<Vec<Vec<u8>>>,
    clickables: RefCell<Vec<Clicky<'a>>>,
}

impl<'a> Clicker<'a> {
    pub fn new(width: u32, height: u32) -> Clicker<'a> {
        Clicker {
            width,
            height,
            buffer: RefCell::new(vec![vec![u8::MAX; height as usize]; width as usize]),
            clickables: RefCell::new(vec![]),
        }
    }

    pub fn add_clickable(&self, clickable: Clicky<'a>) {
        let mut clickables = self.clickables.borrow_mut();

        if clickables.len() == u8::MAX as usize {
            return;
        }

        let (id, bounds) = {
            let _clickable = clickable.borrow();
            let id = _clickable.get_id();

            if id >= u8::MAX as u32 {
                return;
            }

            (id, _clickable.bounds())
        };

        let mut buffer = self.buffer.borrow_mut();

        for i in 0..bounds.height as usize {
            for j in 0..bounds.width as usize {
                buffer[bounds.top as usize + i][bounds.left as usize + j] = id as u8;
            }
        }

        clickables.push(clickable);
    }

    pub fn remove_clickable(&self, id: u32) {
        let mut clickables = self.clickables.borrow_mut();
        if let Some(index) = clickables.iter().position(|c| c.borrow().get_id() == id) {
            let clickable = clickables.remove(index);

            let bounds = clickable.borrow().bounds();
            let mut buffer = self.buffer.borrow_mut();

            for i in 0..bounds.height as usize {
                for j in 0..bounds.width as usize {
                    if buffer[bounds.top as usize + i][bounds.left as usize + j] == index as u8 {
                        buffer[bounds.top as usize + i][bounds.left as usize + j] = u8::MAX;
                    }
                }
            }
        }
    }

    pub fn click(&self, x: i32, y: i32) {
        if !(0 <= x && x < self.width as i32 && 0 <= y && y < self.height as i32) {
            return;
        }

        let id = self.buffer.borrow()[y as usize][x as usize];

        for c in self.clickables.borrow().iter() {
            let mut c = c.borrow_mut();
            if c.get_id() as u8 == id && id != u8::MAX {
                c.click(x as u32, y as u32);
            } else {
                c.no_click();
            }
        }
    }
}

/// Clickable trait
/// Any object that implements this must defined what happens when it gets clicked or when the user doesn't click on this object
pub trait Clickable: Fixed {
    /// get id of current clickable
    fn get_id(&self) -> u32;
    /// click method takes x and y because some objects (e.g. Scrollable) might care about where inside the component the user clicked
    fn click(&mut self, x: u32, y: u32);
    fn no_click(&mut self);
}
