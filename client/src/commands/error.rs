use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("internal error")]
    InternalAnyhow(#[from] anyhow::Error),
    #[error("internal error")]
    InternalIo(#[from] std::io::Error),
    #[error("you are already connected to a lobby")]
    AlreadyConnected,
    #[error("you are not connected to a lobby")]
    NotConnected,
}

pub fn check_error(e: CommandError) {
    match e {
        CommandError::InternalAnyhow(e) => println!("cmd error: {e}"),
        e => println!("cmd error: {e}"),
    }
}
