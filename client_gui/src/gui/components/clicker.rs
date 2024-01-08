use std::cell::RefCell;

use crate::types::RcCell;

use super::Fixed;

type Clicky<'a> = RcCell<dyn Clickable + 'a>;

// structure used to determine element getting clicked in O(1)
// u8 will be an index into the clickables vector
pub struct Clicker<'a> {
    buffer: RefCell<Vec<Vec<u8>>>,
    clickables: RefCell<Vec<Clicky<'a>>>,
}

impl<'a> Clicker<'a> {
    pub fn new(width: u32, height: u32) -> Clicker<'a> {
        Clicker {
            buffer: RefCell::new(vec![vec![u8::MAX; height as usize]; width as usize]),
            clickables: RefCell::new(vec![]),
        }
    }

    pub fn add_clickable(&self, clickable: Clicky<'a>) {
        let mut clickables = self.clickables.borrow_mut();

        if clickables.len() == u8::MAX as usize {
            return;
        }

        let bounds = clickable.borrow().bounds();
        let mut buffer = self.buffer.borrow_mut();

        for i in 0..bounds.height as usize {
            for j in 0..bounds.width as usize {
                buffer[bounds.top as usize + i][bounds.left as usize + j] = clickables.len() as u8;
            }
        }

        clickables.push(clickable);
    }

    pub fn _remove_clickable(&self, index: u8) {
        let mut clickables = self.clickables.borrow_mut();
        let clickable = clickables.remove(index as usize);

        let bounds = clickable.borrow().bounds();
        let mut buffer = self.buffer.borrow_mut();

        for i in 0..bounds.height as usize {
            for j in 0..bounds.width as usize {
                buffer[i][j] = 0;
            }
        }
    }

    pub fn click(&self, x: u32, y: u32) {
        let index = self.buffer.borrow()[y as usize][x as usize] as usize;

        for (i, c) in self.clickables.borrow().iter().enumerate() {
            if i == index && index != u8::MAX as usize {
                c.borrow_mut().click(x, y);
            } else {
                c.borrow_mut().no_click();
            }
        }
    }
}

/// Clickable trait
/// Any object that implements this must defined what happens when it gets clicked or when the user doesn't click on this object
pub trait Clickable: Fixed {
    /// click method takes x and y because some objects (e.g. Scrollable) might care about where inside the component the user clicked
    fn click(&mut self, x: u32, y: u32);
    fn no_click(&mut self);
}
