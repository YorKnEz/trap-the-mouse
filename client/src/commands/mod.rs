mod error;
mod command_handlers;

pub use error::CommandError;
pub use command_handlers::*;

pub trait Command {
    fn execute(&mut self) -> Result<(), CommandError>;
}
