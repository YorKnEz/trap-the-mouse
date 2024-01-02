mod error;
mod event_handlers;

mod event_loop;

use std::sync::{Arc, Mutex};

pub use error::EventError;
pub use event_handlers::*;
pub use event_loop::EventLoop;

pub type BoolMutex = Arc<Mutex<bool>>;

pub type EventQueue = Arc<Mutex<Vec<EventQueueItem>>>;
pub type EventQueueItem = Box<dyn Event + Send>;
