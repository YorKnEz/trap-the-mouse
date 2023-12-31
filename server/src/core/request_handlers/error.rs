use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
    #[error("{message}")]
    API { message: String },
}
