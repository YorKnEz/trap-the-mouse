mod command_handlers;
mod error;

pub use command_handlers::*;
pub use error::CommandError;

pub trait Command {
    fn execute(&mut self) -> Result<(), CommandError>;
}
