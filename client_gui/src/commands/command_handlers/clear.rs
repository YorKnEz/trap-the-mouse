use crate::commands::CommandError;

pub fn clear_cmd() -> Result<(), CommandError> {
    std::process::Command::new("clear").status()?;

    Ok(())
}
