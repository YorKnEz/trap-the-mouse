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
