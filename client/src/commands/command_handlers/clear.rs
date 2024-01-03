use crate::commands::{Command, CommandError};

pub struct ClearCmd {}

impl ClearCmd {
    pub fn new() -> ClearCmd {
        ClearCmd {}
    }
}

impl Command for ClearCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        std::process::Command::new("clear").status()?;

        Ok(())
    }
}
