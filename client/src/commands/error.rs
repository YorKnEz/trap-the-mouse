use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("internal error")]
    InternalAnyhow(#[from] anyhow::Error),
    #[error("internal error")]
    InternalIo(#[from] std::io::Error),
    #[error("{message}")]
    CommandError { message: String },
}

pub fn check_error(e: CommandError) {
    match e {
        CommandError::InternalAnyhow(e) => println!("cmd error: {e}"),
        CommandError::CommandError { message } => println!("cmd error: {message}"),
        e => println!("cmd error: {}", e),
    }
}
